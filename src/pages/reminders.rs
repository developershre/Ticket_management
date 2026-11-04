//! Reminders page — card grid layout with interactive drag-and-drop reordering
//! allowing the user to easily keep track of which reminder comes first.
#![allow(clippy::redundant_closure)]

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::select::SimpleSelect;
use crate::components::ui::separator::Separator;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    ArrowDownIcon, ArrowUpIcon, BellIcon, CheckIcon, ClockIcon, CopyIcon, EyeIcon, FilterIcon,
    GripVerticalIcon, PencilIcon, PlusIcon, SearchIcon, TicketIcon, TrashIcon,
};

/// Reminder item model stored in reactive state with all 12 fields.
#[derive(Clone, PartialEq, Debug)]
pub struct ReminderItem {
    pub id: String, // "REM-{reminder_id}" or "RMD-..."
    pub reminder_id: i64,
    pub employee_code: String,
    pub employee_name: String,
    pub email_address: String,
    pub service_category: String,
    pub reminder_message: String,
    pub send_at: String,
    pub status: String,
    pub email_sent_at: String,
    pub ticket_id: String,
    pub created_at: String,
    pub last_updated_at: String,
    pub server_reminder_id: String,
    pub server_last_updated: String,
    pub cc_emails: String,
    pub repeat_frequency: String,
    pub done: bool,
}

impl ReminderItem {
    pub fn title(&self) -> &str {
        &self.reminder_message
    }
    pub fn when(&self) -> &str {
        &self.send_at
    }
    pub fn tag(&self) -> &str {
        &self.service_category
    }
    pub fn display_name(&self) -> String {
        if !self.employee_name.trim().is_empty() {
            return self.employee_name.trim().to_string();
        }
        let code = self.employee_code.trim();
        if !code.is_empty() {
            if let Some(emp) = crate::db::find_employee_by_code(code) {
                if !emp.emp_name.trim().is_empty() {
                    return emp.emp_name;
                }
            }
            return code.to_string();
        }
        "Unknown".to_string()
    }
}

fn category_badge_variant(tag: &str) -> &'static str {
    match tag {
        "Engineering" => "blue",
        "Meetings" => "purple",
        "Ops" => "amber",
        "Reporting" => "cyan",
        "Planning" => "emerald",
        "Infrastructure" => "orange",
        _ => "outline",
    }
}

fn status_badge_variant(status: &str) -> &'static str {
    match status {
        "Completed" => "emerald",
        "Sent" => "blue",
        "Pending" => "amber",
        "Cancelled" => "destructive",
        _ => "secondary",
    }
}

fn initial_reminders() -> Vec<ReminderItem> {
    // Reminders are read from SQLite only — the local cache of the Google
    // Sheet. When the table is empty, no fake mock items are fabricated.
    crate::db::get_all_reminders()
        .into_iter()
        .map(|r| {
            let done = r.status.eq_ignore_ascii_case("completed")
                || r.status.eq_ignore_ascii_case("sent")
                || r.status.eq_ignore_ascii_case("done");
            let display_id = if !r.server_reminder_id.is_empty() {
                r.server_reminder_id.clone()
            } else {
                format!("REM-{}", r.reminder_id)
            };
            ReminderItem {
                id: display_id,
                reminder_id: r.reminder_id,
                employee_code: r.employee_code,
                employee_name: r.employee_name,
                email_address: r.email_address,
                service_category: r.service_category,
                reminder_message: r.reminder_message,
                send_at: r.send_at,
                status: r.status,
                email_sent_at: r.email_sent_at,
                ticket_id: r.ticket_id,
                created_at: r.created_at,
                last_updated_at: r.last_updated_at,
                server_reminder_id: r.server_reminder_id,
                server_last_updated: r.server_last_updated,
                cc_emails: r.cc_emails,
                repeat_frequency: r.repeat_frequency,
                done,
            }
        })
        .collect()
}

// ============================================================
// Context Menus
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct ReminderContextMenuProps {
    reminder: ReminderItem,
    rank: usize,
    total_count: usize,
    children: Element,
    on_inspect: EventHandler<ReminderItem>,
    on_toggle_done: EventHandler<String>,
    on_move_top: EventHandler<String>,
    on_move_relative: EventHandler<(String, i32)>,
    on_move_bottom: EventHandler<String>,
    on_change_category: EventHandler<(String, String)>,
    on_change_timing: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
    on_send_now: EventHandler<ReminderItem>,
}

