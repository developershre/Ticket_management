#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;

pub mod api;
pub mod auth;
mod components;
pub mod db;
mod icons;
pub mod notifications;
mod pages;
pub mod sheet_crud;
mod state;
pub mod sync;
mod sync_test;
#[cfg(test)]
mod reminder_sync_tests;

use components::app_sidebar::AppSidebar;
use components::fab::Fab;
use components::titlebar::Titlebar;
use components::ui::sidebar::SidebarProvider;
use components::ui::toast::ToastProvider;
use pages::analytics::Analytics;
use pages::auth::{Login, Profile, Register};
use pages::calendar_page::CalendarPage;
use pages::employees::Employees;
use pages::notes::NotesPage;
use pages::reminders::Reminders;
use pages::tickets::Tickets;

/// All app routes.
///
/// There is intentionally no default dioxus navbar: `AppShell` renders a
/// completely custom chrome (custom draggable titlebar, sidebar navigation,
/// FAB) around the router outlet.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Tickets {},
    #[route("/notes")]
    NotesPage {},
    #[route("/employees")]
    Employees {},
    #[route("/reminders")]
    Reminders {},
    #[route("/calendar")]
    CalendarPage {},
    #[route("/analytics")]
    Analytics {},
    #[route("/profile")]
    Profile {},
    #[end_layout]
    #[layout(AuthShell)]
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[end_layout]
    #[route("/:..route")]
    NotFound { route: Vec<String> },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const DND_JS: Asset = asset!("/assets/dnd.js");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Command-line tool to wipe the active database and leave all tables empty
    if args.first().map(String::as_str) == Some("--reset-db") {
        if let Ok(conn) = db::get_connection() {
            let tables = ["USERS", "auth_sessions", "auth_users", "allowed_email_domains", "reminders", "tickets", "NOTES", "sync_queue", "sync_conflicts", "sync_meta", "sheet_changes"];
            for t in tables {
                let _ = conn.execute(&format!("DELETE FROM {}", t), []);
            }
            let _ = conn.execute_batch("VACUUM;");
            println!("[BPSL IT Support] Active database wiped cleanly (all tables 0 rows).");
            std::process::exit(0);
        }
    }

    // Command-line tool to verify and update tables without wiping existing user data
    if args.first().map(String::as_str) == Some("--migrate-db") {
        let active_db = db::db_path();
        match db::init_database() {
            Ok(_) => {
                println!("[BPSL IT Support] Database tables at '{}' verified and updated successfully.", active_db.display());
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("[BPSL IT Support] Migration failed at '{}': {:?}", active_db.display(), e);
                std::process::exit(1);
            }
        }
    }

    // Command-line tool to initialize a single pristine, empty SQLite database with all tables (0 rows, no settings)
    if args.first().map(String::as_str) == Some("--init-clean-db")
        || args.first().map(String::as_str) == Some("--bundle-prep")
    {
        let target = args.get(1).cloned().unwrap_or_else(|| "management.db".to_string());
        if std::path::Path::new(&target).exists() {
            let _ = std::fs::remove_file(&target);
        }
        std::env::set_var("TMS_DB_PATH", &target);
        match db::init_database() {
            Ok(_) => {
                let _ = db::factory_reset_database();
                if let Ok(conn) = db::get_connection() {
                    let tables = ["USERS", "auth_sessions", "auth_users", "allowed_email_domains", "reminders", "tickets", "NOTES", "sync_queue", "sync_conflicts", "sync_meta", "sheet_changes"];
                    println!("[BPSL IT Support] Verified table row counts for '{}':", target);
                    for t in tables {
                        let count: i64 = conn.query_row(&format!("SELECT COUNT(*) FROM {}", t), [], |r| r.get(0)).unwrap_or(-1);
                        println!("  - {}: {} rows", t, count);
                    }
                }
                println!("[BPSL IT Support] Successfully initialized single pristine empty database at '{}' with 0 rows and no pre-entered data or settings.", target);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("[BPSL IT Support] Failed to initialize clean database: {:?}", e);
                std::process::exit(1);
            }
        }
    }

    // Sync test harness (see src/sync_test.rs): configures its own DB path
    // and runs scripted end-to-end scenarios, so it must run before the
    // regular database initialization.
    if args.first().map(String::as_str) == Some("--sync-test") {
        let scenario = args.get(1).cloned().unwrap_or_default();
        std::process::exit(sync_test::run(&scenario));
    }

    // Initialize SQLite database adhering to DATABASE.SQL schema
    let active_db = db::db_path();
    println!("[BPSL IT Support DB] Active database: {}", active_db.display());
    if let Err(e) = db::init_database() {
        eprintln!(
            "[BPSL IT Support DB] Warning: database init failed at '{}': {:?}",
            active_db.display(),
            e
        );
    }

    // Operational aid: run a single sync cycle against the configured API
    // and print the resulting status (no UI).
    if args.first().map(String::as_str) == Some("--sync-once") {
        if args.iter().any(|a| a == "--use-env-api") {
            let _ = db::clear_system_api_config();
            let _ = db::sync_meta_set("last_pull_at", "0");
            let _ = db::sync_meta_set("last_employee_pull_at", "0");
            if let Ok(conn) = db::get_connection() {
                let _ = conn.execute("UPDATE tickets SET server_last_updated = ''", []);
            }
        }
        if let Ok(cfg) = api::ApiConfig::load() {
            println!(
                "[sync] db={} protocol={:?} url={}",
                db::db_path().display(),
                cfg.protocol,
                cfg.base_url
            );
        }
        sync::run_cycle_blocking();
        let s = sync::snapshot();
        println!(
            "[sync] state={:?} pending={} conflicts={} detail={}",
            s.state, s.pending_ops, s.conflicts, s.detail
        );
        if let Ok(conn) = db::get_connection() {
            let tickets: i64 = conn.query_row("SELECT COUNT(*) FROM tickets", [], |r| r.get(0)).unwrap_or(-1);
            let users: i64 = conn.query_row("SELECT COUNT(*) FROM USERS", [], |r| r.get(0)).unwrap_or(-1);
            let c_pend: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE kanban_state = 'Pending'", [], |r| r.get(0)).unwrap_or(0);
            let c_prog: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE kanban_state = 'InProgress'", [], |r| r.get(0)).unwrap_or(0);
            let c_hold: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE kanban_state = 'Hold' OR kanban_state = 'Onhold'", [], |r| r.get(0)).unwrap_or(0);
            let c_solved: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE kanban_state = 'Solved'", [], |r| r.get(0)).unwrap_or(0);
            let c_done: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE kanban_state = 'Completed'", [], |r| r.get(0)).unwrap_or(0);
            let c_cancel: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE kanban_state = 'Cancelled'", [], |r| r.get(0)).unwrap_or(0);
            let s_open: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE status = 'Open'", [], |r| r.get(0)).unwrap_or(0);
            let s_closed: i64 = conn.query_row("SELECT COUNT(*) FROM tickets WHERE status = 'Closed'", [], |r| r.get(0)).unwrap_or(0);
            println!(
                "[sync] local_counts: tickets={} employees={} | columns: Pending={} InProgress={} Hold={} Solved={} Solved(closed)={} Cancelled={} | status: Open={} Closed={}",
                tickets, users, c_pend, c_prog, c_hold, c_solved, c_done, c_cancel, s_open, s_closed
            );
        }
        return;
    }

    // Background sync worker: single thread, independent of the UI
    // lifecycle (20s cadence + immediate wake after local changes).
    sync::start_worker();

    // Desktop: the native window frame (navbar + OS action buttons) is
    // disabled entirely — the custom `Titlebar` component provides the
    // drag region and the minimize/maximize/close buttons instead.
    #[cfg(feature = "desktop")]
    {
        use components::ui::theme::{fetch_system_is_dark, ThemeMode};
        use dioxus::desktop::tao::window::Theme;
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        let initial_mode_str = db::sync_meta_get("ui_theme_mode").unwrap_or_else(|| "system".to_string());
        let initial_mode = ThemeMode::from_str_lossy(&initial_mode_str);
        let is_sys_dark = fetch_system_is_dark();
        let is_initial_dark = match initial_mode {
            ThemeMode::Dark => true,
            ThemeMode::Light => false,
            ThemeMode::System => is_sys_dark,
        };
        let initial_theme = if is_initial_dark {
            Theme::Dark
        } else {
            Theme::Light
        };

        dioxus::LaunchBuilder::new()
            .with_cfg(
                Config::new()
                    .with_disable_drag_drop_handler(true)
                    .with_custom_head(
                        r#"<script>
                        (function() {
                            // 1. Initial theme setup before DOM paint to prevent flash of light theme
                            var savedTheme = '';
                            try {
                                savedTheme = localStorage.getItem('bpsl-theme') || '';
                            } catch(e) {}
                            var sysDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
                            var isDark = savedTheme === 'dark' || (savedTheme !== 'light' && sysDark);
                            var root = document.documentElement;
                            if (isDark) {
                                root.classList.add('dark');
                                root.style.colorScheme = 'dark';
                            } else {
                                root.classList.remove('dark');
                                root.style.colorScheme = 'light';
                            }

                            // 2. Synchronous fast theme switcher function
                            window.__setAppTheme = function(target, persist) {
                                var root = document.documentElement;
                                var body = document.body;
                                var shell = document.getElementById('app-shell');
                                var isDark;
                                if (target === 'dark') {
                                    isDark = true;
                                } else if (target === 'light') {
                                    isDark = false;
                                } else if (target === 'toggle') {
                                    isDark = !root.classList.contains('dark');
                                } else {
                                    isDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches;
                                }

                                var currentlyDark = root.classList.contains('dark');
                                if (target !== 'toggle' && currentlyDark === isDark) {
                                    if (persist) {
                                        try { localStorage.setItem('bpsl-theme', target); } catch(e) {}
                                    }
                                    return;
                                }

                                root.classList.add('theme-transitioning');
                                if (isDark) {
                                    root.classList.add('dark');
                                    if (body) body.classList.add('dark');
                                    if (shell) shell.classList.add('dark');
                                    root.style.colorScheme = 'dark';
                                } else {
                                    root.classList.remove('dark');
                                    if (body) body.classList.remove('dark');
                                    if (shell) shell.classList.remove('dark');
                                    root.style.colorScheme = 'light';
                                }

                                if (persist) {
                                    var modeToStore = (target === 'toggle') ? (isDark ? 'dark' : 'light') : target;
                                    try {
                                        localStorage.setItem('bpsl-theme', modeToStore);
                                    } catch(e) {}
                                }

                                clearTimeout(window.__themeTransitionTimeout);
                                window.__themeTransitionTimeout = setTimeout(function() {
                                    root.classList.remove('theme-transitioning');
                                }, 100);
                            };

                            // 3. Capturing click listener for instantaneous zero-delay DOM updates
                            document.addEventListener('click', function(e) {
                                var toggleBtn = e.target && e.target.closest ? e.target.closest('[data-slot="theme-toggle-btn"]') : null;
                                if (toggleBtn) {
                                    window.__setAppTheme('toggle', true);
                                    return;
                                }
                                var segBtn = e.target && e.target.closest ? e.target.closest('[data-theme-set]') : null;
                                if (segBtn) {
                                    var mode = segBtn.getAttribute('data-theme-set');
                                    if (mode) window.__setAppTheme(mode, true);
                                    return;
                                }
                            }, true);

                            // Drag & drop handlers
                            document.addEventListener('dragstart', function(e) {
                                var target = e.target;
                                var el = target && target.closest ? target.closest('[draggable="true"]') : null;
                                if (!el) return;
                                var id = el.getAttribute('data-drag-id') || el.id || '';
                                if (e.dataTransfer) {
                                    try {
                                        e.dataTransfer.effectAllowed = 'all';
                                        if (id) {
                                            e.dataTransfer.setData('text/plain', id);
                                            e.dataTransfer.setData('application/x-bpsl-id', id);
                                        } else {
                                            e.dataTransfer.setData('text/plain', 'drag-item');
                                        }
                                    } catch(err) {}
                                }
                            }, false);
                            document.addEventListener('dragover', function(e) {
                                e.preventDefault();
                                if (e.dataTransfer) {
                                    try { e.dataTransfer.dropEffect = 'move'; } catch(err) {}
                                }
                            }, false);
                            document.addEventListener('dragenter', function(e) {
                                e.preventDefault();
                                if (e.dataTransfer) {
                                    try { e.dataTransfer.dropEffect = 'move'; } catch(err) {}
                                }
                            }, false);

                            // Global keyboard shortcuts
                            document.addEventListener('keydown', function(e) {
                                if ((e.ctrlKey || e.metaKey) && e.key === ',') {
                                    e.preventDefault();
                                    var btn = document.querySelector('[data-slot="open-settings-btn"]');
                                    if (btn) btn.click();
                                }
                                if ((e.ctrlKey || e.metaKey) && (e.key === 'd' || e.key === 'D') && !e.shiftKey && !e.altKey) {
                                    var activeEl = document.activeElement;
                                    if (activeEl && (activeEl.tagName === 'INPUT' || activeEl.tagName === 'TEXTAREA' || activeEl.isContentEditable)) return;
                                    e.preventDefault();
                                    if (window.__setAppTheme) window.__setAppTheme('toggle', true);
                                    var themeBtn = document.querySelector('[data-slot="theme-toggle-btn"]');
                                    if (themeBtn) themeBtn.click();
                                }
                            }, false);
                        })();
                        </script>"#.to_string(),
                    )
                    .with_window(
                        WindowBuilder::new()
                            .with_title("BPSL IT Support")
                            .with_decorations(false)
                            .with_theme(Some(initial_theme))
                            .with_inner_size(LogicalSize::new(1200.0, 800.0))
                            .with_min_inner_size(LogicalSize::new(940.0, 600.0)),
                    ),
            )
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(App);
    }
}

