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
use crate::components::ui::separator::Separator;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    ArrowDownIcon, ArrowUpIcon, BellIcon, CheckIcon, ClockIcon, CopyIcon, FilterIcon,
    GripVerticalIcon, PlusIcon, SearchIcon, TicketIcon, TrashIcon,
};

/// Reminder item model stored in reactive state with all 12 fields.
#[derive(Clone, PartialEq, Debug)]
pub struct ReminderItem {
    pub id: String, // "REM-{reminder_id}"
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
    let db_reminders = crate::db::get_all_reminders();
    if !db_reminders.is_empty() {
        return db_reminders
            .into_iter()
            .map(|r| {
                let done = r.status.eq_ignore_ascii_case("completed");
                ReminderItem {
                    id: format!("REM-{}", r.reminder_id),
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
                    done,
                }
            })
            .collect();
    }

    vec![
        ReminderItem {
            id: "REM-1".to_string(),
            reminder_id: 1,
            employee_code: "EMP-4019".to_string(),
            employee_name: "David Chen".to_string(),
            email_address: "david.chen@corp.internal".to_string(),
            service_category: "Engineering".to_string(),
            reminder_message: "Review pull request #482".to_string(),
            send_at: "Today · 2:00 PM".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: "IT-1".to_string(),
            created_at: "Today".to_string(),
            last_updated_at: "Today".to_string(),
            done: false,
        },
        ReminderItem {
            id: "REM-2".to_string(),
            reminder_id: 2,
            employee_code: "EMP-5082".to_string(),
            employee_name: "Sarah Miller".to_string(),
            email_address: "sarah.miller@corp.internal".to_string(),
            service_category: "Meetings".to_string(),
            reminder_message: "Team standup notes & updates".to_string(),
            send_at: "Today · 4:30 PM".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: String::new(),
            created_at: "Today".to_string(),
            last_updated_at: "Today".to_string(),
            done: false,
        },
        ReminderItem {
            id: "REM-3".to_string(),
            reminder_id: 3,
            employee_code: "EMP-1024".to_string(),
            employee_name: "Robert Taylor".to_string(),
            email_address: "robert.taylor@corp.internal".to_string(),
            service_category: "Ops".to_string(),
            reminder_message: "Renew domain + SSL certificates".to_string(),
            send_at: "Tomorrow · 9:00 AM".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: "IT-2".to_string(),
            created_at: "Yesterday".to_string(),
            last_updated_at: "Yesterday".to_string(),
            done: false,
        },
    ]
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
    on_toggle_done: EventHandler<String>,
    on_move_top: EventHandler<String>,
    on_move_relative: EventHandler<(String, i32)>,
    on_move_bottom: EventHandler<String>,
    on_change_category: EventHandler<(String, String)>,
    on_change_timing: EventHandler<(String, String)>,
    on_delete: EventHandler<String>,
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
                        let r_id_c = r_id.clone();
                        let handler = props.on_toggle_done;
                        move |_| handler.call(r_id_c.clone())
                    },
                    CheckIcon { class: "size-3.5 text-primary" }
                    if is_done {
                        "Mark as Incomplete"
                    } else {
                        "Mark as Completed"
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

    // Drag-and-drop reactive state
    let mut dragged_id = use_signal(|| Option::<String>::None);
    let mut drag_over_id = use_signal(|| Option::<String>::None);

    // Dialog modals state
    let mut is_create_open = use_signal(|| false);
    let mut is_detail_open = use_signal(|| false);
    let mut active_reminder = use_signal(|| Option::<ReminderItem>::None);

    // Form inputs for Create Reminder dialog (essential fields only)
    let mut new_category = use_signal(|| "Engineering".to_string());
    let mut new_message = use_signal(String::new);
    let mut new_send_at = use_signal(|| "Today · 2:00 PM".to_string());
    let mut new_ticket_id = use_signal(String::new);

    let toast = use_context::<ToastState>();

    let q = query.read().to_lowercase();
    let cat = category_filter.read().clone();

    // Open create dialog
    let mut open_create = move || {
        new_category.set("Engineering".to_string());
        new_message.write().clear();
        new_send_at.set("Today · 2:00 PM".to_string());
        new_ticket_id.write().clear();
        is_create_open.set(true);
    };

    // Save newly created reminder (auto-populates recipient profile and status)
    let mut save_new_reminder = move || {
        let msg = new_message.read().trim().to_string();
        if msg.is_empty() {
            toast.error("Required Field", "Please provide a reminder message.");
            return;
        }

        let input = crate::db::NewReminderInput {
            employee_code: "EMP-1001".to_string(),
            employee_name: "Gamer Rock".to_string(),
            email_address: "admin@taskflow.internal".to_string(),
            service_category: new_category.read().clone(),
            reminder_message: msg.clone(),
            send_at: new_send_at.read().clone(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: new_ticket_id.read().clone(),
        };

        match crate::db::create_reminder_full(&input) {
            Ok(new_id) => {
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
                    done: false,
                };
                reminders.write().insert(0, item);
                if let Some(mut sync) = db_sync {
                    sync.with_mut(|r| r.0 += 1);
                }
                toast.success("Reminder Created", format!("REM-{new_id}: \"{msg}\" created successfully"));
                is_create_open.set(false);
            }
            Err(e) => {
                toast.error("Database Error", format!("Failed to create reminder: {:?}", e));
            }
        }
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
                toast.info("Priority Updated", format!("\"{title}\" is now #{new_rank}"));
            } else {
                list.push(item);
            }
        }
    };

    // Toggle completion status
    let mut toggle_done = move |id: String| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            item.done = !item.done;
            item.status = if item.done { "Completed".to_string() } else { "Pending".to_string() };
            let status = item.status.clone();
            let _ = crate::db::update_reminder_status(item.reminder_id, &status);
            toast.info("Status Changed", format!("Reminder marked {status}"));
        }
    };

    // Delete reminder
    let mut delete_reminder = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let item = list.remove(pos);
            let _ = crate::db::delete_reminder_db(item.reminder_id);
            toast.success("Reminder Deleted", format!("\"{}\" removed", item.reminder_message));
        }
    };

    // Change category
    let mut change_category = move |(id, new_tag): (String, String)| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            item.service_category = new_tag.clone();
            let _ = crate::db::update_reminder_category(item.reminder_id, &new_tag);
            toast.info("Category Changed", format!("\"{}\" is now {new_tag}", item.reminder_message));
        }
    };

    // Change timing
    let mut change_timing = move |(id, new_when): (String, String)| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            item.send_at = new_when.clone();
            let _ = crate::db::update_reminder_send_at(item.reminder_id, &new_when);
            toast.info("Timing Changed", format!("Due: {new_when}"));
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
        let completed_ids: Vec<(String, i64)> = list.iter().filter(|r| r.done).map(|r| (r.id.clone(), r.reminder_id)).collect();
        let count = completed_ids.len();
        if count == 0 {
            toast.info("No Completed Reminders", "There are no completed items to clear");
            return;
        }
        for (_id, rem_id) in &completed_ids {
            let _ = crate::db::delete_reminder_db(*rem_id);
        }
        list.retain(|r| !r.done);
        toast.success("Completed Cleared", format!("Cleared {count} completed reminders"));
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
                                                        active_reminder.set(Some(r_view.clone()));
                                                        is_detail_open.set(true);
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
                                                        span { class: "font-mono text-[11px] text-muted-foreground truncate max-w-[130px]",
                                                            "{reminder.employee_code}"
                                                        }
                                                    }
                                                }

                                                div { class: "flex items-center justify-between border-t border-border/40 pt-2.5 mt-1",
                                                    span { class: "text-[11px] text-muted-foreground truncate max-w-[140px]",
                                                        if is_done {
                                                            "Completed"
                                                        } else {
                                                            "{reminder.employee_name}"
                                                        }
                                                    }
                                                    button {
                                                        r#type: "button",
                                                        title: if is_done { "Mark incomplete" } else { "Mark complete" },
                                                        class: "flex items-center gap-1.5 rounded-md border px-2.5 py-1 text-xs transition-colors cursor-pointer {check_btn_cls}",
                                                        onclick: move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            toggle_done(id_for_toggle.clone());
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
        // Create Reminder Dialog (Streamlined Essential Inputs)
        // ============================================================
        if is_create_open() {
            Dialog {
                open: Some(is_create_open),
                DialogContent {
                    class: "max-w-xl max-h-[90vh] overflow-y-auto",
                    DialogHeader {
                        div { class: "flex items-center gap-2",
                            div { class: "flex size-8 items-center justify-center rounded bg-primary/10 text-primary",
                                BellIcon { class: "size-4" }
                            }
                            div {
                                DialogTitle { "Create Reminder" }
                                DialogDescription { "Schedule a reminder notification. Recipient profile and automated dispatch tracking are handled automatically." }
                            }
                        }
                    }

                    div { class: "flex flex-col gap-3.5 py-2 text-xs",
                        // 1. Reminder Message (Required)
                        div { class: "flex flex-col gap-1",
                            label { class: "text-xs font-semibold text-foreground flex items-center gap-1",
                                "Reminder Message"
                                span { class: "text-destructive", "*" }
                            }
                            textarea {
                                class: "min-h-[75px] rounded-md border border-input bg-card p-2 text-xs text-foreground focus:border-primary focus:outline-none",
                                placeholder: "e.g. Renew domain + SSL certificates before expiry...",
                                value: "{new_message}",
                                oninput: move |e: FormEvent| new_message.set(e.value()),
                            }
                        }

                        // 2. Service Category & Linked Ticket ID
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                            div { class: "flex flex-col gap-1",
                                label { class: "text-[11px] font-semibold text-foreground", "Service / Category" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                    value: "{new_category}",
                                    onchange: move |e: FormEvent| new_category.set(e.value()),
                                    option { value: "Engineering", "Engineering" }
                                    option { value: "Meetings", "Meetings" }
                                    option { value: "Ops", "Ops" }
                                    option { value: "Reporting", "Reporting" }
                                    option { value: "Planning", "Planning" }
                                    option { value: "Infrastructure", "Infrastructure" }
                                    option { value: "General", "General" }
                                }
                            }
                            div { class: "flex flex-col gap-1",
                                label { class: "text-[11px] font-semibold text-foreground", "Ticket ID (optional link)" }
                                Input {
                                    class: "text-xs h-9 w-full font-mono",
                                    placeholder: "e.g. IT-1 or leave blank".to_string(),
                                    value: new_ticket_id.read().clone(),
                                    oninput: move |e: FormEvent| new_ticket_id.set(e.value()),
                                }
                            }
                        }

                        // 3. Scheduling & Delivery Timing
                        div { class: "rounded-md border border-border/70 bg-muted/20 p-3 flex flex-col gap-2",
                            label { class: "text-[11px] font-semibold text-foreground flex items-center gap-1.5",
                                ClockIcon { class: "size-3 text-primary" }
                                "Schedule & Timing"
                            }
                            Input {
                                class: "text-xs h-9 w-full",
                                placeholder: "e.g. Today · 2:00 PM".to_string(),
                                value: new_send_at.read().clone(),
                                oninput: move |e: FormEvent| new_send_at.set(e.value()),
                            }
                            div { class: "flex flex-wrap gap-1.5 pt-0.5",
                                for preset in ["Today · 2:00 PM", "Tomorrow · 9:00 AM", "Friday · 5:00 PM", "Mon · 10:00 AM"] {
                                    {
                                        let preset_str = preset.to_string();
                                        rsx! {
                                            button {
                                                key: "{preset}",
                                                r#type: "button",
                                                class: "text-[10px] px-2 py-0.5 rounded border border-border/70 hover:bg-muted/80 text-muted-foreground hover:text-foreground cursor-pointer transition-colors",
                                                onclick: move |_| new_send_at.set(preset_str.clone()),
                                                "{preset}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // 4. Automated System Defaults Summary Banner
                        div { class: "flex flex-wrap items-center justify-between gap-1.5 p-2.5 rounded bg-muted/30 border border-border/50 text-[10px] text-muted-foreground font-mono",
                            span { class: "font-sans font-medium text-foreground", "⚡ Auto-populated:" }
                            span { "Recipient: Gamer Rock (EMP-1001)" }
                            span { "•" }
                            span { "Status: Pending" }
                            span { "•" }
                            span { "Auto email dispatch" }
                        }
                    }

                    DialogFooter {
                        Button {
                            variant: "outline",
                            onclick: move |_| is_create_open.set(false),
                            "Cancel"
                        }
                        Button {
                            variant: "default",
                            onclick: move |_| save_new_reminder(),
                            "Create Reminder"
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
                                }
                            }
                        }

                        div { class: "flex flex-col gap-3 py-2 text-xs",
                            // Message
                            div { class: "p-3 rounded-md bg-card border border-border/80",
                                span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider block mb-1", "Reminder Message" }
                                p { class: "text-sm font-semibold text-foreground leading-relaxed", "{r.reminder_message}" }
                            }

                            // Recipient & Ticket
                            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2.5",
                                div { class: "p-3 rounded-md bg-muted/20 border border-border/60 flex flex-col gap-1.5",
                                    span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Recipient" }
                                    span { class: "font-semibold text-foreground text-xs", "{r.employee_name}" }
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

                            // Delivery & Timestamps
                            div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5 p-3 rounded-md bg-muted/20 border border-border/60",
                                div { class: "flex flex-col gap-0.5",
                                    span { class: "text-[10px] text-muted-foreground uppercase tracking-wider font-semibold", "Send At" }
                                    span { class: "font-semibold text-foreground", "{r.send_at}" }
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
                                variant: "outline",
                                onclick: move |_| is_detail_open.set(false),
                                "Close"
                            }
                        }
                    }
                }
            }
        }
    }
}

