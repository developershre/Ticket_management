use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const COLUMN_NAMES: &[&str] = &[
    "Pending",
    "InProgress",
    "Hold",
    "Solved",
    "Completed",
    "Cancelled",
];

/// Parses an ID string (e.g. "IT-123", "TC-0004", "123") into an i64 ticket ID.
#[allow(dead_code)]
pub fn parse_ticket_num(id: &str) -> Option<i64> {
    let s = id.trim();
    if let Ok(n) = s.parse::<i64>() {
        return Some(n);
    }
    if let Some(rest) = s
        .strip_prefix("IT-")
        .or_else(|| s.strip_prefix("it-"))
        .or_else(|| s.strip_prefix("TC-"))
        .or_else(|| s.strip_prefix("tc-"))
    {
        let trimmed_zeroes = rest.trim_start_matches('0');
        if let Ok(n) = trimmed_zeroes.parse::<i64>() {
            return Some(n);
        }
        if let Ok(n) = rest.parse::<i64>() {
            return Some(n);
        }
    }
    None
}

/// IT Ticket Item model storing all 23 standardized fields.
#[derive(Clone, PartialEq, Debug)]
pub struct TicketItem {
    pub ticket_id: i64,
    pub employee_code: String,
    pub email_address: String,
    pub category: String,
    pub subject: String,
    pub problem_description: String,
    pub attachment_name: String,
    pub attachment_url: String,
    pub priority: String,
    pub assigned_to: String,
    pub customer: String,
    pub time_spent: f64,
    pub activities: String,
    pub my_activity: String,
    pub deadline: String,
    pub sla_status: String,
    pub sla_deadline: String,
    pub rating_avg_text: String,
    pub kanban_state: String,
    pub stage: String,
    pub status: String,
    pub created_on: String,
    pub last_updated_on: String,

    // Aliases
    pub id: String,
    pub title: String,
    pub employee: String,
    pub assignee: String,
    pub created_at: String,
}