use components::ui::theme::{use_theme, ThemeProvider};

#[component]
fn App() -> Element {
    let auth_state = use_signal(|| {
        let user = crate::auth::current_active_user();
        let token = crate::db::sync_meta_get("auth_active_token");
        crate::auth::AuthState { user, token }
    });
    use_context_provider(|| auth_state);

    rsx! {
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Roboto:ital,wght@0,100..900;1,100..900&family=Roboto+Mono:ital,wght@0,100..700;1,100..700&display=swap",
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Stylesheet { href: TAILWIND_CSS }
        document::Script { src: DND_JS }
        document::Title { "BPSL IT Support" }
        ThemeProvider {
            Router::<Route> {}
        }
    }
}

/// Authentication shell: dedicated minimal layout for /login and /register
/// retaining the custom window titlebar for window dragging and theme control.
#[component]
fn AuthShell() -> Element {
    let theme = use_theme();
    let dark_cls = if theme.is_dark() { "dark" } else { "" };

    rsx! {
        ToastProvider {
            div {
                id: "app-shell",
                class: "{dark_cls} flex h-screen w-full flex-col overflow-hidden bg-background text-foreground",
                Titlebar { app_name: "BPSL IT Support".to_string() }
                main { class: "relative flex flex-1 flex-col overflow-y-auto min-h-0 w-full items-center justify-center p-4",
                    Outlet::<Route> {}
                }
            }
        }
    }
}

