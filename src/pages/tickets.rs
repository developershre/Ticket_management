//! IT Ticket Management — Kanban Board (Pending, InProgress, Onhold, Completed)
//! and full-width interactive DataTable with drag & drop support.
#![allow(clippy::redundant_closure)]

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
use crate::components::ui::data_table::{DataTable, DataTableColumn, DataTableFilter, SortValue};
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator,
    DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
use crate::components::ui::input::Input;
use crate::components::ui::select::SimpleSelect;
use crate::components::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
use crate::components::ui::toast::ToastState;
use crate::icons::{
    CheckIcon, CopyIcon, ExternalLinkIcon, EyeIcon, FileTextIcon, FilterIcon, GripVerticalIcon,
    MoreHorizontalIcon, PaperclipIcon, PencilIcon, PlusIcon, SearchIcon, TrashIcon,
    UploadCloudIcon, XIcon,
};

/// Extract direct viewable image preview URL (supporting Google Drive links and standard image formats).
fn get_image_preview_url(filename: &str, url: &str) -> Option<String> {
    let lower_name = filename.to_lowercase();
    let lower_url = url.to_lowercase();

    let is_img = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"]
        .iter()
        .any(|ext| lower_name.ends_with(ext) || lower_url.contains(ext))
        || lower_name.contains("screenshot")
        || lower_url.contains("drive.google.com");

    if !is_img && url.trim().is_empty() {
        return None;
    }

    let url_trimmed = url.trim();

    // If it's a Google Drive link, extract file ID and convert to direct image preview URL
    if url_trimmed.contains("drive.google.com") {
        if let Some(pos) = url_trimmed.find("/file/d/") {
            let after = &url_trimmed[pos + 8..];
            let id = after
                .split('/')
                .next()
                .unwrap_or(after)
                .split('?')
                .next()
                .unwrap_or(after);
            if !id.is_empty() {
                return Some(format!("https://lh3.googleusercontent.com/d/{id}"));
            }
        } else if let Some(pos) = url_trimmed.find("id=") {
            let after = &url_trimmed[pos + 3..];
            let id = after.split('&').next().unwrap_or(after);
            if !id.is_empty() {
                return Some(format!("https://lh3.googleusercontent.com/d/{id}"));
            }
        }
    }

    if is_img && !url_trimmed.is_empty() {
        return Some(url_trimmed.to_string());
    }

    None
}

fn remove_activity(mut value: Signal<String>, item: &str) {
    let item_upper = item.trim().to_uppercase();
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    list.retain(|x| x != &item_upper);
    value.set(list.join(", ").to_uppercase());
}

fn add_custom_activity(mut value: Signal<String>, mut custom_input: Signal<String>) {
    let text = custom_input.read().trim().to_uppercase();
    if text.is_empty() {
        return;
    }
    let parts: Vec<&str> = text
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return;
    }
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    for p in parts {
        let p_upper = p.to_uppercase();
        if !list.contains(&p_upper) {
            list.push(p_upper);
        }
    }
    value.set(list.join(", ").to_uppercase());
    custom_input.set(String::new());
}

fn pop_last_activity(mut value: Signal<String>) {
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    if list.pop().is_some() {
        value.set(list.join(", ").to_uppercase());
    }
}

#[derive(Props, Clone, PartialEq)]
struct MultiActivitySelectorProps {
    value: Signal<String>,
}

#[component]
fn MultiActivitySelector(props: MultiActivitySelectorProps) -> Element {
    let mut custom_input = use_signal(String::new);
    let value = props.value;

    let raw_val = value.read().clone();
    let selected_items: Vec<String> = raw_val
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();

    rsx! {
        // Tag input container — all badges lie directly inside this input field
        div { class: "min-h-[42px] w-full rounded-md border border-border/80 bg-card px-2.5 py-1.5 flex flex-wrap items-center gap-1.5 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/20 transition-all cursor-text",
            for item in selected_items.iter() {
                {
                    let item_str = item.clone();
                    rsx! {
                        span {
                            key: "{item_str}",
                            class: "inline-flex items-center gap-1 rounded bg-primary/15 px-2 py-0.5 text-[11px] font-bold tracking-wide text-primary border border-primary/30 uppercase select-none shadow-2xs",
                            "{item_str}"
                            button {
                                r#type: "button",
                                class: "hover:text-destructive cursor-pointer ml-0.5 font-bold leading-none select-none text-xs",
                                onclick: {
                                    let to_remove = item_str.clone();
                                    move |_| remove_activity(value, &to_remove)
                                },
                                "×"
                            }
                        }
                    }
                }
            }

            input {
                r#type: "text",
                class: "flex-1 min-w-[160px] bg-transparent text-xs text-foreground placeholder:text-muted-foreground focus:outline-none uppercase font-mono py-0.5",
                placeholder: if selected_items.is_empty() { "Type activity and press Enter..." } else { "Type and press Enter..." },
                value: custom_input.read().clone(),
                oninput: move |e: FormEvent| {
                    let val = e.value();
                    if val.ends_with(',') {
                        custom_input.set(val[..val.len() - 1].to_uppercase());
                        add_custom_activity(value, custom_input);
                    } else {
                        custom_input.set(val.to_uppercase());
                    }
                },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        add_custom_activity(value, custom_input);
                    } else if e.key() == Key::Backspace && custom_input.read().is_empty() {
                        pop_last_activity(value);
                    }
                },
            }
        }
    }
}

/// IT Ticket Item model storing all 23 standardized fields.
#[derive(Clone, PartialEq, Debug)]
pub struct TicketItem {
    // 1. Ticket ID
    pub ticket_id: i64,
    // 2. Employee Code
    pub employee_code: String,
    // 3. Email Address
    pub email_address: String,
    // 4. Category
    pub category: String,
    // 5. Subject
    pub subject: String,
    // 6. Problem Description
    pub problem_description: String,
    // 7. Attachment Name
    pub attachment_name: String,
    // 8. Attachment URL
    pub attachment_url: String,
    // 9. Priority
    pub priority: String,
    // 10. Assigned to
    pub assigned_to: String,
    // 11. Customer
    pub customer: String,
    // 12. Time Spent (hours)
    pub time_spent: f64,
    // 13. Activities
    pub activities: String,
    // 14. My Activity
    pub my_activity: String,
    // 15. Deadline
    pub deadline: String,
    // 16. SLA Status
    pub sla_status: String,
    // 17. SLA Deadline
    pub sla_deadline: String,
    // 18. Rating Avg Text
    pub rating_avg_text: String,
    // 19. Kanban State ("Pending" | "InProgress" | "Onhold" | "Completed")
    pub kanban_state: String,
    // 20. Stage
    pub stage: String,
    // 21. Status
    pub status: String,
    // 22. Created on
    pub created_on: String,
    // 23. Last Updated on
    pub last_updated_on: String,

    // Aliases for convenience & UI compatibility:
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

            // aliases:
            id,
            title: subject.to_string(),
            employee: employee_code.to_string(),
            assignee: assigned_to.to_string(),
            created_at: created_on.to_string(),
        }
    }

    pub fn from_db_ticket(t: crate::db::DbTicket) -> Self {
        let raw_stage = if !t.kanban_state.is_empty() {
            &t.kanban_state
        } else if !t.stage.is_empty() {
            &t.stage
        } else {
            &t.status
        };
        let stage = crate::sync::from_sheet_status(raw_stage).to_string();

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
            &stage,
            &stage,
            &stage,
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
        if let (Some(start), Some(end)) =
            (self.employee_code.find('('), self.employee_code.rfind(')'))
        {
            if end > start {
                let inside = self.employee_code[start + 1..end].trim();
                if !inside.is_empty() {
                    return inside.to_string();
                }
            }
        }
        if let Some(user_part) = self.email_address.split('@').next() {
            let parts: Vec<String> = user_part
                .split('.')
                .map(|p| {
                    let mut c = p.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect();
            if !parts.is_empty() {
                return parts.join(" ");
            }
        }
        self.employee_code.clone()
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
    pub fn open_view(&mut self, t: TicketItem) {
        self.viewing_ticket.set(Some(t));
        self.is_view_open.set(true);
    }

    pub fn open_create(&mut self, default_column: Option<&str>) {
        self.create_emp_name.set(String::new());
        self.create_employee_code.set(String::new());
        self.create_email_address.set(String::new());
        self.create_category.set("Select category".to_string());
        self.create_subject.set(String::new());
        self.create_problem_description.set(String::new());
        self.create_attachment_name.set(String::new());
        self.create_attachment_url.set(String::new());
        self.create_priority.set("Low".to_string());
        self.create_assigned_to.set("IT Support".to_string());
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
        self.edit_emp_name.set(t.emp_name());
        self.edit_employee_code.set(t.emp_code_only().to_string());
        self.edit_email_address.set(t.email_address.clone());
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
        self.edit_stage.set(t.kanban_state.clone());
        self.edit_status.set(t.kanban_state.clone());

        self.editing_ticket.set(Some(t));
        self.is_edit_open.set(true);
        self.is_view_open.set(false);
        self.is_create_open.set(false);
    }

    pub fn update_status(&mut self, ticket_id: &str, new_status: &str) {
        let mut local_num = None;
        let mut list = self.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
            list[pos].status = new_status.to_string();
            list[pos].kanban_state = new_status.to_string();
            list[pos].stage = new_status.to_string();
            local_num = Some(list[pos].ticket_id);
        }
        let num = local_num
            .or_else(|| {
                ticket_id
                    .strip_prefix("IT-")
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| ticket_id.parse::<i64>().ok());
        if let Some(num) = num {
            let _ = crate::db::update_ticket_status(num, new_status);
            crate::sync::enqueue_update(
                num,
                serde_json::json!({
                    "kanban_state": new_status,
                    "stage": new_status,
                    "status": new_status,
                }),
            );
        }
    }

    pub fn update_priority(&mut self, ticket_id: &str, new_priority: &str) {
        let mut local_num = None;
        let mut list = self.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
            list[pos].priority = new_priority.to_string();
            local_num = Some(list[pos].ticket_id);
        }
        let num = local_num
            .or_else(|| {
                ticket_id
                    .strip_prefix("IT-")
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| ticket_id.parse::<i64>().ok());
        if let Some(num) = num {
            let _ = crate::db::update_ticket_priority(num, new_priority);
            crate::sync::enqueue_update(num, serde_json::json!({ "priority": new_priority }));
        }
    }

    pub fn update_category(&mut self, ticket_id: &str, new_category: &str) {
        let mut local_num = None;
        let mut list = self.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
            list[pos].category = new_category.to_string();
            local_num = Some(list[pos].ticket_id);
        }
        let num = local_num
            .or_else(|| {
                ticket_id
                    .strip_prefix("IT-")
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| ticket_id.parse::<i64>().ok());
        if let Some(num) = num {
            let _ = crate::db::update_ticket_category(num, new_category);
            crate::sync::enqueue_update(num, serde_json::json!({ "category": new_category }));
        }
    }

    pub fn update_assignee(&mut self, ticket_id: &str, new_assignee: &str) {
        let mut local_num = None;
        let mut list = self.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == ticket_id) {
            list[pos].assigned_to = new_assignee.to_string();
            list[pos].assignee = new_assignee.to_string();
            local_num = Some(list[pos].ticket_id);
        }
        let num = local_num
            .or_else(|| {
                ticket_id
                    .strip_prefix("IT-")
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| ticket_id.parse::<i64>().ok());
        if let Some(num) = num {
            let _ = crate::db::update_ticket_assignee(num, new_assignee);
            crate::sync::enqueue_update(num, serde_json::json!({ "assigned_to": new_assignee }));
        }
    }

    pub fn move_column_tickets(&mut self, from_col: &str, to_col: &str) {
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
                t.stage = to_col.to_string();
                t.status = to_col.to_string();
                let num = t.ticket_id;
                let _ = crate::db::update_ticket_status(num, to_col);
                crate::sync::enqueue_update(
                    num,
                    serde_json::json!({
                        "kanban_state": to_col,
                        "stage": to_col,
                        "status": to_col,
                    }),
                );
            }
        }
    }

    pub fn delete_ticket(&mut self, ticket_id: &str) {
        // Queue the delete BEFORE the row is removed — the server mapping
        // is read from the row (an unmapped row needs no server call).
        let local_id = self
            .tickets
            .read()
            .iter()
            .find(|t| t.id == ticket_id)
            .map(|t| t.ticket_id)
            .or_else(|| {
                ticket_id
                    .strip_prefix("IT-")
                    .and_then(|s| s.parse::<i64>().ok())
            })
            .or_else(|| ticket_id.parse::<i64>().ok());

        if let Some(num) = local_id {
            crate::sync::enqueue_delete(num);
        }
        let mut list = self.tickets.write();
        list.retain(|t| t.id != ticket_id);
        if let Some(num) = local_id {
            let _ = crate::db::delete_ticket(num);
        }
    }
}

pub const COLUMN_NAMES: &[&str] = &["Pending", "InProgress", "Onhold", "Completed"];

fn initial_tickets() -> Vec<TicketItem> {
    // Tickets are read from SQLite only — the local cache of the Google
    // Sheet. An empty table simply means nothing has been pulled yet; no
    // demo/fallback rows are ever fabricated.
    crate::db::get_all_tickets()
        .into_iter()
        .map(TicketItem::from_db_ticket)
        .collect()
}

fn column_display_label(col: &str) -> &'static str {
    match col {
        "Pending" => "Pending",
        "InProgress" => "In Progress",
        "Onhold" => "On Hold",
        "Completed" => "Completed",
        _ => "Other",
    }
}

