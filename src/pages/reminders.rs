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
use crate::components::ui::input::Input;
use crate::components::ui::separator::Separator;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    ArrowDownIcon, ArrowUpIcon, CheckIcon, CopyIcon, FilterIcon, GripVerticalIcon, PlusIcon,
    SearchIcon, TrashIcon,
};

/// Reminder item model stored in reactive state.
#[derive(Clone, PartialEq, Debug)]
pub struct ReminderItem {
    pub id: String,
    pub title: String,
    pub when: String,
    pub tag: String,
    pub done: bool,
}

fn category_badge_variant(tag: &str) -> &'static str {
    match tag {
        "Engineering" => "default",
        "Meetings" => "secondary",
        "Ops" => "warning",
        "Reporting" => "info",
        "Planning" => "success",
        _ => "outline",
    }
}

fn initial_reminders() -> Vec<ReminderItem> {
    let db_notes = crate::db::get_all_notes();
    if !db_notes.is_empty() {
        return db_notes
            .into_iter()
            .enumerate()
            .map(|(idx, n)| {
                let parts: Vec<&str> = n.content.split(" — ").collect();
                let title = parts.first().copied().unwrap_or(&n.content).to_string();
                let tag = match idx % 5 {
                    0 => "Engineering",
                    1 => "Meetings",
                    2 => "Ops",
                    3 => "Reporting",
                    _ => "Planning",
                };
                let when = match idx % 4 {
                    0 => "Today · 2:00 PM",
                    1 => "Today · 4:30 PM",
                    2 => "Tomorrow · 9:00 AM",
                    _ => "Mon · 10:00 AM",
                };
                ReminderItem {
                    id: format!("rem-{}", n.id),
                    title,
                    when: when.to_string(),
                    tag: tag.to_string(),
                    done: false,
                }
            })
            .collect();
    }

    vec![
        ReminderItem {
            id: "rem-1".to_string(),
            title: "Review pull request #482".to_string(),
            when: "Today · 2:00 PM".to_string(),
            tag: "Engineering".to_string(),
            done: false,
        },
        ReminderItem {
            id: "rem-2".to_string(),
            title: "Team standup notes & updates".to_string(),
            when: "Today · 4:30 PM".to_string(),
            tag: "Meetings".to_string(),
            done: false,
        },
        ReminderItem {
            id: "rem-3".to_string(),
            title: "Renew domain + SSL certificates".to_string(),
            when: "Tomorrow · 9:00 AM".to_string(),
            tag: "Ops".to_string(),
            done: false,
        },
        ReminderItem {
            id: "rem-4".to_string(),
            title: "Send weekly progress summary".to_string(),
            when: "Friday · 5:00 PM".to_string(),
            tag: "Reporting".to_string(),
            done: false,
        },
        ReminderItem {
            id: "rem-5".to_string(),
            title: "Sprint backlog grooming & planning".to_string(),
            when: "Mon · 10:00 AM".to_string(),
            tag: "Planning".to_string(),
            done: false,
        },
        ReminderItem {
            id: "rem-6".to_string(),
            title: "Clean up unused Docker images and staging volumes".to_string(),
            when: "Tue · 03:00 PM".to_string(),
            tag: "Ops".to_string(),
            done: true,
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
                        Badge { variant: "{category_badge_variant(&r.tag)}", class: "text-[10px]", "{r.tag}" }
                    }
                    span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5", "{r.title}" }
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
                        let title_c = r.title.clone();
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
                                let is_current = r.tag == cat_str;
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
                                let is_current = r.when == timing_str;
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
    let mut reminders = use_signal(initial_reminders);
    let mut query = use_signal(String::new);
    let mut category_filter = use_signal(|| "All".to_string());

    // Drag-and-drop reactive state
    let mut dragged_id = use_signal(|| Option::<String>::None);
    let mut drag_over_id = use_signal(|| Option::<String>::None);

    let toast = use_context::<ToastState>();

    let q = query.read().to_lowercase();
    let cat = category_filter.read().clone();

    // Reorder reminder from source position to before target card
    let mut move_reminder = move |src_id: String, target_id: String| {
        if src_id == target_id {
            return;
        }
        let mut list = reminders.write();
        if let Some(src_pos) = list.iter().position(|r| r.id == src_id) {
            let item = list.remove(src_pos);
            let title = item.title.clone();
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
            let status = if item.done { "completed" } else { "pending" };
            toast.info("Status Changed", format!("Reminder marked {status}"));
        }
    };

    // Delete reminder
    let mut delete_reminder = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let title = list[pos].title.clone();
            list.remove(pos);
            if let Some(num) = id.strip_prefix("rem-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::delete_note(num);
            }
            toast.success("Reminder Deleted", format!("\"{title}\" removed"));
        }
    };

    // Change category
    let mut change_category = move |(id, new_tag): (String, String)| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            item.tag = new_tag.clone();
            let title = item.title.clone();
            let when = item.when.clone();
            if let Some(num) = id.strip_prefix("rem-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::update_note(num, &format!("{title} — {when} [{new_tag}]"));
            }
            toast.info("Category Changed", format!("\"{title}\" is now {new_tag}"));
        }
    };

    // Change timing
    let mut change_timing = move |(id, new_when): (String, String)| {
        let mut list = reminders.write();
        if let Some(item) = list.iter_mut().find(|r| r.id == id) {
            item.when = new_when.clone();
            let title = item.title.clone();
            let tag = item.tag.clone();
            if let Some(num) = id.strip_prefix("rem-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::update_note(num, &format!("{title} — {new_when} [{tag}]"));
            }
            toast.info("Timing Changed", format!("Due: {new_when}"));
        }
    };

    // Move to top
    let mut move_to_top = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let item = list.remove(pos);
            let title = item.title.clone();
            list.insert(0, item);
            toast.success("Priority #1", format!("\"{title}\" moved to top"));
        }
    };

    // Move to bottom
    let mut move_to_bottom = move |id: String| {
        let mut list = reminders.write();
        if let Some(pos) = list.iter().position(|r| r.id == id) {
            let item = list.remove(pos);
            let title = item.title.clone();
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

    // Quick add reminder
    let mut quick_add_reminder = move |tag: String| {
        let count = reminders.read().len() + 1;
        let title = format!("New reminder item #{}", count);
        let when = "Tomorrow · 10:00 AM".to_string();
        let real_tag = if tag == "All" || tag.is_empty() { "Engineering".to_string() } else { tag };

        let db_id = crate::db::create_note("EMP-1001", &format!("{title} — {when} [{real_tag}]"));
        let real_id = if let Ok(id) = db_id { format!("rem-{id}") } else { format!("rem-{}", 100 + count) };

        reminders.write().insert(0, ReminderItem {
            id: real_id.clone(),
            title: title.clone(),
            when,
            tag: real_tag,
            done: false,
        });
        toast.success("Reminder Created", format!("\"{title}\" added as #1 priority"));
    };

    // Clear completed reminders
    let mut clear_completed = move || {
        let mut list = reminders.write();
        let completed_ids: Vec<String> = list.iter().filter(|r| r.done).map(|r| r.id.clone()).collect();
        let count = completed_ids.len();
        if count == 0 {
            toast.info("No Completed Reminders", "There are no completed items to clear");
            return;
        }
        for id in &completed_ids {
            if let Some(num) = id.strip_prefix("rem-").and_then(|s| s.parse::<i64>().ok()) {
                let _ = crate::db::delete_note(num);
            }
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
                || item.title.to_lowercase().contains(&q)
                || item.when.to_lowercase().contains(&q)
                || item.tag.to_lowercase().contains(&q);
            let match_cat = cat == "All" || item.tag == cat;
            match_q && match_cat
        })
        .collect();

    rsx! {
        RemindersPageContextMenu {
            on_quick_add: {
                let cat_c = cat.clone();
                move |_| quick_add_reminder(cat_c.clone())
            },
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
                            onclick: {
                                let cat_c = cat.clone();
                                move |_| quick_add_reminder(cat_c.clone())
                            },
                            PlusIcon { class: "size-3.5 text-primary" }
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
                            placeholder: "Search reminders...".to_string(),
                            value: query.read().clone(),
                            oninput: move |e: FormEvent| query.set(e.value()),
                        }
                    }

                    // Category Chips
                    div { class: "flex flex-wrap items-center gap-1.5",
                        for tag_name in ["All", "Engineering", "Meetings", "Ops", "Planning", "Reporting"] {
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
                            "No reminders match your search or filter. Right-click anywhere to add one, or select 'All'."
                        }
                    }
                } else {
                    div { class: "flex-1 overflow-y-auto min-h-0 pr-1 w-full",
                        div { class: "grid grid-cols-1 md:grid-cols-2 xl:grid-cols-4 gap-3.5 w-full pb-6",
                        for reminder in filtered.iter() {
                            {
                                let id = reminder.id.clone();
                                let title = reminder.title.clone();
                                let when = reminder.when.clone();
                                let tag = reminder.tag.clone();
                                let is_done = reminder.done;

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
                                                    Badge {
                                                        variant: "{category_badge_variant(&tag)}",
                                                        class: "text-[10px]",
                                                        "{tag}"
                                                    }
                                                }

                                                div { class: "flex flex-col gap-1.5 py-1",
                                                    h3 { class: "text-sm font-semibold leading-snug {title_cls}", "{title}" }
                                                    div { class: "flex items-center gap-1.5 text-xs text-muted-foreground",
                                                        svg {
                                                            class: "size-3.5",
                                                            view_box: "0 0 24 24",
                                                            fill: "none",
                                                            stroke: "currentColor",
                                                            stroke_width: "2",
                                                            stroke_linecap: "round",
                                                            stroke_linejoin: "round",
                                                            circle { cx: "12", cy: "12", r: "10" }
                                                            polyline { points: "12 6 12 12 16 14" }
                                                        }
                                                        span { "{when}" }
                                                    }
                                                }

                                                div { class: "flex items-center justify-between border-t border-border/40 pt-2.5 mt-1",
                                                    span { class: "text-[11px] text-muted-foreground",
                                                        if is_done {
                                                            "Completed"
                                                        } else if global_rank == 1 {
                                                            "First in order"
                                                        } else {
                                                            "Right-click or drag to reorder"
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
    }
}