/// App shell: custom titlebar + sidebar + page outlet + FAB.
#[component]
fn AppShell() -> Element {
    let nav = use_navigator();
    let theme = use_theme();
    let dark_cls = if theme.is_dark() { "dark" } else { "" };
    let db_sync = use_signal(state::AppDbRevision::default);
    use_context_provider(|| db_sync);

    let app_settings = use_signal(state::AppSettings::load_from_db);
    use_context_provider(|| app_settings);

    let mut open_settings = use_signal(state::OpenSettingsDialog::default);
    use_context_provider(|| open_settings);

    let mut open_sheet_changes = use_signal(state::OpenSheetChangesDialog::default);
    use_context_provider(|| open_sheet_changes);

    let is_sidebar_right = app_settings.read().sidebar_position == "right";

    // Restore and enforce appearance options from SQLite on launch
    use_effect(move || {
        let s = app_settings.peek().clone();
        crate::state::apply_appearance_to_dom(
            &s.accent_color,
            &s.font_scale,
            &s.card_density,
            &s.corner_radius,
            &s.font_family,
            s.oled_dark,
            s.animations_enabled,
        );
        let landing = s.default_landing_page.clone();
        if landing != "tickets" {
            match landing.as_str() {
                "notes" => { let _ = nav.replace(Route::NotesPage {}); }
                "employees" => { let _ = nav.replace(Route::Employees {}); }
                "reminders" => { let _ = nav.replace(Route::Reminders {}); }
                "calendar" => { let _ = nav.replace(Route::CalendarPage {}); }
                "analytics" => { let _ = nav.replace(Route::Analytics {}); }
                _ => {}
            }
        }
    });

    rsx! {
        ToastProvider {
            SidebarProvider {
                div {
                    id: "app-shell",
                    class: "{dark_cls} flex h-screen w-full flex-col overflow-hidden bg-background text-foreground",
                    Titlebar { app_name: "BPSL IT Support".to_string() }
                    div { class: format!("flex flex-1 overflow-hidden {}", if is_sidebar_right { "flex-row-reverse" } else { "" }),
                        AppSidebar {}
                        main { class: "relative flex flex-1 flex-col overflow-hidden min-h-0 w-full",
                            Outlet::<Route> {}
                        }
                    }
                    Fab {}
                    if open_settings.read().0 {
                        components::settings_modal::SettingsModal {
                            on_close: move |_| open_settings.set(state::OpenSettingsDialog(false)),
                        }
                    }
                    if open_sheet_changes.read().0 {
                        components::sheet_changes_modal::SheetChangesModal {
                            on_close: move |_| open_sheet_changes.set(state::OpenSheetChangesDialog(false)),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NotFound(route: Vec<String>) -> Element {
    let path = format!("/{}", route.join("/"));
    rsx! {
        div { class: "flex flex-col items-center justify-center flex-1 min-h-[60vh] p-8 text-center my-auto",
            div { class: "flex size-16 items-center justify-center rounded-2xl bg-muted text-muted-foreground mb-4 shadow-xs",
                svg { class: "size-8", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                    path { d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
                    line { x1: "12", y1: "9", x2: "12", y2: "13" }
                    line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
                }
            }
            h2 { class: "text-2xl font-bold tracking-tight text-foreground", "Page Not Found" }
            p { class: "text-sm text-muted-foreground mt-1 mb-2 max-w-sm",
                "The requested path "
                code { class: "font-mono text-xs px-1.5 py-0.5 rounded bg-muted text-foreground", "{path}" }
                " does not exist."
            }
            p { class: "text-xs text-muted-foreground/80 mb-6", "Check the URL or return to the support ticket board." }
            Link {
                to: Route::Tickets {},
                class: "inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground shadow-xs hover:bg-primary/90 transition-colors cursor-pointer",
                svg { class: "size-4", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                    path { d: "m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" }
                    polyline { points: "9 22 9 12 15 12 15 22" }
                }
                "Back to Support Tickets"
            }
        }
    }
}