impl TicketItem {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ticket_id: i64,
        employee_code: &str,
        email_address: &str,
        category: &str,
        subject: &str,
        problem_description: &str,
        attachment_name: &str,
        attachment_url: &str,
        priority: &str,
        assigned_to: &str,
        customer: &str,
        time_spent: f64,
        activities: &str,
        my_activity: &str,
        deadline: &str,
        sla_status: &str,
        sla_deadline: &str,
        rating_avg_text: &str,
        kanban_state: &str,
        stage: &str,
        status: &str,
        created_on: &str,
        last_updated_on: &str,
    ) -> Self {
        let id = format!("IT-{}", ticket_id);
        Self {
            ticket_id,
            employee_code: employee_code.to_string(),
            email_address: email_address.to_string(),
            category: category.to_string(),
            subject: subject.to_string(),
            problem_description: problem_description.to_string(),
            attachment_name: attachment_name.to_string(),
            attachment_url: attachment_url.to_string(),
            priority: priority.to_string(),
            assigned_to: assigned_to.to_string(),
            customer: customer.to_string(),
            time_spent,
            activities: activities.to_string(),
            my_activity: my_activity.to_string(),
            deadline: deadline.to_string(),
            sla_status: sla_status.to_string(),
            sla_deadline: sla_deadline.to_string(),
            rating_avg_text: rating_avg_text.to_string(),
            kanban_state: kanban_state.to_string(),
            stage: stage.to_string(),
            status: status.to_string(),
            created_on: created_on.to_string(),
            last_updated_on: last_updated_on.to_string(),

            id,
            title: subject.to_string(),
            employee: employee_code.to_string(),
            assignee: assigned_to.to_string(),
            created_at: created_on.to_string(),
        }
    }

    pub fn from_db_ticket(t: crate::db::DbTicket) -> Self {
        let raw_kanban = if !t.kanban_state.trim().is_empty() {
            &t.kanban_state
        } else if !t.stage.trim().is_empty() {
            &t.stage
        } else {
            &t.status
        };
        let kanban_state = crate::sync::from_sheet_status(raw_kanban).to_string();
        let stage = if !t.stage.trim().is_empty() {
            t.stage.clone()
        } else {
            "Triage".to_string()
        };
        let status = if !t.status.trim().is_empty() {
            t.status.clone()
        } else if kanban_state == "Completed" || kanban_state == "Cancelled" {
            "Closed".to_string()
        } else {
            "Open".to_string()
        };

        let mut item = Self::new(
            t.ticket_id,
            &t.employee_code,
            &t.email_address,
            &t.category,
            &t.subject,
            &t.problem_description,
            &t.attachment_name,
            &t.attachment_url,
            &t.priority,
            &t.assigned_to,
            &t.customer,
            t.time_spent,
            &t.activities,
            &t.my_activity,
            &t.deadline,
            &t.sla_status,
            &t.sla_deadline,
            &t.rating_avg_text,
            &kanban_state,
            &stage,
            &status,
            &t.created_on,
            &t.last_updated_on,
        );
        if let Some(ref sid) = t.server_ticket_id {
            if !sid.trim().is_empty() {
                item.id = sid.clone();
            }
        }
        item
    }

    pub fn emp_code_only(&self) -> &str {
        if let Some(idx) = self.employee_code.find('(') {
            self.employee_code[..idx].trim()
        } else {
            self.employee_code.trim()
        }
    }

    pub fn emp_name(&self) -> String {
        let code = self.emp_code_only();
        if !code.is_empty() {
            if let Some(emp) = crate::db::find_employee_by_code(code) {
                if !emp.emp_name.trim().is_empty() {
                    return emp.emp_name;
                }
            }
        }
        if let Some(start) = self.employee_code.find('(') {
            if let Some(end) = self.employee_code.find(')') {
                if end > start + 1 {
                    return self.employee_code[start + 1..end].trim().to_string();
                }
            }
        }
        if !self.customer.trim().is_empty() && self.customer.trim() != "Internal" {
            return self.customer.clone();
        }
        if !code.is_empty() {
            return code.to_string();
        }
        "Unassigned".to_string()
    }

    pub fn emp_email(&self) -> String {
        let code = self.emp_code_only();
        if !code.is_empty() {
            if let Some(emp) = crate::db::find_employee_by_code(code) {
                if !emp.emp_email.trim().is_empty() {
                    return emp.emp_email;
                }
            }
        }
        self.email_address.clone()
    }

    pub fn assignee_name(&self) -> &str {
        match self.assigned_to.as_str() {
            "Shreyansh" => "Shreyansh",
            "kaveri" => "Kaveri",
            "Chaitram" => "Chaitram",
            "IT Support" => "IT Support",
            other if !other.is_empty() => other,
            _ => "Unassigned",
        }
    }

    #[allow(dead_code)]
    pub fn display_tc_id(&self) -> String {
        format!("TC-{:04}", self.ticket_id)
    }

    pub fn ticket_number(&self) -> i64 {
        let digits: String = self.id.chars().filter(|c| c.is_ascii_digit()).collect();
        if let Ok(num) = digits.parse::<i64>() {
            if num > 0 {
                return num;
            }
        }
        self.ticket_id
    }

    #[allow(dead_code)]
    pub fn card_display_title(&self) -> String {
        let prob = self.problem_description.trim();
        if !prob.is_empty() {
            prob.to_string()
        } else if !self.subject.trim().is_empty() {
            self.subject.trim().to_string()
        } else {
            self.title.clone()
        }
    }

    #[allow(dead_code)]
    pub fn customer_display(&self) -> String {
        let name = self.emp_name();
        if !name.is_empty() && name != "Unassigned" {
            name
        } else if !self.customer.trim().is_empty() && self.customer.trim() != "Internal" {
            self.customer.trim().to_string()
        } else {
            "N/A".to_string()
        }
    }

    #[allow(dead_code)]
    pub fn department_display(&self) -> String {
        let code = self.emp_code_only();
        if !code.is_empty() {
            if let Some(emp) = crate::db::find_employee_by_code(code) {
                if !emp.department.trim().is_empty() {
                    return emp.department.trim().to_string();
                }
            }
        }
        if !self.category.trim().is_empty() {
            return self.category.trim().to_string();
        }
        "Admin".to_string()
    }

    #[allow(dead_code)]
    pub fn division_display(&self) -> String {
        let code = self.emp_code_only();
        if !code.is_empty() {
            if let Some(emp) = crate::db::find_employee_by_code(code) {
                if !emp.division.trim().is_empty() {
                    return emp.division.trim().to_string();
                }
                if emp.emp_email.trim().contains('@') {
                    return emp.emp_email.trim().to_string();
                }
            }
        }
        if self.email_address.trim().contains('@') && !self.email_address.trim().is_empty() {
            return self.email_address.trim().to_string();
        }
        "N/A".to_string()
    }

    #[allow(dead_code)]
    pub fn assignee_card_display(&self) -> String {
        let a = self.assigned_to.trim();
        if a.is_empty() || a.eq_ignore_ascii_case("unassigned") {
            "Unassigned".to_string()
        } else {
            a.to_string()
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomKanbanView {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub dot_color: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KanbanViewDef {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub dot_color: String,
    pub is_custom: bool,
}

pub fn load_custom_kanban_views() -> Vec<CustomKanbanView> {
    crate::db::sync_meta_get("kanban_custom_views")
        .and_then(|json| serde_json::from_str::<Vec<CustomKanbanView>>(&json).ok())
        .unwrap_or_default()
}

pub fn save_custom_kanban_views(views: &[CustomKanbanView]) {
    if let Ok(json) = serde_json::to_string(views) {
        let _ = crate::db::sync_meta_set("kanban_custom_views", &json);
    }
}

pub fn load_hidden_views() -> std::collections::HashSet<String> {
    crate::db::sync_meta_get("kanban_hidden_views")
        .map(|s| {
            s.split(',')
                .map(|x| x.trim().to_string())
                .filter(|x| !x.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

pub fn save_hidden_views(hidden: &std::collections::HashSet<String>) {
    let joined = hidden.iter().cloned().collect::<Vec<_>>().join(",");
    let _ = crate::db::sync_meta_set("kanban_hidden_views", &joined);
}

pub fn load_hide_empty_views() -> bool {
    crate::db::sync_meta_get("kanban_hide_empty_views")
        .map(|v| v == "true")
        .unwrap_or(false)
}

pub fn save_hide_empty_views(val: bool) {
    let _ = crate::db::sync_meta_set("kanban_hide_empty_views", if val { "true" } else { "false" });
}

pub fn kanban_grid_layout_class(visible_count: usize) -> &'static str {
    match visible_count {
        0 => "flex items-center justify-center w-full flex-1 min-h-0",
        1 => "flex justify-center w-full flex-1 min-h-0 overflow-y-auto p-1",
        _ => "flex gap-3.5 w-full flex-1 min-h-0 overflow-x-auto pb-2 items-stretch",
    }
}

pub fn initial_tickets() -> Vec<TicketItem> {
    let mut list: Vec<TicketItem> = crate::db::get_all_tickets()
        .into_iter()
        .map(TicketItem::from_db_ticket)
        .collect();
    list.sort_by(|a, b| {
        let num_a = a.ticket_number();
        let num_b = b.ticket_number();
        num_b.cmp(&num_a).then_with(|| b.created_on.cmp(&a.created_on))
    });
    list
}

pub fn column_display_label(col: &str) -> &'static str {
    match col {
        "Pending" => "Pending",
        "InProgress" => "In Progress",
        "Hold" | "Onhold" => "Hold",
        "Solved" => "Solved",
        "Completed" => "Solved (Closed)",
        "Cancelled" => "Cancelled",
        _ => "Custom View",
    }
}

#[allow(dead_code)]
pub fn format_card_datetime(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return "N/A".to_string();
    }
    let formatted = crate::state::arrange_date_time(s);
    if formatted.is_empty() {
        "N/A".to_string()
    } else {
        formatted
    }
}

#[allow(dead_code)]
pub fn card_border_accent_class(col: &str) -> &'static str {
    match col {
        "Hold" | "Onhold" => "border-l-4 border-l-amber-500",
        "Cancelled" => "border-l-4 border-l-red-500",
        "Completed" => "border-l-4 border-l-emerald-600",
        "InProgress" => "border-l-4 border-l-sky-500",
        "Pending" => "border-l-4 border-l-amber-400",
        "Solved" => "border-l-4 border-l-purple-500",
        _ => "border-l-4 border-l-slate-400",
    }
}

#[allow(dead_code)]
pub fn card_status_pill_label(col: &str) -> &'static str {
    match col {
        "Hold" | "Onhold" => "HOLD",
        "Cancelled" => "CANCELLED",
        "Completed" => "SOLVED(CLOSED)",
        "InProgress" => "IN PROGRESS",
        "Pending" => "PENDING",
        "Solved" => "SOLVED",
        _ => "CUSTOM",
    }
}

#[allow(dead_code)]
pub fn card_status_pill_class(col: &str) -> &'static str {
    match col {
        "Hold" | "Onhold" => "bg-amber-100 text-amber-900 border border-amber-200/60 dark:bg-amber-950/40 dark:text-amber-200",
        "Cancelled" => "bg-red-100 text-red-700 border border-red-200/60 dark:bg-red-950/40 dark:text-red-300",
        "Completed" => "bg-emerald-100 text-emerald-800 border border-emerald-200/60 dark:bg-emerald-950/40 dark:text-emerald-300",
        "InProgress" => "bg-sky-100 text-sky-800 border border-sky-200/60 dark:bg-sky-950/40 dark:text-sky-300",
        "Pending" => "bg-amber-100 text-amber-800 border border-amber-200/60 dark:bg-amber-950/40 dark:text-amber-300",
        "Solved" => "bg-purple-100 text-purple-800 border border-purple-200/60 dark:bg-purple-950/40 dark:text-purple-300",
        _ => "bg-slate-100 text-slate-800 border border-slate-200/60 dark:bg-slate-800 dark:text-slate-200",
    }
}

#[allow(dead_code)]
pub fn column_header_accent_class(col: &str) -> &'static str {
    match col {
        "Hold" | "Onhold" => "bg-amber-400",
        "Cancelled" => "bg-red-500",
        "Completed" => "bg-emerald-700",
        "InProgress" => "bg-sky-500",
        "Pending" => "bg-amber-400",
        "Solved" => "bg-purple-600",
        _ => "bg-slate-400",
    }
}