fn column_subtitle(col: &str) -> &'static str {
    match col {
        "Pending" => "New & unassigned",
        "InProgress" => "Active support",
        "Onhold" => "Waiting on user/vendor",
        "Completed" => "Resolved & closed",
        _ => "",
    }
}

fn priority_badge(priority: &str) -> &'static str {
    match priority {
        "Critical" | "High" => "destructive",
        "Medium" => "warning",
        "Low" => "info",
        _ => "secondary",
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "Pending" => "text-amber-500 dark:text-amber-400 border-amber-500/35 bg-amber-500/15",
        "InProgress" => "text-sky-500 dark:text-sky-400 border-sky-500/35 bg-sky-500/15",
        "Onhold" => "text-purple-500 dark:text-purple-400 border-purple-500/35 bg-purple-500/15",
        "Completed" => {
            "text-emerald-500 dark:text-emerald-400 border-emerald-500/35 bg-emerald-500/15"
        }
        _ => "text-muted-foreground border-border bg-muted/40",
    }
}

fn status_badge_variant(status: &str) -> &'static str {
    match status {
        "Pending" => "warning",
        "InProgress" => "info",
        "Onhold" => "purple",
        "Completed" => "success",
        _ => "secondary",
    }
}

fn sla_badge_class(sla: &str) -> &'static str {
    match sla {
        "Within SLA" => {
            "text-emerald-500 dark:text-emerald-400 border-emerald-500/35 bg-emerald-500/15"
        }
        "Approaching Breach" => {
            "text-amber-500 dark:text-amber-400 border-amber-500/35 bg-amber-500/15"
        }
        "Breached" => "text-rose-500 dark:text-rose-400 border-rose-500/35 bg-rose-500/15",
        _ => "text-muted-foreground border-border bg-muted/30",
    }
}

// ============================================================
// Context Menus
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct TicketContextMenuProps {
    ticket: TicketItem,
    children: Element,
    on_view: EventHandler<TicketItem>,
    on_edit: EventHandler<TicketItem>,
}

