//! Calendar Page — Full-page event calendar workstation with Context Menus.
//! Clean, architectural aesthetic with reduced roundness, full-width layouts,
//! rich event details, switchable views, and full right-click context menu actions.
#![allow(clippy::redundant_closure)]

use dioxus::prelude::*;

use crate::components::month_calendar::{
    day_of_week, days_in_month, first_day_of_month, format_date_pretty, is_event_on, CalEvent,
    EventColor, MONTH_NAMES, SHORT_MONTH_NAMES,
};
use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
use crate::components::ui::separator::Separator;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    CalendarIcon, ChevronLeftIcon, ChevronRightIcon, ClockIcon, ExternalLinkIcon, MapPinIcon,
    PlusIcon, SearchIcon, TrashIcon, UsersIcon, VideoIcon, XIcon,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarView {
    Month,
    Week,
    Day,
    Schedule,
}

impl CalendarView {
    pub fn label(&self) -> &'static str {
        match self {
            CalendarView::Month => "Month",
            CalendarView::Week => "Week",
            CalendarView::Day => "Day",
            CalendarView::Schedule => "Schedule",
        }
    }
}

fn priority_badge_variant(priority: &str) -> &'static str {
    match priority {
        "Urgent" => "destructive",
        "High" => "warning",
        "Medium" => "info",
        _ => "outline",
    }
}

fn seed_events(today: (u32, u32, u32)) -> Vec<CalEvent> {
    let (y, m, d) = today;
    let max_days = days_in_month(y, m);

    let d_sub2 = d.saturating_sub(2).max(1);
    let d_sub1 = d.saturating_sub(1).max(1);
    let d_add1 = (d + 1).min(max_days);
    let d_add2 = (d + 2).min(max_days);
    let d_add3 = (d + 3).min(max_days);
    let d_add5 = (d + 5).min(max_days);
    let d_add7 = (d + 7).min(max_days);

    vec![
        CalEvent::new(
            1,
            "Q4 Sprint Planning & Retrospective",
            (y, m, d),
            Some("09:30 AM"),
            Some("10:30 AM"),
            EventColor::Blue,
            "Engineering",
            "Sprint 42 retrospective, velocity review, and sprint 43 backlog refinement with engineering leads.",
            Some("Google Meet · meet.google.com/taskflow-sprint"),
            &["David Chen", "Sarah Miller", "Gamer Rock"],
            "High",
        ),
        CalEvent::new(
            2,
            "Security Architecture & TLS Audit",
            (y, m, d),
            Some("02:00 PM"),
            Some("03:15 PM"),
            EventColor::Purple,
            "DevOps",
            "Audit of TLS 1.3 cipher suites and certbot automated renewal pipeline for wildcard *.staging.corp.internal.",
            Some("Room 402 (DevOps War Room)"),
            &["Elena Rostova", "Robert Taylor"],
            "Urgent",
        ),
        CalEvent::new(
            3,
            "Design System 2.0 Token Review",
            (y, m, d),
            Some("04:30 PM"),
            Some("05:15 PM"),
            EventColor::Purple,
            "Design",
            "Reviewing token consistency, dark-mode contrast ratios, and accessible calendar widget components.",
            Some("Google Meet · meet.google.com/ux-sync"),
            &["Lisa Wong", "Sarah Miller"],
            "Medium",
        ),
        CalEvent {
            id: 4,
            title: "Production Release Window & QA Freeze".to_string(),
            date: (y, m, d),
            end_date: Some((y, m, d_add2)),
            time: Some("All day".to_string()),
            end_time: None,
            color: EventColor::Red,
            category: "Release & QA".to_string(),
            description: "Code freeze for v2.4 core engine deployment; only P0 bugfix commits allowed to release branch."
                .to_string(),
            location: Some("CI/CD Deployment Pipeline".to_string()),
            attendees: vec![
                "Elena Rostova".to_string(),
                "Gamer Rock".to_string(),
                "David Chen".to_string(),
            ],
            priority: "Urgent".to_string(),
        },
        CalEvent::new(
            5,
            "Client Onboarding & Platform Demo",
            (y, m, d_add1),
            Some("11:00 AM"),
            Some("12:00 PM"),
            EventColor::Green,
            "Client Demo",
            "Walkthrough of TaskFlow IT management dashboard, ticketing workflow, and SLA escalation policies.",
            Some("Zoom · meeting ID: 849 2011 4099"),
            &["Mark Vance", "David Chen"],
            "High",
        ),
        CalEvent::new(
            6,
            "MacBook Pro M3 Fleet Provisioning",
            (y, m, d_add2),
            Some("01:30 PM"),
            Some("03:00 PM"),
            EventColor::Amber,
            "IT Operations",
            "Staging standard development environment and MDM profile enrollment for incoming engineering cohort.",
            Some("IT Lab B, Workstation 4"),
            &["Sarah Miller", "James Smith"],
            "Medium",
        ),
        CalEvent::new(
            7,
            "Database Failover & Replication Drill",
            (y, m, d_sub2),
            Some("10:00 AM"),
            Some("11:30 AM"),
            EventColor::Blue,
            "DevOps",
            "Simulating primary node failure in WAL replication cluster to verify zero-downtime leader election.",
            Some("Server Room 2 / Slack #incident-response"),
            &["Elena Rostova", "Robert Taylor"],
            "High",
        ),
        CalEvent::new(
            8,
            "Active Directory Audit & Cleanup",
            (y, m, d_sub1),
            Some("03:30 PM"),
            Some("04:30 PM"),
            EventColor::Amber,
            "IT Operations",
            "Archiving inactive contractor credentials and verifying group policy permissions for finance dept.",
            Some("IT Operations Desk"),
            &["Robert Taylor", "James Smith"],
            "Medium",
        ),
        CalEvent::new(
            9,
            "Weekly IT Operations All-Hands",
            (y, m, d_add3),
            Some("09:00 AM"),
            Some("09:45 AM"),
            EventColor::Green,
            "General",
            "Company-wide review of open tickets, resolution SLA metrics, and upcoming infrastructure maintenance.",
            Some("Main Auditorium & Live Stream"),
            &["David Chen", "Sarah Miller", "James Smith", "Elena Rostova"],
            "Medium",
        ),
        CalEvent::new(
            10,
            "Vendor SOC2 Compliance Walkthrough",
            (y, m, d_add5),
            Some("02:30 PM"),
            Some("04:00 PM"),
            EventColor::Amber,
            "Compliance",
            "Annual external auditor review for SOC2 Type II trust principles and data retention controls.",
            Some("Executive Boardroom A"),
            &["Robert Taylor", "Gamer Rock"],
            "High",
        ),
        CalEvent::new(
            11,
            "Floor 3 Network Switch Upgrade",
            (y, m, d_add7),
            Some("06:00 PM"),
            Some("08:00 PM"),
            EventColor::Blue,
            "DevOps",
            "Firmware update and redundant fiber uplink cutover for building 3 distribution switch.",
            Some("IDF Closet 3-East"),
            &["James Smith", "Elena Rostova"],
            "Medium",
        ),
    ]
}