#[allow(dead_code)]
pub fn column_count_badge_class(col: &str) -> &'static str {
    match col {
        "Hold" | "Onhold" => "bg-amber-400 text-slate-950 font-bold",
        "Cancelled" => "bg-red-500 text-white font-bold",
        "Completed" => "bg-emerald-700 text-white font-bold",
        "InProgress" => "bg-sky-500 text-white font-bold",
        "Pending" => "bg-amber-400 text-slate-950 font-bold",
        "Solved" => "bg-purple-600 text-white font-bold",
        _ => "bg-slate-600 text-white font-bold",
    }
}

#[allow(dead_code)]
pub fn priority_pill_class(prio: &str) -> &'static str {
    match prio {
        "Critical" | "High" => "bg-red-500 text-white",
        "Medium" => "bg-amber-400 text-slate-950",
        "Low" => "bg-sky-500 text-white",
        _ => "bg-slate-200 text-slate-800 dark:bg-slate-700 dark:text-slate-200",
    }
}

#[allow(dead_code)]
pub fn column_subtitle(col: &str) -> &'static str {
    match col {
        "Pending" => "New & open",
        "InProgress" => "Active support",
        "Hold" | "Onhold" => "Waiting / paused",
        "Solved" => "Solved, awaiting close",
        "Completed" => "Resolved & closed",
        "Cancelled" => "Cancelled / duplicate",
        _ => "",
    }
}

