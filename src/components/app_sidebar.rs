//! App sidebar: primary navigation (Tickets, Reminders, Calendar).
//!
//! Built entirely from the shadcn-style kit in `src/components/ui/sidebar`.

use dioxus::prelude::*;

use crate::components::ui::sidebar::{
    Sidebar, SidebarCollapsible, SidebarContent, SidebarFooter, SidebarGroup, SidebarGroupContent,
    SidebarGroupLabel, SidebarHeader, SidebarMenu, SidebarMenuBadge, SidebarMenuButton,
    SidebarMenuItem, SidebarRail, SidebarTrigger,
};
use crate::Route;

// ============================================================
// Sidebar root
// ============================================================

#[component]
pub fn AppSidebar() -> Element {
    let toast = try_use_context::<crate::components::ui::toast::ToastState>();
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();
    // Read badge counts memoized so routing transitions don't re-query SQLite
    let sidebar_data = use_memo(move || {
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        (
            crate::db::note_count(),
            crate::db::ticket_count(),
            crate::db::employee_count(),
            crate::db::reminder_count(),
            crate::db::get_active_sheet_source_id(),
            crate::db::get_all_sheet_sources(),
        )
    });
    let (
        notes_count,
        tickets_count,
        employees_count,
        reminders_count,
        active_sheet_id,
        sheet_sources,
    ) = sidebar_data();

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
                div { class: "flex flex-row items-center gap-2.5 w-full px-1.5 py-1",
                    div { class: "sidebar-text flex flex-col gap-1 w-full",
                        div { class: "relative w-full",
                            select {
                                class: "w-full h-8 pl-2.5 pr-7 text-xs font-medium rounded-md border border-sidebar-border/80 bg-sidebar-accent/50 text-sidebar-foreground cursor-pointer focus:outline-none focus:ring-1 focus:ring-primary truncate appearance-none hover:bg-sidebar-accent/80 transition-colors shadow-2xs",
                                value: "{active_sheet_id}",
                                onchange: {
                                    let sheet_sources_c = sheet_sources.clone();
                                    let active_id_c = active_sheet_id.clone();
                                    move |e: FormEvent| {
                                        let chosen = e.value();
                                        if chosen == "__add_new__" {
                                            if let Some(mut open_settings) = try_use_context::<Signal<crate::state::OpenSettingsDialog>>() {
                                                open_settings.set(crate::state::OpenSettingsDialog(true));
                                            }
                                            return;
                                        }
                                        if chosen == active_id_c {
                                            return;
                                        }
                                        let target_name = sheet_sources_c
                                            .iter()
                                            .find(|s| s.id == chosen)
                                            .map(|s| s.name.clone())
                                            .unwrap_or_else(|| chosen.clone());
                                        match crate::sync::switch_sheet_profile_and_sync(&chosen) {
                                            Ok(_) => {
                                                if let Some(mut r) = db_sync {
                                                    r.with_mut(|rev| rev.0 += 1);
                                                }
                                                if let Some(t) = toast {
                                                    t.success("Switched Sheet", format!("Now syncing from {target_name}."));
                                                }
                                            }
                                            Err(e) => {
                                                if let Some(t) = toast {
                                                    t.error("Switch Failed", e);
                                                }
                                            }
                                        }
                                    }
                                },
                                for src in sheet_sources.iter() {
                                    option {
                                        key: "{src.id}",
                                        value: "{src.id}",
                                        selected: src.id == active_sheet_id,
                                        "{src.name}"
                                    }
                                }
                                option {
                                    value: "__add_new__",
                                    "+ Add Google Sheet..."
                                }
                            }
                            div { class: "pointer-events-none absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground",
                                svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                    path { d: "M6 9l6 6 6-6" }
                                }
                            }
                        }
                    }
                    SidebarTrigger { class: "size-7 border border-sidebar-border/50 hover:bg-sidebar-accent text-muted-foreground hover:text-foreground" }
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
                            NavItem {
                                route: Route::Analytics {},
                                label: "Analytics",
                                icon: rsx! {
                                    crate::icons::BarChart3Icon { class: "size-4" }
                                },
                            }
                        }
                    }
                }

            }

            SidebarFooter {
                SidebarMenu {
                    SidebarMenuItem {
                        SidebarMenuButton {
                            tooltip: Some("Preferences & Customization (Ctrl+,)".to_string()),
                            onclick: move |_| {
                                if let Some(mut open_settings) = try_use_context::<Signal<crate::state::OpenSettingsDialog>>() {
                                    open_settings.set(crate::state::OpenSettingsDialog(true));
                                }
                            },
                            crate::icons::SettingsIcon { class: "size-4 text-muted-foreground" }
                            span { class: "sidebar-text", "Preferences" }
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
