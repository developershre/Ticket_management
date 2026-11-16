use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::input::Input;
use crate::components::ui::select::SimpleSelect;
use crate::icons::{FilterIcon, XIcon};
use crate::pages::tickets::types::TicketItem;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AdvancedFilterState {
    pub is_open: Signal<bool>,
    pub status: Signal<String>,
    pub priority: Signal<String>,
    pub category: Signal<String>,
    pub assignee: Signal<String>,
    pub stage: Signal<String>,
    pub sla_status: Signal<String>,
    pub date_preset: Signal<String>,
    pub has_attachment: Signal<String>,
    pub rating: Signal<String>,
    pub requester: Signal<String>,
}

impl AdvancedFilterState {
    pub fn new() -> Self {
        Self {
            is_open: use_signal(|| false),
            status: use_signal(|| "All".to_string()),
            priority: use_signal(|| "All".to_string()),
            category: use_signal(|| "All".to_string()),
            assignee: use_signal(|| "All".to_string()),
            stage: use_signal(|| "All".to_string()),
            sla_status: use_signal(|| "All".to_string()),
            date_preset: use_signal(|| "all".to_string()),
            has_attachment: use_signal(|| "all".to_string()),
            rating: use_signal(|| "all".to_string()),
            requester: use_signal(String::new),
        }
    }

    pub fn active_count(&self) -> usize {
        let mut count = 0;
        if *self.status.read() != "All" && !self.status.read().is_empty() {
            count += 1;
        }
        if *self.priority.read() != "All" && !self.priority.read().is_empty() {
            count += 1;
        }
        if *self.category.read() != "All" && !self.category.read().is_empty() {
            count += 1;
        }
        if *self.assignee.read() != "All" && !self.assignee.read().is_empty() {
            count += 1;
        }
        if *self.stage.read() != "All" && !self.stage.read().is_empty() {
            count += 1;
        }
        if *self.sla_status.read() != "All" && !self.sla_status.read().is_empty() {
            count += 1;
        }
        if *self.date_preset.read() != "all" && !self.date_preset.read().is_empty() {
            count += 1;
        }
        if *self.has_attachment.read() != "all" && !self.has_attachment.read().is_empty() {
            count += 1;
        }
        if *self.rating.read() != "all" && !self.rating.read().is_empty() {
            count += 1;
        }
        if !self.requester.read().trim().is_empty() {
            count += 1;
        }
        count
    }

    pub fn reset_all(&mut self) {
        self.status.set("All".to_string());
        self.priority.set("All".to_string());
        self.category.set("All".to_string());
        self.assignee.set("All".to_string());
        self.stage.set("All".to_string());
        self.sla_status.set("All".to_string());
        self.date_preset.set("all".to_string());
        self.has_attachment.set("all".to_string());
        self.rating.set("all".to_string());
        self.requester.set(String::new());
    }

