use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::data_table::{DataTableColumn, SortValue};
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator,
    DropdownMenuSub, DropdownMenuSubContent, DropdownMenuSubTrigger, DropdownMenuTrigger,
};
use crate::components::ui::toast::ToastState;
use crate::icons::{CopyIcon, EyeIcon, MoreHorizontalIcon, PencilIcon, TrashIcon};

use super::components::TicketContextMenu;
use super::types::{
    assignee_color_classes, column_display_label, get_assignee_initials, priority_badge,
    sla_badge_class, status_badge_class, status_badge_variant, TicketContextState, TicketItem,
};

#[component]
pub fn TicketIdCell(ticket: TicketItem) -> Element {
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
pub fn TicketTitleCell(ticket: TicketItem) -> Element {
    let mut state = use_context::<TicketContextState>();
    let title = ticket.title.clone();
    let emp_name = ticket.emp_name();
    let emp_code = ticket.emp_code_only().to_string();
    let email = ticket.emp_email();
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
pub fn TicketActionsCell(ticket: TicketItem) -> Element {
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
                            for &(code, label) in &[
                                ("Pending", "Pending"),
                                ("InProgress", "In Progress"),
                                ("Hold", "On Hold"),
                                ("Solved", "Solved"),
                                ("Completed", "Solved (Closed)"),
                                ("Cancelled", "Cancelled"),
                            ] {
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

pub fn render_id_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketIdCell { ticket: t.clone() }
    }
}

pub fn render_title_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketTitleCell { ticket: t.clone() }
    }
}

pub fn render_actions_cell(t: &TicketItem) -> Element {
    rsx! {
        TicketActionsCell { ticket: t.clone() }
    }
}

pub fn render_category_cell(t: &TicketItem) -> Element {
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

pub fn render_status_cell(t: &TicketItem) -> Element {
    let kanban = t.kanban_state.clone();
    let variant = status_badge_variant(&kanban);
    let class = status_badge_class(&kanban);
    let label = column_display_label(&kanban);
    let is_open = t.status.eq_ignore_ascii_case("Open");
    rsx! {
        div { class: "flex items-center gap-1.5 flex-wrap",
            span {
                class: if is_open {
                    "inline-flex items-center gap-1 text-[9px] font-semibold text-emerald-600 dark:text-emerald-400 bg-emerald-500/10 border border-emerald-500/30 rounded px-1.5 py-0.2"
                } else {
                    "inline-flex items-center gap-1 text-[9px] font-semibold text-slate-500 dark:text-slate-400 bg-slate-500/10 border border-slate-500/30 rounded px-1.5 py-0.2"
                },
                span {
                    class: if is_open {
                        "size-1.5 rounded-full bg-emerald-500 animate-pulse"
                    } else {
                        "size-1.5 rounded-full bg-slate-400"
                    }
                }
                "{t.status}"
            }
            Badge { variant: "{variant}", class: "text-[10px] font-semibold {class}", "{label}" }
        }
    }
}

pub fn render_stage_cell(t: &TicketItem) -> Element {
    let stage = t.stage.clone();
    rsx! {
        if !stage.is_empty() && stage != "Triage" && stage != t.kanban_state {
            span {
                class: "inline-flex items-center rounded bg-secondary/80 border border-border/60 px-1.5 py-0.5 text-[10px] font-mono font-medium text-foreground/80 truncate max-w-[150px]",
                title: "Action / Stage: {stage}",
                "{stage}"
            }
        } else {
            span { class: "text-muted-foreground/50 text-[11px]", "—" }
        }
    }
}

pub fn render_sla_cell(t: &TicketItem) -> Element {
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

pub fn render_priority_cell(t: &TicketItem) -> Element {
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

pub fn render_assignee_cell(t: &TicketItem) -> Element {
    let assignee = t.assignee.clone();
    let initials = get_assignee_initials(&assignee);
    let color_cls = assignee_color_classes(&assignee);
    let name = t.assignee_name();
    rsx! {
        div { class: "flex items-center gap-2",
            div {
                class: "flex size-6 shrink-0 items-center justify-center rounded-full text-[10px] font-extrabold uppercase border shadow-2xs {color_cls}",
                title: "Assigned to: {name}",
                "{initials}"
            }
            span { class: "text-xs font-medium text-foreground truncate", "{name}" }
        }
    }
}

pub fn render_created_cell(t: &TicketItem) -> Element {
    let created = t.created_at.clone();
    let disp = crate::state::format_display_12hr(&created);
    rsx! {
        span {
            title: "{created}",
            class: "text-[11px] text-muted-foreground font-medium",
            "{disp}"
        }
    }
}

pub fn ticket_row_id(t: &TicketItem) -> String {
    t.id.clone()
}

#[allow(dead_code)]
pub fn ticket_filter_value(t: &TicketItem) -> String {
    t.status.clone()
}

#[allow(dead_code)]
pub fn ticket_priority_filter_value(t: &TicketItem) -> String {
    t.priority.clone()
}

pub fn ticket_search_text(t: &TicketItem) -> String {
    let emp_code = t.emp_code_only();
    let emp_name = t.emp_name();
    let emp_email = t.emp_email();
    let digits: String = emp_code.chars().filter(|c| c.is_ascii_digit()).collect();
    let prefixed = if !digits.is_empty() { format!("EMP-{digits}") } else { String::new() };
    format!(
        "{} {} {} {} {} {} {} {} {}",
        t.id, t.title, t.category, t.status, t.priority, emp_name, emp_code, emp_email, prefixed
    )
}

pub fn ticket_columns() -> Vec<DataTableColumn<TicketItem>> {
    vec![
        DataTableColumn::new(
            "id",
            "Ticket ID",
            render_id_cell,
            |t: &TicketItem| SortValue::Number(t.ticket_number()),
        ),
        DataTableColumn::new(
            "title",
            "Subject & Requester",
            render_title_cell,
            |t: &TicketItem| SortValue::Text(t.title.clone()),
        ),
        DataTableColumn::new(
            "stage",
            "Action / Stage",
            render_stage_cell,
            |t: &TicketItem| SortValue::Text(t.stage.clone()),
        ),
        DataTableColumn::new(
            "category",
            "Category",
            render_category_cell,
            |t: &TicketItem| SortValue::Text(t.category.clone()),
        ),
        DataTableColumn::new(
            "status",
            "Status",
            render_status_cell,
            |t: &TicketItem| SortValue::Text(t.status.clone()),
        ),
        DataTableColumn::new(
            "sla",
            "SLA Status",
            render_sla_cell,
            |t: &TicketItem| SortValue::Text(t.sla_status.clone()),
        ),
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