pub fn priority_badge(priority: &str) -> &'static str {
    match priority {
        "Critical" | "High" => "destructive",
        "Medium" => "warning",
        "Low" => "info",
        _ => "secondary",
    }
}

pub fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Pending" => "text-amber-500 dark:text-amber-400 border-amber-500/35 bg-amber-500/15",
        "InProgress" => "text-sky-500 dark:text-sky-400 border-sky-500/35 bg-sky-500/15",
        "Hold" | "Onhold" => "text-amber-600 dark:text-amber-400 border-amber-500/35 bg-amber-500/15",
        "Solved" => "text-purple-500 dark:text-purple-400 border-purple-500/35 bg-purple-500/15",
        "Completed" => {
            "text-emerald-500 dark:text-emerald-400 border-emerald-500/35 bg-emerald-500/15"
        }
        "Cancelled" => "text-rose-500 dark:text-rose-400 border-rose-500/35 bg-rose-500/15",
        _ => "text-muted-foreground border-border bg-muted/40",
    }
}

pub fn status_badge_variant(status: &str) -> &'static str {
    match status {
        "Pending" => "warning",
        "InProgress" => "info",
        "Hold" | "Onhold" => "warning",
        "Solved" => "purple",
        "Completed" => "success",
        "Cancelled" => "destructive",
        _ => "secondary",
    }
}