#[component]
pub fn CalendarPage() -> Element {
    let today = crate::state::get_real_current_date();
    let toast = use_context::<ToastState>();

    // Main calendar view state
    let mut selected_date = use_signal(move || Some(today));
    let mut view_year = use_signal(move || today.0);
    let mut view_month = use_signal(move || today.1);
    let mut active_view = use_signal(|| CalendarView::Month);
    let mut category_filter = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);

    // Events reactive store
    let mut events = use_signal(move || seed_events(today));

    // Modals
    let mut detail_event = use_signal(|| None::<CalEvent>);
    let mut is_new_event_open = use_signal(|| false);

    // Form fields for New Event modal
    let mut form_title = use_signal(String::new);
    let mut form_category = use_signal(|| "Engineering".to_string());
    let mut form_time = use_signal(|| "10:00 AM".to_string());
    let mut form_end_time = use_signal(|| "11:00 AM".to_string());
    let mut form_location = use_signal(String::new);
    let mut form_priority = use_signal(|| "Medium".to_string());
    let mut form_attendees = use_signal(|| "David Chen, Sarah Miller".to_string());
    let mut form_description = use_signal(String::new);
    let mut form_day = use_signal(move || today.2);

    let cur_y = view_year();
    let cur_m = view_month();
    let active_selected = (*selected_date.read()).unwrap_or(today);

    // Context menu event mutations
    let on_change_priority = move |(ev_id, new_prio): (u32, String)| {
        if let Some(ev) = events.write().iter_mut().find(|e| e.id == ev_id) {
            ev.priority = new_prio.clone();
            toast.info("Priority Updated", format!("Event priority set to {new_prio}"));
        }
    };

    let on_change_category = move |(ev_id, new_cat): (u32, String)| {
        if let Some(ev) = events.write().iter_mut().find(|e| e.id == ev_id) {
            ev.color = match new_cat.as_str() {
                "Engineering" | "DevOps" => EventColor::Blue,
                "Design" => EventColor::Purple,
                "Client Demo" => EventColor::Green,
                "Release & QA" => EventColor::Red,
                _ => EventColor::Amber,
            };
            ev.category = new_cat.clone();
            toast.info("Category Updated", format!("Event category set to {new_cat}"));
        }
    };

    let on_delete_event = move |ev_id: u32| {
        events.write().retain(|e| e.id != ev_id);
        detail_event.set(None);
        toast.error("Event Deleted", "The event has been removed from your calendar.");
    };

    // Filter events based on active category & search keyword
    let q = search_query.read().trim().to_lowercase();
    let cat = category_filter.read().clone();

    let filtered_events: Vec<CalEvent> = events
        .read()
        .iter()
        .filter(|e| {
            if cat != "All" && e.category != cat {
                return false;
            }
            if !q.is_empty() {
                let match_title = e.title.to_lowercase().contains(&q);
                let match_desc = e.description.to_lowercase().contains(&q);
                let match_loc = e
                    .location
                    .as_ref()
                    .map(|l| l.to_lowercase().contains(&q))
                    .unwrap_or(false);
                let match_attendee = e.attendees.iter().any(|a| a.to_lowercase().contains(&q));
                if !match_title && !match_desc && !match_loc && !match_attendee {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    // Navigation callbacks
    let on_prev = move |_| {
        match active_view() {
            CalendarView::Month | CalendarView::Schedule => {
                let m = view_month();
                let y = view_year();
                if m == 1 {
                    view_month.set(12);
                    view_year.set(y - 1);
                } else {
                    view_month.set(m - 1);
                }
            }
            CalendarView::Week => {
                let (y, m, d) = active_selected;
                let new_d = if d > 7 {
                    (y, m, d - 7)
                } else {
                    let pm = if m == 1 { 12 } else { m - 1 };
                    let py = if m == 1 { y - 1 } else { y };
                    let pmax = days_in_month(py, pm);
                    (py, pm, pmax - (7 - d))
                };
                selected_date.set(Some(new_d));
                view_year.set(new_d.0);
                view_month.set(new_d.1);
            }
            CalendarView::Day => {
                let (y, m, d) = active_selected;
                let new_d = if d > 1 {
                    (y, m, d - 1)
                } else {
                    let pm = if m == 1 { 12 } else { m - 1 };
                    let py = if m == 1 { y - 1 } else { y };
                    let pmax = days_in_month(py, pm);
                    (py, pm, pmax)
                };
                selected_date.set(Some(new_d));
                view_year.set(new_d.0);
                view_month.set(new_d.1);
            }
        }
    };

    let on_next = move |_| {
        match active_view() {
            CalendarView::Month | CalendarView::Schedule => {
                let m = view_month();
                let y = view_year();
                if m == 12 {
                    view_month.set(1);
                    view_year.set(y + 1);
                } else {
                    view_month.set(m + 1);
                }
            }
            CalendarView::Week => {
                let (y, m, d) = active_selected;
                let max_d = days_in_month(y, m);
                let new_d = if d + 7 <= max_d {
                    (y, m, d + 7)
                } else {
                    let nm = if m == 12 { 1 } else { m + 1 };
                    let ny = if m == 12 { y + 1 } else { y };
                    (ny, nm, (d + 7) - max_d)
                };
                selected_date.set(Some(new_d));
                view_year.set(new_d.0);
                view_month.set(new_d.1);
            }
            CalendarView::Day => {
                let (y, m, d) = active_selected;
                let max_d = days_in_month(y, m);
                let new_d = if d < max_d {
                    (y, m, d + 1)
                } else {
                    let nm = if m == 12 { 1 } else { m + 1 };
                    let ny = if m == 12 { y + 1 } else { y };
                    (ny, nm, 1)
                };
                selected_date.set(Some(new_d));
                view_year.set(new_d.0);
                view_month.set(new_d.1);
            }
        }
    };

    let mut handle_today = move || {
        selected_date.set(Some(today));
        view_year.set(today.0);
        view_month.set(today.1);
    };

    // Open New Event modal for specific day
    let mut open_new_event_for = move |target_d: (u32, u32, u32)| {
        selected_date.set(Some(target_d));
        view_year.set(target_d.0);
        view_month.set(target_d.1);
        form_day.set(target_d.2);
        form_title.set(String::new());
        form_location.set(String::new());
        form_description.set(String::new());
        is_new_event_open.set(true);
    };

    let active_detail = detail_event();

    rsx! {
        div { class: "flex flex-col flex-1 w-full h-full p-4 md:p-6 min-h-0 overflow-hidden",

            // ========================================================
            // Unified Top Header & Controls Bar
            // ========================================================
            div { class: "shrink-0 flex flex-col gap-3 border-b border-border/50 pb-3 mb-3",
                div { class: "flex flex-wrap items-center justify-between gap-3",

                    // Left: Brand / Title + Period Navigation
                    div { class: "flex flex-wrap items-center gap-3",
                        div { class: "flex items-center gap-2.5",
                            div { class: "flex size-8 items-center justify-center rounded-md bg-primary/10 text-primary border border-primary/25",
                                CalendarIcon { class: "size-4" }
                            }
                            h1 { class: "text-lg font-bold tracking-tight text-foreground sm:text-xl",
                                "Calendar"
                            }
                        }

                        Separator { orientation: "vertical", class: "h-4 mx-1 hidden sm:block" }

                        // Navigation Buttons
                        div { class: "flex items-center gap-1.5",
                            Button {
                                variant: "outline",
                                size: "sm",
                                class: "h-8 px-2.5",
                                onclick: move |_| handle_today(),
                                "Today"
                            }

                            div { class: "flex items-center rounded-md border border-border/70 bg-card overflow-hidden",
                                Button {
                                    variant: "ghost",
                                    size: "icon-sm",
                                    class: "rounded-none border-r border-border/40",
                                    onclick: on_prev,
                                    title: "Previous".to_string(),
                                    ChevronLeftIcon { class: "size-3.5" }
                                }
                                Button {
                                    variant: "ghost",
                                    size: "icon-sm",
                                    class: "rounded-none",
                                    onclick: on_next,
                                    title: "Next".to_string(),
                                    ChevronRightIcon { class: "size-3.5" }
                                }
                            }

                            // Period Label
                            h2 { class: "ml-2 text-sm sm:text-base font-semibold tracking-tight text-foreground select-none",
                                match active_view() {
                                    CalendarView::Month | CalendarView::Schedule => format!("{} {}", MONTH_NAMES[(cur_m - 1) as usize], cur_y),
                                    CalendarView::Week => {
                                        format!("Week of {}, {} {}", SHORT_MONTH_NAMES[(active_selected.1 - 1) as usize], active_selected.2, active_selected.0)
                                    },
                                    CalendarView::Day => {
                                        format_date_pretty(active_selected.0, active_selected.1, active_selected.2)
                                    },
                                }
                            }
                        }
                    }

                    // Right: Category Filters + Search + View Switcher + New Event
                    div { class: "flex flex-wrap items-center gap-2",

                        // Category Filter Dropdown
                        select {
                            class: "h-8 rounded-md border border-border/70 bg-card px-2.5 text-xs font-medium text-foreground focus:border-primary focus:outline-none cursor-pointer",
                            value: "{category_filter}",
                            onchange: move |e| category_filter.set(e.value()),
                            option { value: "All", "All Categories" }
                            option { value: "Engineering", "Engineering" }
                            option { value: "DevOps", "DevOps" }
                            option { value: "Design", "Design" }
                            option { value: "Client Demo", "Client Demo" }
                            option { value: "IT Operations", "IT Operations" }
                            option { value: "Release & QA", "Release & QA" }
                        }

                        // Search Bar
                        div { class: "relative hidden md:block",
                            SearchIcon { class: "absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" }
                            input {
                                r#type: "text",
                                placeholder: "Search schedule...",
                                class: "h-8 w-40 lg:w-52 rounded-md border border-border/70 bg-card pl-8 pr-2.5 text-xs text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none transition-all",
                                value: "{search_query}",
                                oninput: move |e| search_query.set(e.value()),
                            }
                            if !search_query.read().is_empty() {
                                button {
                                    r#type: "button",
                                    class: "absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground cursor-pointer",
                                    onclick: move |_| search_query.set(String::new()),
                                    XIcon { class: "size-3" }
                                }
                            }
                        }

                        // View Switcher: Month | Week | Day | Schedule
                        div { class: "inline-flex items-center rounded-md border border-border/70 bg-card p-0.5",
                            for v in [CalendarView::Month, CalendarView::Week, CalendarView::Day, CalendarView::Schedule] {
                                {
                                    let is_active = active_view() == v;
                                    let variant = if is_active { "default" } else { "ghost" };
                                    rsx! {
                                        Button {
                                            variant: "{variant}",
                                            size: "sm",
                                            key: "{v.label()}",
                                            class: "h-7 px-2.5 text-xs rounded-[4px]",
                                            onclick: move |_| active_view.set(v),
                                            "{v.label()}"
                                        }
                                    }
                                }
                            }
                        }

                        // "+ New Event" Button
                        Button {
                            variant: "default",
                            size: "sm",
                            class: "h-8 gap-1.5 shadow-xs",
                            onclick: move |_| open_new_event_for(active_selected),
                            PlusIcon { class: "size-3.5" }
                            span { "New Event" }
                        }
                    }
                }
            }

            // ========================================================
            // Full Page Calendar View Area
            // ========================================================
            div { class: "flex-1 overflow-y-auto min-h-0 w-full pr-1",
                match active_view() {
                    CalendarView::Month => rsx! {
                        MonthCalendarGrid {
                            year: cur_y,
                            month: cur_m,
                            today: today,
                            selected_date: selected_date,
                            filtered_events: filtered_events.clone(),
                            on_select_day: move |d| selected_date.set(Some(d)),
                            on_quick_add: move |d| open_new_event_for(d),
                            on_open_event: move |ev| detail_event.set(Some(ev)),
                            on_change_priority: on_change_priority,
                            on_change_category: on_change_category,
                            on_delete_event: on_delete_event,
                            on_switch_view: move |v| active_view.set(v),
                            on_today: move |_| handle_today(),
                        }
                    },
                    CalendarView::Week => rsx! {
                        WeekCalendarGrid {
                            selected_date: active_selected,
                            today: today,
                            filtered_events: filtered_events.clone(),
                            on_select_day: move |d| selected_date.set(Some(d)),
                            on_quick_add: move |d| open_new_event_for(d),
                            on_open_event: move |ev| detail_event.set(Some(ev)),
                            on_change_priority: on_change_priority,
                            on_change_category: on_change_category,
                            on_delete_event: on_delete_event,
                        }
                    },
                    CalendarView::Day => rsx! {
                        DayPlannerView {
                            selected_date: active_selected,
                            today: today,
                            filtered_events: filtered_events.clone(),
                            on_quick_add: move |d| open_new_event_for(d),
                            on_open_event: move |ev| detail_event.set(Some(ev)),
                            on_change_priority: on_change_priority,
                            on_change_category: on_change_category,
                            on_delete_event: on_delete_event,
                        }
                    },
                    CalendarView::Schedule => rsx! {
                        ScheduleTimelineView {
                            today: today,
                            filtered_events: filtered_events.clone(),
                            on_open_event: move |ev| detail_event.set(Some(ev)),
                            on_quick_add: move || open_new_event_for(today),
                            on_change_priority: on_change_priority,
                            on_change_category: on_change_category,
                            on_delete_event: on_delete_event,
                        }
                    },
                }
            }

            // ========================================================
            // Event Details Modal Dialog
            // ========================================================
            if let Some(ev) = active_detail {
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-xs animate-in fade-in-0 duration-150",
                    onclick: move |_| detail_event.set(None),

                    div {
                        class: "relative w-full max-w-lg rounded-lg border border-border/80 bg-background p-5 shadow-2xl transition-all",
                        onclick: move |e| e.stop_propagation(),

                        // Header Ribbon
                        div { class: "flex items-start justify-between gap-3 border-b border-border/50 pb-3",
                            div { class: "flex items-center gap-2",
                                Badge {
                                    variant: "outline",
                                    class: "{ev.color.pill_classes().2} {ev.color.pill_classes().0} {ev.color.pill_classes().1}",
                                    "{ev.category}"
                                }
                                Badge {
                                    variant: "{priority_badge_variant(&ev.priority)}",
                                    "{ev.priority}"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "rounded-sm p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer",
                                onclick: move |_| detail_event.set(None),
                                XIcon { class: "size-4" }
                            }
                        }

                        // Event Body
                        div { class: "mt-4 flex flex-col gap-3.5",
                            h3 { class: "text-lg font-bold tracking-tight text-foreground", "{ev.title}" }

                            // Time & Date info
                            div { class: "flex flex-col gap-2 rounded-md border border-border/60 bg-muted/20 p-3",
                                div { class: "flex items-center gap-2 text-xs font-medium text-foreground",
                                    CalendarIcon { class: "size-3.5 text-primary shrink-0" }
                                    span { "{format_date_pretty(ev.date.0, ev.date.1, ev.date.2)}" }
                                }
                                div { class: "flex items-center gap-2 font-mono text-xs text-muted-foreground",
                                    ClockIcon { class: "size-3.5 text-primary shrink-0" }
                                    span { "{ev.time_display()}" }
                                }
                                if let Some(loc) = &ev.location {
                                    div { class: "flex items-center justify-between gap-2 pt-1 border-t border-border/30",
                                        div { class: "flex items-center gap-2 text-xs text-foreground min-w-0",
                                            VideoIcon { class: "size-3.5 text-primary shrink-0" }
                                            span { class: "truncate font-medium", "{loc}" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "shrink-0 inline-flex items-center gap-1 rounded-sm bg-primary/10 hover:bg-primary/20 px-2 py-0.5 text-[11px] font-semibold text-primary transition-colors cursor-pointer",
                                            onclick: {
                                                let loc_text = loc.clone();
                                                move |_| {
                                                    toast.info("Meeting Link", format!("Location: {loc_text}"));
                                                }
                                            },
                                            span { "Connect" }
                                            ExternalLinkIcon { class: "size-3" }
                                        }
                                    }
                                }
                            }

                            // Attendees
                            if !ev.attendees.is_empty() {
                                div { class: "flex flex-col gap-1.5",
                                    span { class: "text-[11px] font-semibold uppercase tracking-wider text-muted-foreground",
                                        "Participants ({ev.attendees.len()})"
                                    }
                                    div { class: "flex flex-wrap gap-1.5",
                                        for person in ev.attendees.iter() {
                                            div { key: "{person}",
                                                class: "inline-flex items-center gap-1.5 rounded-sm border border-border/60 bg-card px-2 py-0.5 text-xs text-foreground",
                                                span { class: "flex size-4 items-center justify-center rounded-xs bg-primary/15 text-[9px] font-bold text-primary",
                                                    "{person.chars().next().unwrap_or('?')}"
                                                }
                                                span { "{person}" }
                                            }
                                        }
                                    }
                                }
                            }

                            // Description
                            if !ev.description.is_empty() {
                                div { class: "flex flex-col gap-1",
                                    span { class: "text-[11px] font-semibold uppercase tracking-wider text-muted-foreground",
                                        "Agenda & Notes"
                                    }
                                    p { class: "text-xs leading-relaxed text-foreground/90 rounded-md border border-border/40 bg-card p-3",
                                        "{ev.description}"
                                    }
                                }
                            }
                        }

                        // Modal Footer Actions
                        div { class: "mt-5 flex items-center justify-between border-t border-border/50 pt-3.5",
                            Button {
                                variant: "destructive",
                                size: "sm",
                                class: "gap-1.5",
                                onclick: {
                                    let ev_id = ev.id;
                                    move |_| {
                                        events.write().retain(|e| e.id != ev_id);
                                        detail_event.set(None);
                                        toast.error("Event Deleted", "The event has been removed from your calendar.");
                                    }
                                },
                                TrashIcon { class: "size-3.5" }
                                span { "Delete Event" }
                            }

                            Button {
                                variant: "outline",
                                size: "sm",
                                onclick: move |_| detail_event.set(None),
                                "Close"
                            }
                        }
                    }
                }
            }

            // ========================================================
            // Create New Event Modal Dialog
            // ========================================================
            if *is_new_event_open.read() {
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-xs animate-in fade-in-0 duration-150",
                    onclick: move |_| is_new_event_open.set(false),

                    div {
                        class: "relative w-full max-w-lg rounded-lg border border-border/80 bg-background p-5 shadow-2xl transition-all",
                        onclick: move |e| e.stop_propagation(),

                        div { class: "flex items-center justify-between border-b border-border/50 pb-3",
                            div {
                                h3 { class: "text-base font-bold text-foreground", "Create New Event" }
                                p { class: "text-xs text-muted-foreground",
                                    "Schedule a meeting, sprint review, or deployment window."
                                }
                            }
                            button {
                                r#type: "button",
                                class: "rounded-sm p-1 text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer",
                                onclick: move |_| is_new_event_open.set(false),
                                XIcon { class: "size-4" }
                            }
                        }

                        // Form Fields
                        div { class: "mt-4 flex flex-col gap-3",
                            // Title
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Event Title *" }
                                input {
                                    r#type: "text",
                                    placeholder: "e.g. Q4 Infrastructure Architecture Review",
                                    class: "h-8 rounded-md border border-border/80 bg-card px-2.5 text-xs text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none",
                                    value: "{form_title}",
                                    oninput: move |e| form_title.set(e.value()),
                                }
                            }

                            // Category & Priority Row
                            div { class: "grid grid-cols-2 gap-2.5",
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Category" }
                                    select {
                                        class: "h-8 rounded-md border border-border/80 bg-card px-2.5 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                        value: "{form_category}",
                                        onchange: move |e| form_category.set(e.value()),
                                        option { value: "Engineering", "Engineering" }
                                        option { value: "DevOps", "DevOps" }
                                        option { value: "Design", "Design" }
                                        option { value: "Client Demo", "Client Demo" }
                                        option { value: "IT Operations", "IT Operations" }
                                        option { value: "Release & QA", "Release & QA" }
                                        option { value: "General", "General" }
                                    }
                                }

                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Priority" }
                                    select {
                                        class: "h-8 rounded-md border border-border/80 bg-card px-2.5 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                        value: "{form_priority}",
                                        onchange: move |e| form_priority.set(e.value()),
                                        option { value: "Medium", "Medium" }
                                        option { value: "High", "High" }
                                        option { value: "Urgent", "Urgent" }
                                        option { value: "Low", "Low" }
                                    }
                                }
                            }

                            // Date & Time Row
                            div { class: "grid grid-cols-3 gap-2.5",
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Day" }
                                    input {
                                        r#type: "number",
                                        min: "1",
                                        max: "31",
                                        class: "h-8 rounded-md border border-border/80 bg-card px-2.5 font-mono text-xs text-foreground focus:border-primary focus:outline-none",
                                        value: "{form_day}",
                                        oninput: move |e| {
                                            if let Ok(v) = e.value().parse::<u32>() {
                                                form_day.set(v.clamp(1, 31));
                                            }
                                        },
                                    }
                                }

                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Start Time" }
                                    input {
                                        r#type: "text",
                                        placeholder: "10:00 AM",
                                        class: "h-8 rounded-md border border-border/80 bg-card px-2.5 font-mono text-xs text-foreground focus:border-primary focus:outline-none",
                                        value: "{form_time}",
                                        oninput: move |e| form_time.set(e.value()),
                                    }
                                }

                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "End Time" }
                                    input {
                                        r#type: "text",
                                        placeholder: "11:00 AM",
                                        class: "h-8 rounded-md border border-border/80 bg-card px-2.5 font-mono text-xs text-foreground focus:border-primary focus:outline-none",
                                        value: "{form_end_time}",
                                        oninput: move |e| form_end_time.set(e.value()),
                                    }
                                }
                            }

                            // Location / Meeting URL
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Location or Meeting URL" }
                                input {
                                    r#type: "text",
                                    placeholder: "e.g. Google Meet · meet.google.com/xyz, Room 301",
                                    class: "h-8 rounded-md border border-border/80 bg-card px-2.5 text-xs text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none",
                                    value: "{form_location}",
                                    oninput: move |e| form_location.set(e.value()),
                                }
                            }

                            // Attendees
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Participants (comma-separated)" }
                                input {
                                    r#type: "text",
                                    placeholder: "David Chen, Sarah Miller, Gamer Rock",
                                    class: "h-8 rounded-md border border-border/80 bg-card px-2.5 text-xs text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none",
                                    value: "{form_attendees}",
                                    oninput: move |e| form_attendees.set(e.value()),
                                }
                            }

                            // Description
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Agenda Description" }
                                textarea {
                                    rows: "3",
                                    placeholder: "Meeting objectives, discussion topics, and preparation notes...",
                                    class: "rounded-md border border-border/80 bg-card p-2.5 text-xs text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none resize-none",
                                    value: "{form_description}",
                                    oninput: move |e| form_description.set(e.value()),
                                }
                            }
                        }

                        // Footer Buttons
                        div { class: "mt-5 flex items-center justify-end gap-2 border-t border-border/50 pt-3.5",
                            Button {
                                variant: "outline",
                                size: "sm",
                                onclick: move |_| is_new_event_open.set(false),
                                "Cancel"
                            }

                            Button {
                                variant: "default",
                                size: "sm",
                                onclick: move |_| {
                                    let title = form_title.read().trim().to_string();
                                    if title.is_empty() {
                                        toast.error("Title Required", "Please enter a title for the event.");
                                        return;
                                    }

                                    let target_d = (cur_y, cur_m, *form_day.read());
                                    let next_id = events.read().iter().map(|e| e.id).max().unwrap_or(0) + 1;
                                    let attendees_list: Vec<String> = form_attendees.read()
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();

                                    let new_ev = CalEvent {
                                        id: next_id,
                                        title: title.clone(),
                                        date: target_d,
                                        end_date: None,
                                        time: Some(form_time.read().clone()),
                                        end_time: Some(form_end_time.read().clone()),
                                        color: match form_category.read().as_str() {
                                            "Engineering" => EventColor::Blue,
                                            "DevOps" => EventColor::Blue,
                                            "Design" => EventColor::Purple,
                                            "Client Demo" => EventColor::Green,
                                            "Release & QA" => EventColor::Red,
                                            _ => EventColor::Amber,
                                        },
                                        category: form_category.read().clone(),
                                        description: form_description.read().clone(),
                                        location: if form_location.read().trim().is_empty() {
                                            None
                                        } else {
                                            Some(form_location.read().trim().to_string())
                                        },
                                        attendees: attendees_list,
                                        priority: form_priority.read().clone(),
                                    };

                                    events.write().push(new_ev);
                                    selected_date.set(Some(target_d));
                                    is_new_event_open.set(false);
                                    toast.success("Event Scheduled", format!("'{title}' added for {}-{:02}-{:02}", target_d.0, target_d.1, target_d.2));
                                },
                                "Schedule Event"
                            }
                        }
                    }
                }
            }
        }
    }
}

