//! Fully custom window chrome.
//!
//! The native OS window frame (navbar + minimize/maximize/close buttons) is
//! disabled at launch — see `main()` in `main.rs`
//! (`WindowBuilder::with_decorations(false)`). This component provides the
//! replacement: a branded, draggable titlebar with its own window controls.

use dioxus::prelude::*;
use futures_util::StreamExt;

use crate::components::ui::sidebar::SidebarTrigger;
use crate::components::ui::theme::ThemeToggle;
use crate::icons::AppLogo;
use crate::Route;

// ============================================================
// Native-window helpers.
//
// `dioxus::desktop::window()` grabs the window handle without a hook, so it
// is safe inside event handlers. On web builds these are silent no-ops so the
// exact same titlebar renders on every platform.
// ============================================================

#[cfg(feature = "desktop")]
fn drag_window() {
    dioxus::desktop::window().drag();
}

#[cfg(not(feature = "desktop"))]
fn drag_window() {}

#[cfg(feature = "desktop")]
fn toggle_maximize() {
    dioxus::desktop::window().toggle_maximized();
}

#[cfg(not(feature = "desktop"))]
fn toggle_maximize() {}

// ============================================================
// Titlebar
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TitlebarProps {
    #[props(default = "BPSL IT Support".to_string())]
    pub app_name: String,
}

#[component]
pub fn Titlebar(props: TitlebarProps) -> Element {
    let route = use_route::<Route>();
    let page_title = match route {
        Route::Tickets {} => "Tickets",
        Route::NotesPage {} => "Notes",
        Route::Employees {} => "Employees",
        Route::Reminders {} => "Reminders",
        Route::CalendarPage {} => "Calendar",
    };

    // Shared with WindowControls so the maximize/restore icon stays in sync
    // when the drag region is double-clicked.
    let mut maximized = use_signal(|| false);

    rsx! {
        header {
            "data-slot": "titlebar",
            class: "flex py-4 shrink-0 select-none items-center gap-1 border-b border-sidebar-border/60 bg-sidebar",
            // Branding + current page
            div { class: "flex min-w-0 items-center gap-2 pl-2 pr-2",
                SidebarTrigger { class: "size-7 border border-sidebar-border/60 bg-sidebar hover:bg-sidebar-accent text-sidebar-foreground" }
                // Logo mark
                div { class: "flex size-6 shrink-0 items-center justify-center rounded-lg bg-white p-0.5 border border-sidebar-border/60 shadow-xs",
                    AppLogo { class: "size-4" }
                }
                span { class: "truncate text-sm font-bold tracking-tight text-sidebar-foreground",
                    "{props.app_name}"
                }
                span { class: "hidden items-center rounded-md border border-sidebar-border/80 bg-sidebar-accent/40 px-2 py-0.5 text-[11px] font-medium text-sidebar-foreground/70 sm:inline-flex",
                    "{page_title}"
                }
            }

            // ============================================================
            // Custom drag region — drag to move the window,
            // double-click to maximize/restore.
            // ============================================================
            div {
                class: "flex h-full flex-1 items-center justify-center px-4 cursor-default",
                onmousedown: move |_| drag_window(),
                ondoubleclick: move |_| {
                    toggle_maximize();
                    let next = !*maximized.read();
                    maximized.set(next);
                },
            }

            // Theme toggle button
            div { class: "flex items-center px-1.5",
                ThemeToggle { class: "size-7 border-none bg-transparent hover:bg-sidebar-accent text-sidebar-foreground" }
            }

            // Settings button (API config)
            div { class: "flex items-center px-0.5",
                button {
                    r#type: "button",
                    title: "API & System Settings",
                    class: "size-7 inline-flex items-center justify-center rounded-md border-none bg-transparent hover:bg-sidebar-accent text-sidebar-foreground/80 hover:text-sidebar-foreground transition-colors cursor-pointer",
                    onclick: move |_| {
                        if let Some(mut open_settings) = try_use_context::<Signal<crate::state::OpenSettingsDialog>>() {
                            open_settings.set(crate::state::OpenSettingsDialog(true));
                        }
                    },
                    crate::icons::SettingsIcon { class: "size-4" }
                }
            }

            // Background sync status (Google Sheets ↔ SQLite)
            SyncPill {}

            // Fully custom window controls (no-ops on web)
            WindowControls { maximized }
        }
    }
}

// ============================================================
// Sync status pill
// ============================================================