pub fn sla_badge_class(sla: &str) -> &'static str {
    match sla {
        "Within SLA" | "Met" => {
            "text-emerald-500 dark:text-emerald-400 border-emerald-500/35 bg-emerald-500/15"
        }
        "Approaching Breach" => {
            "text-amber-500 dark:text-amber-400 border-amber-500/35 bg-amber-500/15"
        }
        "Breached" => "text-rose-500 dark:text-rose-400 border-rose-500/35 bg-rose-500/15",
        _ => "text-muted-foreground border-border bg-muted/30",
    }
}

pub fn get_assignee_initials(name: &str) -> String {
    let clean = name.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("unassigned") {
        return "--".to_string();
    }
    let parts: Vec<&str> = clean.split_whitespace().collect();
    if parts.len() >= 2 {
        let c1 = parts[0].chars().next().unwrap_or('-');
        let c2 = parts[1].chars().next().unwrap_or('-');
        format!("{}{}", c1.to_ascii_uppercase(), c2.to_ascii_uppercase())
    } else {
        let mut chars = clean.chars();
        match (chars.next(), chars.next()) {
            (Some(c1), Some(c2)) => {
                format!("{}{}", c1.to_ascii_uppercase(), c2.to_ascii_uppercase())
            }
            (Some(c1), None) => c1.to_ascii_uppercase().to_string(),
            _ => "--".to_string(),
        }
    }
}

pub fn assignee_color_classes(name: &str) -> &'static str {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("unassigned") {
        return "bg-muted text-muted-foreground border-border/80";
    }

    let lower = trimmed.to_lowercase();
    if lower.contains("shreyansh") {
        return "bg-indigo-500/20 text-indigo-700 dark:text-indigo-300 border-indigo-500/40 ring-1 ring-indigo-500/20";
    } else if lower.contains("kaveri") {
        return "bg-rose-500/20 text-rose-700 dark:text-rose-300 border-rose-500/40 ring-1 ring-rose-500/20";
    } else if lower.contains("chaitram") {
        return "bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border-emerald-500/40 ring-1 ring-emerald-500/20";
    } else if lower.contains("it support") {
        return "bg-blue-500/20 text-blue-700 dark:text-blue-300 border-blue-500/40 ring-1 ring-blue-500/20";
    }

    const PALETTE: [&str; 8] = [
        "bg-purple-500/20 text-purple-700 dark:text-purple-300 border-purple-500/40 ring-1 ring-purple-500/20",
        "bg-amber-500/20 text-amber-700 dark:text-amber-300 border-amber-500/40 ring-1 ring-amber-500/20",
        "bg-cyan-500/20 text-cyan-700 dark:text-cyan-300 border-cyan-500/40 ring-1 ring-cyan-500/20",
        "bg-teal-500/20 text-teal-700 dark:text-teal-300 border-teal-500/40 ring-1 ring-teal-500/20",
        "bg-orange-500/20 text-orange-700 dark:text-orange-300 border-orange-500/40 ring-1 ring-orange-500/20",
        "bg-fuchsia-500/20 text-fuchsia-700 dark:text-fuchsia-300 border-fuchsia-500/40 ring-1 ring-fuchsia-500/20",
        "bg-blue-500/20 text-blue-700 dark:text-blue-300 border-blue-500/40 ring-1 ring-blue-500/20",
        "bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 border-emerald-500/40 ring-1 ring-emerald-500/20",
    ];

    let hash: usize = trimmed
        .bytes()
        .fold(0usize, |acc, b| acc.wrapping_add(b as usize));
    PALETTE[hash % PALETTE.len()]
}

