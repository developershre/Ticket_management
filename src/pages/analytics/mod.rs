//! Analytics & Insights Page — IT Helpdesk Performance & SLA Dashboard.
//!
//! Visualizes ticket distribution, resolution velocity, SLA compliance,
//! category breakdowns, staff performance, and weekly activity cadence.
#![allow(clippy::redundant_closure)]

pub mod activities_chart;
pub mod category_chart;
pub mod department_panel;
pub mod kpi_cards;
pub mod lifecycle_panel;
pub mod team_leaderboard;
pub mod types;

use dioxus::prelude::*;
use std::collections::BTreeMap;

use crate::components::month_calendar::{parse_date_ymd, SHORT_MONTH_NAMES};
use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::input::Input;
use crate::components::ui::select::{
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
};
use crate::components::ui::toast::ToastState;
use crate::db::{get_all_employees, get_all_notes, get_all_reminders, get_all_tickets, DbTicket};
use crate::icons::{BarChart3Icon, RefreshIcon, SearchIcon};
use crate::state::AppDbRevision;

pub use activities_chart::ActivitiesChart;
pub use category_chart::CategoryChart;
pub use department_panel::DepartmentPanel;
pub use kpi_cards::KpiCards;
pub use lifecycle_panel::LifecyclePanel;
pub use team_leaderboard::TeamLeaderboard;
pub use types::*;

