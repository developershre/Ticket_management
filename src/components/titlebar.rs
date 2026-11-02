//! Fully custom window chrome.
//!
//! The native OS window frame (navbar + minimize/maximize/close buttons) is
//! disabled at launch — see `main()` in `main.rs`
//! (`WindowBuilder::with_decorations(false)`). This component provides the
//! replacement: a branded, draggable titlebar with its own window controls.

use dioxus::prelude::*;

use crate::components::ui::sidebar::SidebarTrigger;
use crate::components::ui::theme::ThemeToggle;
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
    #[props(default = "TaskFlow".to_string())]
    pub app_name: String,
}

#[component]
pub fn Titlebar(props: TitlebarProps) -> Element {
    let route = use_route::<Route>();
    let page_title = match route {
        Route::Tickets {} => "Tickets",
        Route::Reminders {} => "Reminders",
        Route::CalendarPage {} => "Calendar",
    };

    // Shared with WindowControls so the maximize/restore icon stays in sync
    // when the drag region is double-clicked.
    let mut maximized = use_signal(|| false);

    rsx! {
        header {
            "data-slot": "titlebar",
            class: "flex h-10 shrink-0 select-none items-center gap-1 border-b border-sidebar-border/60 bg-sidebar",

            // Sidebar collapse/expand toggle
            SidebarTrigger {}

            // Branding + current page
            div { class: "flex min-w-0 items-center gap-2.5 pl-1.5 pr-2",
                // Logo mark
                div { class: "flex size-6 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground",
                    svg {
                        class: "size-3.5",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2.5",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M20 6 9 17l-5-5" }
                    }
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
                // Sleek quick search visual (Linear / Raycast aesthetic)
                div {
                    class: "hidden items-center gap-2 rounded-lg border border-sidebar-border/70 bg-background/50 px-3 py-1 text-xs text-muted-foreground transition-colors hover:border-primary/30 md:flex",
                    onmousedown: move |e| e.stop_propagation(),
                    svg {
                        class: "size-3.5 text-muted-foreground/70",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        circle { cx: "11", cy: "11", r: "8" }
                        path { d: "m21 21-4.3-4.3" }
                    }
                    span { "Search tasks, reminders or events..." }
                    kbd { class: "ml-4 rounded border border-border/80 bg-muted px-1.5 py-0.5 text-[10px] font-semibold text-muted-foreground",
                        "⌘K"
                    }
                }
            }

            // Theme toggle button
            div { class: "flex items-center px-1.5",
                ThemeToggle { class: "size-7 border-none bg-transparent hover:bg-sidebar-accent text-sidebar-foreground" }
            }

            // Fully custom window controls (no-ops on web)
            WindowControls { maximized }
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