// ====================================================================
// Reusable Event Context Menu
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct EventContextMenuProps {
    event: CalEvent,
    children: Element,
    on_open_event: EventHandler<CalEvent>,
    on_change_priority: EventHandler<(u32, String)>,
    on_change_category: EventHandler<(u32, String)>,
    on_delete: EventHandler<u32>,
}

#[component]
fn EventContextMenu(props: EventContextMenuProps) -> Element {
    let ev = props.event;
    let ev_id = ev.id;
    let toast = use_context::<ToastState>();

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                class: "w-52",
                ContextMenuLabel {
                    span { class: "font-semibold truncate block", "{ev.title}" }
                }
                ContextMenuSeparator {}
                ContextMenuItem {
                    on_select: {
                        let ev_c = ev.clone();
                        let handler = props.on_open_event;
                        move |_| handler.call(ev_c.clone())
                    },
                    "View Full Details"
                }
                if let Some(loc) = &ev.location {
                    {
                        let loc_str = loc.clone();
                        rsx! {
                            ContextMenuItem {
                                on_select: move |_| {
                                    toast.info("Meeting Location", format!("Connect: {loc_str}"));
                                },
                                "Connect / Join Meeting"
                            }
                        }
                    }
                }
                ContextMenuSub {
                    ContextMenuSubTrigger { "Change Priority" }
                    ContextMenuSubContent {
                        for p in ["Urgent", "High", "Medium", "Low"] {
                            {
                                let p_str = p.to_string();
                                let handler = props.on_change_priority;
                                rsx! {
                                    ContextMenuItem {
                                        key: "{p}",
                                        on_select: move |_| handler.call((ev_id, p_str.clone())),
                                        "{p}"
                                    }
                                }
                            }
                        }
                    }
                }
                ContextMenuSub {
                    ContextMenuSubTrigger { "Change Category" }
                    ContextMenuSubContent {
                        for c in ["Engineering", "DevOps", "Design", "Client Demo", "Release & QA", "General"] {
                            {
                                let c_str = c.to_string();
                                let handler = props.on_change_category;
                                rsx! {
                                    ContextMenuItem {
                                        key: "{c}",
                                        on_select: move |_| handler.call((ev_id, c_str.clone())),
                                        "{c}"
                                    }
                                }
                            }
                        }
                    }
                }
                ContextMenuSeparator {}
                ContextMenuItem {
                    destructive: true,
                    on_select: {
                        let handler = props.on_delete;
                        move |_| handler.call(ev_id)
                    },
                    "Delete Event"
                }
            }
        }
    }
}