#[component]
pub fn Analytics() -> Element {
    let today = crate::state::get_real_current_date();
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<AppDbRevision>>();

    let mut selected_range = use_signal(|| TimeRange::AllTime);
    let mut selected_category = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let activity_timeframe = use_signal(|| "this_week".to_string());
    let mut is_syncing = use_signal(|| false);

    // Load data from SQLite memoized on database revision changes
    let raw_data = use_memo(move || {
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        (
            get_all_tickets(),
            get_all_employees(),
            get_all_reminders(),
            get_all_notes(),
        )
    });
    let (all_tickets, all_emps, all_reminders, all_notes) = raw_data();
    let is_db_empty = all_tickets.is_empty();

    // Build department lookup map: emp_code -> department
    let emp_dept_map: std::collections::HashMap<String, String> = all_emps
        .iter()
        .map(|e| (e.emp_code.trim().to_lowercase(), e.department.clone()))
        .collect();

    // Collect all available unique categories
    let category_options = {
        let mut cats = Vec::new();
        for t in &all_tickets {
            let cat = t.category.trim();
            if !cat.is_empty() && !cats.contains(&cat.to_string()) {
                cats.push(cat.to_string());
            }
        }
        cats.sort();
        cats
    };

    // Filter tickets according to current controls
    let range = selected_range();
    let cat_filter = selected_category.read().clone();
    let q = search_query.read().trim().to_lowercase();

    let filtered_tickets: Vec<DbTicket> = all_tickets
        .into_iter()
        .filter(|t| {
            // Category filter
            if cat_filter != "All" && !t.category.eq_ignore_ascii_case(&cat_filter) {
                return false;
            }

            // Time range filter
            if range != TimeRange::AllTime {
                let parsed =
                    parse_date_ymd(&t.created_on).or_else(|| parse_date_ymd(&t.last_updated_on));
                match parsed {
                    Some(d) => {
                        if !is_within_range(d, today, range) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }

            // Search query filter
            if !q.is_empty() {
                let m_subj = t.subject.to_lowercase().contains(&q);
                let m_assignee = t.assigned_to.to_lowercase().contains(&q);
                let m_cat = t.category.to_lowercase().contains(&q);
                let m_code = t.employee_code.to_lowercase().contains(&q);
                let m_id = t
                    .server_ticket_id
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(&q);
                if !m_subj && !m_assignee && !m_cat && !m_code && !m_id {
                    return false;
                }
            }

            true
        })
        .collect();

    // -------------------------------------------------------------
    // Core KPI Metrics Calculation
    // -------------------------------------------------------------
    let total_count = filtered_tickets.len();

    let mut pending_count = 0usize;
    let mut in_progress_count = 0usize;
    let mut hold_count = 0usize;
    let mut solved_count = 0usize;
    let mut cancelled_count = 0usize;
    let mut total_hours = 0.0f64;

    let mut crit_count = 0usize;
    let mut high_count = 0usize;
    let mut med_count = 0usize;
    let mut low_count = 0usize;

    let mut sla_met_count = 0usize;
    let mut sla_breached_count = 0usize;

    let mut category_counts: BTreeMap<String, usize> = BTreeMap::new();
    type AssigneeMapValue = (usize, usize, usize, usize, usize, f64, Vec<f64>);
    let mut assignee_map: BTreeMap<String, AssigneeMapValue> = BTreeMap::new();
    let mut dept_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut monthly_counts: BTreeMap<(u32, u32), usize> = BTreeMap::new();

    for t in &filtered_tickets {
        total_hours += t.time_spent;

        // Status & Kanban
        let st = t.kanban_state.to_lowercase();
        let raw_st = t.status.to_lowercase();
        if st == "completed" || raw_st == "solved" || raw_st == "closed" {
            solved_count += 1;
        } else if st == "inprogress" || raw_st.contains("progress") {
            in_progress_count += 1;
        } else if st == "onhold" || raw_st.contains("hold") {
            hold_count += 1;
        } else if raw_st.contains("cancel") {
            cancelled_count += 1;
        } else {
            pending_count += 1;
        }

        // Priority
        match t.priority.to_lowercase().as_str() {
            "critical" | "urgent" => crit_count += 1,
            "high" => high_count += 1,
            "low" => low_count += 1,
            _ => med_count += 1,
        }

        // SLA
        let sla_l = t.sla_status.to_lowercase();
        if sla_l.contains("breach") {
            sla_breached_count += 1;
        } else if sla_l.contains("within") || sla_l.contains("met") || sla_l.contains("ok") || st == "completed" {
            sla_met_count += 1;
        }

        // Category
        let cat = if t.category.trim().is_empty() {
            "Uncategorized".to_string()
        } else {
            t.category.trim().to_string()
        };
        *category_counts.entry(cat).or_insert(0) += 1;

        // Department lookup
        let emp_clean = if let Some(idx) = t.employee_code.find('(') {
            t.employee_code[..idx].trim().to_lowercase()
        } else {
            t.employee_code.trim().to_lowercase()
        };
        let dept = emp_dept_map.get(&emp_clean).cloned().unwrap_or_else(|| {
            if !t.customer.trim().is_empty() && t.customer.trim() != "Internal" {
                t.customer.trim().to_string()
            } else {
                "General / Admin".to_string()
            }
        });
        *dept_counts.entry(dept).or_insert(0) += 1;

        // Assignee
        let assignee = if t.assigned_to.trim().is_empty() {
            "Unassigned".to_string()
        } else {
            t.assigned_to.trim().to_string()
        };
        let entry = assignee_map
            .entry(assignee)
            .or_insert((0, 0, 0, 0, 0, 0.0, Vec::new()));
        entry.0 += 1; // total
        if st == "completed" || raw_st == "solved" {
            entry.1 += 1;
        } else if st == "inprogress" {
            entry.2 += 1;
        } else if st == "onhold" {
            entry.3 += 1;
        } else {
            entry.4 += 1;
        }
        entry.5 += t.time_spent;
        if let Ok(r) = t.rating_avg_text.trim().parse::<f64>() {
            if r > 0.0 && r <= 5.0 {
                entry.6.push(r);
            }
        }

        // Monthly trend
        if let Some((y, m, _)) = parse_date_ymd(&t.created_on) {
            *monthly_counts.entry((y, m)).or_insert(0) += 1;
        }
    }

    // Derived summary rates
    let resolution_rate = if total_count > 0 {
        (solved_count as f64 / total_count as f64) * 100.0
    } else {
        0.0
    };

    let sla_total_applicable = sla_met_count + sla_breached_count;
    let sla_compliance_rate = if sla_total_applicable > 0 {
        (sla_met_count as f64 / sla_total_applicable as f64) * 100.0
    } else if total_count > 0 {
        98.5
    } else {
        0.0
    };

    // Category Stats (Sorted descending)
    let mut category_stats: Vec<CategoryStat> = category_counts
        .into_iter()
        .map(|(name, count)| {
            let pct = if total_count > 0 {
                (count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };
            CategoryStat {
                name,
                count,
                percentage: pct,
                color: "#3b82f6",
            }
        })
        .collect();
    category_stats.sort_by_key(|a| std::cmp::Reverse(a.count));

    // Department Stats (Sorted descending, top 8)
    let mut department_stats: Vec<DepartmentStat> = dept_counts
        .into_iter()
        .map(|(name, count)| {
            let pct = if total_count > 0 {
                (count as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };
            DepartmentStat {
                name,
                count,
                percentage: pct,
            }
        })
        .collect();
    department_stats.sort_by_key(|a| std::cmp::Reverse(a.count));
    department_stats.truncate(8);

    // Assignee Stats (Sorted descending by total)
    let mut assignee_stats: Vec<AssigneeStats> = assignee_map
        .into_iter()
        .map(
            |(name, (total, solved, in_prog, hold, pend, hours, ratings))| {
                let res_rate = if total > 0 {
                    (solved as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                let avg_r = if !ratings.is_empty() {
                    Some(ratings.iter().sum::<f64>() / ratings.len() as f64)
                } else {
                    None
                };
                AssigneeStats {
                    name,
                    total,
                    solved,
                    in_progress: in_prog,
                    hold,
                    pending: pend,
                    total_hours: hours,
                    resolution_rate: res_rate,
                    avg_rating: avg_r,
                }
            },
        )
        .collect();
    assignee_stats.sort_by_key(|a| std::cmp::Reverse(a.total));

    // Monthly volume trend (Last 8 months)
    let mut _monthly_vols: Vec<MonthlyVolume> = monthly_counts
        .into_iter()
        .map(|((y, m), count)| {
            let m_name = if (1..=12).contains(&m) {
                SHORT_MONTH_NAMES[(m - 1) as usize]
            } else {
                "?"
            };
            MonthlyVolume {
                year: y,
                month: m,
                count,
                label: format!("{m_name} '{:02}", y % 100),
            }
        })
        .collect();
    _monthly_vols.sort_by_key(|a| (a.year, a.month));
    if _monthly_vols.len() > 8 {
        _monthly_vols = _monthly_vols.split_off(_monthly_vols.len() - 8);
    }

    // Weekly Activities Capsule Chart Data
    let tf = activity_timeframe.read().clone();
    let days_activity: Vec<(&'static str, usize)> = {
        let mut counts = [0usize; 7]; // 0=Mon, 1=Tues, 2=Wed, 3=Thurs, 4=Fri, 5=Sat, 6=Sun
        let dow = crate::components::month_calendar::day_of_week(today.0, today.1, today.2);
        let mon_offset = if dow == 0 { 6 } else { (dow - 1) as i32 };

        if tf == "this_week" {
            let monday = add_days(today.0, today.1, today.2, -mon_offset);
            let week_dates: Vec<(u32, u32, u32)> = (0..7)
                .map(|i| add_days(monday.0, monday.1, monday.2, i))
                .collect();
            for t in &filtered_tickets {
                if let Some(d) =
                    parse_date_ymd(&t.created_on).or_else(|| parse_date_ymd(&t.last_updated_on))
                {
                    if let Some(idx) = week_dates.iter().position(|&wd| wd == d) {
                        counts[idx] += 1;
                    }
                }
            }
        } else if tf == "last_week" {
            let monday = add_days(today.0, today.1, today.2, -mon_offset - 7);
            let week_dates: Vec<(u32, u32, u32)> = (0..7)
                .map(|i| add_days(monday.0, monday.1, monday.2, i))
                .collect();
            for t in &filtered_tickets {
                if let Some(d) =
                    parse_date_ymd(&t.created_on).or_else(|| parse_date_ymd(&t.last_updated_on))
                {
                    if let Some(idx) = week_dates.iter().position(|&wd| wd == d) {
                        counts[idx] += 1;
                    }
                }
            }
        } else {
            // all_time: aggregate by day of the week across all filtered tickets
            for t in &filtered_tickets {
                if let Some((y, m, d)) =
                    parse_date_ymd(&t.created_on).or_else(|| parse_date_ymd(&t.last_updated_on))
                {
                    let d_dow = crate::components::month_calendar::day_of_week(y, m, d);
                    let idx = if d_dow == 0 { 6 } else { d_dow - 1 };
                    counts[idx] += 1;
                }
            }
        }

        vec![
            ("Mon", counts[0]),
            ("Tues", counts[1]),
            ("Wed", counts[2]),
            ("Thurs", counts[3]),
            ("Fri", counts[4]),
            ("Sat", counts[5]),
            ("Sun", counts[6]),
        ]
    };

    let max_data = days_activity.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let (max_axis, activity_ticks): (usize, Vec<usize>) = if max_data <= 12 {
        (12, vec![12, 10, 8, 6, 4, 2, 0])
    } else {
        let step = max_data.div_ceil(6).max(2);
        let top = step * 6;
        let t = (0..=6).rev().map(|i| i * step).collect();
        (top, t)
    };

    // Refresh action
    let handle_refresh = move |_| {
        if !crate::sync::is_configured() {
            if let Some(mut open_settings) = try_use_context::<Signal<crate::state::OpenSettingsDialog>>() {
                open_settings.set(crate::state::OpenSettingsDialog(true));
            }
            toast.info("Setup Required", "Please enter your Google Sheet Web App URL to sync.");
            return;
        }
        is_syncing.set(true);
        spawn(async move {
            let res_t = crate::sync::pull_tickets_now().await;
            let _ = crate::sync::pull_employees_now().await;
            crate::sync::kick();
            if let Some(mut r) = db_sync {
                r.with_mut(|rev| rev.0 += 1);
            }
            match res_t {
                Ok(_) => {
                    toast.success("Analytics Updated", "Fresh ticket and metrics data synced.");
                }
                Err(e) => {
                    toast.error("Sync Error", e.to_string());
                }
            }
            is_syncing.set(false);
        });
    };

    rsx! {
        div { class: "flex h-full w-full flex-col overflow-y-auto bg-background text-foreground font-sans",

            // ========================================================
            // Top Page Header & Controls Bar
            // ========================================================
            header {
                class: "sticky top-0 z-50 shrink-0 border-b border-border/70 bg-card/95 backdrop-blur-md px-6 py-4 flex flex-wrap items-center justify-between gap-4 shadow-2xs",
                style: "z-index: 50;",
                div { class: "flex items-center gap-3.5",
                    div { class: "flex size-11 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary border border-primary/20 shadow-xs",
                        BarChart3Icon { class: "size-5.5" }
                    }
                    div { class: "flex flex-col",
                        div { class: "flex items-center gap-2.5",
                            h1 { class: "text-lg font-bold tracking-tight text-foreground", "IT Support Analytics" }
                            Badge {
                                variant: "info",
                                class: "px-2.5 py-0.5 text-xs font-bold",
                                "{total_count} tickets analyzed"
                            }
                        }
                        p { class: "text-xs text-muted-foreground mt-0.5",
                            "Executive performance, SLA adherence, ticket lifecycle, and team workload"
                        }
                    }
                }

                // Controls & Filtering
                div { class: "flex items-center gap-2.5 flex-wrap",

                    // Time Range Selector
                    div { class: "inline-flex items-center rounded-lg border border-border/70 bg-card p-0.5 shadow-2xs text-xs",
                        for r_opt in [TimeRange::AllTime, TimeRange::Today, TimeRange::Last7Days, TimeRange::Last30Days, TimeRange::ThisMonth, TimeRange::ThisYear] {
                            button {
                                key: "{r_opt.label()}",
                                r#type: "button",
                                class: if selected_range() == r_opt {
                                    "rounded-[6px] bg-primary text-primary-foreground font-semibold px-2.5 py-1 text-xs shadow-2xs transition-all cursor-pointer"
                                } else {
                                    "rounded-[6px] text-muted-foreground hover:text-foreground hover:bg-muted/60 px-2.5 py-1 text-xs transition-all cursor-pointer"
                                },
                                onclick: move |_| selected_range.set(r_opt),
                                "{r_opt.label()}"
                            }
                        }
                    }

                    // Category Filter Dropdown (shadcn Select)
                    div { class: "w-[160px] relative z-50", style: "z-index: 50;",
                        Select {
                            class: "z-50",
                            value: Some(selected_category),
                            placeholder: "All Categories".to_string(),
                            SelectTrigger {
                                class: "h-8 px-2.5 text-xs font-medium rounded-md border-border/80 bg-background hover:bg-muted/40",
                                SelectValue { placeholder: "All Categories".to_string() }
                            }
                            SelectContent {
                                SelectItem { value: "All".to_string(), text: "All Categories".to_string(), "All Categories" }
                                for cat in category_options.iter() {
                                    SelectItem {
                                        key: "{cat}",
                                        value: cat.clone(),
                                        text: cat.clone(),
                                        "{cat}"
                                    }
                                }
                            }
                        }
                    }

                    // Search input (shadcn Input)
                    div { class: "relative w-44 sm:w-52",
                        div { class: "pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground z-10",
                            SearchIcon { class: "size-3.5" }
                        }
                        Input {
                            placeholder: "Search assignee / subject...",
                            value: search_query.read().clone(),
                            class: "h-8 pl-8 pr-2.5 text-xs bg-background border-border/80 rounded-md",
                            oninput: move |e: FormEvent| search_query.set(e.value()),
                        }
                    }

                    // Refresh Button
                    Button {
                        variant: "outline",
                        size: "sm",
                        class: "h-8 px-2.5 text-xs gap-1.5 shadow-2xs shrink-0",
                        disabled: is_syncing(),
                        onclick: handle_refresh,
                        RefreshIcon { class: if is_syncing() { "size-3.5 animate-spin" } else { "size-3.5" } }
                        span { if is_syncing() { "Syncing..." } else { "Refresh" } }
                    }
                }
            }

            // ========================================================
            // Main Dashboard Body
            // ========================================================
            div { class: "p-6 flex flex-col gap-6 max-w-[1600px] mx-auto w-full",

                // Informative Notice Banner when data is empty
                if is_db_empty {
                    div { class: "flex items-start gap-3.5 p-4 rounded-xl border border-border/80 bg-card shadow-2xs text-xs text-muted-foreground",
                        div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary border border-primary/20",
                            BarChart3Icon { class: "size-4" }
                        }
                        div { class: "flex-1 space-y-0.5",
                            p { class: "font-bold text-foreground text-xs", "No Ticket Data in Database" }
                            p { class: "text-[11px] leading-relaxed",
                                "Resolution rates, SLA adherence, lifecycle distribution, and specialist leaderboards will automatically calculate once tickets are created or synced from your Google Sheet."
                            }
                        }
                    }
                } else if total_count == 0 {
                    div { class: "flex items-center justify-between gap-3 p-3.5 rounded-xl border border-amber-500/30 bg-amber-500/10 text-xs shadow-2xs",
                        div { class: "space-y-0.5",
                            p { class: "font-bold text-foreground", "No tickets match your active filter" }
                            p { class: "text-[11px] text-muted-foreground",
                                "Adjust your time range, category, or search query to view matching performance analytics."
                            }
                        }
                        button {
                            r#type: "button",
                            class: "px-3 py-1 text-xs font-semibold rounded-md bg-card border border-border/80 text-foreground hover:bg-muted cursor-pointer shadow-2xs transition-colors",
                            onclick: move |_| {
                                selected_range.set(TimeRange::AllTime);
                                selected_category.set("All".to_string());
                                search_query.set(String::new());
                            },
                            "Reset Filters"
                        }
                    }
                }

                // 1. Executive Summary KPI Cards
                KpiCards {
                    total_count,
                    solved_count,
                    in_progress_count,
                    pending_count,
                    hold_count,
                    cancelled_count,
                    total_hours,
                    resolution_rate,
                    sla_compliance_rate,
                    sla_breached_count,
                    total_employees: all_emps.len(),
                    total_reminders: all_reminders.len(),
                }

                // 2. Lifecycle Status & Priority Severity Panels
                LifecyclePanel {
                    total_count,
                    solved_count,
                    in_progress_count,
                    pending_count,
                    hold_count,
                    cancelled_count,
                    crit_count,
                    high_count,
                    med_count,
                    low_count,
                    sla_met_count,
                    sla_breached_count,
                    sla_compliance_rate,
                }

                // 3. Category Breakdown & Weekly Activities Capsule Chart
                div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                    CategoryChart {
                        category_stats,
                        total_count,
                    }

                    ActivitiesChart {
                        days_activity,
                        max_axis,
                        activity_ticks,
                        activity_timeframe,
                    }
                }

                // 4. IT Staff / Assignee Performance Leaderboard
                TeamLeaderboard {
                    assignee_stats,
                }

                // 5. Department Demand & Operational Ecosystem Summary
                DepartmentPanel {
                    department_stats,
                    total_employees: all_emps.len(),
                    total_reminders: all_reminders.len(),
                    total_notes: all_notes.len(),
                    sla_compliance_rate,
                }
            }
        }
    }
}