/// Compact Google Sheets sync indicator. Subscribes to the sync worker's
/// status channel — the worker pushes an update only when something
/// actually changed (no timers, no polling). When a pull cycle changed
/// local ticket data, `AppDbRevision` is bumped so pages reload from
/// SQLite.
#[component]
fn SyncPill() -> Element {
    let mut status = use_signal(crate::sync::snapshot);
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();

    use_effect(move || {
        let mut rx = crate::sync::subscribe_status();
        spawn(async move {
            let mut last_revision = crate::sync::snapshot().revision;
            while let Some(next) = rx.next().await {
                if next.revision != last_revision {
                    last_revision = next.revision;
                    if let Some(mut sync) = db_sync {
                        sync.with_mut(|r| r.0 += 1);
                    }
                }
                status.set(next);
            }
        });
    });

    let snap = status.read();
    let (label, dot_class, tooltip) = match snap.state {
        crate::sync::SyncState::Disabled => (
            "Sync off".to_string(),
            "bg-zinc-400",
            format!("{} · sync resumes once the API is configured", snap.detail),
        ),
        crate::sync::SyncState::Syncing => (
            "Syncing…".to_string(),
            "bg-sky-400 animate-pulse",
            snap.detail.clone(),
        ),
        crate::sync::SyncState::Error => (
            "Sync error".to_string(),
            "bg-red-500",
            format!(
                "{} · {} pending · {} unresolved conflict(s)",
                snap.detail, snap.pending_ops, snap.conflicts
            ),
        ),
        crate::sync::SyncState::Idle => {
            if snap.pending_ops > 0 {
                (
                    format!("{} pending", snap.pending_ops),
                    "bg-amber-400",
                    snap.detail.clone(),
                )
            } else {
                (
                    "Synced".to_string(),
                    "bg-emerald-500",
                    format!(
                        "{}{}",
                        snap.detail,
                        if snap.conflicts > 0 {
                            format!(" · {} unresolved conflict(s)", snap.conflicts)
                        } else {
                            String::new()
                        }
                    ),
                )
            }
        }
    };

    let open_settings = try_use_context::<Signal<crate::state::OpenSettingsDialog>>();

    rsx! {
        button {
            r#type: "button",
            class: "hidden items-center gap-1.5 rounded-md border border-sidebar-border/80 bg-sidebar-accent/40 px-2 py-0.5 text-[11px] font-medium text-sidebar-foreground/70 hover:text-sidebar-foreground hover:bg-sidebar-accent/70 transition-colors cursor-pointer sm:inline-flex",
            title: format!("{tooltip} · Click to configure API"),
            onclick: move |_| {
                if let Some(mut sig) = open_settings {
                    sig.set(crate::state::OpenSettingsDialog(true));
                }
            },
            span { class: "size-1.5 shrink-0 rounded-full {dot_class}" }
            span { "{label}" }
        }
    }
}

// ============================================================
// Window controls: minimize / maximize-restore / close
// ============================================================

#[cfg(feature = "desktop")]
#[component]
fn WindowControls(maximized: Signal<bool>) -> Element {
    let btn = "flex h-full w-11 items-center justify-center text-sidebar-foreground/70 \
               transition-colors cursor-pointer outline-none";

    rsx! {
        div { class: "flex h-full items-stretch border-l border-sidebar-border/60",

            // Minimize
            button {
                r#type: "button",
                title: "Minimize",
                class: "{btn} hover:bg-sidebar-accent hover:text-sidebar-foreground",
                onmousedown: move |e| e.stop_propagation(),
                onclick: move |_| dioxus::desktop::window().set_minimized(true),
                svg {
                    class: "size-3.5",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    path { d: "M5 12h14" }
                }
            }

            // Maximize / restore
            button {
                r#type: "button",
                title: if maximized() { "Restore" } else { "Maximize" },
                class: "{btn} hover:bg-sidebar-accent hover:text-sidebar-foreground",
                onmousedown: move |e| e.stop_propagation(),
                onclick: move |_| {
                    dioxus::desktop::window().toggle_maximized();
                    let next = !*maximized.read();
                    maximized.set(next);
                },
                if maximized() {
                    svg {
                        class: "size-3.5",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M8 3v3a2 2 0 0 1-2 2H3" }
                        path { d: "M21 8h-3a2 2 0 0 1-2-2V3" }
                        path { d: "M3 16h3a2 2 0 0 1 2 2v3" }
                        path { d: "M16 21v-3a2 2 0 0 1 2-2h3" }
                    }
                } else {
                    svg {
                        class: "size-3.5",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        rect { width: "18", height: "18", x: "3", y: "3", rx: "2" }
                    }
                }
            }

            // Close
            button {
                r#type: "button",
                title: "Close",
                class: "{btn} hover:bg-destructive hover:text-destructive-foreground",
                onmousedown: move |e| e.stop_propagation(),
                onclick: move |_| dioxus::desktop::window().close(),
                svg {
                    class: "size-3.5",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    path { d: "M18 6 6 18" }
                    path { d: "m6 6 12 12" }
                }
            }
        }
    }
}

/// Web builds have no native window to control — render nothing.
#[cfg(not(feature = "desktop"))]
#[component]
fn WindowControls(maximized: Signal<bool>) -> Element {
    let _ = maximized;
    rsx! {}
}
