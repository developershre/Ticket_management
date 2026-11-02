//! App sidebar: primary navigation (Tickets, Reminders, Calendar).
//!
//! Built entirely from the shadcn-style kit in `src/components/ui/sidebar`.

use dioxus::prelude::*;

use crate::components::ui::avatar::{Avatar, AvatarFallback};
use crate::components::ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent,
    SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuBadge, SidebarMenuButton,
    SidebarMenuItem, SidebarRail,
};
use crate::Route;

// ============================================================
// Sidebar root
// ============================================================

#[component]
pub fn AppSidebar() -> Element {
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();
    // Read every badge count in one pass; touching `db_sync` registers
    // this component for reloads when the sync worker bumps the revision.
    let (notes_count, tickets_count, employees_count, reminders_count) = {
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        (
            crate::db::get_all_notes().len(),
            crate::db::get_all_tickets().len(),
            crate::db::employee_count(),
            crate::db::get_all_reminders().len(),
        )
    };

    let count_badge = |n: usize| {
        if n > 0 {
            Some(n.to_string())
        } else {
            None
        }
    };

    rsx! {
        Sidebar {
            collapsible: SidebarCollapsible::Icon,

            SidebarHeader {
                div { class: "flex min-w-0 items-center justify-between gap-2 rounded-lg p-1 transition-colors hover:bg-sidebar-accent/50 cursor-pointer",
                    div { class: "flex min-w-0 items-center gap-2.5",
                        div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary text-primary-foreground",
                            svg {
                                class: "size-4",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2.5",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M20 6 9 17l-5-5" }
                            }
                        }
                        div { class: "sidebar-text flex min-w-0 flex-col",
                            span { class: "truncate text-xs font-bold leading-none text-sidebar-foreground",
                                "TaskFlow"
                            }
                            span { class: "mt-0.5 truncate text-[10px] text-sidebar-foreground/60",
                                "Workspace"
                            }
                        }
                    }
                }
            }

            SidebarContent {
                SidebarGroup {
                    SidebarGroupLabel { "Overview" }
                    SidebarGroupContent {
                        SidebarMenu {
                            NavItem {
                                route: Route::Tickets {},
                                label: "IT Tickets",
                                badge: count_badge(tickets_count),
                                icon: rsx! {
                                    svg {
                                        class: "size-4",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" }
                                        path { d: "M13 5v2" }
                                        path { d: "M13 11v2" }
                                        path { d: "M13 17v2" }
                                    }
                                },
                            }
                            NavItem {
                                route: Route::NotesPage {},
                                label: "Notes",
                                badge: if notes_count > 0 { Some(notes_count.to_string()) } else { None },
                                icon: rsx! {
                                    crate::icons::StickyNoteIcon { class: "size-4" }
                                },
                            }
                            NavItem {
                                route: Route::Employees {},
                                label: "Employees",
                                badge: count_badge(employees_count),
                                icon: rsx! {
                                    crate::icons::UsersIcon { class: "size-4" }
                                },
                            }
                            NavItem {
                                route: Route::Reminders {},
                                label: "Reminders",
                                badge: count_badge(reminders_count),
                                icon: rsx! {
                                    svg {
                                        class: "size-4",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                                        path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
                                    }
                                },
                            }
                            NavItem {
                                route: Route::CalendarPage {},
                                label: "Calendar",
                                icon: rsx! {
                                    svg {
                                        class: "size-4",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        rect { width: "18", height: "18", x: "3", y: "4", rx: "2", ry: "2" }
                                        path { d: "M16 2v4" }
                                        path { d: "M8 2v4" }
                                        path { d: "M3 10h18" }
                                    }
                                },
                            }
                        }
                    }
                }

            }

            SidebarFooter {
                div { class: "flex min-w-0 flex-col p-1",
                    div { class: "flex min-w-0 items-center justify-between gap-2 rounded-lg p-1.5 transition-colors hover:bg-sidebar-accent/50 cursor-pointer",
                        div { class: "flex min-w-0 items-center gap-2.5",
                            div { class: "relative",
                                Avatar {
                                    class: "size-7.5 ring-1 ring-border/60",
                                    AvatarFallback { class: "bg-primary text-xs font-bold text-primary-foreground", "GR" }
                                }
                                span { class: "absolute -bottom-0.5 -right-0.5 size-2 rounded-full bg-primary ring-2 ring-sidebar" }
                            }
                            div { class: "sidebar-text flex min-w-0 flex-col",
                                span { class: "truncate text-xs font-semibold text-sidebar-foreground", "GamerRock" }
                                span { class: "truncate text-[10px] text-sidebar-foreground/60", "developer@taskflow.dev" }
                            }
                        }
                    }
                }
            }

            SidebarRail {}
        }
    }
}

// ============================================================
// Nav item — highlights the active route, pushes on click
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct NavItemProps {
    route: Route,
    label: String,
    icon: Element,
    #[props(default)]
    badge: Option<String>,
}

#[component]
fn NavItem(props: NavItemProps) -> Element {
    let current = use_route::<Route>();
    let navigator = use_navigator();

    let active = current == props.route;
    let route = props.route.clone();
    let label = props.label.clone();
    let tooltip = props.label.clone();
    let badge = props.badge.clone();
    let icon = props.icon;

    rsx! {
        SidebarMenuItem {
            SidebarMenuButton {
                active,
                tooltip: Some(tooltip),
                onclick: move |_| {
                    navigator.push(route.clone());
                },
                {icon}
                span { class: "sidebar-text", "{label}" }
                if let Some(count) = badge.clone() {
                    SidebarMenuBadge { "{count}" }
                }
            }
        }
    }
}