/// Global reactive state holder for viewing and modifying tickets
#[derive(Clone, Copy, PartialEq)]
pub struct TicketContextState {
    pub tickets: Signal<Vec<TicketItem>>,
    pub viewing_ticket: Signal<Option<TicketItem>>,
    pub editing_ticket: Signal<Option<TicketItem>>,
    pub is_view_open: Signal<bool>,
    pub is_edit_open: Signal<bool>,

    pub edit_emp_name: Signal<String>,
    pub edit_employee_code: Signal<String>,
    pub edit_email_address: Signal<String>,
    pub edit_category: Signal<String>,
    pub edit_subject: Signal<String>,
    pub edit_problem_description: Signal<String>,
    pub edit_attachment_name: Signal<String>,
    pub edit_attachment_url: Signal<String>,
    pub edit_priority: Signal<String>,
    pub edit_assigned_to: Signal<String>,
    pub edit_customer: Signal<String>,
    pub edit_time_spent: Signal<f64>,
    pub edit_activities: Signal<String>,
    pub edit_my_activity: Signal<String>,
    pub edit_deadline: Signal<String>,
    pub edit_sla_status: Signal<String>,
    pub edit_sla_deadline: Signal<String>,
    pub edit_rating_avg_text: Signal<String>,
    pub edit_kanban_state: Signal<String>,
    pub edit_stage: Signal<String>,
    pub edit_status: Signal<String>,

    pub is_create_open: Signal<bool>,
    pub create_emp_name: Signal<String>,
    pub create_employee_code: Signal<String>,
    pub create_email_address: Signal<String>,
    pub create_category: Signal<String>,
    pub create_subject: Signal<String>,
    pub create_problem_description: Signal<String>,
    pub create_attachment_name: Signal<String>,
    pub create_attachment_url: Signal<String>,
    pub create_priority: Signal<String>,
    pub create_assigned_to: Signal<String>,
    pub create_customer: Signal<String>,
    pub create_time_spent: Signal<f64>,
    pub create_activities: Signal<String>,
    pub create_my_activity: Signal<String>,
    pub create_deadline: Signal<String>,
    pub create_sla_status: Signal<String>,
    pub create_sla_deadline: Signal<String>,
    pub create_rating_avg_text: Signal<String>,
    pub create_kanban_state: Signal<String>,
    pub create_stage: Signal<String>,
    pub create_status: Signal<String>,
}

impl TicketContextState {
    pub fn open_view(&mut self, mut t: TicketItem) {
        let code = t.emp_code_only().to_string();
        if !code.is_empty() {
            if let Some(emp) = crate::db::find_employee_by_code(&code) {
                t.employee = format!("{} ({})", emp.emp_code, emp.emp_name);
                if !emp.emp_email.is_empty() {
                    t.email_address = emp.emp_email;
                }
            }
        }
        self.viewing_ticket.set(Some(t));
        self.is_view_open.set(true);
    }

    pub fn open_create(&mut self, default_column: Option<&str>) {
        let def_priority = crate::db::sync_meta_get("ui_default_priority").unwrap_or_else(|| "Medium".to_string());
        let def_category = crate::db::sync_meta_get("ui_default_category").unwrap_or_else(|| "General".to_string());
        let def_assignee = crate::db::sync_meta_get("ui_default_assignee").unwrap_or_else(|| "IT Support".to_string());
        let def_emp = crate::db::sync_meta_get("system_employee_code").unwrap_or_default();

        self.create_emp_name.set(String::new());
        self.create_employee_code.set(String::new());
        self.create_email_address.set(String::new());
        if !def_emp.is_empty() {
            self.create_employee_code.set(def_emp.clone());
            if let Some(emp) = crate::db::find_employee_by_code(&def_emp) {
                self.create_emp_name.set(emp.emp_name);
                if !emp.emp_email.is_empty() {
                    self.create_email_address.set(emp.emp_email);
                }
            }
        }
        self.create_category.set(def_category);
        self.create_subject.set(String::new());
        self.create_problem_description.set(String::new());
        self.create_attachment_name.set(String::new());
        self.create_attachment_url.set(String::new());
        self.create_priority.set(def_priority);
        self.create_assigned_to.set(def_assignee);
        self.create_customer.set("Internal".to_string());
        self.create_time_spent.set(0.0);
        self.create_activities.set("Ticket created".to_string());
        self.create_my_activity.set("Initial review".to_string());
        self.create_deadline.set(String::new());
        self.create_sla_status.set("Within SLA".to_string());
        self.create_sla_deadline.set(String::new());
        self.create_rating_avg_text.set("Unrated".to_string());
        let col = default_column.unwrap_or("Pending").to_string();
        self.create_kanban_state.set(col.clone());
        self.create_stage.set("Triage".to_string());
        self.create_status.set("Open".to_string());

        self.is_create_open.set(true);
        self.is_view_open.set(false);
        self.is_edit_open.set(false);
    }