// ====================================================================
// Subcomponent 1: Full Month Calendar Grid (Full Width, Crisp Geometry)
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct MonthGridProps {
    year: u32,
    month: u32,
    today: (u32, u32, u32),
    selected_date: Signal<Option<(u32, u32, u32)>>,
    filtered_events: Vec<CalEvent>,
    on_select_day: EventHandler<(u32, u32, u32)>,
    on_quick_add: EventHandler<(u32, u32, u32)>,
    on_open_event: EventHandler<CalEvent>,
    on_change_priority: EventHandler<(u32, String)>,
    on_change_category: EventHandler<(u32, String)>,
    on_delete_event: EventHandler<u32>,
    on_switch_view: EventHandler<CalendarView>,
    on_today: EventHandler<()>,
}

#[component]
fn MonthCalendarGrid(props: MonthGridProps) -> Element {
    let year = props.year;
    let month = props.month;
    let today = props.today;
    let selected = *props.selected_date.read();

    let total_days = days_in_month(year, month);
    let start_sun = first_day_of_month(year, month);
    let start_col = if start_sun == 0 { 6 } else { start_sun - 1 };

    let prev_m = if month == 1 { 12 } else { month - 1 };
    let prev_y = if month == 1 { year - 1 } else { year };
    let prev_days_count = days_in_month(prev_y, prev_m);

    let next_m = if month == 12 { 1 } else { month + 1 };
    let next_y = if month == 12 { year + 1 } else { year };

    let total_cells = start_col as u32 + total_days;
    let grid_cells = if total_cells <= 35 { 35 } else { 42 };
    let trailing_count = grid_cells - total_cells;

    rsx! {
        div { class: "flex flex-col rounded-md border border-border/60 bg-card overflow-hidden",

            // Days of the week header (Mon - Sun)
            div { class: "grid grid-cols-7 border-b border-border/50 bg-muted/30 text-center text-[11px] font-semibold text-muted-foreground",
                for day_name in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                    div { key: "{day_name}", class: "py-2 uppercase tracking-wider select-none",
                        "{day_name}"
                    }
                }
            }

            // Month cells grid
            div { class: "grid grid-cols-7 divide-x divide-border/40",

                // Previous month padding days
                for i in 0..start_col {
                    {
                        let day_num = prev_days_count - (start_col as u32 - 1) + (i as u32);
                        let day_date = (prev_y, prev_m, day_num);
                        let on_select = props.on_select_day;
                        rsx! {
                            div {
                                key: "prev-{day_num}",
                                class: "group relative min-h-[130px] p-2 bg-muted/10 opacity-35 hover:opacity-75 transition-opacity cursor-pointer flex flex-col justify-between",
                                onclick: move |_| on_select.call(day_date),
                                span { class: "text-xs font-mono text-muted-foreground", "{day_num}" }
                            }
                        }
                    }
                }

                // Current month active days
                for day in 1..=total_days {
                    {
                        let day_date = (year, month, day);
                        let is_today = day_date == today;
                        let is_sel = selected == Some(day_date);

                        let day_events: Vec<CalEvent> = props
                            .filtered_events
                            .iter()
                            .filter(|e| is_event_on(e, day_date))
                            .cloned()
                            .collect();

                        let cell_bg = if is_sel {
                            "bg-primary/[0.04] ring-1 ring-inset ring-primary/40"
                        } else if is_today {
                            "bg-primary/[0.02]"
                        } else {
                            "hover:bg-muted/20"
                        };

                        let on_select = props.on_select_day;
                        let on_add = props.on_quick_add;
                        let on_open = props.on_open_event;
                        let on_change_prio = props.on_change_priority;
                        let on_change_cat = props.on_change_category;
                        let on_delete = props.on_delete_event;
                        let on_switch = props.on_switch_view;
                        let on_today = props.on_today;

                        rsx! {
                            ContextMenu {
                                key: "cur-ctx-{day}",
                                ContextMenuTrigger {
                                    class: "size-full",
                                    div {
                                        class: "group relative min-h-[135px] p-2 transition-colors cursor-pointer flex flex-col gap-1.5 {cell_bg}",
                                        onclick: move |_| on_select.call(day_date),

                                        // Day Header
                                        div { class: "flex items-center justify-between",
                                            if is_today {
                                                span { class: "flex size-6 items-center justify-center rounded-sm bg-primary font-bold text-xs text-primary-foreground shadow-xs",
                                                    "{day}"
                                                }
                                            } else if is_sel {
                                                span { class: "flex size-6 items-center justify-center rounded-sm border border-primary font-bold text-xs text-primary",
                                                    "{day}"
                                                }
                                            } else {
                                                span { class: "text-xs font-mono font-medium text-foreground/80 group-hover:text-foreground",
                                                    "{day}"
                                                }
                                            }

                                            // Quick Add "+" button on hover
                                            button {
                                                r#type: "button",
                                                class: "opacity-0 group-hover:opacity-100 flex size-5 items-center justify-center rounded-xs hover:bg-muted text-muted-foreground hover:text-foreground text-xs font-medium transition-all cursor-pointer",
                                                title: "Add event on this day",
                                                onclick: move |e| {
                                                    e.stop_propagation();
                                                    on_add.call(day_date);
                                                },
                                                "+"
                                            }
                                        }

                                        // Event chips stack
                                        div { class: "flex flex-col gap-1 overflow-hidden",
                                            for ev in day_events.iter().take(4) {
                                                {
                                                    let ev_clone = ev.clone();
                                                    let (pill_bg, pill_text, pill_border) = ev.color.pill_classes();
                                                    rsx! {
                                                        EventContextMenu {
                                                            key: "chip-ctx-{ev.id}",
                                                            event: ev_clone.clone(),
                                                            on_open_event: move |e| on_open.call(e),
                                                            on_change_priority: move |p| on_change_prio.call(p),
                                                            on_change_category: move |c| on_change_cat.call(c),
                                                            on_delete: move |id| on_delete.call(id),

                                                            div {
                                                                class: "flex items-center gap-1.5 rounded-sm border {pill_border} {pill_bg} px-1.5 py-0.5 text-[11px] font-medium transition-colors hover:brightness-105 cursor-pointer truncate",
                                                                onclick: move |e| {
                                                                    e.stop_propagation();
                                                                    on_open.call(ev_clone.clone());
                                                                },
                                                                span { class: "size-1.5 shrink-0 rounded-xs {ev.color.dot_class()}" }
                                                                if let Some(t) = &ev.time {
                                                                    span { class: "font-mono text-[9px] opacity-75 shrink-0", "{t}" }
                                                                }
                                                                span { class: "truncate font-semibold {pill_text}", "{ev.title}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if day_events.len() > 4 {
                                                span { class: "text-[10px] font-medium text-muted-foreground hover:text-foreground px-1 select-none",
                                                    "+{day_events.len() - 4} more"
                                                }
                                            }
                                        }
                                    }
                                }
                                ContextMenuContent {
                                    class: "w-48",
                                    ContextMenuLabel { "{format_date_pretty(year, month, day)}" }
                                    ContextMenuSeparator {}
                                    ContextMenuItem {
                                        on_select: move |_| on_add.call(day_date),
                                        "+ Schedule Event Here"
                                    }
                                    ContextMenuItem {
                                        on_select: move |_| on_select.call(day_date),
                                        "Select Date"
                                    }
                                    ContextMenuSeparator {}
                                    ContextMenuItem {
                                        on_select: move |_| {
                                            on_select.call(day_date);
                                            on_switch.call(CalendarView::Day);
                                        },
                                        "Open in Day Planner"
                                    }
                                    ContextMenuItem {
                                        on_select: move |_| {
                                            on_select.call(day_date);
                                            on_switch.call(CalendarView::Week);
                                        },
                                        "Open in Week Planner"
                                    }
                                    ContextMenuSeparator {}
                                    ContextMenuItem {
                                        on_select: move |_| on_today.call(()),
                                        "Jump to Today"
                                    }
                                }
                            }
                        }
                    }
                }

                // Next month trailing padding days
                for next_d in 1..=trailing_count {
                    {
                        let day_date = (next_y, next_m, next_d);
                        let on_select = props.on_select_day;
                        rsx! {
                            div {
                                key: "next-{next_d}",
                                class: "group relative min-h-[130px] p-2 bg-muted/10 opacity-35 hover:opacity-75 transition-opacity cursor-pointer flex flex-col justify-between",
                                onclick: move |_| on_select.call(day_date),
                                span { class: "text-xs font-mono text-muted-foreground", "{next_d}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ====================================================================
// Subcomponent 2: Week Calendar Grid (7-Day Column Timeline)
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct WeekGridProps {
    selected_date: (u32, u32, u32),
    today: (u32, u32, u32),
    filtered_events: Vec<CalEvent>,
    on_select_day: EventHandler<(u32, u32, u32)>,
    on_quick_add: EventHandler<(u32, u32, u32)>,
    on_open_event: EventHandler<CalEvent>,
    on_change_priority: EventHandler<(u32, String)>,
    on_change_category: EventHandler<(u32, String)>,
    on_delete_event: EventHandler<u32>,
}

#[component]
fn WeekCalendarGrid(props: WeekGridProps) -> Element {
    let (sel_y, sel_m, sel_d) = props.selected_date;
    let today = props.today;

    // Determine the Monday of the week containing sel_date
    let dow = day_of_week(sel_y, sel_m, sel_d);
    let offset_from_mon = if dow == 0 { 6 } else { dow - 1 };

    let mut week_days: Vec<(u32, u32, u32)> = Vec::new();
    let max_days = days_in_month(sel_y, sel_m);

    for i in 0..7 {
        let diff = i - offset_from_mon as i32;
        let d = sel_d as i32 + diff;
        if d >= 1 && d <= max_days as i32 {
            week_days.push((sel_y, sel_m, d as u32));
        } else if d < 1 {
            let pm = if sel_m == 1 { 12 } else { sel_m - 1 };
            let py = if sel_m == 1 { sel_y - 1 } else { sel_y };
            let pmax = days_in_month(py, pm);
            week_days.push((py, pm, (pmax as i32 + d) as u32));
        } else {
            let nm = if sel_m == 12 { 1 } else { sel_m + 1 };
            let ny = if sel_m == 12 { sel_y + 1 } else { sel_y };
            week_days.push((ny, nm, (d - max_days as i32) as u32));
        }
    }

    let time_slots = [
        "08:00 AM", "09:00 AM", "10:00 AM", "11:00 AM", "12:00 PM", "01:00 PM", "02:00 PM",
        "03:00 PM", "04:00 PM", "05:00 PM", "06:00 PM", "07:00 PM",
    ];

    rsx! {
        div { class: "flex flex-col rounded-md border border-border/60 bg-card overflow-hidden",

            // 7 Columns header with Day + Date
            div { class: "grid grid-cols-[64px_repeat(7,1fr)] border-b border-border/50 bg-muted/30 divide-x divide-border/40",
                div { class: "p-2 text-center text-xs font-semibold text-muted-foreground", "Time" }
                for (idx, day_date) in week_days.iter().enumerate() {
                    {
                        let is_today = *day_date == today;
                        let is_sel = *day_date == props.selected_date;
                        let dow_name = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][idx];
                        let on_sel = props.on_select_day;
                        let date_copy = *day_date;

                        let header_bg = if is_today {
                            "bg-primary/10 text-primary"
                        } else if is_sel {
                            "bg-accent/40 text-foreground"
                        } else {
                            "text-muted-foreground hover:bg-muted/40"
                        };

                        rsx! {
                            div {
                                key: "{day_date.0}-{day_date.1}-{day_date.2}",
                                class: "flex flex-col items-center justify-center py-2 px-1 cursor-pointer transition-colors {header_bg}",
                                onclick: move |_| on_sel.call(date_copy),
                                span { class: "text-[11px] font-semibold uppercase tracking-wider", "{dow_name}" }
                                span { class: "font-mono text-sm font-bold", "{day_date.2}" }
                            }
                        }
                    }
                }
            }

            // Time Slots Rows
            div { class: "divide-y divide-border/30 max-h-[640px] overflow-y-auto",
                for slot in time_slots.iter() {
                    {
                        let slot_hour = slot[0..2].parse::<u32>().unwrap_or(0);
                        let is_pm = slot.ends_with("PM");
                        let norm_hour = if is_pm && slot_hour != 12 { slot_hour + 12 } else if !is_pm && slot_hour == 12 { 0 } else { slot_hour };

                        rsx! {
                            div {
                                key: "{slot}",
                                class: "grid grid-cols-[64px_repeat(7,1fr)] divide-x divide-border/30 min-h-[56px]",

                                // Time Label
                                div { class: "p-2 text-right font-mono text-[10px] text-muted-foreground/75 select-none",
                                    "{slot}"
                                }

                                // 7 day cells for this hour slot
                                for day_date in week_days.iter() {
                                    {
                                        let date_copy = *day_date;
                                        let events_in_slot: Vec<CalEvent> = props
                                            .filtered_events
                                            .iter()
                                            .filter(|e| {
                                                if !is_event_on(e, date_copy) {
                                                    return false;
                                                }
                                                if let Some(t) = &e.time {
                                                    let t_pm = t.contains("PM");
                                                    let t_hour = t[0..2].trim().parse::<u32>().unwrap_or(0);
                                                    let h = if t_pm && t_hour != 12 { t_hour + 12 } else if !t_pm && t_hour == 12 { 0 } else { t_hour };
                                                    h == norm_hour
                                                } else {
                                                    norm_hour == 9
                                                }
                                            })
                                            .cloned()
                                            .collect();

                                        let on_open = props.on_open_event;
                                        let on_add = props.on_quick_add;
                                        let on_change_prio = props.on_change_priority;
                                        let on_change_cat = props.on_change_category;
                                        let on_delete = props.on_delete_event;

                                        rsx! {
                                            div {
                                                key: "{date_copy.0}-{date_copy.1}-{date_copy.2}-{slot}",
                                                class: "group relative p-1 hover:bg-muted/15 transition-colors flex flex-col gap-1",

                                                for ev in events_in_slot.iter() {
                                                    {
                                                        let ev_clone = ev.clone();
                                                        let (pill_bg, pill_text, pill_border) = ev.color.pill_classes();
                                                        rsx! {
                                                            EventContextMenu {
                                                                key: "week-ev-ctx-{ev.id}",
                                                                event: ev_clone.clone(),
                                                                on_open_event: move |e| on_open.call(e),
                                                                on_change_priority: move |p| on_change_prio.call(p),
                                                                on_change_category: move |c| on_change_cat.call(c),
                                                                on_delete: move |id| on_delete.call(id),

                                                                div {
                                                                    class: "flex flex-col gap-0.5 rounded-sm border {pill_border} {pill_bg} p-1.5 transition-colors hover:brightness-105 cursor-pointer",
                                                                    onclick: move |e| {
                                                                        e.stop_propagation();
                                                                        on_open.call(ev_clone.clone());
                                                                    },
                                                                    span { class: "text-[11px] font-semibold {pill_text} truncate", "{ev.title}" }
                                                                    span { class: "font-mono text-[9px] text-muted-foreground", "{ev.time_display()}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                if events_in_slot.is_empty() {
                                                    button {
                                                        r#type: "button",
                                                        class: "opacity-0 group-hover:opacity-100 size-full flex items-center justify-center text-muted-foreground/60 hover:text-primary transition-all text-xs cursor-pointer",
                                                        title: "Quick schedule",
                                                        onclick: move |_| on_add.call(date_copy),
                                                        "+"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ====================================================================
// Subcomponent 3: Day Planner View (Detailed Hourly Breakdown)
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct DayPlannerProps {
    selected_date: (u32, u32, u32),
    today: (u32, u32, u32),
    filtered_events: Vec<CalEvent>,
    on_quick_add: EventHandler<(u32, u32, u32)>,
    on_open_event: EventHandler<CalEvent>,
    on_change_priority: EventHandler<(u32, String)>,
    on_change_category: EventHandler<(u32, String)>,
    on_delete_event: EventHandler<u32>,
}

#[component]
fn DayPlannerView(props: DayPlannerProps) -> Element {
    let date = props.selected_date;
    let is_today = date == props.today;
    let date_label = format_date_pretty(date.0, date.1, date.2);

    let day_events: Vec<CalEvent> = props
        .filtered_events
        .iter()
        .filter(|e| is_event_on(e, date))
        .cloned()
        .collect();

    let hours = [
        ("08:00 AM", 8),
        ("09:00 AM", 9),
        ("10:00 AM", 10),
        ("11:00 AM", 11),
        ("12:00 PM", 12),
        ("01:00 PM", 13),
        ("02:00 PM", 14),
        ("03:00 PM", 15),
        ("04:00 PM", 16),
        ("05:00 PM", 17),
        ("06:00 PM", 18),
        ("07:00 PM", 19),
    ];

    let on_add = props.on_quick_add;

    rsx! {
        div { class: "flex flex-col gap-4 rounded-md border border-border/60 bg-card p-5",

            // Day Header Banner
            div { class: "flex flex-wrap items-center justify-between gap-3 border-b border-border/50 pb-3.5",
                div {
                    div { class: "flex items-center gap-2",
                        h3 { class: "text-lg font-bold text-foreground", "{date_label}" }
                        if is_today {
                            span { class: "rounded-sm bg-primary/15 px-2 py-0.5 text-[10px] font-bold text-primary",
                                "TODAY"
                            }
                        }
                    }
                    p { class: "text-xs text-muted-foreground",
                        "{day_events.len()} events scheduled"
                    }
                }

                button {
                    r#type: "button",
                    class: "inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90 transition-colors cursor-pointer shadow-xs",
                    onclick: move |_| on_add.call(date),
                    PlusIcon { class: "size-3.5" }
                    span { "Add Event for this Day" }
                }
            }

            // Hourly Planner
            div { class: "flex flex-col divide-y divide-border/30",
                for (hour_label, hour_num) in hours.iter() {
                    {
                        let h_num = *hour_num;
                        let events_at_hour: Vec<CalEvent> = day_events
                            .iter()
                            .filter(|e| {
                                if let Some(t) = &e.time {
                                    let t_pm = t.contains("PM");
                                    let t_hour = t[0..2].trim().parse::<u32>().unwrap_or(0);
                                    let h = if t_pm && t_hour != 12 { t_hour + 12 } else if !t_pm && t_hour == 12 { 0 } else { t_hour };
                                    h == h_num
                                } else {
                                    h_num == 9
                                }
                            })
                            .cloned()
                            .collect();

                        let on_open = props.on_open_event;
                        let on_change_prio = props.on_change_priority;
                        let on_change_cat = props.on_change_category;
                        let on_delete = props.on_delete_event;

                        rsx! {
                            div {
                                key: "{hour_label}",
                                class: "group grid grid-cols-[80px_1fr] gap-4 py-2.5 min-h-[58px] items-start hover:bg-muted/10 transition-colors rounded-xs px-2",

                                // Left Time Label
                                span { class: "font-mono text-xs font-semibold text-muted-foreground pt-1 select-none",
                                    "{hour_label}"
                                }

                                // Center Content
                                div { class: "flex flex-col gap-2",
                                    if events_at_hour.is_empty() {
                                        div { class: "flex items-center justify-between rounded-xs border border-transparent group-hover:border-dashed group-hover:border-border/50 p-1.5 transition-all",
                                            span { class: "text-xs text-muted-foreground/40 italic", "No events scheduled" }
                                            button {
                                                r#type: "button",
                                                class: "opacity-0 group-hover:opacity-100 inline-flex items-center gap-1 text-[11px] font-semibold text-primary hover:underline cursor-pointer",
                                                onclick: move |_| on_add.call(date),
                                                PlusIcon { class: "size-3" }
                                                span { "Schedule at {hour_label}" }
                                            }
                                        }
                                    } else {
                                        for ev in events_at_hour.iter() {
                                            {
                                                let ev_clone = ev.clone();
                                                let (pill_bg, pill_text, pill_border) = ev.color.pill_classes();
                                                rsx! {
                                                    EventContextMenu {
                                                        key: "day-ev-ctx-{ev.id}",
                                                        event: ev_clone.clone(),
                                                        on_open_event: move |e| on_open.call(e),
                                                        on_change_priority: move |p| on_change_prio.call(p),
                                                        on_change_category: move |c| on_change_cat.call(c),
                                                        on_delete: move |id| on_delete.call(id),

                                                        div {
                                                            class: "flex flex-col gap-2 rounded-sm border-l-3 {ev.color.border_accent_class()} border border-border/70 {pill_bg} p-3 transition-colors hover:brightness-105 cursor-pointer shadow-xs",
                                                            onclick: move |_| on_open.call(ev_clone.clone()),

                                                            div { class: "flex flex-wrap items-center justify-between gap-2",
                                                                div { class: "flex items-center gap-2",
                                                                    span { class: "text-sm font-semibold {pill_text}", "{ev.title}" }
                                                                    span { class: "rounded-sm border {pill_border} px-1.5 py-0.2 text-[10px] font-bold {pill_text}",
                                                                        "{ev.category}"
                                                                    }
                                                                }
                                                                Badge {
                                                                    variant: "{priority_badge_variant(&ev.priority)}",
                                                                    class: "text-[10px]",
                                                                    "{ev.priority}"
                                                                }
                                                            }

                                                            div { class: "flex flex-wrap items-center gap-4 font-mono text-xs text-muted-foreground",
                                                                div { class: "flex items-center gap-1.5",
                                                                    ClockIcon { class: "size-3.5 text-primary" }
                                                                    span { "{ev.time_display()}" }
                                                                }
                                                                if let Some(loc) = &ev.location {
                                                                    div { class: "flex items-center gap-1.5 text-foreground/80",
                                                                        MapPinIcon { class: "size-3.5 text-primary" }
                                                                        span { "{loc}" }
                                                                    }
                                                                }
                                                                if !ev.attendees.is_empty() {
                                                                    div { class: "flex items-center gap-1.5",
                                                                        UsersIcon { class: "size-3.5 text-primary" }
                                                                        span { "{ev.attendees.join(\", \")}" }
                                                                    }
                                                                }
                                                            }

                                                            if !ev.description.is_empty() {
                                                                p { class: "text-xs text-foreground/80 line-clamp-2 pt-1 border-t border-border/20",
                                                                    "{ev.description}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ====================================================================
// Subcomponent 4: Schedule Timeline View (Chronological Feed)
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct ScheduleProps {
    today: (u32, u32, u32),
    filtered_events: Vec<CalEvent>,
    on_open_event: EventHandler<CalEvent>,
    on_quick_add: EventHandler<()>,
    on_change_priority: EventHandler<(u32, String)>,
    on_change_category: EventHandler<(u32, String)>,
    on_delete_event: EventHandler<u32>,
}

#[component]
fn ScheduleTimelineView(props: ScheduleProps) -> Element {
    let today = props.today;
    let mut sorted_events = props.filtered_events.clone();
    sorted_events.sort_by_key(|e| (e.date, e.time.clone()));

    let on_add = props.on_quick_add;

    rsx! {
        div { class: "flex flex-col gap-4 rounded-md border border-border/60 bg-card p-5",

            div { class: "flex items-center justify-between border-b border-border/50 pb-3.5",
                div {
                    h3 { class: "text-base font-bold text-foreground", "Schedule Feed" }
                    p { class: "text-xs text-muted-foreground",
                        "Chronological timeline of all upcoming milestones and reviews."
                    }
                }
                button {
                    r#type: "button",
                    class: "inline-flex items-center gap-1.5 rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90 transition-colors cursor-pointer shadow-xs",
                    onclick: move |_| on_add.call(()),
                    PlusIcon { class: "size-3.5" }
                    span { "New Event" }
                }
            }

            if sorted_events.is_empty() {
                div { class: "flex flex-col items-center justify-center gap-2 py-12 text-center",
                    CalendarIcon { class: "size-8 text-muted-foreground/50" }
                    p { class: "text-sm font-semibold text-foreground", "No events match your criteria" }
                    p { class: "text-xs text-muted-foreground", "Try adjusting your search query or category filter." }
                }
            } else {
                div { class: "flex flex-col gap-2.5",
                    for ev in sorted_events.iter() {
                        {
                            let ev_clone = ev.clone();
                            let is_today = ev.date == today;
                            let (pill_bg, pill_text, pill_border) = ev.color.pill_classes();
                            let on_open = props.on_open_event;
                            let on_change_prio = props.on_change_priority;
                            let on_change_cat = props.on_change_category;
                            let on_delete = props.on_delete_event;

                            rsx! {
                                EventContextMenu {
                                    key: "sched-ctx-{ev.id}",
                                    event: ev_clone.clone(),
                                    on_open_event: move |e| on_open.call(e),
                                    on_change_priority: move |p| on_change_prio.call(p),
                                    on_change_category: move |c| on_change_cat.call(c),
                                    on_delete: move |id| on_delete.call(id),

                                    div {
                                        class: "group flex flex-col sm:flex-row sm:items-center justify-between gap-4 rounded-sm border border-border/60 bg-card p-3.5 hover:border-primary/50 hover:bg-muted/20 transition-all cursor-pointer shadow-xs",
                                        onclick: move |_| on_open.call(ev_clone.clone()),

                                        // Left: Date Badge + Title + Details
                                        div { class: "flex items-start gap-3.5 min-w-0",
                                            div { class: "flex size-12 shrink-0 flex-col items-center justify-center rounded-sm border border-border/70 bg-muted/30 font-mono select-none",
                                                span { class: "text-[9px] font-bold uppercase text-primary leading-none",
                                                    "{SHORT_MONTH_NAMES[(ev.date.1 - 1) as usize]}"
                                                }
                                                span { class: "text-base font-bold text-foreground leading-none mt-1",
                                                    "{ev.date.2}"
                                                }
                                            }

                                            div { class: "flex flex-col gap-1 min-w-0",
                                                div { class: "flex flex-wrap items-center gap-2",
                                                    span { class: "text-sm font-semibold text-foreground truncate", "{ev.title}" }
                                                    span { class: "rounded-sm border {pill_border} {pill_bg} px-1.5 py-0.2 text-[10px] font-semibold {pill_text}",
                                                        "{ev.category}"
                                                    }
                                                    Badge {
                                                        variant: "{priority_badge_variant(&ev.priority)}",
                                                        class: "text-[10px]",
                                                        "{ev.priority}"
                                                    }
                                                    if is_today {
                                                        span { class: "rounded-xs bg-primary/15 px-1.5 py-0.2 text-[9px] font-bold text-primary",
                                                            "TODAY"
                                                        }
                                                    }
                                                }

                                                div { class: "flex flex-wrap items-center gap-3 font-mono text-xs text-muted-foreground",
                                                    div { class: "flex items-center gap-1",
                                                        ClockIcon { class: "size-3.5 text-primary" }
                                                        span { "{ev.time_display()}" }
                                                    }
                                                    if let Some(loc) = &ev.location {
                                                        div { class: "flex items-center gap-1 text-foreground/80 truncate max-w-64",
                                                            MapPinIcon { class: "size-3.5 text-primary" }
                                                            span { class: "truncate", "{loc}" }
                                                        }
                                                    }
                                                    if !ev.attendees.is_empty() {
                                                        div { class: "flex items-center gap-1",
                                                            UsersIcon { class: "size-3.5 text-primary" }
                                                            span { "{ev.attendees.len()} participants" }
                                                        }
                                                    }
                                                }

                                                if !ev.description.is_empty() {
                                                    p { class: "text-xs text-muted-foreground line-clamp-1 pt-0.5",
                                                        "{ev.description}"
                                                    }
                                                }
                                            }
                                        }

                                        // Right: Quick Action
                                        div { class: "flex items-center gap-2 shrink-0 self-end sm:self-center",
                                            button {
                                                r#type: "button",
                                                class: "rounded-sm border border-border/70 bg-muted/40 px-2.5 py-1 text-xs font-medium text-foreground hover:bg-accent transition-colors cursor-pointer",
                                                onclick: {
                                                    let ev_c = ev.clone();
                                                    move |e| {
                                                        e.stop_propagation();
                                                        on_open.call(ev_c.clone());
                                                    }
                                                },
                                                "Details"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