#[component]
fn ReminderContextMenu(props: ReminderContextMenuProps) -> Element {
    let r = props.reminder;
    let r_id = r.id.clone();
    let is_done = r.done;
    let rank = props.rank;
    let total = props.total_count;
    let toast = use_context::<ToastState>();

    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    div { class: "flex items-center justify-between gap-2",
                        span { class: "font-mono font-bold text-foreground text-xs", "#{rank} Priority" }
                        Badge { variant: "{category_badge_variant(&r.service_category)}", class: "text-[10px]", "{r.service_category}" }
                    }
                    span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5", "{r.reminder_message}" }
                }
                ContextMenuSeparator {}

                ContextMenuItem {
                    on_select: {
                        let r_c = r.clone();
                        let handler = props.on_inspect;
                        move |_| handler.call(r_c.clone())
                    },
                    EyeIcon { class: "size-3.5 text-muted-foreground" }
                    "View & Edit Details"
                }

                if !is_done {
                    ContextMenuItem {
                        on_select: {
                            let r_id_c = r_id.clone();
                            let handler = props.on_toggle_done;
                            move |_| handler.call(r_id_c.clone())
                        },
                        CheckIcon { class: "size-3.5 text-primary" }
                        "Mark as Completed"
                    }
                }

                if !is_done {
                    ContextMenuItem {
                        on_select: {
                            let r_c = r.clone();
                            let handler = props.on_send_now;
                            move |_| handler.call(r_c.clone())
                        },
                        svg {
                            class: "size-3.5 text-primary",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            line { x1: "22", y1: "2", x2: "11", y2: "13" }
                            polygon { points: "22 2 15 22 11 13 2 9 22 2" }
                        }
                        "Send Reminder Now (Email)"
                    }
                }

                ContextMenuItem {
                    on_select: {
                        let title_c = r.reminder_message.clone();
                        move |_| {
                            toast.success("Copied", format!("Copied: {title_c}"));
                        }
                    },
                    CopyIcon { class: "size-3.5 text-muted-foreground" }
                    "Copy Reminder Text"
                }

                ContextMenuSub {
                    ContextMenuSubTrigger {
                        "Priority & Rank"
                    }
                    ContextMenuSubContent {
                        ContextMenuItem {
                            disabled: rank <= 1,
                            on_select: {
                                let r_id_c = r_id.clone();
                                let handler = props.on_move_top;
                                move |_| handler.call(r_id_c.clone())
                            },
                            ArrowUpIcon { class: "size-3.5 text-primary" }
                            "Move to Top (#1)"
                        }
                        ContextMenuItem {
                            disabled: rank <= 1,
                            on_select: {
                                let r_id_c = r_id.clone();
                                let handler = props.on_move_relative;
                                move |_| handler.call((r_id_c.clone(), -1))
                            },
                            ArrowUpIcon { class: "size-3.5 text-muted-foreground" }
                            "Move Up"
                        }
                        ContextMenuItem {
                            disabled: rank >= total,
                            on_select: {
                                let r_id_c = r_id.clone();
                                let handler = props.on_move_relative;
                                move |_| handler.call((r_id_c.clone(), 1))
                            },
                            ArrowDownIcon { class: "size-3.5 text-muted-foreground" }
                            "Move Down"
                        }
                        ContextMenuItem {
                            disabled: rank >= total,
                            on_select: {
                                let r_id_c = r_id.clone();
                                let handler = props.on_move_bottom;
                                move |_| handler.call(r_id_c.clone())
                            },
                            ArrowDownIcon { class: "size-3.5 text-muted-foreground" }
                            "Move to Bottom"
                        }
                    }
                }

                if !is_done {
                    ContextMenuSub {
                        ContextMenuSubTrigger {
                            "Change Category"
                        }
                        ContextMenuSubContent {
                            for cat in ["Engineering", "Meetings", "Ops", "Planning", "Reporting"] {
                                {
                                    let cat_str = cat.to_string();
                                    let is_current = r.service_category == cat_str;
                                    let r_id_c = r_id.clone();
                                    let handler = props.on_change_category;
                                    rsx! {
                                        ContextMenuItem {
                                            key: "{cat}",
                                            disabled: is_current,
                                            on_select: move |_| handler.call((r_id_c.clone(), cat_str.clone())),
                                            if is_current {
                                                span { class: "text-primary font-semibold flex items-center gap-1.5",
                                                    CheckIcon { class: "size-3.5 text-primary" }
                                                    "{cat}"
                                                }
                                            } else {
                                                span { "{cat}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    ContextMenuSub {
                        ContextMenuSubTrigger {
                            "Set Due Timing"
                        }
                        ContextMenuSubContent {
                            for timing in ["Today · 2:00 PM", "Today · 5:00 PM", "Tomorrow · 9:00 AM", "Tomorrow · 2:00 PM", "Next Week · Monday"] {
                                {
                                    let timing_str = timing.to_string();
                                    let is_current = r.send_at == timing_str;
                                    let r_id_c = r_id.clone();
                                    let handler = props.on_change_timing;
                                    rsx! {
                                        ContextMenuItem {
                                            key: "{timing}",
                                            disabled: is_current,
                                            on_select: move |_| handler.call((r_id_c.clone(), timing_str.clone())),
                                            if is_current {
                                                span { class: "text-primary font-semibold flex items-center gap-1.5",
                                                    CheckIcon { class: "size-3.5 text-primary" }
                                                    "{timing}"
                                                }
                                            } else {
                                                span { "{timing}" }
                                            }
                                        }
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
                        let r_id_c = r_id.clone();
                        let handler = props.on_delete;
                        move |_| handler.call(r_id_c.clone())
                    },
                    TrashIcon { class: "size-3.5 text-destructive" }
                    "Delete Reminder"
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
struct RemindersPageContextMenuProps {
    children: Element,
    on_quick_add: EventHandler<()>,
    on_clear_completed: EventHandler<()>,
    on_set_filter: EventHandler<String>,
}

#[component]
fn RemindersPageContextMenu(props: RemindersPageContextMenuProps) -> Element {
    rsx! {
        ContextMenu {
            ContextMenuTrigger {
                class: "contents",
                {props.children}
            }
            ContextMenuContent {
                class: "w-52 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    span { class: "font-semibold text-foreground text-xs", "Reminders Actions" }
                }
                ContextMenuSeparator {}
                ContextMenuItem {
                    on_select: {
                        let handler = props.on_quick_add;
                        move |_| handler.call(())
                    },
                    PlusIcon { class: "size-3.5 text-primary" }
                    "Quick Add Reminder"
                }
                ContextMenuItem {
                    on_select: {
                        let handler = props.on_clear_completed;
                        move |_| handler.call(())
                    },
                    TrashIcon { class: "size-3.5 text-muted-foreground" }
                    "Clear Completed"
                }
                ContextMenuSeparator {}
                ContextMenuSub {
                    ContextMenuSubTrigger {
                        "Filter by Category"
                    }
                    ContextMenuSubContent {
                        for cat in &["All", "Engineering", "Meetings", "Ops", "Planning", "Reporting"] {
                            {
                                let cat_str = cat.to_string();
                                let handler = props.on_set_filter;
                                rsx! {
                                    ContextMenuItem {
                                        key: "filter-{cat}",
                                        on_select: move |_| handler.call(cat_str.clone()),
                                        span { "{cat}" }
                                    }
                                }
                            }
                        }
                    }
                }
                ContextMenuItem {
                    on_select: {
                        let handler = props.on_set_filter;
                        move |_| handler.call("All".to_string())
                    },
                    FilterIcon { class: "size-3.5 text-muted-foreground" }
                    "Reset Filter (Show All)"
                }
            }
        }
    }
}

#[component]
pub fn Reminders() -> Element {
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();

    let mut reminders = use_signal(initial_reminders);
    let mut query = use_signal(String::new);
    let mut category_filter = use_signal(|| "All".to_string());

    // Auto-reload from SQLite when background sync updates the DB revision
    use_effect(move || {
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        let list = initial_reminders();
        if *reminders.peek() != list {
            reminders.set(list);
        }
    });

    // Fetch employee directory in real-time on mount
    use_effect(move || {
        spawn(async move {
            let _ = crate::sync::pull_employees_now().await;
        });
    });

    // Drag-and-drop reactive state
    let mut dragged_id = use_signal(|| Option::<String>::None);
    let mut drag_over_id = use_signal(|| Option::<String>::None);

    // Dialog modals state
    let mut is_create_open = use_signal(|| false);
    let mut is_detail_open = use_signal(|| false);
    let mut active_reminder = use_signal(|| Option::<ReminderItem>::None);

    // Form inputs for Create Reminder dialog
    let mut new_employee_code = use_signal(String::new);
    let mut new_email_address = use_signal(String::new);
    let mut new_employee_name = use_signal(String::new);
    let mut new_category = use_signal(|| "Select category".to_string());
    let mut new_priority = use_signal(|| "Low".to_string());
    let mut new_subject = use_signal(String::new);
    let mut new_problem_description = use_signal(String::new);
    let mut new_cc_emails = use_signal(String::new);
    let mut new_repeat_frequency = use_signal(|| "daily".to_string());
    let mut new_send_at = use_signal(String::new);

    // Edit Reminder modal state
    let mut is_editing = use_signal(|| false);
    let mut edit_message = use_signal(String::new);
    let mut edit_category = use_signal(|| "General".to_string());
    let mut edit_send_at = use_signal(String::new);
    let mut edit_repeat_frequency = use_signal(|| "daily".to_string());
    let mut edit_cc_emails = use_signal(String::new);

    let directory_employees = crate::db::get_all_employees();
    let toast = use_context::<ToastState>();

    let mut open_detail = move |r: ReminderItem| {
        edit_message.set(r.reminder_message.clone());
        edit_category.set(r.service_category.clone());
        edit_send_at.set(r.send_at.clone());
        let freq = if r.repeat_frequency.trim().is_empty() {
            "daily".to_string()
        } else {
            r.repeat_frequency.clone()
        };
        edit_repeat_frequency.set(freq);
        edit_cc_emails.set(r.cc_emails.clone());
        is_editing.set(false);
        active_reminder.set(Some(r));
        is_detail_open.set(true);
    };

    let q = query.read().to_lowercase();
    let cat = category_filter.read().clone();

    // Open create dialog
    let mut open_create = move || {
        new_employee_code.write().clear();
        new_email_address.write().clear();
        new_employee_name.write().clear();
        new_category.set("Select category".to_string());
        new_priority.set("Low".to_string());
        new_subject.write().clear();
        new_problem_description.write().clear();
        new_cc_emails.write().clear();
        new_repeat_frequency.set("daily".to_string());
        new_send_at.write().clear();
        is_create_open.set(true);
    };

    // Save newly created reminder
    let mut save_new_reminder = move || {
        let emp_code = new_employee_code.read().trim().to_string();
        if emp_code.is_empty() {
            toast.error("Validation Error", "Please enter Employee Code");
            return;
        }

        let email_val = new_email_address.read().trim().to_string();
        if email_val.is_empty() {
            toast.error("Validation Error", "Please enter Email Address");
            return;
        }

        let cat_val = new_category.read().trim().to_string();
        if cat_val.is_empty() || cat_val == "Select category" {
            toast.error("Validation Error", "Please select a Category");
            return;
        }

        let subj_val = new_subject.read().trim().to_string();
        if subj_val.is_empty() {
            toast.error("Validation Error", "Please enter Subject");
            return;
        }

        let desc_val = new_problem_description.read().trim().to_string();
        if desc_val.is_empty() {
            toast.error("Validation Error", "Please describe the problem");
            return;
        }

        let mut emp_name = new_employee_name.read().trim().to_string();
        if emp_name.is_empty() {
            if let Some(u) = crate::db::find_employee_by_code(&emp_code) {
                emp_name = u.emp_name;
            } else {
                emp_name = emp_code.clone();
            }
        }

        let reminder_msg = format!("{subj_val}: {desc_val}");
        let cc_val = new_cc_emails.read().trim().to_string();
        let send_at_val = new_send_at.read().trim().to_string();
        let final_send_at = if send_at_val.is_empty() {
            let now_str = crate::state::format_utc(crate::sync::now_epoch());
            crate::sync::parse_to_iso8601(&now_str)
        } else {
            crate::sync::parse_to_iso8601(&send_at_val)
        };
        let freq_val = new_repeat_frequency.read().trim().to_lowercase();
        let final_freq = if freq_val == "once" { "once" } else { "daily" };

        let input = crate::db::NewReminderInput {
            employee_code: emp_code.clone(),
            employee_name: emp_name.clone(),
            email_address: email_val.clone(),
            service_category: cat_val.clone(),
            reminder_message: reminder_msg,
            send_at: final_send_at,
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: String::new(),
            cc_emails: cc_val.clone(),
            repeat_frequency: final_freq.to_string(),
        };

        match crate::db::create_reminder_full(&input) {
            Ok(new_id) => {
                crate::sync::enqueue_reminder_create(new_id);
                let item = ReminderItem {
                    id: format!("REM-{new_id}"),
                    reminder_id: new_id,
                    employee_code: input.employee_code,
                    employee_name: input.employee_name,
                    email_address: input.email_address,
                    service_category: input.service_category,
                    reminder_message: input.reminder_message,
                    send_at: input.send_at,
                    status: input.status,
                    email_sent_at: input.email_sent_at,
                    ticket_id: input.ticket_id,
                    created_at: "Just now".to_string(),
                    last_updated_at: "Just now".to_string(),
                    server_reminder_id: String::new(),
                    server_last_updated: String::new(),
                    cc_emails: cc_val,
                    repeat_frequency: input.repeat_frequency,
                    done: false,
                };
                reminders.write().insert(0, item);
                if let Some(mut sync) = db_sync {
                    sync.with_mut(|r| r.0 += 1);
                }
                toast.success(
                    "Reminder Scheduled",
                    format!("Reminder for {emp_name} ({emp_code}) created successfully"),
                );
                is_create_open.set(false);
            }
            Err(e) => {
                toast.error(
                    "Database Error",
                    format!("Failed to create reminder: {:?}", e),
                );
            }
        }
    };

    // Save modifications to existing reminder
    let mut save_reminder_edits = move || {
        let Some(curr) = active_reminder.read().clone() else { return; };
        let rem_id = curr.reminder_id;
        let msg = edit_message.read().trim().to_string();
        if msg.is_empty() {
            toast.error("Validation Error", "Reminder message cannot be empty");
            return;
        }
        let cat = edit_category.read().trim().to_string();
        let send_at = edit_send_at.read().trim().to_string();
        let final_send_at = if send_at.is_empty() { curr.send_at.clone() } else { send_at };
        let freq = edit_repeat_frequency.read().trim().to_lowercase();
        let final_freq = if freq == "once" { "once" } else { "daily" };
        let cc = edit_cc_emails.read().trim().to_string();

        let updated_input = crate::db::NewReminderInput {
            employee_code: curr.employee_code.clone(),
            employee_name: curr.employee_name.clone(),
            email_address: curr.email_address.clone(),
            service_category: cat.clone(),
            reminder_message: msg.clone(),
            send_at: final_send_at.clone(),
            status: curr.status.clone(),
            email_sent_at: curr.email_sent_at.clone(),
            ticket_id: curr.ticket_id.clone(),
            cc_emails: cc.clone(),
            repeat_frequency: final_freq.to_string(),
        };

        if let Err(e) = crate::db::update_reminder_full(rem_id, &updated_input) {
            toast.error("Update Failed", format!("Failed to update reminder: {e}"));
            return;
        }

        let patch = serde_json::json!({
            "service_category": cat,
            "reminder_message": msg,
            "send_at": crate::sync::parse_to_iso8601(&final_send_at),
            "repeat_frequency": final_freq,
            "cc_emails": cc,
        });
        crate::sync::enqueue_reminder_update(rem_id, patch);

        let mut updated_item = curr.clone();
        updated_item.service_category = updated_input.service_category;
        updated_item.reminder_message = updated_input.reminder_message;
        updated_item.send_at = updated_input.send_at;
        updated_item.repeat_frequency = updated_input.repeat_frequency;
        updated_item.cc_emails = updated_input.cc_emails;
        let now_str = crate::state::format_utc(crate::sync::now_epoch());
        updated_item.last_updated_at = now_str;

        // Update in-memory reminders list
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.reminder_id == rem_id) {
            list[pos] = updated_item.clone();
        }
        active_reminder.set(Some(updated_item));
        is_editing.set(false);
        if let Some(mut sync) = db_sync {
            sync.with_mut(|r| r.0 += 1);
        }
        toast.success("Reminder Updated", "Changes saved and queued for sync");
    };

    // Reorder reminder from source position to before target card
    let mut move_reminder = move |src_id: String, target_id: String| {
        if src_id == target_id {
            return;
        }
        let mut list = reminders.write();
        if let Some(src_pos) = list.iter().position(|r| r.id == src_id) {
            let item = list.remove(src_pos);
            let title = item.title().to_string();
            if let Some(target_pos) = list.iter().position(|r| r.id == target_id) {
                list.insert(target_pos, item);
                let new_rank = target_pos + 1;
                toast.info(
                    "Priority Updated",
                    format!("\"{title}\" is now #{new_rank}"),
                );
            } else {
                list.push(item);
            }
        }
    };

    // Toggle completion status
    let mut toggle_done = move |id: String| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            if item.done || item.status.eq_ignore_ascii_case("completed") {
                toast.warning("Finalized", "Completed reminders cannot be modified.");
                return;
            }
            item.done = true;
            item.status = "Completed".to_string();
            let rem_id = item.reminder_id;
            let _ = crate::db::update_reminder_status(rem_id, "Completed");
            let _ = crate::db::cancel_queued_updates_for_reminder(rem_id);
            crate::sync::enqueue_reminder_update(rem_id, serde_json::json!({ "status": "Completed" }));
            toast.info("Status Changed", "Reminder marked Completed");
            if let Some(mut sync) = db_sync {
                sync.with_mut(|r| r.0 += 1);
            }
        }
    };

    // Immediate send now (calls reminders.sendNow API)
    let send_now = move |item: ReminderItem, force: bool| {
        let server_id = item.server_reminder_id.clone();
        let rem_id = item.reminder_id;
        let title = item.reminder_message.clone();

        // `reminders.sendNow` requires the sheet's Reminder ID. The local
        // row id is never sent — firing with it would only produce NOT_FOUND.
        if server_id.is_empty() {
            toast.info(
                "Not synced yet",
                "This reminder hasn't reached the Google Sheet yet. Wait for the sync to finish, then send it again.",
            );
            return;
        }

        spawn(async move {
            // The API call happens first; the local "sent" marker is only
            // written after the server confirms (never before).
            match crate::sync::send_reminder_now_immediate(&server_id, force).await {
                Ok(_) => {
                    let now_str = crate::state::format_utc(crate::sync::now_epoch());
                    let _ = crate::db::set_reminder_sent(rem_id, &now_str);
                    if let Some(mut sync) = db_sync {
                        sync.with_mut(|r| r.0 += 1);
                    }
                    toast.success(
                        "Reminder Email Sent",
                        format!("Secure ticket form link sent for \"{title}\" ({server_id})"),
                    );
                }
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("SCHEMA_MISMATCH") {
                        toast.error(
                            "Send Failed",
                            "The Google Sheet 'Reminders' tab headers must match the layout with 'CC Emails' and 'Repeat Frequency'.",
                        );
                    } else if err_str.contains("already sent today") || err_str.contains("once-per-day") {
                        toast.warning(
                            "Already Sent Today",
                            "An email was already sent today for this reminder. Use 'Force Resend' to bypass the daily guard.",
                        );
                    } else {
                        toast.error("Send Failed", format!("Failed to send reminder email: {e}"));
                    }
                }
            }
        });
    };

    // Delete reminder
    let mut delete_reminder = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let item = list.remove(pos);
            let rem_id = item.reminder_id;
            let server_id = item.server_reminder_id.clone();
            let title = item.reminder_message.clone();
            drop(list);

            // The API has no `reminders.delete` (documented action list has
            // 17 actions, none of which delete reminders). The closest
            // supported operation is `status: "Cancelled"`, which stops the
            // scheduled email; the row itself stays in the sheet and will be
            // re-cached locally as Cancelled on the next pull.
            if !server_id.is_empty() {
                crate::sync::enqueue_reminder_delete(rem_id);
                let _ = crate::db::delete_reminder_db(rem_id);
            } else {
                let _ = crate::db::cancel_queued_ops_for_entity("reminder", rem_id);
                let _ = crate::db::delete_reminder_db(rem_id);
            }
            toast.success(
                "Reminder Deleted",
                format!("\"{title}\" removed and synced with Google Sheets"),
            );
            if let Some(mut sync) = db_sync {
                sync.with_mut(|r| r.0 += 1);
            }
        }
    };

    // Change category
    let mut change_category = move |(id, new_tag): (String, String)| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            if item.done || item.status.eq_ignore_ascii_case("completed") {
                toast.warning("Finalized", "Completed reminders cannot be updated.");
                return;
            }
            item.service_category = new_tag.clone();
            let rem_id = item.reminder_id;
            let _ = crate::db::update_reminder_category(rem_id, &new_tag);
            crate::sync::enqueue_reminder_update(
                rem_id,
                serde_json::json!({ "service_category": new_tag }),
            );
            toast.info(
                "Category Changed",
                format!("\"{}\" is now {new_tag}", item.reminder_message),
            );
            if let Some(mut sync) = db_sync {
                sync.with_mut(|r| r.0 += 1);
            }
        }
    };

    // Change timing
    let mut change_timing = move |(id, new_when): (String, String)| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            if item.done || item.status.eq_ignore_ascii_case("completed") {
                toast.warning("Finalized", "Completed reminders cannot be updated.");
                return;
            }
            item.send_at = new_when.clone();
            let rem_id = item.reminder_id;
            let _ = crate::db::update_reminder_send_at(rem_id, &new_when);
            crate::sync::enqueue_reminder_update(
                rem_id,
                serde_json::json!({ "send_at": crate::sync::parse_to_iso8601(&new_when) }),
            );
            toast.info("Timing Changed", format!("Due: {new_when}"));
            if let Some(mut sync) = db_sync {
                sync.with_mut(|r| r.0 += 1);
            }
        }
    };

    // Move to top
    let mut move_to_top = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let item = list.remove(pos);
            let title = item.title().to_string();
            list.insert(0, item);
            toast.success("Priority #1", format!("\"{title}\" moved to top"));
        }
    };

    // Move to bottom
    let mut move_to_bottom = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let item = list.remove(pos);
            let title = item.title().to_string();
            list.push(item);
            toast.info("Priority Updated", format!("\"{title}\" moved to bottom"));
        }
    };

    // Move relative (-1 or +1)
    let mut move_relative = move |(id, dir): (String, i32)| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let len = list.len();
            let new_pos = (pos as isize + dir as isize).clamp(0, (len - 1) as isize) as usize;
            if new_pos != pos {
                let item = list.remove(pos);
                list.insert(new_pos, item);
            }
        }
    };

    // Clear completed reminders
    let mut clear_completed = move || {
        let mut list = reminders.write();
        let completed_ids: Vec<(String, i64, String)> = list
            .iter()
            .filter(|r| r.done)
            .map(|r| (r.id.clone(), r.reminder_id, r.server_reminder_id.clone()))
            .collect();
        let count = completed_ids.len();
        if count == 0 {
            toast.info(
                "No Completed Reminders",
                "There are no completed items to clear",
            );
            return;
        }
        for (_id, rem_id, _server_id) in &completed_ids {
            let _ = crate::db::cancel_queued_updates_for_reminder(*rem_id);
        }
        list.retain(|r| !r.done);
        toast.success(
            "Completed Cleared",
            format!("Cleared {count} completed reminders"),
        );
        if let Some(mut sync) = db_sync {
            sync.with_mut(|r| r.0 += 1);
        }
    };

    let all_reminders = reminders.read().clone();
    let total_count = all_reminders.len();

    let filtered: Vec<ReminderItem> = all_reminders
        .into_iter()
        .filter(|item| {
            let match_q = q.is_empty()
                || item.title().to_lowercase().contains(&q)
                || item.when().to_lowercase().contains(&q)
                || item.tag().to_lowercase().contains(&q)
                || item.employee_code.to_lowercase().contains(&q)
                || item.employee_name.to_lowercase().contains(&q)
                || item.ticket_id.to_lowercase().contains(&q);
            let match_cat = cat == "All" || item.tag() == cat;
            match_q && match_cat
        })
        .collect();

    rsx! {
        RemindersPageContextMenu {
            on_quick_add: move |_| open_create(),
            on_clear_completed: move |_| clear_completed(),
            on_set_filter: move |f| category_filter.set(f),

            div { class: "flex flex-col flex-1 w-full h-full p-4 md:p-6 min-h-0 overflow-hidden",

                // Page header
                div { class: "flex flex-wrap items-center justify-between gap-3 border-b border-border/50 pb-3 mb-3 shrink-0",
                    div { class: "flex items-center gap-3",
                        h1 { class: "text-xl font-bold tracking-tight text-foreground sm:text-2xl",
                            "Reminders"
                        }
                        Badge {
                            variant: "secondary",
                            class: "font-semibold",
                            "{total_count} items"
                        }
                    }
                    div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                        Button {
                            variant: "outline",
                            size: "sm",
                            class: "h-8 gap-1.5",
                            title: "Sync reminders with Google Sheets".to_string(),
                            onclick: move |_| {
                                let _ = crate::db::sync_meta_set("last_reminder_pull_at", "0");
                                crate::sync::kick();
                                if let Some(mut sync) = db_sync {
                                    sync.with_mut(|r| r.0 += 1);
                                }
                                toast.info("Syncing", "Pulling reminders from Google Sheets…");
                            },
                            svg {
                                class: "size-3.5",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
                                path { d: "M21 3v5h-5" }
                                path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
                                path { d: "M8 16H3v5" }
                            }
                            "Sync"
                        }
                        Button {
                            variant: "default",
                            size: "sm",
                            class: "h-8 gap-1.5 shadow-sm",
                            onclick: move |_| open_create(),
                            PlusIcon { class: "size-3.5" }
                            "New Reminder"
                        }
                        Separator { orientation: "vertical", class: "h-3.5 mx-1 hidden sm:block" }
                        div { class: "flex items-center gap-1.5",
                            GripVerticalIcon { class: "size-3.5 text-primary" }
                            span { "Drag cards to set priority (#1 comes first)" }
                        }
                    }
                }

                // Search & Category Chips toolbar
                div { class: "shrink-0 flex flex-wrap items-center justify-between gap-3 mb-3",
                    div { class: "relative flex flex-1 min-w-[200px] max-w-sm items-center",
                        div { class: "pointer-events-none absolute left-3 flex items-center text-muted-foreground",
                            SearchIcon { class: "size-4" }
                        }
                        Input {
                            class: "pl-9 text-xs h-9 w-full",
                            placeholder: "Search reminders by message, employee, ticket...".to_string(),
                            value: query.read().clone(),
                            oninput: move |e: FormEvent| query.set(e.value()),
                        }
                    }

                    // Category Chips
                    div { class: "flex flex-wrap items-center gap-1.5",
                        for tag_name in ["All", "Engineering", "Meetings", "Ops", "Planning", "Reporting", "Infrastructure"] {
                            {
                                let is_sel = cat == tag_name;
                                let btn_variant = if is_sel { "default" } else { "outline" };
                                rsx! {
                                    Button {
                                        variant: "{btn_variant}",
                                        size: "sm",
                                        key: "cat-{tag_name}",
                                        class: "h-7 px-2.5 text-xs",
                                        onclick: move |_| category_filter.set(tag_name.to_string()),
                                        "{tag_name}"
                                    }
                                }
                            }
                        }
                    }
                }

                // Card Grid Layout with Drag-and-Drop
                if filtered.is_empty() {
                    div { class: "flex flex-1 flex-col items-center justify-center rounded-md border border-dashed border-border p-12 text-center",
                        div { class: "flex size-12 items-center justify-center rounded-full bg-muted text-muted-foreground mb-3",
                            SearchIcon { class: "size-6" }
                        }
                        h3 { class: "text-sm font-semibold text-foreground", "No reminders found" }
                        p { class: "mt-1 text-xs text-muted-foreground max-w-xs",
                            "No reminders match your search or filter. Click 'New Reminder' above or right-click to add one."
                        }
                    }
                } else {
                    div { class: "flex-1 overflow-y-auto min-h-0 pr-1 w-full",
                        div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3.5 w-full pb-6",
                            for reminder in filtered.iter() {
                            {
                                let id = reminder.id.clone();
                                let title = reminder.title().to_string();
                                let when = reminder.when().to_string();
                                let tag = reminder.tag().to_string();
                                let is_done = reminder.done;
                                let reminder_c = reminder.clone();

                                // Calculate master priority rank (1-based index in full list)
                                let global_rank = reminders.read().iter().position(|r| r.id == id).unwrap_or(0) + 1;
                                let is_dragged = dragged_id.read().as_deref() == Some(&id);
                                let is_card_over = drag_over_id.read().as_deref() == Some(&id) && !is_dragged;
                                let card_cls = if is_dragged {
                                    "opacity-40 scale-[0.99] border-dashed border-primary ring-1 ring-primary/40 bg-muted/40 cursor-grabbing rounded-md p-3.5 flex flex-col justify-between gap-3 shadow-xs"
                                } else if is_done {
                                    "group flex cursor-grab active:cursor-grabbing flex-col justify-between gap-3 rounded-md border border-border/50 bg-card/50 opacity-70 p-3.5 transition-colors duration-150 hover:opacity-100 hover:border-primary/40 shadow-xs"
                                } else {
                                    "group flex cursor-grab active:cursor-grabbing flex-col justify-between gap-3 rounded-md border border-border/80 bg-card p-3.5 transition-colors duration-150 hover:border-primary/60 hover:bg-accent/15 shadow-xs"
                                };
                                let title_cls = if is_done {
                                    "line-through text-muted-foreground"
                                } else {
                                    "text-foreground group-hover:text-primary transition-colors"
                                };
                                let check_btn_cls = if is_done {
                                    "border-primary bg-primary text-primary-foreground font-semibold"
                                } else {
                                    "border-border/80 bg-background text-muted-foreground hover:border-primary hover:text-foreground"
                                };
                                let id_for_start = id.clone();
                                let id_for_enter = id.clone();
                                let id_for_over = id.clone();
                                let id_for_drop = id.clone();
                                let id_for_toggle = id.clone();
                                let id_for_up = id.clone();
                                let id_for_down = id.clone();
                                let can_up = global_rank > 1;
                                let can_down = global_rank < total_count;

                                rsx! {
                                    div {
                                        key: "rem-card-wrap-{id}",
                                        class: "flex flex-col gap-1.5",
                                        ondragenter: move |e: DragEvent| {
                                            e.prevent_default();
                                            if drag_over_id.read().as_deref() != Some(&id_for_enter) {
                                                drag_over_id.set(Some(id_for_enter.clone()));
                                            }
                                        },
                                        ondragover: move |e: DragEvent| {
                                            e.prevent_default();
                                            if drag_over_id.read().as_deref() != Some(&id_for_over) {
                                                drag_over_id.set(Some(id_for_over.clone()));
                                            }
                                        },
                                        ondrop: move |e: DragEvent| {
                                            e.prevent_default();
                                            e.stop_propagation();
                                            let src = e.data().data_transfer().get_as_text()
                                                .filter(|s| !s.is_empty())
                                                .or_else(|| dragged_id.read().clone());
                                            if let Some(src) = src {
                                                move_reminder(src, id_for_drop.clone());
                                            }
                                            dragged_id.set(None);
                                            drag_over_id.set(None);
                                        },

                                        if is_card_over {
                                            div { class: "h-1 w-full rounded-full bg-primary animate-pulse my-0.5" }
                                        }

                                        ReminderContextMenu {
                                            reminder: reminder.clone(),
                                            rank: global_rank,
                                            total_count: total_count,
                                            on_toggle_done: move |id| toggle_done(id),
                                            on_move_top: move |id| move_to_top(id),
                                            on_move_relative: move |(id, dir)| move_relative((id, dir)),
                                            on_move_bottom: move |id| move_to_bottom(id),
                                            on_change_category: move |(id, cat)| change_category((id, cat)),
                                            on_change_timing: move |(id, timing)| change_timing((id, timing)),
                                            on_delete: move |id| delete_reminder(id),
                                            on_inspect: move |r| open_detail(r),
                                            on_send_now: move |r| send_now(r, false),

                                            div {
                                                draggable: "true",
                                                "data-drag-id": "{id}",
                                                class: "select-none {card_cls}",
                                                ondragstart: move |_| {
                                                    dragged_id.set(Some(id_for_start.clone()));
                                                },
                                                ondragend: move |_| {
                                                    dragged_id.set(None);
                                                    drag_over_id.set(None);
                                                },
                                                onclick: {
                                                    let r_view = reminder_c.clone();
                                                    move |_| {
                                                        open_detail(r_view.clone());
                                                    }
                                                },

                                                div { class: "flex items-center justify-between",
                                                    div { class: "flex items-center gap-2",
                                                        GripVerticalIcon { class: "size-4 text-muted-foreground/40 group-hover:text-primary transition-colors cursor-grab" }
                                                        if global_rank == 1 {
                                                            Badge {
                                                                variant: "default",
                                                                class: "font-mono text-[11px] font-bold",
                                                                "#1 Next Up"
                                                            }
                                                        } else {
                                                            Badge {
                                                                variant: "secondary",
                                                                class: "font-mono text-[11px]",
                                                                "#{global_rank}"
                                                            }
                                                        }
                                                        // Quick up/down reorder buttons
                                                        div { class: "flex items-center gap-0.5 pointer-events-auto",
                                                            if can_up {
                                                                button {
                                                                    r#type: "button",
                                                                    title: "Move up in priority",
                                                                    class: "flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer text-xs font-bold",
                                                                    onclick: move |e: MouseEvent| {
                                                                        e.stop_propagation();
                                                                        let mut list = reminders.write();
                                                                        if let Some(pos) = list.iter().position(|r| r.id == id_for_up) {
                                                                            if pos > 0 {
                                                                                list.swap(pos, pos - 1);
                                                                            }
                                                                        }
                                                                    },
                                                                    "↑"
                                                                }
                                                            }
                                                            if can_down {
                                                                button {
                                                                    r#type: "button",
                                                                    title: "Move down in priority",
                                                                    class: "flex size-5 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer text-xs font-bold",
                                                                    onclick: move |e: MouseEvent| {
                                                                        e.stop_propagation();
                                                                        let mut list = reminders.write();
                                                                        if let Some(pos) = list.iter().position(|r| r.id == id_for_down) {
                                                                            if pos + 1 < list.len() {
                                                                                list.swap(pos, pos + 1);
                                                                            }
                                                                        }
                                                                    },
                                                                    "↓"
                                                                }
                                                            }
                                                        }
                                                    }
                                                    div { class: "flex items-center gap-1.5 flex-wrap",
                                                        Badge {
                                                            variant: "{category_badge_variant(&tag)}",
                                                            class: "text-[10px]",
                                                            "{tag}"
                                                        }
                                                        Badge {
                                                            variant: "{status_badge_variant(&reminder.status)}",
                                                            class: "text-[10px]",
                                                            "{reminder.status}"
                                                        }
                                                        span {
                                                            class: "rounded px-1.5 py-0.5 text-[9px] font-semibold border bg-muted/60 text-muted-foreground uppercase",
                                                            "{reminder.repeat_frequency}"
                                                        }
                                                        if !reminder.ticket_id.is_empty() {
                                                            span { class: "inline-flex items-center gap-1 rounded bg-muted/70 px-1.5 py-0.5 text-[10px] font-mono text-muted-foreground",
                                                                TicketIcon { class: "size-3 text-primary" }
                                                                "{reminder.ticket_id}"
                                                            }
                                                        }
                                                    }
                                                }

                                                div { class: "flex flex-col gap-1.5 py-1",
                                                    h3 { class: "text-sm font-semibold leading-snug {title_cls}", "{title}" }
                                                    div { class: "flex items-center justify-between text-xs text-muted-foreground pt-0.5",
                                                        div { class: "flex items-center gap-1.5",
                                                            ClockIcon { class: "size-3.5 text-muted-foreground" }
                                                            span { "{when}" }
                                                        }
                                                        span { class: "text-[11px] font-medium text-foreground truncate max-w-[140px]",
                                                            "{reminder.display_name()}"
                                                        }
                                                    }
                                                }

                                                div { class: "flex items-center justify-between border-t border-border/40 pt-2.5 mt-1",
                                                    div { class: "flex items-center gap-1.5 truncate max-w-[150px]",
                                                        span { class: "text-[11px] text-muted-foreground truncate",
                                                            if is_done {
                                                                "Completed"
                                                            } else {
                                                                "{reminder.display_name()}"
                                                            }
                                                        }
                                                        if !reminder.employee_code.is_empty() {
                                                            span { class: "font-mono text-[10px] text-muted-foreground/60",
                                                                "#{reminder.employee_code}"
                                                            }
                                                        }
                                                    }
                                                    div { class: "flex items-center gap-1.5",
                                                        button {
                                                            r#type: "button",
                                                            title: "Send secure ticket form link email now",
                                                            class: "flex items-center gap-1 rounded-md border border-border/80 bg-background px-2 py-1 text-xs text-muted-foreground hover:border-primary hover:text-primary transition-colors cursor-pointer",
                                                            onclick: {
                                                                let r_send = reminder_c.clone();
                                                                move |e: MouseEvent| {
                                                                    e.stop_propagation();
                                                                    send_now(r_send.clone(), false);
                                                                }
                                                            },
                                                            svg {
                                                                class: "size-3",
                                                                view_box: "0 0 24 24",
                                                                fill: "none",
                                                                stroke: "currentColor",
                                                                stroke_width: "2",
                                                                stroke_linecap: "round",
                                                                stroke_linejoin: "round",
                                                                line { x1: "22", y1: "2", x2: "11", y2: "13" }
                                                                polygon { points: "22 2 15 22 11 13 2 9 22 2" }
                                                            }
                                                            span { "Send" }
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            disabled: is_done,
                                                            title: if is_done { "Completed (finalized)" } else { "Mark complete" },
                                                            class: format!("flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs transition-colors {check_btn_cls} {}", if is_done { "opacity-75 cursor-default" } else { "cursor-pointer" }),
                                                            onclick: move |e: MouseEvent| {
                                                                e.stop_propagation();
                                                                if !is_done {
                                                                    toggle_done(id_for_toggle.clone());
                                                                }
                                                            },
                                                            if is_done {
                                                                svg {
                                                                    class: "size-3 text-primary-foreground",
                                                                    view_box: "0 0 24 24",
                                                                    fill: "none",
                                                                    stroke: "currentColor",
                                                                    stroke_width: "3",
                                                                    stroke_linecap: "round",
                                                                    stroke_linejoin: "round",
                                                                    path { d: "M20 6 9 17l-5-5" }
                                                                }
                                                            }
                                                            span {
                                                                if is_done {
                                                                    "Done"
                                                                } else {
                                                                    "Mark Done"
                                                                }
                                                            }
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            title: "Delete reminder & sync with Google Sheets",
                                                            class: "flex items-center gap-1 rounded-md border border-border/80 bg-background px-2 py-1 text-xs text-muted-foreground hover:border-destructive hover:text-destructive hover:bg-destructive/10 transition-colors cursor-pointer",
                                                            onclick: {
                                                                let id_del = reminder_c.id.clone();
                                                                move |e: MouseEvent| {
                                                                    e.stop_propagation();
                                                                    delete_reminder(id_del.clone());
                                                                }
                                                            },
                                                            TrashIcon { class: "size-3" }
                                                            span { "Delete" }
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

        // ============================================================
        // Create Reminder / Support Ticket Dialog (Matching exact form)
        // ============================================================
        if is_create_open() {
            Dialog {
                open: Some(is_create_open),
                DialogContent {
                    class: "max-w-2xl max-h-[92vh] overflow-y-auto p-6 sm:p-8 rounded-xl bg-card border border-border shadow-2xl",
                    DialogHeader {
                        class: "space-y-1 pb-3 text-left",
                        DialogTitle { class: "text-2xl font-bold tracking-tight text-foreground",
                            "Create Support Ticket"
                        }
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
                            crate::components::employee_search::EmployeeSearchInput {
                                employee_code: new_employee_code,
                                employee_name: new_employee_name,
                                email_address: new_email_address,
                                placeholder: "Type code (e.g. 11061) or name (e.g. Chaitram)...".to_string(),
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
                                    value: new_employee_code.read().clone(),
                                    oninput: move |e: FormEvent| {
                                        let val = e.value();
                                        new_employee_code.set(val.clone());
                                        let trimmed = val.trim();
                                        if !trimmed.is_empty() {
                                            if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                                                new_employee_name.set(emp.emp_name);
                                                if !emp.emp_email.is_empty() {
                                                    new_email_address.set(emp.emp_email);
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
                                    value: new_email_address.read().clone(),
                                    oninput: move |e: FormEvent| new_email_address.set(e.value()),
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
                                value: new_employee_name.read().clone(),
                                oninput: move |e: FormEvent| new_employee_name.set(e.value()),
                            }
                        }

                        // Row 2.5: CC Recipients (Optional, synced to CC Emails column)
                        div { class: "flex flex-col gap-1.5",
                            label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                span { "CC Recipients (Optional)" }
                                span { class: "text-[10px] text-muted-foreground font-normal",
                                    "Comma-separated emails (e.g. manager@example.com)"
                                }
                            }
                            Input {
                                class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                placeholder: "e.g. it.manager@example.com, lead@example.com".to_string(),
                                value: new_cc_emails.read().clone(),
                                oninput: move |e: FormEvent| new_cc_emails.set(e.value()),
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
                                    value: new_category,
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
                                    value: new_priority,
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

                        // Row 3.5: Repeat Frequency & Scheduled Send At
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Repeat Frequency" }
                                SimpleSelect {
                                    value: new_repeat_frequency,
                                    placeholder: "Select frequency...".to_string(),
                                    options: vec![
                                        ("daily".to_string(), "Daily (every 24h)".to_string()),
                                        ("once".to_string(), "Once (one-time send)".to_string()),
                                    ],
                                }
                            }
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                    span { "Schedule Date & Time" }
                                    span { class: "text-[10px] text-muted-foreground font-normal", "(Leave empty for now)" }
                                }
                                Input {
                                    class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                    placeholder: "e.g. 2026-11-06T10:30:00+05:30".to_string(),
                                    value: new_send_at.read().clone(),
                                    oninput: move |e: FormEvent| new_send_at.set(e.value()),
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
                                value: new_subject.read().clone(),
                                oninput: move |e: FormEvent| new_subject.set(e.value()),
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
                                value: "{new_problem_description}",
                                oninput: move |e: FormEvent| new_problem_description.set(e.value()),
                            }
                        }
                    }

                    DialogFooter {
                        class: "flex items-center justify-start gap-3 pt-3",
                        button {
                            r#type: "button",
                            class: "inline-flex items-center justify-center rounded-md bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-2.5 text-sm shadow-xs transition-colors cursor-pointer",
                            onclick: move |_| save_new_reminder(),
                            "Create Reminder"
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
        }

        // ============================================================
        // View / Edit Reminder Dialog
        // ============================================================
        if is_detail_open() {
            if let Some(ref r) = active_reminder() {
                Dialog {
                    open: Some(is_detail_open),
                    DialogContent {
                        class: "max-w-2xl max-h-[90vh] overflow-y-auto",
                        DialogHeader {
                            div { class: "flex items-center justify-between",
                                div { class: "flex items-center gap-2",
                                    div { class: "flex size-8 items-center justify-center rounded bg-primary/10 text-primary",
                                        BellIcon { class: "size-4" }
                                    }
                                    div {
                                        DialogTitle { "{r.id}" }
                                        DialogDescription { "Full reminder specifications, recipient identity, and delivery tracking." }
                                    }
                                }
                                div { class: "flex items-center gap-1.5",
                                    Badge { variant: "{category_badge_variant(&r.service_category)}", "{r.service_category}" }
                                    Badge { variant: "{status_badge_variant(&r.status)}", "{r.status}" }
                                    if r.status.eq_ignore_ascii_case("pending") {
                                        Button {
                                            variant: if is_editing() { "secondary" } else { "outline" },
                                            class: "h-7 text-xs px-2.5 ml-1",
                                            onclick: move |_| is_editing.toggle(),
                                            if is_editing() { "Cancel Editing" } else { "Edit" }
                                        }
                                    }
                                }
                            }
                        }

                        if is_editing() {
                            div { class: "flex flex-col gap-3.5 py-2 text-xs",
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground", "Reminder Message" }
                                    textarea {
                                        class: "min-h-[100px] rounded-md border border-input bg-card p-2.5 text-xs text-foreground focus:border-primary focus:ring-1 focus:ring-primary/20 focus:outline-none resize-y",
                                        value: "{edit_message}",
                                        oninput: move |e: FormEvent| edit_message.set(e.value()),
                                    }
                                }
                                div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                                    div { class: "flex flex-col gap-1.5",
                                        label { class: "text-xs font-semibold text-foreground", "Service Category" }
                                        SimpleSelect {
                                            value: edit_category,
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
                                        label { class: "text-xs font-semibold text-foreground", "Repeat Frequency" }
                                        SimpleSelect {
                                            value: edit_repeat_frequency,
                                            placeholder: "Select frequency...".to_string(),
                                            options: vec![
                                                ("daily".to_string(), "Daily (every 24h)".to_string()),
                                                ("once".to_string(), "Once (one-time send)".to_string()),
                                            ],
                                        }
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Scheduled Send At" }
                                        span { class: "text-[10px] text-muted-foreground", "e.g. 2026-11-06T10:30:00+05:30" }
                                    }
                                    Input {
                                        class: "h-9 text-xs rounded-md border border-input bg-card px-3 focus:border-primary",
                                        value: edit_send_at.read().clone(),
                                        oninput: move |e: FormEvent| edit_send_at.set(e.value()),
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "CC Recipients (Optional)" }
                                        span { class: "text-[10px] text-muted-foreground", "Comma-separated emails" }
                                    }
                                    Input {
                                        class: "h-9 text-xs rounded-md border border-input bg-card px-3 focus:border-primary",
                                        placeholder: "e.g. it.manager@example.com, lead@example.com".to_string(),
                                        value: edit_cc_emails.read().clone(),
                                        oninput: move |e: FormEvent| edit_cc_emails.set(e.value()),
                                    }
                                }
                            }

                            DialogFooter {
                                Button {
                                    variant: "outline",
                                    onclick: move |_| is_editing.set(false),
                                    "Cancel"
                                }
                                Button {
                                    variant: "default",
                                    onclick: move |_| save_reminder_edits(),
                                    "Save Changes"
                                }
                            }
                        } else {
                            div { class: "flex flex-col gap-3 py-2 text-xs",
                                // Message
                                div { class: "p-3 rounded-md bg-card border border-border/80",
                                    span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider block mb-1", "Reminder Message" }
                                    p { class: "text-sm font-semibold text-foreground leading-relaxed", "{r.reminder_message}" }
                                }

                                if !r.cc_emails.is_empty() {
                                    div { class: "p-3 rounded-md bg-card border border-border/80 flex flex-col gap-1.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "CC Recipients" }
                                        div { class: "flex flex-wrap gap-1.5",
                                            for email in r.cc_emails.split([',', ';']) {
                                                {
                                                    let email_t = email.trim();
                                                    if !email_t.is_empty() {
                                                        rsx! {
                                                            span { class: "font-mono text-xs px-2 py-0.5 rounded-md bg-muted text-foreground border border-border/60", "{email_t}" }
                                                        }
                                                    } else {
                                                        rsx! {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Recipient & Ticket
                                div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2.5",
                                    div { class: "p-3 rounded-md bg-muted/20 border border-border/60 flex flex-col gap-1.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Recipient" }
                                        span { class: "font-semibold text-foreground text-xs", "{r.display_name()}" }
                                        span { class: "font-mono text-muted-foreground text-[11px]", "Code: {r.employee_code}" }
                                        span { class: "text-muted-foreground text-[11px]", "Email: {r.email_address}" }
                                    }
                                    div { class: "p-3 rounded-md bg-muted/20 border border-border/60 flex flex-col gap-1.5",
                                        span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Associated Ticket" }
                                        if r.ticket_id.is_empty() {
                                            span { class: "text-muted-foreground italic text-xs", "No ticket linked (Standalone Reminder)" }
                                        } else {
                                            div { class: "flex items-center gap-2 mt-1",
                                                TicketIcon { class: "size-4 text-primary" }
                                                span { class: "font-mono font-bold text-foreground text-xs", "{r.ticket_id}" }
                                            }
                                        }
                                    }
                                }

                                // Delivery & Timestamps & Repeat Frequency
                                div { class: "grid grid-cols-2 sm:grid-cols-5 gap-2.5 p-3 rounded-md bg-muted/20 border border-border/60",
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-[10px] text-muted-foreground uppercase tracking-wider font-semibold", "Send At" }
                                        span { class: "font-semibold text-foreground", "{r.send_at}" }
                                    }
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-[10px] text-muted-foreground uppercase tracking-wider font-semibold", "Frequency" }
                                        span { class: "font-semibold text-foreground uppercase text-[11px]", "{r.repeat_frequency}" }
                                    }
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-[10px] text-muted-foreground uppercase tracking-wider font-semibold", "Email Sent At" }
                                        span { class: "text-foreground font-mono text-[11px]", "{r.email_sent_at}" }
                                    }
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-[10px] text-muted-foreground uppercase tracking-wider font-semibold", "Created At" }
                                        span { class: "text-muted-foreground font-mono text-[11px]", "{r.created_at}" }
                                    }
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-[10px] text-muted-foreground uppercase tracking-wider font-semibold", "Last Updated" }
                                        span { class: "text-muted-foreground font-mono text-[11px]", "{r.last_updated_at}" }
                                    }
                                }
                            }

                            DialogFooter {
                                Button {
                                    variant: "destructive",
                                    onclick: {
                                        let id_del = r.id.clone();
                                        move |_| {
                                            delete_reminder(id_del.clone());
                                            is_detail_open.set(false);
                                        }
                                    },
                                    TrashIcon { class: "size-3.5 mr-1.5" }
                                    "Delete"
                                }
                                if r.status.eq_ignore_ascii_case("pending") {
                                    Button {
                                        variant: "secondary",
                                        onclick: move |_| is_editing.set(true),
                                        PencilIcon { class: "size-3.5 mr-1.5" }
                                        "Edit Details"
                                    }
                                }
                                Button {
                                    variant: "outline",
                                    onclick: move |_| is_detail_open.set(false),
                                    "Close"
                                }
                                Button {
                                    variant: "default",
                                    onclick: {
                                        let r_send = r.clone();
                                        move |_| {
                                            send_now(r_send.clone(), false);
                                            is_detail_open.set(false);
                                        }
                                    },
                                    svg {
                                        class: "size-3.5 mr-1.5",
                                        view_box: "0 0 24 24",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                        line { x1: "22", y1: "2", x2: "11", y2: "13" }
                                        polygon { points: "22 2 15 22 11 13 2 9 22 2" }
                                    }
                                    "Send Email Now"
                                }
                                if !r.email_sent_at.is_empty() && r.email_sent_at != "Not sent yet" && r.email_sent_at != "Not sent" {
                                    Button {
                                        variant: "outline",
                                        title: "Bypasses once-per-calendar-day guard to re-send reminder email",
                                        class: "text-amber-600 border-amber-500/30 hover:bg-amber-500/10",
                                        onclick: {
                                            let r_send = r.clone();
                                            move |_| {
                                                send_now(r_send.clone(), true);
                                                is_detail_open.set(false);
                                            }
                                        },
                                        "Force Resend"
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
