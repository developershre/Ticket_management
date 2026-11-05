//! Calendar Page — Interactive Workstation for Tickets, Reminders, and Notes.
//! Shows activity by selected date or inclusive date range of 2 dates.
//! Clean, architectural aesthetic with Roboto typography and real-time SQLite sync.
#![allow(clippy::redundant_closure)]

use dioxus::prelude::*;

use crate::components::month_calendar::{
    days_in_month, extract_time_str, first_day_of_month, format_date_pretty, format_ymd,
    parse_date_ymd, DateRange, MONTH_NAMES, SHORT_MONTH_NAMES,
};
use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::toast::ToastState;
use crate::db::{
    get_all_notes, get_all_reminders, get_all_tickets, DbNote, DbReminder, DbTicket,
};
use crate::icons::{
    BellIcon, CalendarIcon, ChevronLeftIcon, ChevronRightIcon, ClockIcon, EyeIcon, FilterIcon,
    GridIcon, ListIcon, SearchIcon, StickyNoteIcon, TicketIcon, UsersIcon, XIcon,
};
use crate::pages::tickets::{assignee_color_classes, get_assignee_initials};
use crate::state::AppDbRevision;

/// Entity kind for filtering and badge styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarItemKind {
    Ticket,
    Reminder,
    Note,
}

#[allow(dead_code)]
impl CalendarItemKind {
    pub fn label(&self) -> &'static str {
        match self {
            CalendarItemKind::Ticket => "Ticket",
            CalendarItemKind::Reminder => "Reminder",
            CalendarItemKind::Note => "Note",
        }
    }
}

/// Category / Type filter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemFilterType {
    All,
    Tickets,
    Reminders,
    Notes,
}

/// Selection mode: single day or range of two dates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateSelectionMode {
    SingleDate,
    DateRange,
}

/// Calendar presentation format: Month Grid or Schedule Timeline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CalendarViewMode {
    Month,
    Schedule,
}

/// Polymorphic entity wrapper so any ticket, reminder, or note can be inspected.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, PartialEq)]
pub enum CalendarEntityData {
    Ticket(DbTicket),
    Reminder(DbReminder),
    Note(DbNote),
}

/// Unified calendar entry created from SQLite records.
#[derive(Clone, Debug, PartialEq)]
pub struct CalendarEntry {
    pub key: String,
    pub kind: CalendarItemKind,
    pub title: String,
    pub subtitle: String,
    pub date: (u32, u32, u32),
    pub time_str: String,
    pub priority: String,
    pub status: String,
    pub employee: String,
    pub details: String,
    pub tag: Option<String>,
    pub data: CalendarEntityData,
}

fn priority_badge_variant(priority: &str) -> &'static str {
    match priority.to_lowercase().as_str() {
        "urgent" => "destructive",
        "high" => "warning",
        "medium" => "info",
        _ => "outline",
    }
}

fn status_badge_classes(status: &str) -> (&'static str, &'static str) {
    match status.to_lowercase().as_str() {
        "completed" | "closed" | "done" => (
            "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border-emerald-500/25",
            "Completed",
        ),
        "inprogress" | "in progress" => (
            "bg-blue-500/15 text-blue-600 dark:text-blue-400 border-blue-500/25",
            "In Progress",
        ),
        "onhold" | "on hold" => (
            "bg-amber-500/15 text-amber-600 dark:text-amber-400 border-amber-500/25",
            "On Hold",
        ),
        "pending" | "open" | "new" => (
            "bg-sky-500/15 text-sky-600 dark:text-sky-400 border-sky-500/25",
            "Pending",
        ),
        _ => ("bg-muted text-muted-foreground border-border", "Active"),
    }
}