    pub fn matches(&self, t: &TicketItem) -> bool {
        let status = self.status.read();
        if *status != "All" && !status.is_empty() {
            match status.as_str() {
                "Open" => {
                    if !t.status.eq_ignore_ascii_case("Open") {
                        return false;
                    }
                }
                "Closed" => {
                    if !t.status.eq_ignore_ascii_case("Closed") {
                        return false;
                    }
                }
                specific => {
                    if !t.kanban_state.eq_ignore_ascii_case(specific) {
                        return false;
                    }
                }
            }
        }

        let priority = self.priority.read();
        if *priority != "All" && !priority.is_empty() {
            if !t.priority.eq_ignore_ascii_case(&priority) {
                return false;
            }
        }

        let category = self.category.read();
        if *category != "All" && !category.is_empty() {
            if !t.category.eq_ignore_ascii_case(&category) {
                return false;
            }
        }

        let assignee = self.assignee.read();
        if *assignee != "All" && !assignee.is_empty() {
            if assignee.eq_ignore_ascii_case("unassigned") {
                let a = t.assigned_to.trim();
                if !a.is_empty() && !a.eq_ignore_ascii_case("unassigned") {
                    return false;
                }
            } else if !t.assigned_to.eq_ignore_ascii_case(&assignee) {
                return false;
            }
        }

        let stage = self.stage.read();
        if *stage != "All" && !stage.is_empty() {
            if !t.stage.eq_ignore_ascii_case(&stage) {
                return false;
            }
        }

        let sla = self.sla_status.read();
        if *sla != "All" && !sla.is_empty() {
            if !t.sla_status.eq_ignore_ascii_case(&sla) {
                return false;
            }
        }

        let attach = self.has_attachment.read();
        if *attach != "all" && !attach.is_empty() {
            let has = !t.attachment_name.trim().is_empty() || !t.attachment_url.trim().is_empty();
            if *attach == "yes" && !has {
                return false;
            }
            if *attach == "no" && has {
                return false;
            }
        }

        let rating = self.rating.read();
        if *rating != "all" && !rating.is_empty() {
            let r = t.rating_avg_text.trim();
            let is_unrated = r.is_empty() || r.eq_ignore_ascii_case("unrated");
            match rating.as_str() {
                "unrated" => {
                    if !is_unrated {
                        return false;
                    }
                }
                "rated" => {
                    if is_unrated {
                        return false;
                    }
                }
                "5" => {
                    if !r.starts_with('5') {
                        return false;
                    }
                }
                "4+" => {
                    if !r.starts_with('5') && !r.starts_with('4') {
                        return false;
                    }
                }
                "3+" => {
                    if !r.starts_with('5') && !r.starts_with('4') && !r.starts_with('3') {
                        return false;
                    }
                }
                _ => {}
            }
        }

        let req = self.requester.read();
        let query_str = req.trim().to_lowercase();
        if !query_str.is_empty() {
            let name = t.emp_name().to_lowercase();
            let code = t.emp_code_only().to_lowercase();
            let email = t.email_address.to_lowercase();
            let cust = t.customer.to_lowercase();
            let subj = t.subject.to_lowercase();
            let desc = t.problem_description.to_lowercase();
            if !name.contains(&query_str)
                && !code.contains(&query_str)
                && !email.contains(&query_str)
                && !cust.contains(&query_str)
                && !subj.contains(&query_str)
                && !desc.contains(&query_str)
            {
                return false;
            }
        }

        let date_preset = self.date_preset.read();
        if *date_preset != "all" && !date_preset.is_empty() {
            if let Some((y, m, d, _, _)) = crate::state::parse_datetime_flexible(&t.created_on) {
                let ticket_secs = crate::state::epoch_from_civil(y, m, d);
                let now_secs = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|dur| dur.as_secs() as i64)
                    .unwrap_or(0);
                match date_preset.as_str() {
                    "today" => {
                        if now_secs.saturating_sub(ticket_secs) > 86_400 {
                            return false;
                        }
                    }
                    "yesterday" => {
                        if now_secs.saturating_sub(ticket_secs) > 2 * 86_400 {
                            return false;
                        }
                    }
                    "week" => {
                        if now_secs.saturating_sub(ticket_secs) > 7 * 86_400 {
                            return false;
                        }
                    }
                    "month" => {
                        if now_secs.saturating_sub(ticket_secs) > 30 * 86_400 {
                            return false;
                        }
                    }
                    _ => {}
                }
            }
        }