    pub fn open_edit(&mut self, t: TicketItem) {
        let code = t.emp_code_only().to_string();
        self.edit_employee_code.set(code.clone());
        if let Some(emp) = crate::db::find_employee_by_code(&code) {
            self.edit_emp_name.set(emp.emp_name);
            if !emp.emp_email.is_empty() {
                self.edit_email_address.set(emp.emp_email);
            } else {
                self.edit_email_address.set(t.email_address.clone());
            }
        } else {
            self.edit_emp_name.set(t.emp_name());
            self.edit_email_address.set(t.email_address.clone());
        }
        self.edit_category.set(t.category.clone());
        self.edit_subject.set(t.subject.clone());
        self.edit_problem_description
            .set(t.problem_description.clone());
        self.edit_attachment_name.set(t.attachment_name.clone());
        self.edit_attachment_url.set(t.attachment_url.clone());
        self.edit_priority.set(t.priority.clone());
        self.edit_assigned_to.set(t.assigned_to.clone());
        self.edit_customer.set(t.customer.clone());
        self.edit_time_spent.set(t.time_spent);
        self.edit_activities.set(t.activities.clone());
        self.edit_my_activity.set(t.my_activity.to_uppercase());
        self.edit_deadline.set(t.deadline.clone());
        self.edit_sla_status.set(t.sla_status.clone());
        self.edit_sla_deadline.set(t.sla_deadline.clone());
        self.edit_rating_avg_text.set(t.rating_avg_text.clone());
        self.edit_kanban_state.set(t.kanban_state.clone());
        self.edit_stage.set(t.stage.clone());
        self.edit_status.set(t.status.clone());

        self.editing_ticket.set(Some(t));
        self.is_edit_open.set(true);
        self.is_view_open.set(false);
        self.is_create_open.set(false);
    }