#[component]
fn TicketContextMenu(props: TicketContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t = props.ticket;
    let t_id = t.id.clone();
    let current_status = t.kanban_state.clone();
    let current_prio = t.priority.clone();
    let current_cat = t.category.clone();
    let current_assignee = t.assigned_to.clone();

    rsx! {
        ContextMenu {
            ContextMenuTrigger { class: "contents", {props.children} }
            ContextMenuContent { class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    div { class: "flex items-center justify-between gap-2",
                        span { class: "font-mono font-bold text-foreground text-xs",
                            "{t_id}"
                        }
                        span { class: "rounded px-1.5 py-0.2 text-[10px] font-semibold bg-muted text-muted-foreground",
                            "{t.priority}"
                        }
                    }
                    span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5",
                        "{t.emp_name()} • {t.emp_code_only()}"
                    }
                }
                ContextMenuSeparator {}

                ContextMenuItem {
                    on_select: {
                        let t_c = t.clone();
                        let handler = props.on_view;
                        move |_| handler.call(t_c.clone())
                    },
                    EyeIcon { class: "size-3.5 text-muted-foreground" }
                    "View Ticket"
                }

                ContextMenuItem {
                    on_select: {
                        let t_c = t.clone();
                        let handler = props.on_edit;
                        move |_| handler.call(t_c.clone())
                    },
                    PencilIcon { class: "size-3.5 text-muted-foreground" }
                    "Modify Ticket"
                }

                ContextMenuSeparator {}

                ContextMenuSub {
                    ContextMenuSubTrigger { "Change Status" }
                    ContextMenuSubContent {
                        for & (code , label) in &[
                            ("Pending", "Pending"),
                            ("InProgress", "In Progress"),
                            ("Onhold", "On Hold"),
                            ("Completed", "Completed"),
                        ]
                        {
                            {
                                let is_active = current_status == code;
                                let id_c = t_id.clone();
                                let label_str = label.to_string();
                                rsx! {
                                    ContextMenuItem {
                                        key: "status-{code}",
                                        on_select: move |_| {
                                            state.update_status(&id_c, code);
                                            toast.success("Status Updated", format!("{id_c} moved to {label_str}"));
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{label}" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuSub {
                    ContextMenuSubTrigger { "Change Priority" }
                    ContextMenuSubContent {
                        for & prio in &["Urgent", "High", "Medium", "Low"] {
                            {
                                let is_active = current_prio == prio;
                                let id_c = t_id.clone();
                                rsx! {
                                    ContextMenuItem {
                                        key: "prio-{prio}",
                                        on_select: move |_| {
                                            state.update_priority(&id_c, prio);
                                            toast.info("Priority Updated", format!("{id_c} priority set to {prio}"));
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{prio}" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuSub {
                    ContextMenuSubTrigger { "Change Category" }
                    ContextMenuSubContent {
                        for & cat in &["Hardware", "Network", "Software", "Access", "Email"] {
                            {
                                let is_active = current_cat == cat;
                                let id_c = t_id.clone();
                                let cat_str = cat.to_string();
                                rsx! {
                                    ContextMenuItem {
                                        key: "cat-{cat}",
                                        on_select: move |_| {
                                            state.update_category(&id_c, cat);
                                            toast.info("Category Updated", format!("{id_c} category set to {cat_str}"));
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{cat}" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuSub {
                    ContextMenuSubTrigger { "Assign To" }
                    ContextMenuSubContent {
                        for & (code , name) in &[
                            ("Shreyansh", "Shreyansh"),
                            ("kaveri", "Kaveri"),
                            ("Chaitram", "Chaitram"),
                            ("IT Support", "IT Support"),
                            ("", "Unassigned"),
                        ]
                        {
                            {
                                let is_active = current_assignee == code;
                                let id_c = t_id.clone();
                                let name_str = name.to_string();
                                rsx! {
                                    ContextMenuItem {
                                        key: "assignee-{code}",
                                        on_select: move |_| {
                                            state.update_assignee(&id_c, code);
                                            toast.success("Assignee Updated", format!("{id_c} assigned to {name_str}"));
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{name}" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuItem {
                    on_select: {
                        let id_c = t_id.clone();
                        move |_| {
                            toast.info("Copied", format!("{id_c} copied to clipboard"));
                        }
                    },
                    CopyIcon { class: "size-3.5 text-muted-foreground" }
                    "Copy Ticket ID"
                }

                ContextMenuSeparator {}

                ContextMenuItem {
                    destructive: true,
                    on_select: {
                        let id_c = t_id.clone();
                        move |_| {
                            state.delete_ticket(&id_c);
                            toast.error("Deleted", format!("{id_c} removed"));
                        }
                    },
                    TrashIcon { class: "size-3.5 text-destructive" }
                    "Delete Ticket"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct TableFloatingContextMenuProps {
    ticket: TicketItem,
    open: Signal<bool>,
    x: Signal<f64>,
    y: Signal<f64>,
    on_view: EventHandler<TicketItem>,
    on_edit: EventHandler<TicketItem>,
}

#[component]
fn TableFloatingContextMenu(props: TableFloatingContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t = props.ticket;
    let t_id = t.id.clone();
    let current_status = t.kanban_state.clone();
    let current_prio = t.priority.clone();
    let current_cat = t.category.clone();
    let current_assignee = t.assigned_to.clone();

    use_context_provider(|| crate::components::ui::context_menu::ContextMenuState {
        open: props.open,
        x: props.x,
        y: props.y,
    });

    rsx! {
        ContextMenuContent { class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
            ContextMenuLabel {
                div { class: "flex items-center justify-between gap-2",
                    span { class: "font-mono font-bold text-foreground text-xs", "{t_id}" }
                    span { class: "rounded px-1.5 py-0.2 text-[10px] font-semibold bg-muted text-muted-foreground",
                        "{t.priority}"
                    }
                }
                span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5",
                    "{t.emp_name()} • {t.emp_code_only()}"
                }
            }
            ContextMenuSeparator {}

            ContextMenuItem {
                on_select: {
                    let t_c = t.clone();
                    let handler = props.on_view;
                    move |_| handler.call(t_c.clone())
                },
                EyeIcon { class: "size-3.5 text-muted-foreground" }
                "View Ticket"
            }

            ContextMenuItem {
                on_select: {
                    let t_c = t.clone();
                    let handler = props.on_edit;
                    move |_| handler.call(t_c.clone())
                },
                PencilIcon { class: "size-3.5 text-muted-foreground" }
                "Modify Ticket"
            }

            ContextMenuSeparator {}

            ContextMenuSub {
                ContextMenuSubTrigger { "Change Status" }
                ContextMenuSubContent {
                    for & (code , label) in &[
                        ("Pending", "Pending"),
                        ("InProgress", "In Progress"),
                        ("Onhold", "On Hold"),
                        ("Completed", "Completed"),
                    ]
                    {
                        {
                            let is_active = current_status == code;
                            let id_c = t_id.clone();
                            let label_str = label.to_string();
                            rsx! {
                                ContextMenuItem {
                                    key: "status-{code}",
                                    on_select: move |_| {
                                        state.update_status(&id_c, code);
                                        toast.success("Status Updated", format!("{id_c} moved to {label_str}"));
                                    },
                                    div { class: "flex items-center justify-between w-full",
                                        span { "{label}" }
                                        if is_active {
                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ContextMenuSub {
                ContextMenuSubTrigger { "Change Priority" }
                ContextMenuSubContent {
                    for & prio in &["Urgent", "High", "Medium", "Low"] {
                        {
                            let is_active = current_prio == prio;
                            let id_c = t_id.clone();
                            rsx! {
                                ContextMenuItem {
                                    key: "prio-{prio}",
                                    on_select: move |_| {
                                        state.update_priority(&id_c, prio);
                                        toast.info("Priority Updated", format!("{id_c} priority set to {prio}"));
                                    },
                                    div { class: "flex items-center justify-between w-full",
                                        span { "{prio}" }
                                        if is_active {
                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ContextMenuSub {
                ContextMenuSubTrigger { "Change Category" }
                ContextMenuSubContent {
                    for & cat in &["Hardware", "Network", "Software", "Access", "Email"] {
                        {
                            let is_active = current_cat == cat;
                            let id_c = t_id.clone();
                            let cat_str = cat.to_string();
                            rsx! {
                                ContextMenuItem {
                                    key: "cat-{cat}",
                                    on_select: move |_| {
                                        state.update_category(&id_c, cat);
                                        toast.info("Category Updated", format!("{id_c} category set to {cat_str}"));
                                    },
                                    div { class: "flex items-center justify-between w-full",
                                        span { "{cat}" }
                                        if is_active {
                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ContextMenuSub {
                ContextMenuSubTrigger { "Assign To" }
                ContextMenuSubContent {
                    for & (code , name) in &[
                        ("Shreyansh", "Shreyansh"),
                        ("kaveri", "Kaveri"),
                        ("Chaitram", "Chaitram"),
                        ("IT Support", "IT Support"),
                        ("", "Unassigned"),
                    ]
                    {
                        {
                            let is_active = current_assignee == code;
                            let id_c = t_id.clone();
                            let name_str = name.to_string();
                            rsx! {
                                ContextMenuItem {
                                    key: "assignee-{code}",
                                    on_select: move |_| {
                                        state.update_assignee(&id_c, code);
                                        toast.success("Assignee Updated", format!("{id_c} assigned to {name_str}"));
                                    },
                                    div { class: "flex items-center justify-between w-full",
                                        span { "{name}" }
                                        if is_active {
                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            ContextMenuItem {
                on_select: {
                    let id_c = t_id.clone();
                    move |_| {
                        toast.info("Copied", format!("{id_c} copied to clipboard"));
                    }
                },
                CopyIcon { class: "size-3.5 text-muted-foreground" }
                "Copy Ticket ID"
            }

            ContextMenuSeparator {}

            ContextMenuItem {
                destructive: true,
                on_select: {
                    let id_c = t_id.clone();
                    move |_| {
                        state.delete_ticket(&id_c);
                        toast.error("Deleted", format!("{id_c} removed"));
                    }
                },
                TrashIcon { class: "size-3.5 text-destructive" }
                "Delete Ticket"
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ColumnContextMenuProps {
    column: String,
    children: Element,
    on_quick_add: EventHandler<String>,
    on_filter_column: EventHandler<String>,
    on_clear_filters: EventHandler<()>,
}

#[component]
fn ColumnContextMenu(props: ColumnContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let col = props.column;
    let col_disp = column_display_label(&col);

    rsx! {
        ContextMenu {
            ContextMenuTrigger { class: "contents", {props.children} }
            ContextMenuContent { class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    span { class: "font-semibold text-foreground text-xs", "Column: {col_disp}" }
                }
                ContextMenuSeparator {}
                ContextMenuItem {
                    on_select: {
                        let col_c = col.clone();
                        let handler = props.on_quick_add;
                        move |_| handler.call(col_c.clone())
                    },
                    PlusIcon { class: "size-3.5 text-primary" }
                    "Quick Add Ticket"
                }
                ContextMenuItem {
                    on_select: {
                        let col_c = col.clone();
                        let handler = props.on_filter_column;
                        move |_| handler.call(col_c.clone())
                    },
                    FilterIcon { class: "size-3.5 text-muted-foreground" }
                    "Filter: Only {col_disp}"
                }
                ContextMenuSub {
                    ContextMenuSubTrigger { "Move All Tickets To" }
                    ContextMenuSubContent {
                        for & target_col in COLUMN_NAMES {
                            {
                                let is_current = target_col == col.as_str();
                                let from_c = col.clone();
                                let target_label = column_display_label(target_col);
                                rsx! {
                                    ContextMenuItem {
                                        key: "move-all-{target_col}",
                                        disabled: is_current,
                                        on_select: move |_| {
                                            state.move_column_tickets(&from_c, target_col);
                                            toast.info("Tickets Moved", format!("All tickets moved to {target_label}"));
                                        },
                                        span { "{target_label}" }
                                    }
                                }
                            }
                        }
                    }
                }
                ContextMenuSeparator {}
                ContextMenuItem {
                    on_select: {
                        let handler = props.on_clear_filters;
                        move |_| handler.call(())
                    },
                    "Clear Column Filter"
                }
            }
        }
    }
}

// ============================================================
// DataTable Cell Renderers
// ============================================================

#[component]
fn TicketIdCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let id = ticket.id.clone();
    let t_view = ticket.clone();
    let t_edit = ticket.clone();
    rsx! {
        TicketContextMenu {
            ticket: ticket.clone(),
            on_view: move |_| state.open_view(t_view.clone()),
            on_edit: move |_| state.open_edit(t_edit.clone()),
            span { class: "font-mono font-bold text-foreground text-xs hover:text-primary transition-colors cursor-context-menu",
                "{id}"
            }
        }
    }
}

#[component]
fn TicketTitleCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let title = ticket.title.clone();
    let emp_name = ticket.emp_name();
    let emp_code = ticket.emp_code_only().to_string();
    let email = ticket.email_address.clone();
    let t_view = ticket.clone();
    let t_edit = ticket.clone();
    rsx! {
        TicketContextMenu {
            ticket: ticket.clone(),
            on_view: move |_| state.open_view(t_view.clone()),
            on_edit: move |_| state.open_edit(t_edit.clone()),
            div { class: "flex flex-col min-w-0 py-0.5 cursor-context-menu",
                span { class: "font-semibold text-foreground truncate text-xs hover:text-primary transition-colors",
                    "{title}"
                }
                div { class: "flex items-center gap-1.5 text-[11px] text-muted-foreground truncate",
                    span { class: "font-medium text-foreground/80", "{emp_name}" }
                    span { "•" }
                    span { class: "font-mono text-[10px]", "{emp_code}" }
                    span { "•" }
                    span { class: "truncate opacity-80", "{email}" }
                }
            }
        }
    }
}

#[component]
fn TicketActionsCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t_view = ticket.clone();
    let t_edit = ticket.clone();
    let t_del = ticket.clone();
    let t_copy = ticket.clone();
    rsx! {
        div { class: "flex items-center gap-1",
            button {
                r#type: "button",
                title: "View Ticket Details",
                class: "cursor-pointer inline-flex size-7 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    state.open_view(t_view.clone());
                },
                EyeIcon { class: "size-3.5" }
            }
            button {
                r#type: "button",
                title: "Modify Ticket",
                class: "cursor-pointer inline-flex size-7 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    state.open_edit(t_edit.clone());
                },
                PencilIcon { class: "size-3.5" }
            }
            DropdownMenu {
                DropdownMenuTrigger { class: "cursor-pointer inline-flex size-7 items-center justify-center rounded-sm text-muted-foreground hover:text-foreground hover:bg-muted/80 transition-colors",
                    MoreHorizontalIcon { class: "size-3.5" }
                }
                DropdownMenuContent {
                    class: "w-44 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                    align: "end".to_string(),
                    DropdownMenuLabel { "Actions" }
                    DropdownMenuSeparator {}
                    DropdownMenuItem {
                        onclick: {
                            let mut state = state;
                            let t = t_view.clone();
                            move |_| state.open_view(t.clone())
                        },
                        EyeIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "View Details"
                    }
                    DropdownMenuItem {
                        onclick: {
                            let mut state = state;
                            let t = t_edit.clone();
                            move |_| state.open_edit(t.clone())
                        },
                        PencilIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "Edit Ticket"
                    }
                    DropdownMenuItem {
                        onclick: {
                            let id = t_copy.id.clone();
                            move |_| {
                                toast.info("Copied", format!("{id} copied to clipboard"));
                            }
                        },
                        CopyIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "Copy ID"
                    }
                    DropdownMenuSeparator {}
                    DropdownMenuSub {
                        DropdownMenuSubTrigger { "Change Status" }
                        DropdownMenuSubContent {
                            for & (code , label) in &[
                                ("Pending", "Pending"),
                                ("InProgress", "In Progress"),
                                ("Onhold", "On Hold"),
                                ("Completed", "Completed"),
                            ]
                            {
                                {
                                    let mut state = state;
                                    let id = t_view.id.clone();
                                    let label_str = label.to_string();
                                    rsx! {
                                        DropdownMenuItem {
                                            key: "act-status-{code}",
                                            onclick: move |_| {
                                                state.update_status(&id, code);
                                                toast.success("Status Updated", format!("{id} moved to {label_str}"));
                                            },
                                            span { "{label}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    DropdownMenuSub {
                        DropdownMenuSubTrigger { "Change Priority" }
                        DropdownMenuSubContent {
                            for & prio in &["Urgent", "High", "Medium", "Low"] {
                                {
                                    let mut state = state;
                                    let id = t_view.id.clone();
                                    rsx! {
                                        DropdownMenuItem {
                                            key: "act-prio-{prio}",
                                            onclick: move |_| {
                                                state.update_priority(&id, prio);
                                                toast.info("Priority Updated", format!("{id} priority set to {prio}"));
                                            },
                                            span { "{prio}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    DropdownMenuSeparator {}
                    DropdownMenuItem {
                        variant: "destructive".to_string(),
                        onclick: {
                            let mut state = state;
                            let id = t_del.id.clone();
                            move |_| {
                                state.delete_ticket(&id);
                                toast.error("Deleted", format!("{id} removed"));
                            }
                        },
                        TrashIcon { class: "size-3.5 text-destructive mr-1.5" }
                        "Delete"
                    }
                }
            }
        }
    }
}

fn render_id_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketIdCell { ticket: t.clone() }
    }
}

fn render_title_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketTitleCell { ticket: t.clone() }
    }
}

fn render_actions_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketActionsCell { ticket: t.clone() }
    }
}

fn render_category_cell(t: &TicketItem) -> Element {
    let cat = t.category.clone();
    let badge_cls = match cat.as_str() {
        "Network" => "text-sky-500 dark:text-sky-400 border-sky-500/35 bg-sky-500/15",
        "Hardware" => "text-amber-500 dark:text-amber-400 border-amber-500/35 bg-amber-500/15",
        "Software" => "text-purple-500 dark:text-purple-400 border-purple-500/35 bg-purple-500/15",
        "Security" => "text-rose-500 dark:text-rose-400 border-rose-500/35 bg-rose-500/15",
        "Infrastructure" => "text-cyan-500 dark:text-cyan-400 border-cyan-500/35 bg-cyan-500/15",
        _ => "text-emerald-500 dark:text-emerald-400 border-emerald-500/35 bg-emerald-500/15",
    };
    rsx! {
        span { class: "rounded-md px-2 py-0.5 text-[10px] font-semibold border {badge_cls}",
            "{cat}"
        }
    }
}

fn render_status_cell(t: &TicketItem) -> Element {
    let status = t.status.clone();
    let variant = status_badge_variant(&status);
    let class = status_badge_class(&status);
    let label = column_display_label(&status);
    rsx! {
        Badge { variant: "{variant}", class: "text-[10px] font-semibold {class}", "{label}" }
    }
}

fn render_sla_cell(t: &TicketItem) -> Element {
    let sla = t.sla_status.clone();
    let class = sla_badge_class(&sla);
    rsx! {
        if !sla.is_empty() {
            span { class: "rounded px-1.5 py-0.5 text-[10px] font-semibold border {class}",
                "{sla}"
            }
        } else {
            span { class: "text-muted-foreground/60 text-xs", "—" }
        }
    }
}

fn render_priority_cell(t: &TicketItem) -> Element {
    let priority = t.priority.clone();
    let variant = priority_badge(&priority);
    rsx! {
        Badge {
            variant: "{variant}",
            class: "text-[10px] px-1.5 py-0.2 font-semibold",
            "{priority}"
        }
    }
}

fn render_assignee_cell(t: &TicketItem) -> Element {
    let assignee = t.assignee.clone();
    rsx! {
        div { class: "flex items-center gap-1.5",
            div { class: "flex size-6 items-center justify-center rounded-full bg-muted text-[10px] font-bold text-foreground border border-border/80",
                "{assignee}"
            }
        }
    }
}

fn render_created_cell(t: &TicketItem) -> Element {
    let created = t.created_at.clone();
    rsx! {
        span { class: "text-[11px] text-muted-foreground font-mono", "{created}" }
    }
}

fn ticket_row_id(t: &TicketItem) -> String {
    t.id.clone()
}

fn ticket_search_text(t: &TicketItem) -> String {
    format!(
        "{} {} {} {} {} {} {} {} {}",
        t.id,
        t.subject,
        t.employee_code,
        t.email_address,
        t.category,
        t.priority,
        t.assigned_to,
        t.customer,
        t.sla_status
    )
}

fn ticket_filter_value(t: &TicketItem) -> String {
    t.status.clone()
}

fn ticket_priority_filter_value(t: &TicketItem) -> String {
    t.priority.clone()
}

fn ticket_columns() -> Vec<DataTableColumn<TicketItem>> {
    vec![
        DataTableColumn::new("id", "Ticket ID", render_id_cell, |t: &TicketItem| {
            SortValue::Text(t.id.clone())
        }),
        DataTableColumn::new(
            "title",
            "Subject & Requester",
            render_title_cell,
            |t: &TicketItem| SortValue::Text(t.title.clone()),
        ),
        DataTableColumn::new(
            "category",
            "Category",
            render_category_cell,
            |t: &TicketItem| SortValue::Text(t.category.clone()),
        ),
        DataTableColumn::new("status", "Status", render_status_cell, |t: &TicketItem| {
            SortValue::Text(t.status.clone())
        }),
        DataTableColumn::new("sla", "SLA Status", render_sla_cell, |t: &TicketItem| {
            SortValue::Text(t.sla_status.clone())
        }),
        DataTableColumn::new(
            "priority",
            "Priority",
            render_priority_cell,
            |t: &TicketItem| SortValue::Text(t.priority.clone()),
        ),
        DataTableColumn::new(
            "assignee",
            "Assignee",
            render_assignee_cell,
            |t: &TicketItem| SortValue::Text(t.assignee.clone()),
        ),
        DataTableColumn::new(
            "created",
            "Reported",
            render_created_cell,
            |t: &TicketItem| SortValue::Text(t.created_at.clone()),
        ),
        DataTableColumn::actions("actions", "Actions", render_actions_cell),
    ]
}

#[component]
pub fn Tickets() -> Element {
    let mut tickets = use_signal(initial_tickets);
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();
    use_effect(move || {
        if let Some(sync) = db_sync {
            let _ = sync();
            let new_list = initial_tickets();
            if *tickets.peek() != new_list {
                tickets.set(new_list);
            }
        }
    });

    let mut search_query = use_signal(String::new);
    let mut selected_priority = use_signal(|| "All".to_string());

    // Drag-and-drop reactive state
    let mut dragged_id = use_signal(|| Option::<String>::None);
    let mut drag_over_col = use_signal(|| Option::<String>::None);
    let mut drag_over_card = use_signal(|| Option::<String>::None);

    // View & Edit Ticket Modal signals
    let viewing_ticket = use_signal(|| Option::<TicketItem>::None);
    let mut editing_ticket = use_signal(|| Option::<TicketItem>::None);
    let mut is_view_open = use_signal(|| false);
    let mut is_edit_open = use_signal(|| false);

    let mut edit_emp_name = use_signal(String::new);
    let mut edit_employee_code = use_signal(String::new);
    let mut edit_email_address = use_signal(String::new);
    let mut edit_category = use_signal(|| "General".to_string());
    let mut edit_subject = use_signal(String::new);
    let mut edit_problem_description = use_signal(String::new);
    let mut edit_attachment_name = use_signal(String::new);
    let mut edit_attachment_url = use_signal(String::new);
    let mut edit_priority = use_signal(|| "Medium".to_string());
    let mut edit_assigned_to = use_signal(|| "Shreyansh".to_string());
    let mut edit_customer = use_signal(|| "Internal".to_string());
    let mut edit_time_spent = use_signal(|| 0.0f64);
    let mut edit_activities = use_signal(String::new);
    let edit_my_activity = use_signal(String::new);
    let mut edit_deadline = use_signal(String::new);
    let mut edit_sla_status = use_signal(|| "Within SLA".to_string());
    let mut edit_sla_deadline = use_signal(String::new);
    let mut edit_rating_avg_text = use_signal(String::new);
    let mut edit_kanban_state = use_signal(|| "Pending".to_string());
    let mut edit_stage = use_signal(|| "Pending".to_string());
    let mut edit_status = use_signal(|| "Pending".to_string());

    // Fetch employee directory in real-time on mount
    use_effect(move || {
        spawn(async move {
            let _ = crate::sync::pull_employees_now().await;
        });
    });

    let directory_employees = crate::db::get_all_employees();

    let mut is_create_open = use_signal(|| false);
    let mut create_emp_name = use_signal(String::new);
    let mut create_employee_code = use_signal(String::new);
    let mut create_email_address = use_signal(String::new);
    let mut create_category = use_signal(|| "Select category".to_string());
    let mut create_subject = use_signal(String::new);
    let mut create_problem_description = use_signal(String::new);
    let create_attachment_name = use_signal(String::new);
    let create_attachment_url = use_signal(String::new);
    let mut create_priority = use_signal(|| "Low".to_string());
    let create_assigned_to = use_signal(|| "IT Support".to_string());
    let create_customer = use_signal(|| "Internal".to_string());
    let create_time_spent = use_signal(|| 0.0f64);
    let create_activities = use_signal(|| "Ticket created".to_string());
    let create_my_activity = use_signal(|| "Initial review".to_string());
    let create_deadline = use_signal(String::new);
    let create_sla_status = use_signal(|| "Within SLA".to_string());
    let create_sla_deadline = use_signal(String::new);
    let create_rating_avg_text = use_signal(|| "Unrated".to_string());
    let create_kanban_state = use_signal(|| "Pending".to_string());
    let create_stage = use_signal(|| "Triage".to_string());
    let create_status = use_signal(|| "Open".to_string());

    // Table row floating context menu signals
    let mut table_ctx_open = use_signal(|| false);
    let mut table_ctx_x = use_signal(|| 0.0f64);
    let mut table_ctx_y = use_signal(|| 0.0f64);
    let mut table_ctx_ticket = use_signal(|| Option::<TicketItem>::None);

    let mut state = TicketContextState {
        tickets,
        viewing_ticket,
        editing_ticket,
        is_view_open,
        is_edit_open,
        edit_emp_name,
        edit_employee_code,
        edit_email_address,
        edit_category,
        edit_subject,
        edit_problem_description,
        edit_attachment_name,
        edit_attachment_url,
        edit_priority,
        edit_assigned_to,
        edit_customer,
        edit_time_spent,
        edit_activities,
        edit_my_activity,
        edit_deadline,
        edit_sla_status,
        edit_sla_deadline,
        edit_rating_avg_text,
        edit_kanban_state,
        edit_stage,
        edit_status,

        is_create_open,
        create_emp_name,
        create_employee_code,
        create_email_address,
        create_category,
        create_subject,
        create_problem_description,
        create_attachment_name,
        create_attachment_url,
        create_priority,
        create_assigned_to,
        create_customer,
        create_time_spent,
        create_activities,
        create_my_activity,
        create_deadline,
        create_sla_status,
        create_sla_deadline,
        create_rating_avg_text,
        create_kanban_state,
        create_stage,
        create_status,
    };
    use_context_provider(|| state);

    let toast = use_context::<ToastState>();

    let q = search_query.read().to_lowercase();
    let filter_prio = selected_priority.read().clone();

    // Move ticket to end of a column
    let mut move_to_column_end = move |src_id: String, target_col: String| {
        let mut list = tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == src_id) {
            let mut item = list.remove(pos);
            let num = item.ticket_id;
            let old_status = item.status.clone();
            item.status = target_col.clone();
            item.kanban_state = target_col.clone();
            item.stage = target_col.clone();
            list.push(item);

            // Persist status change to SQLite management.db
            let _ = crate::db::update_ticket_status(num, &target_col);
            crate::sync::enqueue_update(
                num,
                serde_json::json!({
                    "kanban_state": target_col.clone(),
                    "stage": target_col.clone(),
                    "status": target_col.clone(),
                }),
            );

            if old_status != target_col {
                let label = column_display_label(&target_col);
                toast.success("Status Updated", format!("{src_id} moved to {label}"));
            }
        }
    };

    // Move ticket before a specific target card in a column
    let mut move_card_before = move |src_id: String, target_card_id: String, target_col: String| {
        if src_id == target_card_id {
            return;
        }
        let mut list = tickets.write();
        if let Some(src_pos) = list.iter().position(|t| t.id == src_id) {
            let mut item = list.remove(src_pos);
            let num = item.ticket_id;
            let old_status = item.status.clone();
            item.status = target_col.clone();
            item.kanban_state = target_col.clone();
            item.stage = target_col.clone();
            if let Some(target_pos) = list.iter().position(|t| t.id == target_card_id) {
                list.insert(target_pos, item);
            } else {
                list.push(item);
            }

            // Persist status change to SQLite management.db
            let _ = crate::db::update_ticket_status(num, &target_col);
            crate::sync::enqueue_update(
                num,
                serde_json::json!({
                    "kanban_state": target_col.clone(),
                    "stage": target_col.clone(),
                    "status": target_col.clone(),
                }),
            );

            if old_status != target_col {
                let label = column_display_label(&target_col);
                toast.success("Moved & Reordered", format!("{src_id} moved to {label}"));
            } else {
                toast.info("Ticket Reordered", format!("{src_id} repositioned"));
            }
        }
    };

    // Reset drag signals helper
    let mut clear_drag_state = move || {
        dragged_id.set(None);
        drag_over_col.set(None);
        drag_over_card.set(None);
    };

    // Save new ticket from Create Ticket dialog (essential inputs matching Create Support Ticket form)
    let mut save_create = move || {
        let emp_code = create_employee_code.read().trim().to_string();
        if emp_code.is_empty() {
            toast.error("Validation Error", "Please enter Employee Code");
            return;
        }

        let email_val = create_email_address.read().trim().to_string();
        if email_val.is_empty() {
            toast.error("Validation Error", "Please enter Email Address");
            return;
        }

        let cat_val = create_category.read().trim().to_string();
        if cat_val.is_empty() || cat_val == "Select category" {
            toast.error("Validation Error", "Please select a Category");
            return;
        }

        let subj_val = create_subject.read().trim().to_string();
        if subj_val.is_empty() {
            toast.error("Validation Error", "Please enter Subject");
            return;
        }

        let desc_val = create_problem_description.read().trim().to_string();
        if desc_val.is_empty() {
            toast.error("Validation Error", "Please describe the problem");
            return;
        }

        let mut name_val = create_emp_name.read().trim().to_string();
        if name_val.is_empty() {
            if let Some(u) = crate::db::find_employee_by_code(&emp_code) {
                name_val = u.emp_name;
            }
        }
        let emp_combined = if name_val.is_empty() {
            emp_code.clone()
        } else {
            format!("{} ({})", emp_code, name_val)
        };

        let att_name_val = String::new();
        let att_url_val = String::new();
        let prio_val = create_priority.read().clone();
        let assigned_val = "IT Support".to_string();

        // Automated behind-the-scenes defaults:
        let cust_val = "Internal".to_string();
        let time_val = 0.0f64;
        let act_val = "Ticket created".to_string();
        let my_act_val = "Initial triage & review".to_string();

        // SLA deadline automated based on priority tier:
        let sla_dead_val = match prio_val.as_str() {
            "Critical" => "Within 4 Hours (P1)".to_string(),
            "High" => "Within 24 Hours (P2)".to_string(),
            "Low" => "Within 72 Hours (P4)".to_string(),
            _ => "Within 48 Hours (P3)".to_string(),
        };
        let deadline_val = match prio_val.as_str() {
            "Critical" => "Today · Urgent".to_string(),
            "High" => "Tomorrow · 5:00 PM".to_string(),
            _ => "Within 3 Business Days".to_string(),
        };
        let sla_stat_val = "Within SLA".to_string();
        let rating_val = "Unrated".to_string();

        let kanban_val = create_kanban_state.read().clone();
        let stage_val = "Triage".to_string();
        let status_val = "Open".to_string();

        let input = crate::db::NewTicketInput {
            employee_code: emp_combined.clone(),
            email_address: email_val.clone(),
            category: cat_val.clone(),
            subject: subj_val.clone(),
            problem_description: desc_val.clone(),
            attachment_name: att_name_val.clone(),
            attachment_url: att_url_val.clone(),
            priority: prio_val.clone(),
            assigned_to: assigned_val.clone(),
            customer: cust_val.clone(),
            time_spent: time_val,
            activities: act_val.clone(),
            my_activity: my_act_val.clone(),
            deadline: deadline_val.clone(),
            sla_status: sla_stat_val.clone(),
            sla_deadline: sla_dead_val.clone(),
            rating_avg_text: rating_val.clone(),
            kanban_state: kanban_val.clone(),
            stage: stage_val.clone(),
            status: status_val.clone(),
        };

        match crate::db::create_ticket_full(&input) {
            Ok(new_id) => {
                let real_id_str = format!("IT-{new_id}");
                let now_str = "Just now".to_string();
                let new_ticket = TicketItem::new(
                    new_id,
                    &emp_combined,
                    &email_val,
                    &cat_val,
                    &subj_val,
                    &desc_val,
                    &att_name_val,
                    &att_url_val,
                    &prio_val,
                    &assigned_val,
                    &cust_val,
                    time_val,
                    &act_val,
                    &my_act_val,
                    &deadline_val,
                    &sla_stat_val,
                    &sla_dead_val,
                    &rating_val,
                    &kanban_val,
                    &stage_val,
                    &status_val,
                    &now_str,
                    &now_str,
                );
                tickets.write().push(new_ticket);

                // Queue the create for background sync (immediate wake).
                crate::sync::enqueue_create(new_id);

                if let Some(mut sync) = db_sync {
                    sync.with_mut(|r| r.0 += 1);
                }

                toast.success(
                    "Ticket Created",
                    format!("{real_id_str} created successfully"),
                );
                is_create_open.set(false);
            }
            Err(e) => {
                toast.error(
                    "Database Error",
                    format!("Failed to create ticket: {:?}", e),
                );
            }
        }
    };

    // Save changes from Edit / Modify Ticket dialog
    let mut save_edit = move || {
        if let Some(mut current) = editing_ticket.read().clone() {
            let subj_val = edit_subject.read().trim().to_string();
            if subj_val.is_empty() {
                toast.error("Validation Error", "Ticket subject cannot be empty");
                return;
            }

            let code_val = edit_employee_code.read().trim().to_string();
            let name_val = edit_emp_name.read().trim().to_string();
            let emp_combined = if name_val.is_empty() {
                code_val.clone()
            } else if code_val.is_empty() {
                name_val.clone()
            } else {
                format!("{} ({})", code_val, name_val)
            };

            let email_val = edit_email_address.read().trim().to_string();
            let cat_val = edit_category.read().clone();
            let desc_val = edit_problem_description.read().clone();
            let att_name_val = edit_attachment_name.read().clone();
            let att_url_val = edit_attachment_url.read().clone();
            let prio_val = edit_priority.read().clone();
            let assigned_val = edit_assigned_to.read().clone();
            let cust_val = edit_customer.read().clone();
            let time_val = *edit_time_spent.read();
            let act_val = edit_activities.read().clone();
            let my_act_val = edit_my_activity.read().clone();
            let deadline_val = edit_deadline.read().clone();
            let sla_stat_val = edit_sla_status.read().clone();
            let sla_dead_val = edit_sla_deadline.read().clone();
            let rating_val = edit_rating_avg_text.read().clone();

            // The stage represents the Kanban board stages
            let kanban_val = edit_kanban_state.read().clone();
            let stage_val = kanban_val.clone();
            let status_val = kanban_val.clone();

            // Only send fields the user actually changed — the API takes
            // partial updates ("send ticket_id and only the fields to
            // update") and rejects unknown fields. `activities` maps to the
            // server's `activities_text`; `time_spent` stays numeric.
            let mut api_fields = serde_json::Map::new();
            {
                let mut put_str = |key: &str, old: &str, new: &str| {
                    if old != new {
                        api_fields
                            .insert(key.to_string(), serde_json::Value::String(new.to_string()));
                    }
                };
                put_str("employee_code", &current.employee_code, &emp_combined);
                put_str("email_address", &current.email_address, &email_val);
                put_str("category", &current.category, &cat_val);
                put_str("subject", &current.subject, &subj_val);
                put_str(
                    "problem_description",
                    &current.problem_description,
                    &desc_val,
                );
                put_str("attachment_name", &current.attachment_name, &att_name_val);
                put_str("attachment_url", &current.attachment_url, &att_url_val);
                put_str("priority", &current.priority, &prio_val);
                put_str("assigned_to", &current.assigned_to, &assigned_val);
                put_str("customer", &current.customer, &cust_val);
                put_str("activities_text", &current.activities, &act_val);
                put_str("my_activity", &current.my_activity, &my_act_val);
                put_str("deadline", &current.deadline, &deadline_val);
                put_str("sla_status", &current.sla_status, &sla_stat_val);
                put_str("sla_deadline", &current.sla_deadline, &sla_dead_val);
                put_str("rating_avg_text", &current.rating_avg_text, &rating_val);
                if current.time_spent != time_val {
                    if let Some(n) = serde_json::Number::from_f64(time_val) {
                        api_fields.insert("time_spent".to_string(), serde_json::Value::Number(n));
                    }
                }
                if current.kanban_state != kanban_val {
                    api_fields.insert(
                        "kanban_state".to_string(),
                        serde_json::Value::String(kanban_val.clone()),
                    );
                    api_fields.insert(
                        "stage".to_string(),
                        serde_json::Value::String(stage_val.clone()),
                    );
                    api_fields.insert(
                        "status".to_string(),
                        serde_json::Value::String(status_val.clone()),
                    );
                }
            }

            current.subject = subj_val.clone();
            current.title = subj_val.clone();
            current.employee_code = emp_combined.clone();
            current.employee = emp_combined.clone();
            current.email_address = email_val.clone();
            current.category = cat_val.clone();
            current.problem_description = desc_val.clone();
            current.attachment_name = att_name_val.clone();
            current.attachment_url = att_url_val.clone();
            current.priority = prio_val.clone();
            current.assigned_to = assigned_val.clone();
            current.assignee = assigned_val.clone();
            current.customer = cust_val.clone();
            current.time_spent = time_val;
            current.activities = act_val.clone();
            current.my_activity = my_act_val.clone();
            current.deadline = deadline_val.clone();
            current.sla_status = sla_stat_val.clone();
            current.sla_deadline = sla_dead_val.clone();
            current.rating_avg_text = rating_val.clone();
            current.kanban_state = kanban_val.clone();
            current.status = kanban_val.clone();
            current.stage = stage_val.clone();
            current.last_updated_on = "Just now".to_string();

            let mut list = tickets.write();
            if let Some(pos) = list.iter().position(|t| t.id == current.id) {
                list[pos] = current.clone();
            }

            let num = current.ticket_id;
            let _ = crate::db::update_ticket_full(
                num,
                &emp_combined,
                &email_val,
                &cat_val,
                &subj_val,
                &desc_val,
                &att_name_val,
                &att_url_val,
                &prio_val,
                &assigned_val,
                &cust_val,
                time_val,
                &act_val,
                &my_act_val,
                &deadline_val,
                &sla_stat_val,
                &sla_dead_val,
                &rating_val,
                &kanban_val,
                &stage_val,
                &status_val,
            );

            // Queue only the changed fields for background sync.
            if !api_fields.is_empty() {
                crate::sync::enqueue_update(num, serde_json::Value::Object(api_fields));
            }

            toast.success(
                "Ticket Modified",
                format!("{} modified successfully", current.id),
            );
            state.is_edit_open.set(false);
            state.editing_ticket.set(None);
        }
    };

    // Quick add ticket to column
    let mut quick_add_ticket = move |col: String| {
        let count = tickets.read().len() + 1;
        let fallback_id = format!("IT-{}", 200 + count);
        let col_disp = column_display_label(&col);
        let title = format!("New request ({col_disp})");
        // Sheet directory only — the configured employee, or nothing. No
        // fabricated employee codes/emails are written to the sheet.
        let employee = crate::db::configured_employee_code();
        let email = crate::db::employee_email(&employee);
        let customer = crate::db::employee_name(&employee);
        let category = "General".to_string();
        let priority = "Medium".to_string();
        let assignee = "GR".to_string();
        let created_at = "Just now".to_string();

        if employee.is_empty() {
            toast.error(
                "No employee configured",
                "Set TMS_EMPLOYEE_CODE (env or .env) to an existing sheet Employee Code before quick-adding tickets.",
            );
            return;
        }

        let db_id = crate::db::create_ticket(
            &title,
            &category,
            &priority,
            &employee,
            &assignee,
            &col,
            "Quick ticket created via column quick-add action",
        );
        let real_id = if let Ok(id) = db_id {
            format!("IT-{id}")
        } else {
            fallback_id
        };
        // Queue the create for background sync (only when the row exists).
        if let Ok(id) = db_id {
            crate::sync::enqueue_create(id);
        }
        let numeric_id = db_id.unwrap_or(200 + count as i64);

        tickets.write().push(TicketItem::new(
            numeric_id,
            &employee,
            &email,
            &category,
            &title,
            "Quick ticket created via column quick-add action",
            "",
            "",
            &priority,
            &assignee,
            &customer,
            0.0,
            "Ticket created",
            "Initial assignment to team lead",
            "",
            "Within SLA",
            "",
            "",
            &col,
            "Triage",
            "Open",
            &created_at,
            &created_at,
        ));
        toast.success("Ticket Created", format!("{real_id} created in {col_disp}"));
    };

    rsx! {
        div { class: "flex flex-col flex-1 w-full h-full p-4 md:p-6 min-h-0 overflow-hidden",

            // Page header
            div { class: "flex items-center justify-between border-b border-border/50 pb-3 mb-3 shrink-0",
                div { class: "flex items-center gap-3",
                    h1 { class: "text-xl font-bold tracking-tight text-foreground sm:text-2xl",
                        "IT Tickets"
                    }
                    Badge { variant: "secondary", class: "font-semibold",
                        "{tickets.read().len()} tickets"
                    }
                }
                Button {
                    variant: "default",
                    size: "sm",
                    class: "gap-1.5 h-8 px-3 text-xs font-semibold shadow-sm cursor-pointer",
                    onclick: move |_| state.open_create(None),
                    PlusIcon { class: "size-4" }
                    "New Ticket"
                }
            }

            Tabs {
                value: "board".to_string(),
                class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden",
                TabsList {
                    TabsTrigger { value: "board".to_string(), "Kanban Board" }
                    TabsTrigger { value: "table".to_string(), "IT Data Table" }
                }

                // --------------------------------------------------------
                // Kanban Board View (Onhold, pending, InProgress, Completed)
                // --------------------------------------------------------
                TabsContent {
                    value: "board".to_string(),
                    class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden gap-3 pt-2",

                    // Search & Filter toolbar for Kanban
                    div { class: "shrink-0 flex flex-wrap items-center justify-between gap-3",
                        div { class: "relative flex flex-1 min-w-[200px] max-w-sm items-center",
                            div { class: "pointer-events-none absolute left-3 flex items-center text-muted-foreground",
                                SearchIcon { class: "size-4" }
                            }
                            Input {
                                class: "pl-9 text-xs h-9 w-full",
                                placeholder: "Filter tickets by ID, title, requester, or category...".to_string(),
                                value: search_query.read().clone(),
                                oninput: move |e: FormEvent| search_query.set(e.value()),
                            }
                        }

                        div { class: "flex items-center gap-1.5",
                            span { class: "text-xs font-medium text-muted-foreground mr-1",
                                "Priority:"
                            }
                            for prio in ["All", "High", "Medium", "Low"] {
                                {
                                    let is_sel = filter_prio == prio;
                                    let btn_variant = if is_sel { "default" } else { "outline" };
                                    rsx! {
                                        Button {
                                            variant: "{btn_variant}",
                                            size: "sm",
                                            key: "prio-{prio}",
                                            class: "h-7 px-2.5 text-xs",
                                            onclick: move |_| selected_priority.set(prio.to_string()),
                                            "{prio}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // 4 Kanban Columns side-by-side in full width
                    div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3.5 w-full flex-1 min-h-0 overflow-hidden",
                        for & column in COLUMN_NAMES {
                            {
                                let col_display = column_display_label(column);
                                let col_sub = column_subtitle(column);
                                let col_dot = match column {
                                    "InProgress" => "bg-sky-500 animate-pulse",
                                    "Pending" => "bg-warning",
                                    "Onhold" => "bg-purple-500",
                                    _ => "bg-success",
                                };

                                let all_tickets = tickets.read().clone();
                                let column_tickets: Vec<TicketItem> = all_tickets
                                    .into_iter()
                                    .filter(|t| {
                                        let st = if !t.kanban_state.is_empty() {
                                            t.kanban_state.as_str()
                                        } else if !t.stage.is_empty() {
                                            t.stage.as_str()
                                        } else {
                                            t.status.as_str()
                                        };
                                        match (st, column) {
                                            ("InProgress", "InProgress") | ("In Progress", "InProgress") => true,
                                            ("Pending", "Pending") | ("Open", "Pending") => true,
                                            ("Onhold", "Onhold") | ("On Hold", "Onhold") => true,
                                            ("Completed", "Completed")

                                            // Column header

                                            // Column cards container (acts as full drop target)

                                            // Insertion indicator before card

                                            // Inner card content
                                            // Header: ID, Status badge, Priority badge, and direct View & Modify buttons!
                                            // Direct View & Modify buttons on the Kanban card

                                            // Requester identity: Name, Code & Email

                                            // Footer with quick state transition buttons & assignee

                                            // Drop zone at bottom of column
                                            | ("Resolved", "Completed")
                                            | ("Closed", "Completed") => true,
                                            (s, c) => s == c || s.eq_ignore_ascii_case(c),
                                        }
                                    })
                                    .filter(|t| {
                                        let match_q = q.is_empty() || t.id.to_lowercase().contains(&q)
                                            || t.title.to_lowercase().contains(&q)
                                            || t.employee.to_lowercase().contains(&q)
                                            || t.emp_name().to_lowercase().contains(&q)
                                            || t.emp_code_only().to_lowercase().contains(&q)
                                            || t.email_address.to_lowercase().contains(&q)
                                            || t.category.to_lowercase().contains(&q)
                                            || t.assignee.to_lowercase().contains(&q);
                                        let match_prio = filter_prio == "All" || t.priority == filter_prio;
                                        match_q && match_prio
                                    })
                                    .collect();
                                let is_col_over = drag_over_col.read().as_deref() == Some(column);
                                let col_highlight = if is_col_over {
                                    "ring-1 ring-primary border-primary bg-primary/5"
                                } else {
                                    ""
                                };
                                rsx! {
                                    ColumnContextMenu {
                                        column: column.to_string(),
                                        on_quick_add: move |col| quick_add_ticket(col),
                                        on_filter_column: move |col| search_query.set(col),
                                        on_clear_filters: move |_| {
                                            search_query.set(String::new());
                                            selected_priority.set("All".to_string());
                                        },

                                        div {
                                            key: "board-col-{column}",
                                            class: "flex flex-col h-full min-h-0 rounded-md border border-border/70 bg-card/75 p-3 transition-colors duration-150 {col_highlight}",
                                            ondragenter: move |e: DragEvent| {
                                                e.prevent_default();
                                                if drag_over_col.read().as_deref() != Some(column) {
                                                    drag_over_col.set(Some(column.to_string()));
                                                }
                                            },
                                            ondragover: move |e: DragEvent| {
                                                e.prevent_default();
                                                if drag_over_col.read().as_deref() != Some(column) {
                                                    drag_over_col.set(Some(column.to_string()));
                                                }
                                            },
                                            ondrop: move |e: DragEvent| {
                                                e.prevent_default();
                                                let src = e
                                                    .data()
                                                    .data_transfer()
                                                    .get_as_text()
                                                    .filter(|s| !s.is_empty())
                                                    .or_else(|| dragged_id.read().clone());
                                                if let Some(s) = src {
                                                    move_to_column_end(s, column.to_string());
                                                }
                                                clear_drag_state();
                                            },

                                            div { class: "flex items-center justify-between px-1 border-b border-border/40 pb-2.5",
                                                div { class: "flex flex-col gap-0.5",
                                                    div { class: "flex items-center gap-2",
                                                        span { class: "size-2 rounded-full {col_dot}" }
                                                        h2 { class: "text-xs font-bold uppercase tracking-wider text-foreground",
                                                            "{col_display}"
                                                        }
                                                    }
                                                    span { class: "text-[11px] text-muted-foreground", "{col_sub}" }
                                                }
                                                Badge {
                                                    variant: "outline",
                                                    class: "text-[10px] font-mono tabular-nums text-muted-foreground",
                                                    "{column_tickets.len()}"
                                                }
                                            }

                                            div {
                                                class: "flex flex-col gap-2 min-h-0 flex-1 overflow-y-auto pr-1",
                                                ondragenter: move |e: DragEvent| {
                                                    e.prevent_default();
                                                    if drag_over_col.read().as_deref() != Some(column) {
                                                        drag_over_col.set(Some(column.to_string()));
                                                    }
                                                },
                                                ondragover: move |e: DragEvent| {
                                                    e.prevent_default();
                                                    if drag_over_col.read().as_deref() != Some(column) {
                                                        drag_over_col.set(Some(column.to_string()));
                                                    }
                                                },
                                                ondrop: move |e: DragEvent| {
                                                    e.prevent_default();
                                                    let src = e
                                                        .data()
                                                        .data_transfer()
                                                        .get_as_text()
                                                        .filter(|s| !s.is_empty())
                                                        .or_else(|| dragged_id.read().clone());
                                                    if let Some(s) = src {
                                                        move_to_column_end(s, column.to_string());
                                                    }
                                                    clear_drag_state();
                                                },

                                                for ticket in column_tickets.iter() {
                                                    {
                                                        let id = ticket.id.clone();
                                                        let title = ticket.title.clone();
                                                        let priority = ticket.priority.clone();
                                                        let category = ticket.category.clone();
                                                        let assignee = ticket.assignee.clone();
                                                        let ticket_status = ticket.status.clone();
                                                        let sla_status = ticket.sla_status.clone();
                                                        let deadline = ticket.deadline.clone();

                                                        let is_dragged = dragged_id.read().as_deref() == Some(&id);
                                                        let is_card_over = drag_over_card.read().as_deref() == Some(&id) && !is_dragged;

                                                        let card_cls = if is_dragged {
                                                            "opacity-40 scale-[0.99] border-dashed border-primary ring-1 ring-primary/40 bg-muted/40 cursor-grabbing rounded-md border p-3 flex flex-col gap-2"
                                                        } else if ticket_status == "Completed" {
                                                            "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-success/30 bg-card p-3 transition-colors duration-150 hover:border-success/60 hover:bg-success/5 shadow-xs"
                                                        } else if ticket_status == "InProgress" {
                                                            "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-primary/40 bg-card p-3 transition-colors duration-150 hover:border-primary/70 hover:bg-primary/5 shadow-xs"
                                                        } else {
                                                            "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-border/70 bg-card p-3 transition-colors duration-150 hover:border-primary/50 hover:bg-accent/20 shadow-xs"
                                                        };

                                                        let id_for_start = id.clone();
                                                        let id_for_enter = id.clone();
                                                        let id_for_over = id.clone();
                                                        let id_for_drop = id.clone();

                                                        rsx! {
                                                            div {
                                                                key: "board-card-wrap-{id}",
                                                                class: "flex flex-col gap-1.5",
                                                                ondragenter: move |e: DragEvent| {
                                                                    e.prevent_default();
                                                                    if drag_over_card.read().as_deref() != Some(&id_for_enter) {
                                                                        drag_over_card.set(Some(id_for_enter.clone()));
                                                                    }
                                                                    if drag_over_col.read().as_deref() != Some(column) {
                                                                        drag_over_col.set(Some(column.to_string()));
                                                                    }
                                                                },
                                                                ondragover: move |e: DragEvent| {
                                                                    e.prevent_default();
                                                                    if drag_over_card.read().as_deref() != Some(&id_for_over) {
                                                                        drag_over_card.set(Some(id_for_over.clone()));
                                                                    }
                                                                    if drag_over_col.read().as_deref() != Some(column) {
                                                                        drag_over_col.set(Some(column.to_string()));
                                                                    }
                                                                },
                                                                ondrop: move |e: DragEvent| {
                                                                    e.prevent_default();
                                                                    e.stop_propagation();
                                                                    let src = e




                                                                        .data()
                                                                        .data_transfer()
                                                                        .get_as_text()
                                                                        .filter(|s| !s.is_empty())
                                                                        .or_else(|| dragged_id.read().clone());
                                                                    if let Some(s) = src {
                                                                        move_card_before(s, id_for_drop.clone(), column.to_string());
                                                                    }
                                                                    clear_drag_state();
                                                                },

                                                                if is_card_over {
                                                                    div { class: "h-1 w-full rounded-full bg-primary animate-pulse my-0.5" }
                                                                }

                                                                TicketContextMenu {
                                                                    ticket: ticket.clone(),
                                                                    on_view: move |t| state.open_view(t),
                                                                    on_edit: move |t| state.open_edit(t),

                                                                    div {
                                                                        draggable: "true",
                                                                        "data-drag-id": "{id}",
                                                                        class: "select-none {card_cls}",
                                                                        ondragstart: move |_| {
                                                                            dragged_id.set(Some(id_for_start.clone()));
                                                                        },
                                                                        ondragend: move |_| {
                                                                            clear_drag_state();
                                                                        },
                                                                        ondoubleclick: {
                                                                            let t_dbl = ticket.clone();
                                                                            move |_| state.open_view(t_dbl.clone())
                                                                        },

                                                                        div { class: "pointer-events-none flex flex-col gap-2 w-full",
                                                                            div { class: "flex items-center justify-between gap-1.5",
                                                                                div { class: "flex items-center gap-1.5 min-w-0",
                                                                                    GripVerticalIcon { class: "size-3.5 shrink-0 text-muted-foreground/40 group-hover:text-primary transition-colors cursor-grab" }
                                                                                    span { class: "font-mono text-[11px] font-bold text-muted-foreground group-hover:text-primary transition-colors",
                                                                                        "{id}"
                                                                                    }
                                                                                }
                                                                                div { class: "flex items-center gap-1 shrink-0",
                                                                                    Badge {
                                                                                        variant: "{status_badge_variant(&ticket_status)}",
                                                                                        class: "text-[10px] font-semibold {status_badge_class(&ticket_status)}",
                                                                                        "{column_display_label(&ticket_status)}"
                                                                                    }
                                                                                    Badge {
                                                                                        variant: "{priority_badge(&priority)}",
                                                                                        class: "text-[10px] px-1.5 py-0.2",
                                                                                        "{priority}"
                                                                                    }
                                                                                    div { class: "flex items-center gap-0.5 pointer-events-auto ml-1 border-l border-border/50 pl-1",
                                                                                        button {
                                                                                            r#type: "button",
                                                                                            title: "View Ticket Details",
                                                                                            class: "cursor-pointer rounded p-1 text-muted-foreground hover:text-primary hover:bg-muted/80 transition-colors",
                                                                                            onclick: {
                                                                                                let t_v = ticket.clone();
                                                                                                move |e: MouseEvent| {
                                                                                                    e.stop_propagation();
                                                                                                    state.open_view(t_v.clone());
                                                                                                }
                                                                                            },
                                                                                            EyeIcon { class: "size-3.5" }
                                                                                        }
                                                                                        button {
                                                                                            r#type: "button",
                                                                                            title: "Modify Ticket",
                                                                                            class: "cursor-pointer rounded p-1 text-muted-foreground hover:text-primary hover:bg-muted/80 transition-colors",
                                                                                            onclick: {
                                                                                                let t_e = ticket.clone();
                                                                                                move |e: MouseEvent| {
                                                                                                    e.stop_propagation();
                                                                                                    state.open_edit(t_e.clone());
                                                                                                }
                                                                                            },
                                                                                            PencilIcon { class: "size-3.5" }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }

                                                                            p { class: "text-sm font-semibold leading-snug text-foreground group-hover:text-primary/95 transition-colors line-clamp-2",
                                                                                "{title}"
                                                                            }

                                                                            div { class: "flex flex-col gap-0.5 text-[11px] pt-0.5",
                                                                                div { class: "flex items-center justify-between gap-1.5 text-muted-foreground",
                                                                                    span { class: "font-medium text-foreground truncate", "{ticket.emp_name()}" }
                                                                                    span { class: "font-mono text-[10px] text-muted-foreground shrink-0",
                                                                                        "{ticket.emp_code_only()}"
                                                                                    }
                                                                                }
                                                                                div { class: "flex items-center justify-between gap-1 text-[10px] text-muted-foreground/80",
                                                                                    span { class: "truncate", "{ticket.email_address}" }
                                                                                    if !sla_status.is_empty() {
                                                                                        span { class: "rounded px-1.5 py-0.2 text-[9px] font-semibold {sla_badge_class(&sla_status)} shrink-0",
                                                                                            "{sla_status}"
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }

                                                                            if !deadline.is_empty() {
                                                                                div { class: "flex items-center justify-between text-[10px] text-muted-foreground/80 pt-1 border-t border-border/30",
                                                                                    span { class: "font-mono text-warning font-medium", "Due: {deadline}" }
                                                                                }
                                                                            }
                                                                        }

                                                                        div { class: "mt-1 flex items-center justify-between pt-2 border-t border-border/40 text-xs",
                                                                            div { class: "flex items-center gap-1 pointer-events-auto",
                                                                                span { class: "rounded-md bg-muted/80 px-2 py-0.5 text-[10px] font-medium text-muted-foreground mr-1",
                                                                                    "{category}"
                                                                                }
                                                                                for & target_col in COLUMN_NAMES {
                                                                                    if target_col != column {
                                                                                        {
                                                                                            let t_id = id.clone();
                                                                                            let t_target = target_col.to_string();
                                                                                            let short_label = match target_col {
                                                                                                "Pending" => "Pend",
                                                                                                "InProgress" => "InProg",
                                                                                                "Onhold" => "Hold",
                                                                                                _ => "Done",
                                                                                            };
                                                                                            let move_title = format!("Change stage to {}", column_display_label(target_col));
                                                                                            rsx! {
                                                                                                button {
                                                                                                    r#type: "button",
                                                                                                    key: "mv-{id}-{target_col}",
                                                                                                    title: "{move_title}",
                                                                                                    class: "cursor-pointer rounded px-1.5 py-0.5 text-[9px] font-semibold text-muted-foreground border border-border/60 hover:text-foreground hover:border-primary/50 hover:bg-primary/10 transition-colors",
                                                                                                    onclick: move |e: MouseEvent| {
                                                                                                        e.stop_propagation();
                                                                                                        move_to_column_end(t_id.clone(), t_target.clone());
                                                                                                    },
                                                                                                    "→ {short_label}"
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            div {
                                                                                title: "{ticket.assignee_name()} ({assignee})",
                                                                                class: "flex size-5.5 shrink-0 items-center justify-center rounded-full text-[10px] font-bold bg-muted text-foreground border border-border/80 pointer-events-auto cursor-help",
                                                                                "{assignee}"
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                div {
                                                    class: if is_col_over { "flex items-center justify-center rounded-md border-2 border-dashed border-primary bg-primary/10 py-2.5 text-xs font-semibold text-primary transition-all animate-pulse" } else { "flex items-center justify-center rounded-md border border-dashed border-border/40 py-2 text-[11px] text-muted-foreground/50 transition-all hover:border-border hover:text-muted-foreground hover:bg-accent/20 cursor-pointer" },
                                                    onclick: {
                                                        let col_c = column.to_string();
                                                        move |_| quick_add_ticket(col_c.clone())
                                                    },
                                                    ondragenter: move |e: DragEvent| {
                                                        e.prevent_default();
                                                        if drag_over_col.read().as_deref() != Some(column) {
                                                            drag_over_col.set(Some(column.to_string()));
                                                        }
                                                        drag_over_card.set(None);
                                                    },
                                                    ondragover: move |e: DragEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        if drag_over_col.read().as_deref() != Some(column) {
                                                            drag_over_col.set(Some(column.to_string()));
                                                        }
                                                        drag_over_card.set(None);
                                                    },
                                                    ondrop: move |e: DragEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        let src = e
                                                            .data()
                                                            .data_transfer()
                                                            .get_as_text()
                                                            .filter(|s| !s.is_empty())
                                                            .or_else(|| dragged_id.read().clone());
                                                        if let Some(src) = src {
                                                            move_to_column_end(src, column.to_string());
                                                        }
                                                        clear_drag_state();
                                                    },
                                                    if is_col_over {
                                                        span { "Drop ticket here ({col_display})" }
                                                    } else {
                                                        span { "+ Quick Add Ticket ({col_display})" }
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
                // --------------------------------------------------------
                // IT Data Table View (Full Width, Sortable, Filterable, Paged)
                // --------------------------------------------------------
                TabsContent {
                    value: "table".to_string(),
                    class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden pt-2",
                    DataTable {
                        data: tickets.read().clone(),
                        columns: ticket_columns(),
                        row_id: ticket_row_id,
                        search_text: ticket_search_text,
                        filter_label: "Status".to_string(),
                        filter: vec![
                            DataTableFilter::new("Pending", "Pending"),
                            DataTableFilter::new("InProgress", "In Progress"),
                            DataTableFilter::new("Onhold", "On Hold"),
                            DataTableFilter::new("Completed", "Completed"),
                        ],
                        filter_value: ticket_filter_value as fn(&TicketItem) -> String,
                        filter2_label: "Priority".to_string(),
                        filter2: vec![
                            DataTableFilter::new("Urgent", "Urgent"),
                            DataTableFilter::new("High", "High"),
                            DataTableFilter::new("Medium", "Medium"),
                            DataTableFilter::new("Low", "Low"),
                        ],
                        filter2_value: ticket_priority_filter_value as fn(&TicketItem) -> String,
                        on_row_click: move |t: TicketItem| state.open_view(t),
                        on_row_context_menu: move |(t, coords): (TicketItem, (f64, f64))| {
                            table_ctx_ticket.set(Some(t));
                            table_ctx_x.set(coords.0);
                            table_ctx_y.set(coords.1);
                            table_ctx_open.set(true);
                        },
                        page_size: 15,
                        class: "w-full flex-1 min-h-0".to_string(),
                    }
                }
            }

            // ============================================================
            // View Ticket Dialog Modal
            // ============================================================
            // ============================================================
            // View Ticket Dialog Modal (Clean & Focused on Essential Fields)
            // ============================================================
            Dialog { open: is_view_open,
                DialogContent { class: "max-w-4xl sm:max-w-4xl w-full rounded-md border border-border/80 bg-card p-6 sm:p-7 shadow-2xl max-h-[88vh] overflow-y-auto",
                    if let Some(ticket) = viewing_ticket.read().as_ref() {
                        {
                            let t_for_edit = ticket.clone();
                            let status_lbl = column_display_label(&ticket.kanban_state);
                            let status_badge_cls = status_badge_class(&ticket.kanban_state);
                            let prio_variant = priority_badge(&ticket.priority);
                            let sla_cls = sla_badge_class(&ticket.sla_status);
                            let initial_letter = ticket.emp_name().chars().next().unwrap_or('U');

                            rsx! {
                                DialogHeader {
                                    div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-border/50 pb-3",
                                        div { class: "flex items-center gap-2",
                                            span { class: "font-mono text-xs font-bold text-foreground bg-muted px-2.5 py-0.5 rounded border border-border/80",
                                                "{ticket.id}"
                                            }
                                            Badge {
                                                variant: "{prio_variant}",
                                                class: "text-[10px] font-semibold px-2 py-0.5",
                                                "{ticket.priority}"
                                            }
                                            Badge {
                                                variant: "{status_badge_variant(&ticket.kanban_state)}",
                                                class: "text-[10px] font-semibold {status_badge_cls}",
                                                "{status_lbl}"
                                            }
                                        }
                                        if !ticket.sla_status.is_empty() {
                                            span { class: "rounded px-2 py-0.5 text-[10px] font-semibold border {sla_cls}",
                                                "SLA: {ticket.sla_status}"
                                            }
                                        }
                                    }

                                    // 1. Created By (Requester Identity)

                                    // 2. Assignment, Stage, Category & Customer

                                    // 3. Problem Description

                                    // 4. Attachment (Option of inserting / viewing attachment)

                                    // 5. Timing, SLA Targets & Time Spent

                                    // 6. Activities & My Activity

                                    DialogTitle { class: "text-lg font-bold text-foreground mt-3 leading-snug", "{ticket.subject}" }
                                    DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                        "Reported on {ticket.created_on} · Last updated: {ticket.last_updated_on}"
                                    }
                                }

                                div { class: "flex flex-col gap-4 py-3 text-xs",
                                    div { class: "flex flex-col gap-1.5 rounded-md bg-muted/30 border border-border/40 p-3",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Created By (Requester)"
                                        }
                                        div { class: "flex items-center justify-between gap-2 flex-wrap",
                                            div { class: "flex items-center gap-2.5",
                                                div { class: "flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-foreground font-bold text-xs border border-border/80",
                                                    "{initial_letter}"
                                                }
                                                div { class: "flex flex-col min-w-0",
                                                    span { class: "font-semibold text-foreground text-xs", "{ticket.emp_name()}" }
                                                    span { class: "font-mono text-[11px] text-muted-foreground",
                                                        "{ticket.emp_code_only()}"
                                                    }
                                                }
                                            }
                                            div { class: "flex items-center gap-1.5 text-xs text-muted-foreground",
                                                span { class: "font-mono text-foreground/80 bg-background/60 px-2 py-0.5 rounded border border-border/50",
                                                    "{ticket.email_address}"
                                                }
                                            }
                                        }
                                    }

                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Assigned To"
                                            }
                                            span { class: "font-semibold text-foreground text-xs truncate", "{ticket.assignee_name()}" }
                                            span { class: "text-[10px] text-muted-foreground font-mono", "({ticket.assigned_to})" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Stage / State"
                                            }
                                            span { class: "font-semibold text-foreground text-xs", "{status_lbl}" }
                                            span { class: "text-[10px] text-muted-foreground", "{ticket.stage}" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Category"
                                            }
                                            span { class: "font-semibold text-foreground text-xs", "{ticket.category}" }
                                            span { class: "text-[10px] text-muted-foreground", "IT Support" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Customer / Dept"
                                            }
                                            span { class: "font-semibold text-foreground text-xs truncate",
                                                if ticket.customer.is_empty() {
                                                    "Internal"
                                                } else {
                                                    "{ticket.customer}"
                                                }
                                            }
                                            span { class: "text-[10px] text-muted-foreground", "Client Scope" }
                                        }
                                    }

                                    div { class: "flex flex-col gap-1.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Problem Description"
                                        }
                                        div { class: "rounded-md bg-muted/40 p-3 text-xs leading-relaxed text-foreground border border-border/40 font-mono whitespace-pre-wrap",
                                            if ticket.problem_description.is_empty() {
                                                "No detailed description provided."
                                            } else {
                                                "{ticket.problem_description}"
                                            }
                                        }
                                    }

                                    div { class: "flex flex-col gap-1.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5",
                                            PaperclipIcon { class: "size-3.5 text-primary" }
                                            "Attached File"
                                        }
                                        if !ticket.attachment_name.is_empty() || !ticket.attachment_url.is_empty() {
                                            {
                                                let att_name = ticket.attachment_name.clone();
                                                let att_url = ticket.attachment_url.clone();
                                                let img_preview = get_image_preview_url(&att_name, &att_url);
                                                rsx! {
                                                    div { class: "flex flex-col gap-2 rounded-md bg-muted/30 border border-border/60 p-3",
                                                        div { class: "flex items-center justify-between",
                                                            div { class: "flex items-center gap-2.5 min-w-0",
                                                                div { class: "flex size-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary",
                                                                    FileTextIcon { class: "size-4" }
                                                                }
                                                                div { class: "flex flex-col min-w-0",
                                                                    span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                                                        "{att_name}"
                                                                    }
                                                                    span { class: "text-[10px] text-muted-foreground font-mono truncate",
                                                                        "{att_url}"
                                                                    }
                                                                }
                                                            }
                                                            div { class: "flex items-center gap-2 shrink-0",
                                                                span { class: "text-[10px] bg-primary/10 text-primary px-2 py-0.5 rounded font-medium",
                                                                    "Attached"
                                                                }
                                                            }
                                                        }
                                                        if let Some(preview_src) = img_preview {
                                                            div { class: "mt-1 flex flex-col items-center justify-center rounded-md border border-border/70 bg-card p-2 shadow-xs",
                                                                img {
                                                                    src: "{preview_src}",
                                                                    alt: "{att_name}",
                                                                    class: "max-h-72 max-w-full rounded object-contain",
                                                                    loading: "lazy",
                                                                }
                                                                if !att_url.is_empty() {
                                                                    a {
                                                                        href: "{att_url}",
                                                                        target: "_blank",
                                                                        class: "mt-2 inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                                                        ExternalLinkIcon { class: "size-3" }
                                                                        "Open full image in browser"
                                                                    }
                                                                }
                                                            }
                                                        } else if !att_url.is_empty() {
                                                            a {
                                                                href: "{att_url}",
                                                                target: "_blank",
                                                                class: "inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                                                ExternalLinkIcon { class: "size-3" }
                                                                "Open attachment link"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            div { class: "rounded-md bg-muted/20 border border-border/40 p-2.5 text-xs text-muted-foreground flex items-center gap-2",
                                                PaperclipIcon { class: "size-3.5 text-muted-foreground/60" }
                                                "No file attached to this ticket."
                                            }
                                        }
                                    }

                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Target Deadline"
                                            }
                                            span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                                if ticket.deadline.is_empty() {
                                                    "None set"
                                                } else {
                                                    "{ticket.deadline}"
                                                }
                                            }
                                            span { class: "text-[10px] text-muted-foreground", "Resolution Target" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "SLA Deadline"
                                            }
                                            span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                                if ticket.sla_deadline.is_empty() {
                                                    "None set"
                                                } else {
                                                    "{ticket.sla_deadline}"
                                                }
                                            }
                                            span { class: "text-[10px] text-muted-foreground", "SLA Breach Target" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Time Spent"
                                            }
                                            span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                                "{ticket.time_spent:.1} hrs"
                                            }
                                            span { class: "text-[10px] text-muted-foreground", "Logged Work" }
                                        }
                                        div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                            span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Rating Avg"
                                            }
                                            span { class: "font-semibold text-foreground text-xs truncate",
                                                if ticket.rating_avg_text.is_empty() {
                                                    "Unrated"
                                                } else {
                                                    "{ticket.rating_avg_text}"
                                                }
                                            }
                                            span { class: "text-[10px] text-muted-foreground", "Client Feedback" }
                                        }
                                    }

                                    if !ticket.activities.is_empty() || !ticket.my_activity.is_empty() {
                                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2.5",
                                            if !ticket.activities.is_empty() {
                                                div { class: "flex flex-col gap-1 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                                    span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                        "Logged Activities"
                                                    }
                                                    span { class: "text-xs text-foreground/90 leading-relaxed", "{ticket.activities}" }
                                                }
                                            }
                                            if !ticket.my_activity.is_empty() {
                                                div { class: "flex flex-col gap-1 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                                    span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                        "My Next Activity"
                                                    }
                                                    span { class: "text-xs text-foreground/90 leading-relaxed", "{ticket.my_activity}" }
                                                }
                                            }
                                        }
                                    }
                                }

                                DialogFooter { class: "flex items-center justify-between gap-2 pt-3 border-t border-border/50",
                                    Button {
                                        variant: "outline",
                                        size: "sm",
                                        onclick: move |_| is_view_open.set(false),
                                        "Close"
                                    }
                                    Button {
                                        variant: "default",
                                        size: "sm",
                                        class: "gap-1.5",
                                        onclick: move |_| state.open_edit(t_for_edit.clone()),
                                        PencilIcon { class: "size-3.5" }
                                        "Modify Ticket"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Create Support Ticket Dialog (Matching exact layout)
            // ============================================================
            Dialog { open: is_create_open,
                DialogContent { class: "max-w-2xl sm:max-w-2xl w-full rounded-xl border border-border bg-card p-6 sm:p-8 shadow-2xl max-h-[92vh] overflow-y-auto",
                    DialogHeader {
                        class: "space-y-1 pb-3 text-left",
                        DialogTitle { class: "text-2xl font-bold tracking-tight text-foreground", "Create Support Ticket" }
                        DialogDescription { class: "text-sm text-muted-foreground",
                            "Enter your employee details and describe the issue."
                        }
                    }

                    div { class: "flex flex-col gap-4 py-2 text-sm",
                        // Row 0: Select Employee feature from table
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                span { "Select Employee from Directory" }
                                span { class: "text-[10px] text-muted-foreground font-normal",
                                    "{directory_employees.len()} employees loaded"
                                }
                            }
                            SimpleSelect {
                                value: create_employee_code,
                                placeholder: "-- Choose an Employee (auto-fills code & details) --".to_string(),
                                onchange: move |val: String| {
                                    let trimmed = val.trim().to_string();
                                    if !trimmed.is_empty() {
                                        if let Some(emp) = crate::db::find_employee_by_code(&trimmed) {
                                            create_emp_name.set(emp.emp_name);
                                            if !emp.emp_email.is_empty() {
                                                create_email_address.set(emp.emp_email);
                                            }
                                        }
                                    }
                                },
                                options: directory_employees
                                    .iter()
                                    .map(|emp| {
                                        (
                                            emp.emp_code.clone(),
                                            format!("{} - {} ({})", emp.emp_code, emp.emp_name, emp.department),
                                        )
                                    })
                                    .collect(),
                            }
                        }

                        // Row 1: Employee Code * | Email Address *
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground flex items-center gap-0.5",
                                    "Employee Code"
                                    span { class: "text-destructive font-bold", " *" }
                                }
                                Input {
                                    class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                    placeholder: "".to_string(),
                                    value: create_employee_code.read().clone(),
                                    oninput: move |e: FormEvent| {
                                        let val = e.value();
                                        create_employee_code.set(val.clone());
                                        let trimmed = val.trim();
                                        if !trimmed.is_empty() {
                                            if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                                                create_emp_name.set(emp.emp_name);
                                                if !emp.emp_email.is_empty() {
                                                    create_email_address.set(emp.emp_email);
                                                }
                                            }
                                        }
                                    },
                                }
                            }
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground flex items-center gap-0.5",
                                    "Email Address"
                                    span { class: "text-destructive font-bold", " *" }
                                }
                                Input {
                                    class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                    placeholder: "".to_string(),
                                    value: create_email_address.read().clone(),
                                    oninput: move |e: FormEvent| create_email_address.set(e.value()),
                                }
                            }
                        }

                        // Row 2: Employee Name (Automatically filled from Employee sheet)
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs font-semibold text-foreground",
                                "Employee Name"
                            }
                            Input {
                                class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                placeholder: "Automatically filled from Employee sheet".to_string(),
                                value: create_emp_name.read().clone(),
                                oninput: move |e: FormEvent| create_emp_name.set(e.value()),
                            }
                        }

                        // Row 3: Category * | Priority
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground flex items-center gap-0.5",
                                    "Category"
                                    span { class: "text-destructive font-bold", " *" }
                                }
                                SimpleSelect {
                                    value: create_category,
                                    placeholder: "Select category...".to_string(),
                                    options: vec![
                                        ("IT Support".to_string(), "IT Support".to_string()),
                                        ("Hardware".to_string(), "Hardware".to_string()),
                                        ("Software".to_string(), "Software".to_string()),
                                        ("Network".to_string(), "Network".to_string()),
                                        ("Access & Accounts".to_string(), "Access & Accounts".to_string()),
                                        ("Security".to_string(), "Security".to_string()),
                                        ("Infrastructure".to_string(), "Infrastructure".to_string()),
                                        ("General".to_string(), "General".to_string()),
                                        ("Other".to_string(), "Other".to_string()),
                                    ],
                                }
                            }
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground",
                                    "Priority"
                                }
                                SimpleSelect {
                                    value: create_priority,
                                    placeholder: "Select priority...".to_string(),
                                    options: vec![
                                        ("Low".to_string(), "Low".to_string()),
                                        ("Medium".to_string(), "Medium".to_string()),
                                        ("High".to_string(), "High".to_string()),
                                        ("Critical".to_string(), "Critical".to_string()),
                                    ],
                                }
                            }
                        }

                        // Row 4: Subject *
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs font-semibold text-foreground flex items-center gap-0.5",
                                "Subject"
                                span { class: "text-destructive font-bold", " *" }
                            }
                            Input {
                                class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                placeholder: "".to_string(),
                                value: create_subject.read().clone(),
                                oninput: move |e: FormEvent| create_subject.set(e.value()),
                            }
                        }

                        // Row 5: Problem Description *
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs font-semibold text-foreground flex items-center gap-0.5",
                                "Problem Description"
                                span { class: "text-destructive font-bold", " *" }
                            }
                            textarea {
                                class: "min-h-[130px] rounded-md border border-input bg-card p-3 text-sm text-foreground focus:border-primary focus:ring-1 focus:ring-primary/20 focus:outline-none resize-y",
                                placeholder: "".to_string(),
                                value: "{create_problem_description}",
                                oninput: move |e: FormEvent| create_problem_description.set(e.value()),
                            }
                        }
                    }

                    DialogFooter {
                        class: "flex items-center justify-start gap-3 pt-3",
                        button {
                            r#type: "button",
                            class: "inline-flex items-center justify-center rounded-md bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-2.5 text-sm shadow-xs transition-colors cursor-pointer",
                            onclick: move |_| save_create(),
                            "Submit Ticket"
                        }
                        button {
                            r#type: "button",
                            class: "inline-flex items-center justify-center rounded-md border border-border bg-card hover:bg-muted text-foreground font-medium px-4 py-2.5 text-sm transition-colors cursor-pointer",
                            onclick: move |_| is_create_open.set(false),
                            "Cancel"
                        }
                    }
                }
            }

            // ============================================================
            // Modify Ticket Dialog Modal (Full 23 Fields & Attachment Option)
            // ============================================================
            Dialog { open: is_edit_open,
                DialogContent { class: "max-w-4xl sm:max-w-4xl w-full rounded-md border border-border/80 bg-card p-6 sm:p-7 shadow-2xl max-h-[88vh] overflow-y-auto",
                    if let Some(ticket) = editing_ticket.read().as_ref() {
                        {
                            let t_id = ticket.id.clone();
                            rsx! {
                                DialogHeader {
                                    div { class: "flex items-center gap-2",
                                        span { class: "font-mono text-xs font-semibold text-foreground bg-muted border border-border/70 px-2.5 py-0.5 rounded",
                                            "{t_id}"
                                        }

                                        // 1. Subject

                                        // 2. Created By (Requester Details)

                                        // 3. Classification, Stage & Assignment

                                        // 4. Workflow Stage & Status

                                        // 5. Problem Description

                                        // 6. Attachment Option (Insert, Replace or Remove)

                                        // 7. Target Deadline, SLA Status & Time Spent

                                        // 8. Activities, My Activity & Rating Text

                                        DialogTitle { class: "text-lg font-bold text-foreground", "Modify Ticket" }
                                    }
                                    DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                        "Update ticket details, requester information, assignment, SLA, and attachments."
                                    }
                                }

                                div { class: "flex flex-col gap-4 py-3 text-xs",
                                    div { class: "flex flex-col gap-1",
                                        label { class: "text-xs font-semibold text-foreground", "Subject" }
                                        Input {
                                            class: "text-xs h-9 w-full",
                                            placeholder: "Enter ticket subject...".to_string(),
                                            value: edit_subject.read().clone(),
                                            oninput: move |e: FormEvent| edit_subject.set(e.value()),
                                        }
                                    }

                                    div { class: "rounded-md border border-border/60 bg-muted/20 p-3 flex flex-col gap-2.5",
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                "Created By (Requester)"
                                            }
                                            span { class: "text-[10px] text-muted-foreground",
                                                "Select employee to auto-fill"
                                            }
                                        }
                                        select {
                                            class: "h-8 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                            value: "{edit_employee_code}",
                                            onchange: move |e: FormEvent| {
                                                let val = e.value();
                                                edit_employee_code.set(val.clone());
                                                let trimmed = val.trim();
                                                if !trimmed.is_empty() {
                                                    if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                                                        edit_emp_name.set(emp.emp_name);
                                                        if !emp.emp_email.is_empty() {
                                                            edit_email_address.set(emp.emp_email);
                                                        }
                                                    }
                                                }
                                            },
                                            option { value: "", "-- Choose from Employee Directory (auto-fills) --" }
                                            for emp in directory_employees.iter() {
                                                option {
                                                    key: "{emp.emp_id}",
                                                    value: "{emp.emp_code}",
                                                    "{emp.emp_code} - {emp.emp_name} ({emp.department})"
                                                }
                                            }
                                        }
                                        div { class: "grid grid-cols-1 sm:grid-cols-3 gap-2.5",
                                            div { class: "flex flex-col gap-1",
                                                label { class: "text-[11px] font-semibold text-foreground", "Employee Name" }
                                                Input {
                                                    class: "text-xs h-9 w-full",
                                                    placeholder: "e.g. Employee Name".to_string(),
                                                    value: edit_emp_name.read().clone(),
                                                    oninput: move |e: FormEvent| edit_emp_name.set(e.value()),
                                                }
                                            }
                                            div { class: "flex flex-col gap-1",
                                                label { class: "text-[11px] font-semibold text-foreground", "Employee Code" }
                                                Input {
                                                    class: "text-xs h-9 w-full",
                                                    placeholder: "e.g. 10001".to_string(),
                                                    value: edit_employee_code.read().clone(),
                                                    oninput: move |e: FormEvent| edit_employee_code.set(e.value()),
                                                }
                                            }
                                            div { class: "flex flex-col gap-1",
                                                label { class: "text-[11px] font-semibold text-foreground", "Email Address" }
                                                Input {
                                                    class: "text-xs h-9 w-full",
                                                    placeholder: "e.g. employee@bpsl.net".to_string(),
                                                    value: edit_email_address.read().clone(),
                                                    oninput: move |e: FormEvent| edit_email_address.set(e.value()),
                                                }
                                            }
                                        }
                                    }

                                    div { class: "grid grid-cols-2 sm:grid-cols-5 gap-2.5",
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Category" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_category}",
                                                onchange: move |e: FormEvent| edit_category.set(e.value()),
                                                option { value: "IT Support", "IT Support" }
                                                option { value: "Hardware", "Hardware" }
                                                option { value: "Software", "Software" }
                                                option { value: "Network", "Network" }
                                                option { value: "Access & Accounts", "Access & Accounts" }
                                                option { value: "Security", "Security" }
                                                option { value: "Infrastructure", "Infrastructure" }
                                                option { value: "General", "General" }
                                                option { value: "Other", "Other" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Priority" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_priority}",
                                                onchange: move |e: FormEvent| edit_priority.set(e.value()),
                                                option { value: "High", "High" }
                                                option { value: "Medium", "Medium" }
                                                option { value: "Low", "Low" }
                                                option { value: "Critical", "Critical" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Assigned To" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_assigned_to}",
                                                onchange: move |e: FormEvent| edit_assigned_to.set(e.value()),
                                                option { value: "", "Unassigned" }
                                                option { value: "Shreyansh", "Shreyansh" }
                                                option { value: "kaveri", "Kaveri" }
                                                option { value: "Chaitram", "Chaitram" }
                                                option { value: "IT Support", "IT Support" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Status" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_kanban_state}",
                                                onchange: move |e: FormEvent| {
                                                    let val = e.value();
                                                    edit_kanban_state.set(val.clone());
                                                    edit_stage.set(val.clone());
                                                    edit_status.set(val);
                                                },
                                                option { value: "Pending", "Pending" }
                                                option { value: "InProgress", "In Progress" }
                                                option { value: "Onhold", "On Hold" }
                                                option { value: "Completed", "Completed" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Customer / Dept" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "e.g. Internal, Operations".to_string(),
                                                value: edit_customer.read().clone(),
                                                oninput: move |e: FormEvent| edit_customer.set(e.value()),
                                            }
                                        }
                                    }

                                    div { class: "flex flex-col gap-1",
                                        label { class: "text-[11px] font-semibold text-foreground", "Problem Description" }
                                        textarea {
                                            class: "min-h-[85px] rounded-md border border-border/80 bg-card p-2 text-xs text-foreground focus:border-primary focus:outline-none font-mono",
                                            placeholder: "Detailed description of the issue...",
                                            value: "{edit_problem_description}",
                                            oninput: move |e: FormEvent| edit_problem_description.set(e.value()),
                                        }
                                    }

                                    div { class: "rounded-md border border-border/70 bg-muted/20 p-3.5 flex flex-col gap-2.5",
                                        div { class: "flex items-center justify-between",
                                            span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5",
                                                PaperclipIcon { class: "size-3.5 text-primary" }
                                                "Attachment (Optional)"
                                            }
                                            if !edit_attachment_name.read().is_empty() || !edit_attachment_url.read().is_empty() {
                                                button {
                                                    r#type: "button",
                                                    class: "text-[11px] text-destructive hover:underline flex items-center gap-1 cursor-pointer",
                                                    onclick: move |_| {
                                                        edit_attachment_name.set(String::new());
                                                        edit_attachment_url.set(String::new());
                                                    },
                                                    XIcon { class: "size-3" }
                                                    "Clear Attachment"
                                                }
                                            }
                                        }

                                        if edit_attachment_name.read().is_empty() && edit_attachment_url.read().is_empty() {
                                            div { class: "flex flex-col gap-2",
                                                label { class: "flex flex-col items-center justify-center border-2 border-dashed border-border/80 hover:border-primary/60 bg-card/60 hover:bg-card p-4 rounded-lg cursor-pointer transition-all duration-150 group",
                                                    input {
                                                        r#type: "file",
                                                        class: "hidden",
                                                        onchange: move |e: FormEvent| {
                                                            let raw = e.value();
                                                            let clean = raw.split(['/', '\\']).last().unwrap_or(&raw).to_string();
                                                            if !clean.is_empty() {
                                                                edit_attachment_name.set(clean.clone());
                                                                edit_attachment_url.set(format!("/attachments/{}", clean));
                                                            }
                                                        },
                                                    }
                                                    UploadCloudIcon { class: "size-6 text-muted-foreground group-hover:text-primary transition-colors mb-1" }
                                                    span { class: "text-xs font-medium text-foreground group-hover:text-primary transition-colors",
                                                        "Click to browse and insert an attachment (optional)"
                                                    }
                                                    span { class: "text-[10px] text-muted-foreground",
                                                        "Upload diagnostic logs, screenshots, checklists, or error traces"
                                                    }
                                                }
                                                div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2 pt-1",
                                                    Input {
                                                        class: "text-xs h-8",
                                                        placeholder: "Or specify attachment filename... (e.g. screenshot.png)".to_string(),
                                                        value: edit_attachment_name.read().clone(),
                                                        oninput: move |e: FormEvent| edit_attachment_name.set(e.value()),
                                                    }
                                                    Input {
                                                        class: "text-xs h-8",
                                                        placeholder: "Or attachment URL/Path... (e.g. https://drive.google.com/...)".to_string(),
                                                        value: edit_attachment_url.read().clone(),
                                                        oninput: move |e: FormEvent| edit_attachment_url.set(e.value()),
                                                    }
                                                }
                                            }
                                        } else {
                                            {
                                                let att_name = edit_attachment_name.read().clone();
                                                let att_url = edit_attachment_url.read().clone();
                                                let img_preview = get_image_preview_url(&att_name, &att_url);
                                                rsx! {
                                                    div { class: "flex flex-col gap-2 p-3 rounded-md border border-primary/30 bg-primary/5",
                                                        div { class: "flex items-center justify-between",
                                                            div { class: "flex items-center gap-2.5 min-w-0",
                                                                div { class: "flex size-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary",
                                                                    FileTextIcon { class: "size-4" }
                                                                }
                                                                div { class: "flex flex-col min-w-0",
                                                                    span { class: "font-mono text-xs font-semibold text-foreground truncate",
                                                                        "{att_name}"
                                                                    }
                                                                    span { class: "text-[10px] text-muted-foreground font-mono truncate",
                                                                        if att_url.is_empty() {
                                                                            "Local attachment"
                                                                        } else {
                                                                            "{att_url}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            span { class: "text-[10px] bg-primary/20 text-primary px-2 py-0.5 rounded font-medium shrink-0",
                                                                "Attached"
                                                            }
                                                        }

                                                        // Actual image preview when attachment is an image or Google Drive image link
                                                        if let Some(preview_src) = img_preview {
                                                            div { class: "mt-1 flex flex-col items-center justify-center rounded-md border border-border/70 bg-card p-2 shadow-xs",
                                                                img {
                                                                    src: "{preview_src}",
                                                                    alt: "{att_name}",
                                                                    class: "max-h-72 max-w-full rounded object-contain transition-all hover:scale-[1.01]",
                                                                    loading: "lazy",
                                                                }
                                                                if !att_url.is_empty() {
                                                                    a {
                                                                        href: "{att_url}",
                                                                        target: "_blank",
                                                                        class: "mt-2 inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                                                        ExternalLinkIcon { class: "size-3" }
                                                                        "Open full image in browser"
                                                                    }
                                                                }
                                                            }
                                                        } else if !att_url.is_empty() {
                                                            div { class: "mt-1",
                                                                a {
                                                                    href: "{att_url}",
                                                                    target: "_blank",
                                                                    class: "inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                                                    ExternalLinkIcon { class: "size-3" }
                                                                    "Open attachment link"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3",
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Target Deadline" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "e.g. Today · 05:00 PM or 2026-11-05".to_string(),
                                                value: edit_deadline.read().clone(),
                                                oninput: move |e: FormEvent| edit_deadline.set(e.value()),
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "SLA Status" }
                                            select {
                                                class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                                value: "{edit_sla_status}",
                                                onchange: move |e: FormEvent| edit_sla_status.set(e.value()),
                                                option { value: "Within SLA", "Within SLA" }
                                                option { value: "Approaching Breach", "Approaching Breach" }
                                                option { value: "Breached", "Breached" }
                                                option { value: "Excluded", "Excluded" }
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "SLA Deadline" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "e.g. 2026-11-04 12:00".to_string(),
                                                value: edit_sla_deadline.read().clone(),
                                                oninput: move |e: FormEvent| edit_sla_deadline.set(e.value()),
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Time Spent (hrs)" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "0.0".to_string(),
                                                value: edit_time_spent.read().to_string(),
                                                oninput: move |e: FormEvent| {
                                                    if let Ok(v) = e.value().parse::<f64>() {
                                                        edit_time_spent.set(v);
                                                    }
                                                },
                                            }
                                        }
                                    }

                                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Activities Summary" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "e.g. Diagnostic logs analyzed...".to_string(),
                                                value: edit_activities.read().clone(),
                                                oninput: move |e: FormEvent| edit_activities.set(e.value()),
                                            }
                                        }
                                        div { class: "flex flex-col gap-1",
                                            label { class: "text-[11px] font-semibold text-foreground", "Rating Text" }
                                            Input {
                                                class: "text-xs h-9 w-full",
                                                placeholder: "e.g. 4.9 / 5.0".to_string(),
                                                value: edit_rating_avg_text.read().clone(),
                                                oninput: move |e: FormEvent| edit_rating_avg_text.set(e.value()),
                                            }
                                        }
                                    }

                                    div { class: "flex flex-col gap-1.5 pt-1",
                                        label { class: "text-[11px] font-semibold text-foreground flex items-center justify-between",
                                            span { "My Activity" }
                                            span { class: "text-[10px] text-muted-foreground font-normal", "Type & press Enter to create badge (ALL CAPS)" }
                                        }
                                        MultiActivitySelector { value: edit_my_activity }
                                    }
                                }

                                DialogFooter { class: "flex items-center justify-end gap-2 pt-3 border-t border-border/50",
                                    Button {
                                        variant: "outline",
                                        size: "sm",
                                        onclick: move |_| {
                                            is_edit_open.set(false);
                                            editing_ticket.set(None);
                                        },
                                        "Cancel"
                                    }
                                    Button { variant: "default", size: "sm", onclick: move |_| save_edit(), "Save Changes" }
                                }
                            }
                        }
                    }
                }
            }

            if table_ctx_open() {
                if let Some(ticket) = table_ctx_ticket.read().clone() {
                    TableFloatingContextMenu {
                        ticket,
                        open: table_ctx_open,
                        x: table_ctx_x,
                        y: table_ctx_y,
                        on_view: move |t| state.open_view(t),
                        on_edit: move |t| state.open_edit(t),
                    }
                }
            }
        }
    }
}
