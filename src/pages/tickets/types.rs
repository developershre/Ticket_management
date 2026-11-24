use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

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
    pub tag: String,

    // Pre-computed employee & customer display cache (zero-cost in render loops)
    pub cached_emp_name: String,
    pub cached_emp_email: String,
    pub cached_customer: String,
    pub cached_department: String,
    pub cached_division: String,

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

        let code_only = if let Some(idx) = employee_code.find('(') {
            employee_code[..idx].trim()
        } else {
            employee_code.trim()
        };

        let emp_opt = if !code_only.is_empty() {
            crate::db::find_employee_by_code(code_only)
        } else {
            None
        };

        let cached_emp_name = if let Some(ref emp) = emp_opt {
            if !emp.emp_name.trim().is_empty() {
                emp.emp_name.clone()
            } else if !customer.trim().is_empty() && customer.trim() != "Internal" {
                customer.to_string()
            } else if !code_only.is_empty() {
                code_only.to_string()
            } else {
                "Unassigned".to_string()
            }
        } else if let Some(start) = employee_code.find('(') {
            if let Some(end) = employee_code.find(')') {
                if end > start + 1 {
                    employee_code[start + 1..end].trim().to_string()
                } else if !customer.trim().is_empty() && customer.trim() != "Internal" {
                    customer.to_string()
                } else if !code_only.is_empty() {
                    code_only.to_string()
                } else {
                    "Unassigned".to_string()
                }
            } else if !customer.trim().is_empty() && customer.trim() != "Internal" {
                customer.to_string()
            } else if !code_only.is_empty() {
                code_only.to_string()
            } else {
                "Unassigned".to_string()
            }
        } else if !customer.trim().is_empty() && customer.trim() != "Internal" {
            customer.to_string()
        } else if !code_only.is_empty() {
            code_only.to_string()
        } else {
            "Unassigned".to_string()
        };

        let cached_emp_email = if let Some(ref emp) = emp_opt {
            if !emp.emp_email.trim().is_empty() {
                emp.emp_email.clone()
            } else {
                email_address.to_string()
            }
        } else {
            email_address.to_string()
        };

        let cached_customer = if !cached_emp_name.is_empty() && cached_emp_name != "Unassigned" {
            cached_emp_name.clone()
        } else if !customer.trim().is_empty() && customer.trim() != "Internal" {
            customer.trim().to_string()
        } else {
            "N/A".to_string()
        };

        let cached_department = if let Some(ref emp) = emp_opt {
            if !emp.department.trim().is_empty() {
                emp.department.trim().to_string()
            } else if !category.trim().is_empty() {
                category.trim().to_string()
            } else {
                "Admin".to_string()
            }
        } else if !category.trim().is_empty() {
            category.trim().to_string()
        } else {
            "Admin".to_string()
        };

        let cached_division = if let Some(ref emp) = emp_opt {
            if !emp.division.trim().is_empty() {
                emp.division.trim().to_string()
            } else if emp.emp_email.trim().contains('@') {
                emp.emp_email.trim().to_string()
            } else if email_address.trim().contains('@') && !email_address.trim().is_empty() {
                email_address.trim().to_string()
            } else {
                "N/A".to_string()
            }
        } else if email_address.trim().contains('@') && !email_address.trim().is_empty() {
            email_address.trim().to_string()
        } else {
            "N/A".to_string()
        };

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
            tag: String::new(),

            cached_emp_name,
            cached_emp_email,
            cached_customer,
            cached_department,
            cached_division,

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
        } else if !t.stage.trim().is_empty() && is_generic_status(&t.stage) {
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
            kanban_state.clone()
        };

        let resolved_my_activity = resolve_ticket_activity_all(&t.my_activity, &t.stage, &t.activities, &t.tag);
        let my_activity = if !resolved_my_activity.is_empty() {
            resolved_my_activity
        } else {
            t.my_activity.clone()
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
            &my_activity,
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
        item.tag = if !t.tag.trim().is_empty() {
            t.tag.clone()
        } else {
            my_activity.clone()
        };
        if let Some(ref sid) = t.server_ticket_id {
            if !sid.trim().is_empty() {
                item.id = sid.clone();
            }
        }
        item
    }

    pub fn from_server_ticket(t: &crate::api::ServerTicket, local_id: i64) -> Self {
        let raw_kanban = if !t.kanban_state.trim().is_empty() {
            &t.kanban_state
        } else if !t.stage.trim().is_empty() && is_generic_status(&t.stage) {
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
            kanban_state.clone()
        };

        let resolved_my_activity = resolve_ticket_activity_all(&t.my_activity, &t.stage, &t.activities, "");
        let my_activity = if !resolved_my_activity.is_empty() {
            resolved_my_activity
        } else {
            t.my_activity.clone()
        };

        let mut item = Self::new(
            local_id,
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
            &my_activity,
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
        item.tag = my_activity;
        if !t.ticket_id.trim().is_empty() {
            item.id = t.ticket_id.clone();
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

    #[inline]
    pub fn emp_name(&self) -> String {
        if !self.cached_emp_name.is_empty() {
            self.cached_emp_name.clone()
        } else {
            "Unassigned".to_string()
        }
    }

    #[inline]
    pub fn emp_email(&self) -> String {
        self.cached_emp_email.clone()
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
    #[inline]
    pub fn customer_display(&self) -> String {
        if !self.cached_customer.is_empty() {
            self.cached_customer.clone()
        } else {
            "N/A".to_string()
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn department_display(&self) -> String {
        if !self.cached_department.is_empty() {
            self.cached_department.clone()
        } else {
            "Admin".to_string()
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub fn division_display(&self) -> String {
        if !self.cached_division.is_empty() {
            self.cached_division.clone()
        } else {
            "N/A".to_string()
        }
    }

    pub fn is_closed(&self) -> bool {
        let st = self.status.trim();
        let kb = self.kanban_state.trim();
        st.eq_ignore_ascii_case("closed")
            || st.eq_ignore_ascii_case("close")
            || st.eq_ignore_ascii_case("completed")
            || st.eq_ignore_ascii_case("solved(closed)")
            || st.eq_ignore_ascii_case("solved (closed)")
            || st.eq_ignore_ascii_case("cancelled")
            || st.eq_ignore_ascii_case("canceled")
            || kb.eq_ignore_ascii_case("completed")
            || kb.eq_ignore_ascii_case("solved(closed)")
            || kb.eq_ignore_ascii_case("solved (closed)")
            || kb.eq_ignore_ascii_case("cancelled")
            || kb.eq_ignore_ascii_case("canceled")
    }

    #[allow(dead_code)]
    #[inline]
    pub fn is_open(&self) -> bool {
        !self.is_closed()
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

/// Helper to strip leading "Actioned:" or "Actioned: " prefix (case-insensitive)
pub fn strip_actioned_prefix(val: &str) -> String {
    let trimmed = val.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("actioned:") {
        trimmed[9..].trim().to_string()
    } else if lower.starts_with("actioned :") {
        trimmed[10..].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Returns true if a status/stage string is a generic workflow column or placeholder
pub fn is_generic_status(val: &str) -> bool {
    let lower = val.trim().to_lowercase();
    matches!(
        lower.as_str(),
        "" | "triage"
            | "pending"
            | "pending review"
            | "in progress"
            | "inprogress"
            | "hold"
            | "on hold"
            | "onhold"
            | "solved"
            | "solved(closed)"
            | "solved (closed)"
            | "completed"
            | "closed"
            | "close"
            | "cancelled"
            | "canceled"
            | "open"
            | "done"
            | "unassigned"
            | "new"
            | "assigned"
            | "investigation"
            | "resolution"
    )
}

/// Extracts activity tags from Google Apps Script logged text like:
/// "Processed WB,SALE ORDER,AMD in 1h 34m" -> "WB,SALE ORDER,AMD"
pub fn extract_processed_activities(val: &str) -> Option<String> {
    let trimmed = val.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("processed ") {
        let rest = &trimmed[10..];
        if let Some(in_pos) = rest.rfind(" in ") {
            let act_part = rest[..in_pos].trim();
            if !act_part.is_empty() {
                return Some(act_part.to_string());
            }
        } else {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Returns true if an individual tag token is an empty or generic placeholder
pub fn is_generic_placeholder(val: &str) -> bool {
    let lower = val.trim().to_lowercase();
    matches!(
        lower.as_str(),
        "" | "triage"
            | "pending review"
            | "unassigned"
            | "none"
            | "n/a"
            | "ticket created"
            | "initial review"
    )
}

/// Normalizes a comma, newline, or semicolon-separated activity string into clean uppercase badges
pub fn normalize_activity_badges(val: &str) -> String {
    let clean = strip_actioned_prefix(val);
    if is_generic_status(&clean) || is_generic_placeholder(&clean) {
        return String::new();
    }
    let parts: Vec<String> = clean
        .split([',', '\n', '\r', ';'])
        .map(|s| strip_actioned_prefix(s).trim().to_uppercase())
        .filter(|s| !s.is_empty() && !is_generic_placeholder(s) && !is_generic_status(s))
        .collect();

    let mut deduped: Vec<String> = Vec::new();
    for p in parts {
        if !deduped.contains(&p) {
            deduped.push(p);
        }
    }
    deduped.join(", ")
}

fn append_activity_tokens(all_badges: &mut Vec<String>, raw: &str) {
    let norm = normalize_activity_badges(raw);
    if !norm.is_empty() {
        for item in norm.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if !all_badges.iter().any(|x: &String| x.eq_ignore_ascii_case(item)) {
                all_badges.push(item.to_string());
            }
        }
    }
}

/// Resolves all actual activities for a ticket across stage, tag, my_activity, and activities.
/// Collects and deduplicates all unique activity badges cleanly.
pub fn resolve_ticket_activity_all(
    my_activity: &str,
    stage: &str,
    activities: &str,
    tag: &str,
) -> String {
    let mut all_badges: Vec<String> = Vec::new();

    // 1. Gather from stage (primary store in Google Sheet per user configuration)
    append_activity_tokens(&mut all_badges, stage);

    // 2. Gather from tag
    append_activity_tokens(&mut all_badges, tag);

    // 3. Gather from my_activity (with Actioned: prefix stripped)
    append_activity_tokens(&mut all_badges, my_activity);

    // 4. Gather from activities field (extract if "Processed ... in ...")
    let cleaned_act = activities.trim();
    if !cleaned_act.is_empty() {
        if let Some(extracted) = extract_processed_activities(cleaned_act) {
            append_activity_tokens(&mut all_badges, &extracted);
        }
    }

    if !all_badges.is_empty() {
        return all_badges.join(", ");
    }

    // 5. Fallback: only if no badges were found anywhere else, check if activities
    // directly contains comma-separated action tokens or an uppercase code
    if !cleaned_act.is_empty()
        && !cleaned_act.eq_ignore_ascii_case("ticket created")
        && !is_generic_placeholder(cleaned_act)
        && !cleaned_act.to_lowercase().contains("open for")
        && (cleaned_act.contains(',') || cleaned_act.chars().any(|c| c.is_alphabetic()) && cleaned_act.chars().all(|c| !c.is_alphabetic() || c.is_uppercase()))
    {
        append_activity_tokens(&mut all_badges, cleaned_act);
    }

    all_badges.join(", ")
}

/// Backward-compatible wrapper resolving activities with an empty tag.
#[allow(dead_code)]
pub fn resolve_ticket_activity(my_activity: &str, stage: &str, activities: &str) -> String {
    resolve_ticket_activity_all(my_activity, stage, activities, "")
}

/// Standardizes a column / status ID to its corresponding Google Sheet status and macro/granular status.
pub fn target_col_to_sheet_status(col: &str) -> (&str, &'static str) {
    match col {
        "Pending" => ("Pending", "Pending"),
        "InProgress" => ("In Progress", "InProgress"),
        "Hold" | "Onhold" => ("Hold", "Hold"),
        "Solved" => ("Solved", "Solved"),
        "Completed" => ("Solved(closed)", "Closed"),
        "Cancelled" => ("Cancelled", "Cancelled"),
        _ => (col, "Pending"),
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
    match crate::sync::from_sheet_status(status) {
        "Pending" => "text-amber-500 dark:text-amber-400 border-amber-500/35 bg-amber-500/15",
        "InProgress" => "text-sky-500 dark:text-sky-400 border-sky-500/35 bg-sky-500/15",
        "Hold" => "text-amber-600 dark:text-amber-400 border-amber-500/35 bg-amber-500/15",
        "Solved" => "text-purple-500 dark:text-purple-400 border-purple-500/35 bg-purple-500/15",
        "Completed" => {
            "text-emerald-500 dark:text-emerald-400 border-emerald-500/35 bg-emerald-500/15"
        }
        "Cancelled" => "text-rose-500 dark:text-rose-400 border-rose-500/35 bg-rose-500/15",
        _ => "text-muted-foreground border-border bg-muted/40",
    }
}

pub fn status_badge_variant(status: &str) -> &'static str {
    match crate::sync::from_sheet_status(status) {
        "Pending" => "warning",
        "InProgress" => "info",
        "Hold" => "warning",
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
        let resolved = resolve_ticket_activity_all(&t.my_activity, &t.stage, &t.activities, &t.tag);
        if !resolved.is_empty() {
            t.my_activity = resolved.clone();
            t.tag = resolved;
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
        // Find the freshest version from memory or database so even after an edit it's always completely up-to-date
        let t = {
            let list = self.tickets.read();
            list.iter()
                .find(|item| item.id == t.id || (t.ticket_id > 0 && item.ticket_id == t.ticket_id))
                .cloned()
                .unwrap_or(t)
        };
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
        let resolved_act = resolve_ticket_activity_all(&t.my_activity, &t.stage, &t.activities, &t.tag);
        let act_to_show = if !resolved_act.is_empty() {
            resolved_act
        } else if !t.tag.trim().is_empty() && !is_generic_status(&t.tag) {
            normalize_activity_badges(&t.tag)
        } else if !t.stage.trim().is_empty() && !is_generic_status(&t.stage) {
            normalize_activity_badges(&t.stage)
        } else if !t.my_activity.trim().is_empty() && !is_generic_status(&t.my_activity) {
            normalize_activity_badges(&t.my_activity)
        } else {
            String::new()
        };
        self.edit_my_activity.set(act_to_show.clone());
        self.edit_deadline.set(t.deadline.clone());
        self.edit_sla_status.set(t.sla_status.clone());
        self.edit_sla_deadline.set(t.sla_deadline.clone());
        self.edit_rating_avg_text.set(t.rating_avg_text.clone());
        self.edit_kanban_state.set(t.kanban_state.clone());
        let stage_display = if !act_to_show.is_empty() {
            act_to_show
        } else {
            t.stage.clone()
        };
        self.edit_stage.set(stage_display);
        self.edit_status.set(t.status.clone());

        self.editing_ticket.set(Some(t));
        self.is_edit_open.set(true);
        self.is_view_open.set(false);
        self.is_create_open.set(false);
    }

    pub fn update_status(&mut self, ticket_id: &str, new_status: &str) {
        let mut local_num = None;
        let mut target_stage = None;
        let (sheet_kanban, derived_macro_status) = target_col_to_sheet_status(new_status);

        {
            let mut list = self.tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
                list[pos].status = derived_macro_status.to_string();
                list[pos].kanban_state = new_status.to_string();
                target_stage = Some(list[pos].stage.clone());
                local_num = Some(list[pos].ticket_id);
            }
        }
        let num = local_num.or_else(|| crate::db::find_ticket_local_id(ticket_id));
        if let Some(num) = num {
            let _ = crate::db::update_ticket_status(num, new_status);
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
        let (sheet_kanban, derived_macro_status) = target_col_to_sheet_status(to_col);

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
                t.status = derived_macro_status.to_string();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_ticket_activity_from_all_fields() {
        // Stage has activities
        let r1 = resolve_ticket_activity_all("", "WB,OFFLINE,ENTRY", "", "");
        assert_eq!(r1, "WB, OFFLINE, ENTRY");

        // Actioned prefix in my_activity with stage generic
        let r2 = resolve_ticket_activity_all("Actioned: Party Bill Date,Change", "Solved(closed)", "", "");
        assert_eq!(r2, "PARTY BILL DATE, CHANGE");

        // Tag field
        let r3 = resolve_ticket_activity_all("", "Pending", "", "PO, AMD, APPROVAL");
        assert_eq!(r3, "PO, AMD, APPROVAL");

        // Activities field with Processed ... in ...
        let r4 = resolve_ticket_activity_all("", "In Progress", "Processed WB,SALE ORDER,AMD in 1h 34m", "");
        assert_eq!(r4, "WB, SALE ORDER, AMD");

        // Direct activities field
        let r5 = resolve_ticket_activity_all("", "Closed", "Laser Unit,Clean,Pick-up Roller,Change", "");
        assert_eq!(r5, "LASER UNIT, CLEAN, PICK-UP ROLLER, CHANGE");

        // Multiline / newline-separated activities
        let r6 = resolve_ticket_activity_all("", "SMPS Change\nCPU FAN Change\nSSD Change", "", "");
        assert_eq!(r6, "SMPS CHANGE, CPU FAN CHANGE, SSD CHANGE");

        // Semicolon-separated activities
        let r7 = resolve_ticket_activity_all("", "ERP; Restart", "", "");
        assert_eq!(r7, "ERP, RESTART");

        // Deduplication across stage and my_activity
        let r8 = resolve_ticket_activity_all("PO, AMD", "PO, AMD, APPROVAL", "", "");
        assert_eq!(r8, "PO, AMD, APPROVAL");
    }

    #[test]
    fn test_multiple_ticket_states_mapping_and_preservation() {
        // Verify target_col_to_sheet_status returns appropriate sheet_kanban and granular status
        assert_eq!(target_col_to_sheet_status("Pending"), ("Pending", "Pending"));
        assert_eq!(target_col_to_sheet_status("InProgress"), ("In Progress", "InProgress"));
        assert_eq!(target_col_to_sheet_status("Hold"), ("Hold", "Hold"));
        assert_eq!(target_col_to_sheet_status("Solved"), ("Solved", "Solved"));
        assert_eq!(target_col_to_sheet_status("Completed"), ("Solved(closed)", "Closed"));
        assert_eq!(target_col_to_sheet_status("Cancelled"), ("Cancelled", "Cancelled"));

        // Verify to_sheet_status preserves granular statuses
        assert_eq!(crate::sync::to_sheet_status("Pending"), "Open");
        assert_eq!(crate::sync::to_sheet_status("InProgress"), "In Progress");
        assert_eq!(crate::sync::to_sheet_status("Hold"), "On Hold");
        assert_eq!(crate::sync::to_sheet_status("Solved"), "Solved");
        assert_eq!(crate::sync::to_sheet_status("Completed"), "Closed");
        assert_eq!(crate::sync::to_sheet_status("Cancelled"), "Cancelled");

        // Verify to_bpsl_status preserves granular statuses
        assert_eq!(crate::sync::to_bpsl_status("Pending"), "Pending");
        assert_eq!(crate::sync::to_bpsl_status("InProgress"), "InProgress");
        assert_eq!(crate::sync::to_bpsl_status("Hold"), "Hold");
        assert_eq!(crate::sync::to_bpsl_status("Solved"), "Solved");
        assert_eq!(crate::sync::to_bpsl_status("Closed"), "Closed");
        assert_eq!(crate::sync::to_bpsl_status("Cancelled"), "Cancelled");

        // Verify TicketItem is_open vs is_closed
        let mut ticket = TicketItem::new(
            1, "EMP1", "emp@test.com", "IT", "Test", "Test issue", "", "",
            "Medium", "Shreyansh", "Customer", 0.0, "", "", "", "Within SLA", "", "Unrated",
            "InProgress", "Triage", "InProgress", "2026-11-24", "2026-11-24",
        );
        assert!(ticket.is_open());
        assert!(!ticket.is_closed());

        ticket.kanban_state = "Hold".to_string();
        ticket.status = "Hold".to_string();
        assert!(ticket.is_open());
        assert!(!ticket.is_closed());

        ticket.kanban_state = "Solved".to_string();
        ticket.status = "Solved".to_string();
        assert!(ticket.is_open());
        assert!(!ticket.is_closed());

        ticket.kanban_state = "Completed".to_string();
        ticket.status = "Closed".to_string();
        assert!(ticket.is_closed());
        assert!(!ticket.is_open());

        ticket.kanban_state = "Cancelled".to_string();
        ticket.status = "Cancelled".to_string();
        assert!(ticket.is_closed());
        assert!(!ticket.is_open());
    }
}
