#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;

pub mod api;
mod components;
pub mod db;
mod icons;
mod pages;
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
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");
const DND_JS: Asset = asset!("/assets/dnd.js");

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Command-line tool to wipe the active database and leave all tables empty
    if args.first().map(String::as_str) == Some("--reset-db") {
        if let Ok(conn) = db::get_connection() {
            let tables = ["USERS", "reminders", "tickets", "NOTES", "sync_queue", "sync_conflicts", "sync_meta"];
            for t in tables {
                let _ = conn.execute(&format!("DELETE FROM {}", t), []);
            }
            let _ = conn.execute_batch("VACUUM;");
            println!("[BPSL IT Support] Active database wiped cleanly (all tables 0 rows).");
            std::process::exit(0);
        }
    }

    // Command-line tool to initialize a clean, empty SQLite database with all tables
    if args.first().map(String::as_str) == Some("--init-clean-db") {
        let target = args.get(1).cloned().unwrap_or_else(|| "management.db".to_string());
        std::env::set_var("TMS_DB_PATH", &target);
        let _ = db::init_database();
        if let Ok(conn) = db::get_connection() {
            let tables = ["USERS", "reminders", "tickets", "NOTES", "sync_queue", "sync_conflicts", "sync_meta"];
            for t in tables {
                let _ = conn.execute(&format!("DELETE FROM {}", t), []);
            }
            let _ = conn.execute_batch("VACUUM;");
        }
        match db::init_database() {
            Ok(_) => {
                println!("[BPSL IT Support] Successfully initialized clean empty database at '{}'", target);
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
    if let Err(e) = db::init_database() {
        eprintln!(
            "[BPSL IT Support DB] Warning: database init failed: {:?}",
            e
        );
    }

    // Operational aid: run a single sync cycle against the configured API
    // and print the resulting status (no UI).
    if args.first().map(String::as_str) == Some("--sync-once") {
        sync::run_cycle_blocking();
        let s = sync::snapshot();
        println!(
            "[sync] state={:?} pending={} conflicts={} detail={}",
            s.state, s.pending_ops, s.conflicts, s.detail
        );
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
        use components::ui::theme::fetch_system_is_dark;
        use dioxus::desktop::tao::window::Theme;
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        let is_sys_dark = fetch_system_is_dark();
        let initial_theme = if is_sys_dark {
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
    rsx! {
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Link { rel: "preconnect", href: "https://fonts.gstatic.com" }
        document::Link {
            rel: "stylesheet",
            href: "https://fonts.googleapis.com/css2?family=Roboto:ital,wght@0,100..900;1,100..900&family=Roboto+Mono:ital,wght@0,100..700;1,100..700&display=swap",
        }
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Script { src: DND_JS }
        document::Title { "BPSL IT Support" }
        ThemeProvider {
            Router::<Route> {}
        }
    }
}

/// App shell: custom titlebar + sidebar + page outlet + FAB.
#[component]
fn AppShell() -> Element {
    let theme = use_theme();
    let dark_cls = if theme.is_dark() { "dark" } else { "" };
    let db_sync = use_signal(state::AppDbRevision::default);
    use_context_provider(|| db_sync);

    let app_settings = use_signal(state::AppSettings::load_from_db);
    use_context_provider(|| app_settings);

    let mut open_settings = use_signal(state::OpenSettingsDialog::default);
    use_context_provider(|| open_settings);

    let is_sidebar_right = app_settings.read().sidebar_position == "right";

    rsx! {
        ToastProvider {
            SidebarProvider {
                div { class: "{dark_cls} flex h-screen w-full flex-col overflow-hidden bg-background text-foreground transition-colors duration-200",
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
                }
            }
        }
    }
}