    pub fn update_status(&mut self, ticket_id: &str, new_status: &str) {
        let mut local_num = None;
        let mut target_stage = None;
        let derived_macro_status = if new_status.eq_ignore_ascii_case("Completed")
            || new_status.eq_ignore_ascii_case("Solved(closed)")
            || new_status.eq_ignore_ascii_case("Solved (closed)")
            || new_status.eq_ignore_ascii_case("Closed")
            || new_status.eq_ignore_ascii_case("Cancelled")
        {
            "Closed".to_string()
        } else {
            "Open".to_string()
        };

        {
            let mut list = self.tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
                list[pos].status = derived_macro_status.clone();
                list[pos].kanban_state = new_status.to_string();
                target_stage = Some(list[pos].stage.clone());
                local_num = Some(list[pos].ticket_id);
            }
        }
        let num = local_num.or_else(|| crate::db::find_ticket_local_id(ticket_id));
        if let Some(num) = num {
            let _ = crate::db::update_ticket_status(num, new_status);
            let sheet_kanban = match new_status {
                "InProgress" => "In Progress",
                "Hold" => "Hold",
                "Solved" => "Solved",
                "Completed" => "Solved(closed)",
                "Cancelled" => "Cancelled",
                other => other,
            };
            let mut payload = serde_json::json!({
                "kanban_state": sheet_kanban,
                "status": derived_macro_status,
            });
            if let Some(stg) = target_stage {
                if !stg.is_empty() {
                    payload["stage"] = serde_json::json!(stg);
                }
            }
            crate::sync::enqueue_update_with_server_id(num, Some(ticket_id.to_string()), payload);
        }
    }

    pub fn update_priority(&mut self, ticket_id: &str, new_priority: &str) {
        let mut local_num = None;
        {
            let mut list = self.tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
                list[pos].priority = new_priority.to_string();
                local_num = Some(list[pos].ticket_id);
            }
        }
        let num = local_num.or_else(|| crate::db::find_ticket_local_id(ticket_id));
        if let Some(num) = num {
            let _ = crate::db::update_ticket_priority(num, new_priority);
            crate::sync::enqueue_update_with_server_id(
                num,
                Some(ticket_id.to_string()),
                serde_json::json!({ "priority": new_priority }),
            );
        }
    }

    pub fn update_category(&mut self, ticket_id: &str, new_category: &str) {
        let mut local_num = None;
        {
            let mut list = self.tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
                list[pos].category = new_category.to_string();
                local_num = Some(list[pos].ticket_id);
            }
        }
        let num = local_num.or_else(|| crate::db::find_ticket_local_id(ticket_id));
        if let Some(num) = num {
            let _ = crate::db::update_ticket_category(num, new_category);
            crate::sync::enqueue_update_with_server_id(
                num,
                Some(ticket_id.to_string()),
                serde_json::json!({ "category": new_category }),
            );
        }
    }

    pub fn update_assignee(&mut self, ticket_id: &str, new_assignee: &str) {
        let mut local_num = None;
        {
            let mut list = self.tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
                list[pos].assigned_to = new_assignee.to_string();
                list[pos].assignee = new_assignee.to_string();
                local_num = Some(list[pos].ticket_id);
            }
        }
        let num = local_num.or_else(|| crate::db::find_ticket_local_id(ticket_id));
        if let Some(num) = num {
            let _ = crate::db::update_ticket_assignee(num, new_assignee);
            crate::sync::enqueue_update_with_server_id(
                num,
                Some(ticket_id.to_string()),
                serde_json::json!({ "assigned_to": new_assignee }),
            );
        }
    }

    pub fn move_column_tickets(&mut self, from_col: &str, to_col: &str) {
        let derived_macro_status = if to_col.eq_ignore_ascii_case("Completed")
            || to_col.eq_ignore_ascii_case("Solved(closed)")
            || to_col.eq_ignore_ascii_case("Solved (closed)")
            || to_col.eq_ignore_ascii_case("Closed")
            || to_col.eq_ignore_ascii_case("Cancelled")
        {
            "Closed".to_string()
        } else {
            "Open".to_string()
        };
        let sheet_kanban = match to_col {
            "InProgress" => "In Progress",
            "Hold" => "Hold",
            "Solved" => "Solved",
            "Completed" => "Solved(closed)",
            "Cancelled" => "Cancelled",
            other => other,
        };

        let mut list = self.tickets.write();
        for t in list.iter_mut() {
            let current = if !t.kanban_state.is_empty() {
                t.kanban_state.as_str()
            } else if !t.stage.is_empty() {
                t.stage.as_str()
            } else {
                t.status.as_str()
            };
            if current == from_col {
                t.kanban_state = to_col.to_string();
                t.status = derived_macro_status.clone();
                let num = t.ticket_id;
                let _ = crate::db::update_ticket_status(num, to_col);
                let mut payload = serde_json::json!({
                    "kanban_state": sheet_kanban,
                    "status": derived_macro_status,
                });
                if !t.stage.is_empty() {
                    payload["stage"] = serde_json::json!(t.stage.clone());
                }
                crate::sync::enqueue_update(num, payload);
            }
        }
    }

    pub fn delete_ticket(&mut self, ticket_id: &str) {
        let local_id = self
            .tickets
            .read()
            .iter()
            .find(|t| t.id == ticket_id)
            .map(|t| t.ticket_id)
            .or_else(|| crate::db::find_ticket_local_id(ticket_id));

        if let Some(num) = local_id {
            crate::sync::enqueue_delete(num);
            let _ = crate::db::delete_ticket(num);
        }
        let mut list = self.tickets.write();
        list.retain(|t| t.id != ticket_id);
    }
}