#[component]
pub fn CalendarPage() -> Element {
    let today = crate::state::get_real_current_date();
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<AppDbRevision>>();

    // Calendar navigation state
    let mut view_year = use_signal(move || today.0);
    let mut view_month = use_signal(move || today.1);
    let mut view_mode = use_signal(|| CalendarViewMode::Month);

    // Selection mode: Single Date vs Range of 2 Dates
    let mut selection_mode = use_signal(|| DateSelectionMode::SingleDate);
    let mut selected_date = use_signal(move || today);
    let mut date_range = use_signal(|| None::<DateRange>);
    let mut range_pick_step = use_signal(|| 0u8);

    // Filtering & search
    let mut item_filter = use_signal(|| ItemFilterType::All);
    let mut search_query = use_signal(String::new);

    // Detail modal state
    let mut inspect_item = use_signal(|| None::<CalendarEntityData>);

    let cur_y = view_year();
    let cur_m = view_month();
    let sel_d = selected_date();

    // -------------------------------------------------------------
    // Load and unify all tickets, reminders, and notes from SQLite
    // -------------------------------------------------------------
    let all_entries = use_memo(move || {
        if let Some(sync) = db_sync {
            let _ = sync();
        }

        let mut entries = Vec::new();

        // 1. Tickets
        let tickets = get_all_tickets();
        for t in tickets {
            let date = parse_date_ymd(&t.created_on)
                .or_else(|| parse_date_ymd(&t.last_updated_on))
                .unwrap_or(today);
            let time_str = extract_time_str(&t.created_on);
            let display_id = t
                .server_ticket_id
                .clone()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| format!("TKT-{}", t.ticket_id));

            entries.push(CalendarEntry {
                key: format!("ticket-{}", t.ticket_id),
                kind: CalendarItemKind::Ticket,
                title: if t.subject.is_empty() {
                    "Untitled Support Ticket".to_string()
                } else {
                    t.subject.clone()
                },
                subtitle: format!(
                    "Assigned: {}",
                    if t.assigned_to.is_empty() {
                        "Unassigned"
                    } else {
                        &t.assigned_to
                    }
                ),
                date,
                time_str,
                priority: t.priority.clone(),
                status: t.kanban_state.clone(),
                employee: if !t.employee_code.is_empty() {
                    t.employee_code.clone()
                } else {
                    t.email_address.clone()
                },
                details: t.problem_description.clone(),
                tag: Some(format!("{display_id} • {}", t.category)),
                data: CalendarEntityData::Ticket(t),
            });
        }

        // 2. Reminders
        let reminders = get_all_reminders();
        for r in reminders {
            let date = parse_date_ymd(&r.created_at).unwrap_or(today);
            let time_str = if !r.send_at.is_empty() {
                format!("Due: {}", crate::state::format_display_12hr(&r.send_at))
            } else {
                crate::state::format_display_12hr(&r.created_at)
            };
            let display_id = if !r.server_reminder_id.is_empty() {
                r.server_reminder_id.clone()
            } else {
                format!("REM-{}", r.reminder_id)
            };

            entries.push(CalendarEntry {
                key: format!("reminder-{}", r.reminder_id),
                kind: CalendarItemKind::Reminder,
                title: if r.reminder_message.is_empty() {
                    "Untitled Reminder".to_string()
                } else {
                    r.reminder_message.clone()
                },
                subtitle: format!("Recipient: {} ({})", r.employee_name, r.employee_code),
                date,
                time_str,
                priority: "Medium".to_string(),
                status: r.status.clone(),
                employee: r.employee_name.clone(),
                details: format!(
                    "Category: {} | Email: {}",
                    r.service_category, r.email_address
                ),
                tag: Some(display_id),
                data: CalendarEntityData::Reminder(r),
            });
        }

        // 3. Notes
        let notes = get_all_notes();
        for n in notes {
            let date = parse_date_ymd(&n.created_at).unwrap_or(today);
            let time_str = extract_time_str(&n.created_at);
            let (title, subtitle) = if let Some((t, d)) = n.content.split_once(" — ") {
                (t.trim().to_string(), d.trim().to_string())
            } else if let Some((t, d)) = n.content.split_once('\n') {
                (t.trim().to_string(), d.trim().to_string())
            } else {
                (n.content.clone(), String::new())
            };

            entries.push(CalendarEntry {
                key: format!("note-{}", n.id),
                kind: CalendarItemKind::Note,
                title: if title.is_empty() {
                    format!("Note #{}", n.id)
                } else {
                    title
                },
                subtitle: if !subtitle.is_empty() {
                    subtitle
                } else if !n.emp_code.is_empty() {
                    format!("Employee: {}", n.emp_code)
                } else {
                    "Quick Note".to_string()
                },
                date,
                time_str,
                priority: "Low".to_string(),
                status: "Note".to_string(),
                employee: n.emp_code.clone(),
                details: n.content.clone(),
                tag: Some(format!("NOTE-{}", n.id)),
                data: CalendarEntityData::Note(n),
            });
        }

        // Sort descending by date, then key
        entries.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| b.key.cmp(&a.key)));
        entries
    });

    let entries = all_entries();

    // -------------------------------------------------------------
    // Filter items based on active selection (Single Date or Range)
    // -------------------------------------------------------------
    let q = search_query.read().trim().to_lowercase();
    let current_filter = item_filter();
    let current_mode = selection_mode();
    let active_range = date_range();

    let filtered_items: Vec<CalendarEntry> = entries
        .iter()
        .filter(|e| {
            // 1. Date match
            let date_matches = match current_mode {
                DateSelectionMode::SingleDate => e.date == sel_d,
                DateSelectionMode::DateRange => {
                    if let Some((start, end)) = active_range {
                        e.date >= start && e.date <= end
                    } else {
                        e.date == sel_d
                    }
                }
            };
            if !date_matches {
                return false;
            }

            // 2. Kind / Type filter
            let kind_matches = match current_filter {
                ItemFilterType::All => true,
                ItemFilterType::Tickets => e.kind == CalendarItemKind::Ticket,
                ItemFilterType::Reminders => e.kind == CalendarItemKind::Reminder,
                ItemFilterType::Notes => e.kind == CalendarItemKind::Note,
            };
            if !kind_matches {
                return false;
            }

            // 3. Search query
            if !q.is_empty() {
                let m_title = e.title.to_lowercase().contains(&q);
                let m_sub = e.subtitle.to_lowercase().contains(&q);
                let m_emp = e.employee.to_lowercase().contains(&q);
                let m_det = e.details.to_lowercase().contains(&q);
                let m_tag = e
                    .tag
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&q))
                    .unwrap_or(false);
                if !m_title && !m_sub && !m_emp && !m_det && !m_tag {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    // Activity summary counts for current selected date/range
    let (total_tickets, total_reminders, total_notes) = {
        let in_scope: Vec<&CalendarEntry> = entries
            .iter()
            .filter(|e| match current_mode {
                DateSelectionMode::SingleDate => e.date == sel_d,
                DateSelectionMode::DateRange => {
                    if let Some((start, end)) = active_range {
                        e.date >= start && e.date <= end
                    } else {
                        e.date == sel_d
                    }
                }
            })
            .collect();

        let t = in_scope
            .iter()
            .filter(|e| e.kind == CalendarItemKind::Ticket)
            .count();
        let r = in_scope
            .iter()
            .filter(|e| e.kind == CalendarItemKind::Reminder)
            .count();
        let n = in_scope
            .iter()
            .filter(|e| e.kind == CalendarItemKind::Note)
            .count();
        (t, r, n)
    };

    // Navigation callbacks
    let mut on_prev_month = move |_| {
        let m = view_month();
        let y = view_year();
        if m == 1 {
            view_month.set(12);
            view_year.set(y - 1);
        } else {
            view_month.set(m - 1);
        }
    };

    let mut on_next_month = move |_| {
        let m = view_month();
        let y = view_year();
        if m == 12 {
            view_month.set(1);
            view_year.set(y + 1);
        } else {
            view_month.set(m + 1);
        }
    };

    let mut on_goto_today = move |_| {
        view_year.set(today.0);
        view_month.set(today.1);
        selected_date.set(today);
        if selection_mode() == DateSelectionMode::DateRange {
            date_range.set(Some((today, today)));
        }
    };

    // Day click handler supporting both Single Date and 2-Date Range selection
    let mut handle_day_click = move |clicked: (u32, u32, u32)| {
        match selection_mode() {
            DateSelectionMode::SingleDate => {
                selected_date.set(clicked);
                view_year.set(clicked.0);
                view_month.set(clicked.1);
            }
            DateSelectionMode::DateRange => {
                let step = *range_pick_step.read();
                if step == 0 {
                    date_range.set(Some((clicked, clicked)));
                    range_pick_step.set(1);
                    selected_date.set(clicked);
                    toast.info(
                        "Start Date Selected",
                        format!("Start: {}. Click a second date for the end.", format_ymd(clicked)),
                    );
                } else {
                    let current_start = date_range().map(|(s, _)| s).unwrap_or(clicked);
                    let (s, e) = if clicked < current_start {
                        (clicked, current_start)
                    } else {
                        (current_start, clicked)
                    };
                    date_range.set(Some((s, e)));
                    range_pick_step.set(0);
                    toast.success(
                        "Date Range Active",
                        format!("Viewing items created between {} and {}", format_ymd(s), format_ymd(e)),
                    );
                }
            }
        }
    };

    rsx! {
        div { class: "flex h-full w-full flex-col overflow-hidden bg-background font-sans",

            // ========================================================
            // Top Workspace Header Toolbar
            // ========================================================
            div { class: "shrink-0 border-b border-border/70 bg-card/60 backdrop-blur-md px-6 py-3.5 flex flex-wrap items-center justify-between gap-3 shadow-2xs",

                // Month / Year Title + Prev / Next Controls
                div { class: "flex items-center gap-3",
                    div { class: "flex items-center gap-1.5",
                        span { class: "text-lg font-bold tracking-tight text-foreground",
                            "{MONTH_NAMES[(cur_m - 1) as usize]} {cur_y}"
                        }
                    }

                    div { class: "flex items-center gap-1 rounded-md border border-border/60 bg-muted/30 p-0.5",
                        button {
                            r#type: "button",
                            class: "flex size-7 items-center justify-center rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                            title: "Previous month",
                            onclick: move |e| on_prev_month(e),
                            ChevronLeftIcon { class: "size-4" }
                        }
                        button {
                            r#type: "button",
                            class: "flex size-7 items-center justify-center rounded-sm hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                            title: "Next month",
                            onclick: move |e| on_next_month(e),
                            ChevronRightIcon { class: "size-4" }
                        }
                    }

                    Button {
                        variant: "outline",
                        size: "sm",
                        class: "h-7 text-xs px-2.5",
                        onclick: move |e| on_goto_today(e),
                        "Today"
                    }
                }

                // Center: Mode Switcher (Single Date vs Date Range)
                div { class: "flex items-center gap-2",
                    div { class: "inline-flex items-center rounded-md border border-border/70 bg-card p-0.5 shadow-2xs",
                        button {
                            r#type: "button",
                            class: if current_mode == DateSelectionMode::SingleDate {
                                "flex items-center gap-1.5 rounded-[4px] bg-primary text-primary-foreground font-semibold px-3 py-1 text-xs shadow-2xs transition-all cursor-pointer"
                            } else {
                                "flex items-center gap-1.5 rounded-[4px] text-muted-foreground hover:text-foreground hover:bg-muted/60 px-3 py-1 text-xs transition-all cursor-pointer"
                            },
                            onclick: move |_| {
                                selection_mode.set(DateSelectionMode::SingleDate);
                                range_pick_step.set(0);
                            },
                            CalendarIcon { class: "size-3.5" }
                            span { "Single Date" }
                        }

                        button {
                            r#type: "button",
                            class: if current_mode == DateSelectionMode::DateRange {
                                "flex items-center gap-1.5 rounded-[4px] bg-primary text-primary-foreground font-semibold px-3 py-1 text-xs shadow-2xs transition-all cursor-pointer"
                            } else {
                                "flex items-center gap-1.5 rounded-[4px] text-muted-foreground hover:text-foreground hover:bg-muted/60 px-3 py-1 text-xs transition-all cursor-pointer"
                            },
                            onclick: move |_| {
                                selection_mode.set(DateSelectionMode::DateRange);
                                if date_range().is_none() {
                                    date_range.set(Some((today, today)));
                                }
                            },
                            FilterIcon { class: "size-3.5" }
                            span { "Date Range (2 Dates)" }
                        }
                    }

                    // View Mode Switcher: Grid vs Timeline
                    div { class: "inline-flex items-center rounded-md border border-border/70 bg-card p-0.5 shadow-2xs",
                        button {
                            r#type: "button",
                            class: if view_mode() == CalendarViewMode::Month {
                                "flex items-center gap-1 rounded-[4px] bg-muted font-semibold text-foreground px-2 py-1 text-xs cursor-pointer"
                            } else {
                                "flex items-center gap-1 rounded-[4px] text-muted-foreground hover:text-foreground px-2 py-1 text-xs cursor-pointer"
                            },
                            title: "Month Calendar Grid",
                            onclick: move |_| view_mode.set(CalendarViewMode::Month),
                            GridIcon { class: "size-3.5" }
                            span { "Grid" }
                        }
                        button {
                            r#type: "button",
                            class: if view_mode() == CalendarViewMode::Schedule {
                                "flex items-center gap-1 rounded-[4px] bg-muted font-semibold text-foreground px-2 py-1 text-xs cursor-pointer"
                            } else {
                                "flex items-center gap-1 rounded-[4px] text-muted-foreground hover:text-foreground px-2 py-1 text-xs cursor-pointer"
                            },
                            title: "Schedule Agenda View",
                            onclick: move |_| view_mode.set(CalendarViewMode::Schedule),
                            ListIcon { class: "size-3.5" }
                            span { "Agenda" }
                        }
                    }
                }

                // Right: Search Input
                div { class: "relative w-full sm:w-64",
                    div { class: "pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground",
                        SearchIcon { class: "size-3.5" }
                    }
                    input {
                        r#type: "text",
                        placeholder: "Search created items...",
                        class: "h-8 w-full rounded-md border border-border/80 bg-background/80 pl-8 pr-7 text-xs text-foreground placeholder:text-muted-foreground/70 focus:border-primary focus:outline-none transition-colors",
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
            }

            // ========================================================
            // Date Range Control Ribbon (shown when Date Range is active)
            // ========================================================
            if current_mode == DateSelectionMode::DateRange {
                div { class: "shrink-0 border-b border-border/70 bg-primary/5 px-6 py-2 flex flex-wrap items-center justify-between gap-3 text-xs",
                    div { class: "flex items-center gap-2",
                        span { class: "font-semibold text-primary flex items-center gap-1.5",
                            CalendarIcon { class: "size-4" }
                            "Active Date Range:"
                        }
                        if let Some((start, end)) = active_range {
                            span { class: "font-mono font-bold bg-background border border-primary/30 text-foreground px-2 py-0.5 rounded shadow-2xs",
                                "{format_ymd(start)} \u{2013} {format_ymd(end)}"
                            }
                        } else {
                            span { class: "italic text-muted-foreground", "Click 2 dates on the calendar below" }
                        }

                        if *range_pick_step.read() == 1 {
                            span { class: "text-amber-500 font-medium animate-pulse",
                                "• Select second date to complete range"
                            }
                        }
                    }

                    // Quick range preset pills
                    div { class: "flex items-center gap-1.5",
                        span { class: "text-[11px] text-muted-foreground font-medium", "Presets:" }
                        button {
                            r#type: "button",
                            class: "rounded border border-border/70 bg-background px-2 py-0.5 text-[11px] font-medium text-foreground hover:bg-muted hover:border-primary/50 cursor-pointer transition-colors",
                            onclick: move |_| {
                                date_range.set(Some((today, today)));
                                range_pick_step.set(0);
                            },
                            "Today"
                        }
                        button {
                            r#type: "button",
                            class: "rounded border border-border/70 bg-background px-2 py-0.5 text-[11px] font-medium text-foreground hover:bg-muted hover:border-primary/50 cursor-pointer transition-colors",
                            onclick: move |_| {
                                let (y, m, d) = today;
                                let start = if d > 7 {
                                    (y, m, d - 6)
                                } else {
                                    let pm = if m == 1 { 12 } else { m - 1 };
                                    let py = if m == 1 { y - 1 } else { y };
                                    let pmax = days_in_month(py, pm);
                                    (py, pm, pmax - (6 - d))
                                };
                                date_range.set(Some((start, today)));
                                range_pick_step.set(0);
                            },
                            "Last 7 Days"
                        }
                        button {
                            r#type: "button",
                            class: "rounded border border-border/70 bg-background px-2 py-0.5 text-[11px] font-medium text-foreground hover:bg-muted hover:border-primary/50 cursor-pointer transition-colors",
                            onclick: move |_| {
                                let max_d = days_in_month(cur_y, cur_m);
                                date_range.set(Some(((cur_y, cur_m, 1), (cur_y, cur_m, max_d))));
                                range_pick_step.set(0);
                            },
                            "This Month"
                        }
                        button {
                            r#type: "button",
                            class: "rounded border border-border/70 bg-background px-2 py-0.5 text-[11px] font-medium text-destructive hover:bg-destructive/10 cursor-pointer transition-colors",
                            onclick: move |_| {
                                selection_mode.set(DateSelectionMode::SingleDate);
                                range_pick_step.set(0);
                            },
                            "Reset to Single Date"
                        }
                    }
                }
            }

            // ========================================================
            // Main Responsive Split-Screen Workstation
            // ========================================================
            div { class: "flex flex-1 overflow-hidden min-h-0",

                // ----------------------------------------------------
                // LEFT: Interactive Calendar Grid / Timeline
                // ----------------------------------------------------
                div { class: "flex flex-1 flex-col overflow-y-auto p-4 min-h-0 border-r border-border/60",
                    match view_mode() {
                        CalendarViewMode::Month => rsx! {
                            MonthCalendarView {
                                year: cur_y,
                                month: cur_m,
                                today: today,
                                selected_date: sel_d,
                                date_range: active_range,
                                selection_mode: current_mode,
                                entries: entries.clone(),
                                on_day_click: move |d| handle_day_click(d),
                            }
                        },
                        CalendarViewMode::Schedule => rsx! {
                            ScheduleTimelineListView {
                                year: cur_y,
                                month: cur_m,
                                today: today,
                                selected_date: sel_d,
                                date_range: active_range,
                                selection_mode: current_mode,
                                entries: entries.clone(),
                                on_select_day: move |d| {
                                    selected_date.set(d);
                                    selection_mode.set(DateSelectionMode::SingleDate);
                                },
                                on_open_item: move |item| inspect_item.set(Some(item)),
                            }
                        },
                    }
                }

                // ----------------------------------------------------
                // RIGHT: Created Activity Panel (Tickets, Reminders, Notes)
                // ----------------------------------------------------
                div { class: "flex w-full md:w-[460px] lg:w-[500px] shrink-0 flex-col overflow-hidden bg-card/40 backdrop-blur-xs",

                    // Panel Header & Active Date/Range Title
                    div { class: "shrink-0 border-b border-border/70 bg-card/80 p-4",
                        div { class: "flex items-start justify-between gap-2",
                            div { class: "flex flex-col gap-0.5 min-w-0",
                                span { class: "text-[11px] font-bold uppercase tracking-wider text-muted-foreground",
                                    if current_mode == DateSelectionMode::DateRange {
                                        "Activity in Date Range"
                                    } else {
                                        "Activity on Selected Date"
                                    }
                                }
                                {
                                    let range_display_text = if current_mode == DateSelectionMode::DateRange {
                                        if let Some((start, end)) = active_range {
                                            format!("{} \u{2013} {}", format_ymd(start), format_ymd(end))
                                        } else {
                                            "No Date Range Active".to_string()
                                        }
                                    } else {
                                        format_date_pretty(sel_d.0, sel_d.1, sel_d.2)
                                    };
                                    rsx! {
                                        h2 { class: "text-base font-bold text-foreground truncate",
                                            "{range_display_text}"
                                        }
                                    }
                                }
                            }

                            span { class: "rounded-full bg-primary/10 text-primary border border-primary/20 px-2.5 py-0.5 text-xs font-bold shrink-0",
                                "{filtered_items.len()} items"
                            }
                        }

                        // Summary Metric Cards
                        div { class: "grid grid-cols-3 gap-2 mt-3",
                            div { class: "flex flex-col rounded-md border border-blue-500/20 bg-blue-500/5 p-2 text-center",
                                span { class: "text-[10px] font-semibold text-blue-600 dark:text-blue-400 uppercase tracking-wide", "Tickets" }
                                span { class: "text-base font-extrabold text-blue-700 dark:text-blue-300", "{total_tickets}" }
                            }
                            div { class: "flex flex-col rounded-md border border-amber-500/20 bg-amber-500/5 p-2 text-center",
                                span { class: "text-[10px] font-semibold text-amber-600 dark:text-amber-400 uppercase tracking-wide", "Reminders" }
                                span { class: "text-base font-extrabold text-amber-700 dark:text-amber-300", "{total_reminders}" }
                            }
                            div { class: "flex flex-col rounded-md border border-emerald-500/20 bg-emerald-500/5 p-2 text-center",
                                span { class: "text-[10px] font-semibold text-emerald-600 dark:text-emerald-400 uppercase tracking-wide", "Notes" }
                                span { class: "text-base font-extrabold text-emerald-700 dark:text-emerald-300", "{total_notes}" }
                            }
                        }

                        // Entity Filter Tabs: All | Tickets | Reminders | Notes
                        div { class: "flex items-center gap-1.5 mt-3 pt-3 border-t border-border/50",
                            button {
                                r#type: "button",
                                class: if current_filter == ItemFilterType::All {
                                    "rounded-md bg-primary text-primary-foreground font-bold px-2.5 py-1 text-xs cursor-pointer shadow-2xs"
                                } else {
                                    "rounded-md text-muted-foreground hover:bg-muted hover:text-foreground px-2.5 py-1 text-xs cursor-pointer"
                                },
                                onclick: move |_| item_filter.set(ItemFilterType::All),
                                "All ({total_tickets + total_reminders + total_notes})"
                            }
                            button {
                                r#type: "button",
                                class: if current_filter == ItemFilterType::Tickets {
                                    "rounded-md bg-blue-600 text-white font-bold px-2.5 py-1 text-xs cursor-pointer shadow-2xs"
                                } else {
                                    "rounded-md text-muted-foreground hover:bg-muted hover:text-foreground px-2.5 py-1 text-xs cursor-pointer"
                                },
                                onclick: move |_| item_filter.set(ItemFilterType::Tickets),
                                "Tickets ({total_tickets})"
                            }
                            button {
                                r#type: "button",
                                class: if current_filter == ItemFilterType::Reminders {
                                    "rounded-md bg-amber-600 text-white font-bold px-2.5 py-1 text-xs cursor-pointer shadow-2xs"
                                } else {
                                    "rounded-md text-muted-foreground hover:bg-muted hover:text-foreground px-2.5 py-1 text-xs cursor-pointer"
                                },
                                onclick: move |_| item_filter.set(ItemFilterType::Reminders),
                                "Reminders ({total_reminders})"
                            }
                            button {
                                r#type: "button",
                                class: if current_filter == ItemFilterType::Notes {
                                    "rounded-md bg-emerald-600 text-white font-bold px-2.5 py-1 text-xs cursor-pointer shadow-2xs"
                                } else {
                                    "rounded-md text-muted-foreground hover:bg-muted hover:text-foreground px-2.5 py-1 text-xs cursor-pointer"
                                },
                                onclick: move |_| item_filter.set(ItemFilterType::Notes),
                                "Notes ({total_notes})"
                            }
                        }
                    }

                    // Items List Area
                    div { class: "flex-1 overflow-y-auto p-4 flex flex-col gap-2.5 min-h-0",
                        if filtered_items.is_empty() {
                            div { class: "flex flex-col items-center justify-center py-16 text-center text-muted-foreground gap-2",
                                CalendarIcon { class: "size-10 text-muted-foreground/40 stroke-1" }
                                span { class: "text-sm font-semibold text-foreground/80", "No items recorded" }
                                p { class: "text-xs max-w-xs text-muted-foreground",
                                    "No tickets, reminders, or notes were created on this date or range."
                                }
                            }
                        } else {
                            for item in filtered_items.iter() {
                                {
                                    let item_clone = item.clone();
                                    let (border_color, badge_icon, badge_bg, badge_label) = match item.kind {
                                        CalendarItemKind::Ticket => (
                                            "border-l-blue-500",
                                            rsx! { TicketIcon { class: "size-3 text-blue-600 dark:text-blue-400" } },
                                            "bg-blue-500/10 text-blue-600 dark:text-blue-400 border-blue-500/20",
                                            "Ticket",
                                        ),
                                        CalendarItemKind::Reminder => (
                                            "border-l-amber-500",
                                            rsx! { BellIcon { class: "size-3 text-amber-600 dark:text-amber-400" } },
                                            "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/20",
                                            "Reminder",
                                        ),
                                        CalendarItemKind::Note => (
                                            "border-l-emerald-500",
                                            rsx! { StickyNoteIcon { class: "size-3 text-emerald-600 dark:text-emerald-400" } },
                                            "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/20",
                                            "Note",
                                        ),
                                    };

                                    let (status_cls, status_txt) = status_badge_classes(&item.status);

                                    rsx! {
                                        div {
                                            key: "{item.key}",
                                            class: "group relative flex flex-col gap-2 rounded-lg border border-border/80 bg-card p-3 shadow-2xs hover:border-primary/50 hover:shadow-sm transition-all border-l-4 {border_color} cursor-pointer",
                                            onclick: move |_| inspect_item.set(Some(item_clone.data.clone())),

                                            // Top Row: Type & Tag + Time + Action
                                            div { class: "flex items-center justify-between gap-2",
                                                div { class: "flex items-center gap-1.5 flex-wrap min-w-0",
                                                    span { class: "inline-flex items-center gap-1 rounded px-1.5 py-0.5 text-[10px] font-bold border {badge_bg}",
                                                        {badge_icon}
                                                        span { "{badge_label}" }
                                                    }
                                                    if let Some(t) = &item.tag {
                                                        span { class: "font-mono text-[10px] text-muted-foreground truncate",
                                                            "{t}"
                                                        }
                                                    }
                                                }

                                                div { class: "flex items-center gap-2 shrink-0",
                                                    if !item.time_str.is_empty() {
                                                        span { class: "flex items-center gap-1 font-mono text-[10px] text-muted-foreground",
                                                            ClockIcon { class: "size-3" }
                                                            span { "{item.time_str}" }
                                                        }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        class: "opacity-0 group-hover:opacity-100 p-1 text-muted-foreground hover:text-primary transition-opacity cursor-pointer",
                                                        title: "Inspect Details",
                                                        EyeIcon { class: "size-3.5" }
                                                    }
                                                }
                                            }

                                            // Title & Content
                                            div { class: "flex flex-col gap-0.5",
                                                h4 { class: "text-xs font-bold text-foreground leading-snug group-hover:text-primary transition-colors line-clamp-1",
                                                    "{item.title}"
                                                }
                                                if !item.details.is_empty() {
                                                    p { class: "text-[11px] text-muted-foreground line-clamp-2 leading-relaxed",
                                                        "{item.details}"
                                                    }
                                                }
                                            }

                                            // Bottom Metadata Ribbon
                                            div { class: "flex items-center justify-between gap-2 pt-1.5 border-t border-border/40 text-[11px]",
                                                // Employee or Assigned user
                                                div { class: "flex items-center gap-1.5 min-w-0",
                                                    if item.kind == CalendarItemKind::Ticket {
                                                        span { class: "flex size-5 shrink-0 items-center justify-center rounded-full text-[9px] font-extrabold uppercase border shadow-2xs {assignee_color_classes(&item.employee)}",
                                                            "{get_assignee_initials(&item.employee)}"
                                                        }
                                                        span { class: "truncate font-medium text-foreground/80", "{item.subtitle}" }
                                                    } else {
                                                        UsersIcon { class: "size-3 text-muted-foreground shrink-0" }
                                                        span { class: "truncate text-muted-foreground", "{item.subtitle}" }
                                                    }
                                                }

                                                // Status / Priority Badges
                                                div { class: "flex items-center gap-1.5 shrink-0",
                                                    if item.kind == CalendarItemKind::Ticket {
                                                        Badge {
                                                            variant: "{priority_badge_variant(&item.priority)}",
                                                            class: "text-[10px] py-0 h-4.5 px-1.5",
                                                            "{item.priority}"
                                                        }
                                                    }
                                                    span { class: "rounded border px-1.5 py-0 text-[10px] font-medium {status_cls}",
                                                        "{status_txt}"
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

            // ========================================================
            // Detail Inspection Modal Dialog
            // ========================================================
            if let Some(detail) = inspect_item() {
                div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 p-4 backdrop-blur-xs animate-in fade-in-0 duration-150",
                    onclick: move |_| inspect_item.set(None),

                    div {
                        class: "relative w-full max-w-lg rounded-lg border border-border/80 bg-background p-6 shadow-2xl transition-all",
                        onclick: move |e| e.stop_propagation(),

                        match detail {
                            CalendarEntityData::Ticket(t) => {
                                let display_ticket_id = t.server_ticket_id.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| format!("TKT-{}", t.ticket_id));
                                rsx! {
                                    div { class: "flex flex-col gap-4",
                                        div { class: "flex items-start justify-between border-b border-border/60 pb-3",
                                            div { class: "flex flex-col gap-1",
                                                div { class: "flex items-center gap-2",
                                                    span { class: "rounded bg-blue-500/15 text-blue-600 dark:text-blue-400 border border-blue-500/25 px-2 py-0.5 text-xs font-bold",
                                                        "Support Ticket"
                                                    }
                                                    span { class: "font-mono text-xs font-bold text-foreground",
                                                        "{display_ticket_id}"
                                                    }
                                                }
                                                h3 { class: "text-base font-bold text-foreground leading-snug mt-1", "{t.subject}" }
                                            }
                                            button {
                                                r#type: "button",
                                                class: "p-1 rounded text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer",
                                                onclick: move |_| inspect_item.set(None),
                                                XIcon { class: "size-4" }
                                            }
                                        }

                                    div { class: "grid grid-cols-2 gap-3 text-xs bg-muted/20 p-3 rounded-md border border-border/50",
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Category" }
                                            span { class: "font-semibold text-foreground", "{t.category}" }
                                        }
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Priority" }
                                            span { class: "font-semibold text-foreground", "{t.priority}" }
                                        }
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Assigned To" }
                                            span { class: "font-semibold text-foreground", "{t.assigned_to}" }
                                        }
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Status" }
                                            span { class: "font-semibold text-foreground", "{t.kanban_state}" }
                                        }
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Created Date" }
                                            span { class: "font-mono font-medium text-foreground", "{t.created_on}" }
                                        }
                                        {
                                            let emp_clean = if let Some(idx) = t.employee_code.find('(') {
                                                t.employee_code[..idx].trim()
                                            } else {
                                                t.employee_code.trim()
                                            };
                                            let emp_db = crate::db::find_employee_by_code(emp_clean);
                                            let emp_name = emp_db.as_ref().map(|e| e.emp_name.clone()).unwrap_or_else(|| t.employee_code.clone());
                                            rsx! {
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Employee Name" }
                                                    span { class: "font-semibold text-foreground truncate", "{emp_name}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Employee Code" }
                                                    span { class: "font-mono font-medium text-foreground", "{emp_clean}" }
                                                }
                                            }
                                        }
                                    }

                                    if !t.problem_description.is_empty() {
                                        div { class: "flex flex-col gap-1",
                                            span { class: "text-[11px] font-bold uppercase tracking-wider text-muted-foreground", "Problem Description" }
                                            p { class: "text-xs text-foreground/90 bg-card border border-border/60 p-3 rounded-md leading-relaxed whitespace-pre-wrap",
                                                "{t.problem_description}"
                                            }
                                        }
                                    }

                                    div { class: "flex justify-end pt-3 border-t border-border/50",
                                        Button {
                                            variant: "outline",
                                            size: "sm",
                                            onclick: move |_| inspect_item.set(None),
                                            "Close"
                                        }
                                    }
                                }
                            }
                        },
                            CalendarEntityData::Reminder(r) => {
                                rsx! {
                                    div { class: "flex flex-col gap-4",
                                    div { class: "flex items-start justify-between border-b border-border/60 pb-3",
                                        div { class: "flex flex-col gap-1",
                                            div { class: "flex items-center gap-2",
                                                span { class: "rounded bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/25 px-2 py-0.5 text-xs font-bold",
                                                    "Reminder"
                                                }
                                                span { class: "font-mono text-xs font-bold text-foreground",
                                                    "#{r.reminder_id}"
                                                }
                                            }
                                            h3 { class: "text-base font-bold text-foreground leading-snug mt-1", "{r.reminder_message}" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "p-1 rounded text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer",
                                            onclick: move |_| inspect_item.set(None),
                                            XIcon { class: "size-4" }
                                        }
                                    }

                                    {
                                        let emp_code_clean = r.employee_code.trim();
                                        let emp_db = crate::db::find_employee_by_code(emp_code_clean);
                                        let recipient_name = emp_db.as_ref().map(|e| e.emp_name.clone()).unwrap_or_else(|| r.employee_name.clone());
                                        let recipient_email = emp_db.as_ref().map(|e| e.emp_email.clone()).filter(|e| !e.is_empty()).unwrap_or_else(|| r.email_address.clone());
                                        rsx! {
                                            div { class: "grid grid-cols-2 gap-3 text-xs bg-muted/20 p-3 rounded-md border border-border/50",
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Recipient Name" }
                                                    span { class: "font-semibold text-foreground", "{recipient_name}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Employee Code" }
                                                    span { class: "font-mono font-semibold text-foreground", "{r.employee_code}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Status" }
                                                    span { class: "font-semibold text-foreground", "{r.status}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Scheduled Send Time" }
                                                    span { class: "font-mono font-medium text-foreground", title: "{r.send_at}", "{crate::state::format_display_12hr(&r.send_at)}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Repeat Frequency" }
                                                    span { class: "font-mono font-semibold text-foreground uppercase", "{r.repeat_frequency}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Created At" }
                                                    span { class: "font-mono font-medium text-foreground", title: "{r.created_at}", "{crate::state::format_display_12hr(&r.created_at)}" }
                                                }
                                                div { class: "flex flex-col gap-0.5",
                                                    span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Recipient Email" }
                                                    span { class: "font-mono text-[11px] text-foreground truncate", "{recipient_email}" }
                                                }
                                                if !r.cc_emails.is_empty() {
                                                    div { class: "col-span-2 flex flex-col gap-0.5 pt-1 border-t border-border/30",
                                                        span { class: "text-muted-foreground text-[10px] uppercase font-bold", "CC Recipients" }
                                                        span { class: "font-mono text-[11px] text-foreground truncate", "{r.cc_emails}" }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "flex justify-end pt-3 border-t border-border/50",
                                        Button {
                                            variant: "outline",
                                            size: "sm",
                                            onclick: move |_| inspect_item.set(None),
                                            "Close"
                                        }
                                    }
                                }
                            }
                        },
                            CalendarEntityData::Note(n) => {
                                rsx! {
                                    div { class: "flex flex-col gap-4",
                                    div { class: "flex items-start justify-between border-b border-border/60 pb-3",
                                        div { class: "flex flex-col gap-1",
                                            span { class: "rounded bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/25 px-2 py-0.5 text-xs font-bold w-fit",
                                                "Quick Note"
                                            }
                                            h3 { class: "text-base font-bold text-foreground leading-snug mt-1", "Note #{n.id}" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "p-1 rounded text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer",
                                            onclick: move |_| inspect_item.set(None),
                                            XIcon { class: "size-4" }
                                        }
                                    }

                                    div { class: "grid grid-cols-2 gap-3 text-xs bg-muted/20 p-3 rounded-md border border-border/50",
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Employee Code" }
                                            span { class: "font-mono font-semibold text-foreground", "{n.emp_code}" }
                                        }
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-muted-foreground text-[10px] uppercase font-bold", "Created At" }
                                            span { class: "font-mono font-medium text-foreground", "{n.created_at}" }
                                        }
                                    }

                                    div { class: "flex flex-col gap-1",
                                        span { class: "text-[11px] font-bold uppercase tracking-wider text-muted-foreground", "Note Content" }
                                        p { class: "text-xs text-foreground/90 bg-card border border-border/60 p-3 rounded-md leading-relaxed whitespace-pre-wrap",
                                            "{n.content}"
                                        }
                                    }

                                    div { class: "flex justify-end pt-3 border-t border-border/50",
                                        Button {
                                            variant: "outline",
                                            size: "sm",
                                            onclick: move |_| inspect_item.set(None),
                                            "Close"
                                        }
                                    }
                                }
                            }
                        },
                        }
                    }
                }
            }
        }
    }
}

// ====================================================================
// Month Calendar Grid View
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct MonthGridProps {
    year: u32,
    month: u32,
    today: (u32, u32, u32),
    selected_date: (u32, u32, u32),
    date_range: Option<DateRange>,
    selection_mode: DateSelectionMode,
    entries: Vec<CalendarEntry>,
    on_day_click: EventHandler<(u32, u32, u32)>,
}

#[component]
fn MonthCalendarView(props: MonthGridProps) -> Element {
    let year = props.year;
    let month = props.month;
    let today = props.today;
    let selected = props.selected_date;
    let range = props.date_range;
    let mode = props.selection_mode;

    let total_days = days_in_month(year, month);
    let start_sun = first_day_of_month(year, month);
    // Convert Sunday=0 to Monday=0
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
        div { class: "flex flex-col rounded-lg border border-border/70 bg-card overflow-hidden shadow-2xs select-none",

            // Day of Week Header (Mon - Sun)
            div { class: "grid grid-cols-7 border-b border-border/60 bg-muted/40 text-center text-xs font-bold text-muted-foreground",
                for day_name in ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"] {
                    div { key: "{day_name}", class: "py-2.5 uppercase tracking-wider",
                        "{day_name}"
                    }
                }
            }

            // Month Grid
            div { class: "grid grid-cols-7 divide-x divide-y divide-border/40",

                // Previous month padding
                for i in 0..start_col {
                    {
                        let day_num = prev_days_count - (start_col as u32 - 1) + (i as u32);
                        let day_date = (prev_y, prev_m, day_num);
                        let on_click = props.on_day_click;
                        rsx! {
                            div {
                                key: "prev-{day_num}",
                                class: "min-h-[105px] p-2 bg-muted/15 opacity-40 hover:opacity-75 transition-opacity cursor-pointer flex flex-col justify-between",
                                onclick: move |_| on_click.call(day_date),
                                span { class: "text-xs font-mono text-muted-foreground", "{day_num}" }
                            }
                        }
                    }
                }

                // Active days of current month
                for day in 1..=total_days {
                    {
                        let day_date = (year, month, day);
                        let is_today = day_date == today;
                        let on_click = props.on_day_click;

                        // Check range highlighting
                        let (is_in_range, is_range_start, is_range_end) = match (mode, range) {
                            (DateSelectionMode::DateRange, Some((s, e))) => {
                                (day_date >= s && day_date <= e, day_date == s, day_date == e)
                            }
                            _ => (false, false, false),
                        };

                        let is_single_selected = mode == DateSelectionMode::SingleDate && day_date == selected;

                        // Cell styling
                        let cell_cls = if is_range_start && is_range_end {
                            "bg-primary/20 ring-2 ring-inset ring-primary"
                        } else if is_range_start {
                            "bg-primary/15 ring-2 ring-inset ring-primary rounded-l-md"
                        } else if is_range_end {
                            "bg-primary/15 ring-2 ring-inset ring-primary rounded-r-md"
                        } else if is_in_range {
                            "bg-primary/10"
                        } else if is_single_selected {
                            "bg-primary/10 ring-2 ring-inset ring-primary/80"
                        } else if is_today {
                            "bg-primary/[0.03]"
                        } else {
                            "hover:bg-muted/30"
                        };

                        // Count items created on this day
                        let tickets_count = props.entries.iter().filter(|e| e.date == day_date && e.kind == CalendarItemKind::Ticket).count();
                        let reminders_count = props.entries.iter().filter(|e| e.date == day_date && e.kind == CalendarItemKind::Reminder).count();
                        let notes_count = props.entries.iter().filter(|e| e.date == day_date && e.kind == CalendarItemKind::Note).count();

                        rsx! {
                            div {
                                key: "cur-{day}",
                                class: "relative min-h-[110px] p-2 flex flex-col justify-between transition-colors cursor-pointer {cell_cls}",
                                onclick: move |_| on_click.call(day_date),

                                // Day Header
                                div { class: "flex items-center justify-between",
                                    if is_today {
                                        span { class: "flex size-6 items-center justify-center rounded-full bg-primary font-bold text-xs text-primary-foreground shadow-2xs",
                                            "{day}"
                                        }
                                    } else if is_single_selected || is_range_start || is_range_end {
                                        span { class: "flex size-6 items-center justify-center rounded-full border-2 border-primary font-bold text-xs text-primary bg-background shadow-2xs",
                                            "{day}"
                                        }
                                    } else {
                                        span { class: "text-xs font-mono font-medium text-foreground/80",
                                            "{day}"
                                        }
                                    }

                                    // Total created indicator
                                    if tickets_count + reminders_count + notes_count > 0 {
                                        span { class: "text-[10px] font-bold text-muted-foreground/70",
                                            "{tickets_count + reminders_count + notes_count}"
                                        }
                                    }
                                }

                                // Badges stack for created items
                                div { class: "flex flex-col gap-1 mt-1.5",
                                    if tickets_count > 0 {
                                        {
                                            let s_suffix = if tickets_count > 1 { "s" } else { "" };
                                            rsx! {
                                                span { class: "inline-flex items-center gap-1 rounded bg-blue-500/15 text-blue-700 dark:text-blue-300 border border-blue-500/25 px-1.5 py-0.5 text-[10px] font-semibold truncate",
                                                    TicketIcon { class: "size-2.5 shrink-0" }
                                                    span { "{tickets_count} ticket{s_suffix}" }
                                                }
                                            }
                                        }
                                    }
                                    if reminders_count > 0 {
                                        {
                                            let s_suffix = if reminders_count > 1 { "s" } else { "" };
                                            rsx! {
                                                span { class: "inline-flex items-center gap-1 rounded bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/25 px-1.5 py-0.5 text-[10px] font-semibold truncate",
                                                    BellIcon { class: "size-2.5 shrink-0" }
                                                    span { "{reminders_count} reminder{s_suffix}" }
                                                }
                                            }
                                        }
                                    }
                                    if notes_count > 0 {
                                        {
                                            let s_suffix = if notes_count > 1 { "s" } else { "" };
                                            rsx! {
                                                span { class: "inline-flex items-center gap-1 rounded bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/25 px-1.5 py-0.5 text-[10px] font-semibold truncate",
                                                    StickyNoteIcon { class: "size-2.5 shrink-0" }
                                                    span { "{notes_count} note{s_suffix}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Next month trailing padding
                for j in 0..trailing_count {
                    {
                        let day_num = j + 1;
                        let day_date = (next_y, next_m, day_num);
                        let on_click = props.on_day_click;
                        rsx! {
                            div {
                                key: "next-{day_num}",
                                class: "min-h-[105px] p-2 bg-muted/15 opacity-40 hover:opacity-75 transition-opacity cursor-pointer flex flex-col justify-between",
                                onclick: move |_| on_click.call(day_date),
                                span { class: "text-xs font-mono text-muted-foreground", "{day_num}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ====================================================================
// Schedule Timeline / Agenda List View
// ====================================================================

#[derive(Props, Clone, PartialEq)]
struct ScheduleProps {
    year: u32,
    month: u32,
    today: (u32, u32, u32),
    selected_date: (u32, u32, u32),
    date_range: Option<DateRange>,
    selection_mode: DateSelectionMode,
    entries: Vec<CalendarEntry>,
    on_select_day: EventHandler<(u32, u32, u32)>,
    on_open_item: EventHandler<CalendarEntityData>,
}

#[component]
fn ScheduleTimelineListView(props: ScheduleProps) -> Element {
    let mode = props.selection_mode;
    let range = props.date_range;
    let selected = props.selected_date;

    // Filter entries based on range/date
    let active_entries: Vec<CalendarEntry> = props
        .entries
        .iter()
        .filter(|e| match mode {
            DateSelectionMode::SingleDate => e.date == selected,
            DateSelectionMode::DateRange => {
                if let Some((s, end)) = range {
                    e.date >= s && e.date <= end
                } else {
                    e.date == selected
                }
            }
        })
        .cloned()
        .collect();

    rsx! {
        div { class: "flex flex-col gap-4 p-2",
            if active_entries.is_empty() {
                div { class: "flex flex-col items-center justify-center py-24 text-muted-foreground gap-2",
                    CalendarIcon { class: "size-12 text-muted-foreground/30 stroke-1" }
                    span { class: "text-sm font-bold text-foreground/80", "No activities recorded" }
                    p { class: "text-xs text-muted-foreground", "No tickets, reminders, or notes found for this timeframe." }
                }
            } else {
                for item in active_entries.iter() {
                    {
                        let item_data = item.data.clone();
                        let on_open = props.on_open_item;
                        let (type_cls, type_label) = match item.kind {
                            CalendarItemKind::Ticket => ("bg-blue-500/10 text-blue-600 border-blue-500/25", "Ticket"),
                            CalendarItemKind::Reminder => ("bg-amber-500/10 text-amber-600 border-amber-500/25", "Reminder"),
                            CalendarItemKind::Note => ("bg-emerald-500/10 text-emerald-600 border-emerald-500/25", "Note"),
                        };

                        rsx! {
                            div {
                                key: "{item.key}",
                                class: "flex items-start gap-4 p-3.5 rounded-lg border border-border/70 bg-card hover:border-primary/50 transition-all cursor-pointer shadow-2xs",
                                onclick: move |_| on_open.call(item_data.clone()),

                                // Date badge
                                div { class: "flex flex-col items-center justify-center rounded-md border border-border bg-muted/30 px-3 py-2 shrink-0 min-w-[70px]",
                                    span { class: "text-[10px] font-bold uppercase tracking-wider text-muted-foreground",
                                        "{SHORT_MONTH_NAMES[(item.date.1 - 1) as usize]}"
                                    }
                                    span { class: "text-lg font-extrabold text-foreground",
                                        "{item.date.2}"
                                    }
                                }

                                // Details
                                div { class: "flex flex-1 flex-col gap-1 min-w-0",
                                    div { class: "flex items-center gap-2 flex-wrap",
                                        span { class: "rounded px-1.5 py-0.5 text-[10px] font-bold border {type_cls}",
                                            "{type_label}"
                                        }
                                        if let Some(t) = &item.tag {
                                            span { class: "font-mono text-[11px] text-muted-foreground", "{t}" }
                                        }
                                        if !item.time_str.is_empty() {
                                            span { class: "text-[10px] text-muted-foreground font-mono ml-auto", "{item.time_str}" }
                                        }
                                    }
                                    h4 { class: "text-sm font-bold text-foreground leading-snug truncate", "{item.title}" }
                                    p { class: "text-xs text-muted-foreground line-clamp-2", "{item.details}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