        true
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AdvancedFilterBarProps {
    pub filter: AdvancedFilterState,
    pub total_count: usize,
    pub filtered_count: usize,
    pub categories: Vec<String>,
    pub assignees: Vec<String>,
    pub stages: Vec<String>,
}

#[component]
pub fn AdvancedFilterBar(props: AdvancedFilterBarProps) -> Element {
    let mut filter = props.filter;
    let total_count = props.total_count;
    let filtered_count = props.filtered_count;
    let active_count = filter.active_count();
    let is_open = *filter.is_open.read();

    let mut is_open_sig = filter.is_open;
    let mut status_sig = filter.status;
    let mut priority_sig = filter.priority;
    let mut category_sig = filter.category;
    let mut assignee_sig = filter.assignee;
    let mut stage_sig = filter.stage;
    let mut sla_sig = filter.sla_status;
    let mut date_sig = filter.date_preset;
    let mut attach_sig = filter.has_attachment;
    let mut rating_sig = filter.rating;
    let mut req_sig = filter.requester;

    // Build Category dropdown options
    let mut cat_opts = vec![("All".to_string(), "All Categories".to_string())];
    for c in &props.categories {
        cat_opts.push((c.clone(), c.clone()));
    }

    // Build Assignee dropdown options
    let mut ass_opts = vec![
        ("All".to_string(), "All Assignees".to_string()),
        ("Unassigned".to_string(), "Unassigned Only".to_string()),
    ];
    for a in &props.assignees {
        ass_opts.push((a.clone(), a.clone()));
    }

    // Build Stage dropdown options
    let mut stage_opts = vec![("All".to_string(), "All Stages".to_string())];
    for s in &props.stages {
        stage_opts.push((s.clone(), s.clone()));
    }

    rsx! {
        div { class: "flex flex-col gap-2 w-full shrink-0",
            // Toolbar row
            div { class: "flex flex-wrap items-center justify-between gap-2.5 p-2 rounded-lg bg-card/75 border border-border/70 shadow-2xs backdrop-blur-sm",
                div { class: "flex flex-wrap items-center gap-2 min-w-0",
                    // Advanced Filter Toggle Button
                    button {
                        r#type: "button",
                        class: if is_open || active_count > 0 {
                            "flex items-center gap-1.5 h-8 px-2.5 rounded-md text-xs font-semibold bg-primary/10 border border-primary text-primary hover:bg-primary/15 transition-all cursor-pointer shadow-2xs"
                        } else {
                            "flex items-center gap-1.5 h-8 px-2.5 rounded-md text-xs font-semibold bg-background border border-border hover:bg-muted text-foreground transition-all cursor-pointer"
                        },
                        onclick: move |_| is_open_sig.set(!is_open),
                        FilterIcon { class: "size-3.5" }
                        span { "Advanced Filter" }
                        if active_count > 0 {
                            span { class: "size-4 rounded-full bg-primary text-primary-foreground text-[10px] font-bold flex items-center justify-center ml-0.5",
                                "{active_count}"
                            }
                        }
                    }

                    // Quick Status Filter Pills
                    div { class: "flex items-center gap-1 bg-muted/50 p-0.5 rounded-md border border-border/40 text-xs",
                        button {
                            r#type: "button",
                            class: if *status_sig.read() == "All" {
                                "px-2 py-1 rounded font-semibold bg-background text-foreground shadow-2xs"
                            } else {
                                "px-2 py-1 rounded text-muted-foreground hover:text-foreground hover:bg-background/50"
                            },
                            onclick: move |_| status_sig.set("All".to_string()),
                            "All"
                        }
                        button {
                            r#type: "button",
                            class: if *status_sig.read() == "Open" {
                                "px-2 py-1 rounded font-semibold bg-background text-emerald-600 dark:text-emerald-400 shadow-2xs"
                            } else {
                                "px-2 py-1 rounded text-muted-foreground hover:text-foreground hover:bg-background/50"
                            },
                            onclick: move |_| status_sig.set("Open".to_string()),
                            "Open"
                        }
                        button {
                            r#type: "button",
                            class: if *status_sig.read() == "Closed" {
                                "px-2 py-1 rounded font-semibold bg-background text-slate-600 dark:text-slate-400 shadow-2xs"
                            } else {
                                "px-2 py-1 rounded text-muted-foreground hover:text-foreground hover:bg-background/50"
                            },
                            onclick: move |_| status_sig.set("Closed".to_string()),
                            "Closed"
                        }
                    }

                    // Quick Priority: Critical/High toggle
                    button {
                        r#type: "button",
                        class: if *priority_sig.read() == "Critical" || *priority_sig.read() == "High" {
                            "h-8 px-2.5 rounded-md text-xs font-semibold bg-rose-500/10 border border-rose-500/40 text-rose-600 dark:text-rose-400 hover:bg-rose-500/20 transition-all cursor-pointer"
                        } else {
                            "h-8 px-2.5 rounded-md text-xs font-medium bg-background border border-border/70 text-muted-foreground hover:text-foreground hover:bg-muted transition-all cursor-pointer"
                        },
                        onclick: move |_| {
                            if *priority_sig.read() == "Critical" || *priority_sig.read() == "High" {
                                priority_sig.set("All".to_string());
                            } else {
                                priority_sig.set("High".to_string());
                            }
                        },
                        "🔥 High Priority"
                    }

                    // Results Counter
                    span { class: "text-xs font-medium text-muted-foreground px-1 hidden sm:inline-block",
                        "Showing "
                        span { class: "font-bold text-foreground", "{filtered_count}" }
                        " of {total_count} tickets"
                    }
                }

                // Right controls: Clear all
                if active_count > 0 {
                    div { class: "flex items-center gap-1.5 shrink-0",
                        Button {
                            variant: "ghost",
                            size: "sm",
                            class: "h-7 px-2 text-xs text-muted-foreground hover:text-destructive gap-1 cursor-pointer font-medium",
                            onclick: move |_| filter.reset_all(),
                            XIcon { class: "size-3.5" }
                            "Clear Filters ({active_count})"
                        }
                    }
                }
            }

            // Expanded Advanced Filter Card
            if is_open {
                div { class: "rounded-lg border border-border/80 bg-card p-3.5 shadow-sm flex flex-col gap-3 transition-all animate-in fade-in-50 duration-150",
                    div { class: "flex items-center justify-between pb-2 border-b border-border/50",
                        div { class: "flex items-center gap-2",
                            FilterIcon { class: "size-4 text-primary" }
                            h3 { class: "text-xs font-bold uppercase tracking-wider text-foreground", "Advanced Ticket Filters" }
                        }
                        button {
                            r#type: "button",
                            title: "Close advanced filter panel",
                            class: "cursor-pointer rounded p-1 text-muted-foreground hover:text-foreground hover:bg-muted transition-colors",
                            onclick: move |_| is_open_sig.set(false),
                            XIcon { class: "size-3.5" }
                        }
                    }

                    // Multi-Column Grid
                    div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3",
                        // Column 1: Status & Workflow
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-[11px] font-semibold text-muted-foreground", "Ticket Status / Kanban State" }
                            SimpleSelect {
                                value: status_sig,
                                placeholder: "All Statuses".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: vec![
                                    ("All".to_string(), "All Statuses".to_string()),
                                    ("Open".to_string(), "Open Tickets (All Active)".to_string()),
                                    ("Closed".to_string(), "Closed Tickets (Resolved)".to_string()),
                                    ("Pending".to_string(), "Pending".to_string()),
                                    ("InProgress".to_string(), "In Progress".to_string()),
                                    ("Hold".to_string(), "On Hold".to_string()),
                                    ("Solved".to_string(), "Solved".to_string()),
                                    ("Completed".to_string(), "Solved (Closed)".to_string()),
                                    ("Cancelled".to_string(), "Cancelled".to_string()),
                                ],
                            }

                            label { class: "text-[11px] font-semibold text-muted-foreground pt-1", "Action / Workflow Stage" }
                            SimpleSelect {
                                value: stage_sig,
                                placeholder: "All Stages".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: stage_opts,
                            }
                        }

                        // Column 2: Priority & Category
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-[11px] font-semibold text-muted-foreground", "Priority Level" }
                            SimpleSelect {
                                value: priority_sig,
                                placeholder: "All Priorities".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: vec![
                                    ("All".to_string(), "All Priorities".to_string()),
                                    ("Critical".to_string(), "Critical".to_string()),
                                    ("High".to_string(), "High".to_string()),
                                    ("Medium".to_string(), "Medium".to_string()),
                                    ("Low".to_string(), "Low".to_string()),
                                ],
                            }

                            label { class: "text-[11px] font-semibold text-muted-foreground pt-1", "Category" }
                            SimpleSelect {
                                value: category_sig,
                                placeholder: "All Categories".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: cat_opts,
                            }
                        }

                        // Column 3: Assignee & SLA
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-[11px] font-semibold text-muted-foreground", "Assigned Technician" }
                            SimpleSelect {
                                value: assignee_sig,
                                placeholder: "All Assignees".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: ass_opts,
                            }

                            label { class: "text-[11px] font-semibold text-muted-foreground pt-1", "SLA Status" }
                            SimpleSelect {
                                value: sla_sig,
                                placeholder: "All SLA Statuses".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: vec![
                                    ("All".to_string(), "All SLA Statuses".to_string()),
                                    ("Within SLA".to_string(), "Within SLA".to_string()),
                                    ("Approaching Breach".to_string(), "Approaching Breach".to_string()),
                                    ("Breached".to_string(), "Breached".to_string()),
                                ],
                            }
                        }

                        // Column 4: Date, Attachments & Rating
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-[11px] font-semibold text-muted-foreground", "Created Date Period" }
                            SimpleSelect {
                                value: date_sig,
                                placeholder: "Any Time".to_string(),
                                class: "h-8 text-xs".to_string(),
                                options: vec![
                                    ("all".to_string(), "All Time".to_string()),
                                    ("today".to_string(), "Today".to_string()),
                                    ("yesterday".to_string(), "Yesterday & Today".to_string()),
                                    ("week".to_string(), "Last 7 Days".to_string()),
                                    ("month".to_string(), "Last 30 Days".to_string()),
                                ],
                            }

                            div { class: "grid grid-cols-2 gap-2 pt-1",
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-[11px] font-semibold text-muted-foreground", "Attachment" }
                                    SimpleSelect {
                                        value: attach_sig,
                                        placeholder: "Any".to_string(),
                                        class: "h-8 text-xs".to_string(),
                                        options: vec![
                                            ("all".to_string(), "All".to_string()),
                                            ("yes".to_string(), "With Attachment".to_string()),
                                            ("no".to_string(), "None".to_string()),
                                        ],
                                    }
                                }
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-[11px] font-semibold text-muted-foreground", "Rating" }
                                    SimpleSelect {
                                        value: rating_sig,
                                        placeholder: "Any".to_string(),
                                        class: "h-8 text-xs".to_string(),
                                        options: vec![
                                            ("all".to_string(), "All Ratings".to_string()),
                                            ("5".to_string(), "★ 5 Stars".to_string()),
                                            ("4+".to_string(), "★ 4+ Stars".to_string()),
                                            ("rated".to_string(), "Rated Only".to_string()),
                                            ("unrated".to_string(), "Unrated".to_string()),
                                        ],
                                    }
                                }
                            }
                        }
                    }

                    // Search Requester / Department / Subject bar
                    div { class: "pt-1 flex flex-col gap-1",
                        label { class: "text-[11px] font-semibold text-muted-foreground", "Search within Requester, Employee Code, Subject, or Details" }
                        Input {
                            value: req_sig.read().clone(),
                            placeholder: "Type employee name, code (e.g. 11098), department, division, or keyword...".to_string(),
                            class: "h-8 text-xs bg-background".to_string(),
                            oninput: move |e: FormEvent| req_sig.set(e.value()),
                        }
                    }
                }
            }

            // Active Filter Chips Row
            if active_count > 0 {
                div { class: "flex flex-wrap items-center gap-1.5 px-0.5",
                    span { class: "text-[11px] font-semibold text-muted-foreground mr-1", "Active Filters:" }

                    if *status_sig.read() != "All" && !status_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-primary/10 border border-primary/30 text-primary text-xs font-medium",
                            "Status: {status_sig}"
                            button {
                                r#type: "button",
                                title: "Clear status filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| status_sig.set("All".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if *priority_sig.read() != "All" && !priority_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-700 dark:text-amber-400 text-xs font-medium",
                            "Priority: {priority_sig}"
                            button {
                                r#type: "button",
                                title: "Clear priority filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| priority_sig.set("All".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if *category_sig.read() != "All" && !category_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-secondary border border-border text-foreground text-xs font-medium",
                            "Category: {category_sig}"
                            button {
                                r#type: "button",
                                title: "Clear category filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| category_sig.set("All".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if *assignee_sig.read() != "All" && !assignee_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-indigo-500/10 border border-indigo-500/30 text-indigo-700 dark:text-indigo-400 text-xs font-medium",
                            "Assignee: {assignee_sig}"
                            button {
                                r#type: "button",
                                title: "Clear assignee filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| assignee_sig.set("All".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if *stage_sig.read() != "All" && !stage_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-secondary border border-border text-foreground text-xs font-medium",
                            "Stage: {stage_sig}"
                            button {
                                r#type: "button",
                                title: "Clear stage filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| stage_sig.set("All".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if *sla_sig.read() != "All" && !sla_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-rose-500/10 border border-rose-500/30 text-rose-700 dark:text-rose-400 text-xs font-medium",
                            "SLA: {sla_sig}"
                            button {
                                r#type: "button",
                                title: "Clear SLA filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| sla_sig.set("All".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if *date_sig.read() != "all" && !date_sig.read().is_empty() {
                        {
                            let date_label = match date_sig.read().as_str() {
                                "today" => "Today",
                                "yesterday" => "Yesterday & Today",
                                "week" => "Last 7 Days",
                                "month" => "Last 30 Days",
                                _ => "Custom Date",
                            };
                            rsx! {
                                span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-cyan-500/10 border border-cyan-500/30 text-cyan-700 dark:text-cyan-400 text-xs font-medium",
                                    "Date: {date_label}"
                                    button {
                                        r#type: "button",
                                        title: "Clear date filter",
                                        class: "cursor-pointer hover:text-destructive",
                                        onclick: move |_| date_sig.set("all".to_string()),
                                        XIcon { class: "size-3" }
                                    }
                                }
                            }
                        }
                    }

                    if *attach_sig.read() != "all" && !attach_sig.read().is_empty() {
                        {
                            let label = if *attach_sig.read() == "yes" { "Has Attachment" } else { "No Attachment" };
                            rsx! {
                                span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-secondary border border-border text-foreground text-xs font-medium",
                                    "{label}"
                                    button {
                                        r#type: "button",
                                        title: "Clear attachment filter",
                                        class: "cursor-pointer hover:text-destructive",
                                        onclick: move |_| attach_sig.set("all".to_string()),
                                        XIcon { class: "size-3" }
                                    }
                                }
                            }
                        }
                    }

                    if *rating_sig.read() != "all" && !rating_sig.read().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-amber-500/10 border border-amber-500/30 text-amber-700 dark:text-amber-400 text-xs font-medium",
                            "Rating: {rating_sig}"
                            button {
                                r#type: "button",
                                title: "Clear rating filter",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| rating_sig.set("all".to_string()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    if !req_sig.read().trim().is_empty() {
                        span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md bg-secondary border border-border text-foreground text-xs font-medium",
                            "Query: \"{req_sig}\""
                            button {
                                r#type: "button",
                                title: "Clear requester query",
                                class: "cursor-pointer hover:text-destructive",
                                onclick: move |_| req_sig.set(String::new()),
                                XIcon { class: "size-3" }
                            }
                        }
                    }

                    button {
                        r#type: "button",
                        class: "cursor-pointer text-xs font-semibold text-primary hover:underline ml-1",
                        onclick: move |_| filter.reset_all(),
                        "Reset all"
                    }
                }
            }
        }
    }
}
