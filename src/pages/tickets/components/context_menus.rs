use dioxus::prelude::*;

use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
use crate::components::ui::toast::ToastState;
use crate::icons::{
    CheckIcon, CopyIcon, EyeIcon, EyeOffIcon, FilterIcon, PencilIcon, PlusIcon, TrashIcon,
};
use super::super::types::{TicketContextState, TicketItem};

#[derive(Props, Clone, PartialEq)]
pub struct TicketContextMenuProps {
    pub ticket: TicketItem,
    pub children: Element,
    pub on_view: EventHandler<TicketItem>,
    pub on_edit: EventHandler<TicketItem>,
}

#[component]
pub fn TicketContextMenu(props: TicketContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t = props.ticket;
    let t_id = t.id.clone();
    let current_status = if !t.kanban_state.is_empty() {
        t.kanban_state.clone()
    } else {
        t.status.clone()
    };
    let current_prio = t.priority.clone();
    let current_cat = t.category.clone();
    let current_assignee = t.assigned_to.clone();

    rsx! {
        ContextMenu {
            ContextMenuTrigger { class: "contents", {props.children} }
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
                        for &(code, label) in &[
                            ("Pending", "Pending"),
                            ("InProgress", "In Progress"),
                            ("Hold", "On Hold"),
                            ("Solved", "Solved"),
                            ("Completed", "Solved (Closed)"),
                            ("Cancelled", "Cancelled"),
                        ] {
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
                        ] {
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
pub struct TableFloatingContextMenuProps {
    pub ticket: TicketItem,
    pub open: Signal<bool>,
    pub x: Signal<f64>,
    pub y: Signal<f64>,
    pub on_view: EventHandler<TicketItem>,
    pub on_edit: EventHandler<TicketItem>,
}

#[component]
pub fn TableFloatingContextMenu(props: TableFloatingContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let t = props.ticket;
    let t_id = t.id.clone();
    let current_status = if !t.kanban_state.is_empty() {
        t.kanban_state.clone()
    } else {
        t.status.clone()
    };
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
                    for &(code, label) in &[
                        ("Pending", "Pending"),
                        ("InProgress", "In Progress"),
                        ("Hold", "On Hold"),
                        ("Solved", "Solved"),
                        ("Completed", "Solved (Closed)"),
                        ("Cancelled", "Cancelled"),
                    ] {
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
                    ] {
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
pub struct ColumnContextMenuProps {
    pub column: String,
    pub title: String,
    pub children: Element,
    pub on_quick_add: EventHandler<String>,
    pub on_filter_column: EventHandler<String>,
    pub on_clear_filters: EventHandler<()>,
    pub on_hide_column: EventHandler<String>,
    pub is_custom: bool,
    pub on_delete_column: EventHandler<String>,
    #[props(default)]
    pub all_targets: Vec<(String, String)>,
}

#[component]
pub fn ColumnContextMenu(props: ColumnContextMenuProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let col = props.column;
    let col_disp = props.title;

    rsx! {
        ContextMenu {
            ContextMenuTrigger { class: "contents", {props.children} }
            ContextMenuContent { class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    span { class: "font-semibold text-foreground text-xs", "View: {col_disp}" }
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
                ContextMenuItem {
                    on_select: {
                        let col_c = col.clone();
                        let handler = props.on_hide_column;
                        move |_| handler.call(col_c.clone())
                    },
                    EyeOffIcon { class: "size-3.5 text-muted-foreground" }
                    "Hide This View"
                }
                if props.is_custom {
                    ContextMenuItem {
                        on_select: {
                            let col_c = col.clone();
                            let handler = props.on_delete_column;
                            move |_| handler.call(col_c.clone())
                        },
                        TrashIcon { class: "size-3.5 text-destructive" }
                        "Delete Custom View"
                    }
                }
                if !props.all_targets.is_empty() {
                    ContextMenuSub {
                        ContextMenuSubTrigger { "Move All Tickets To" }
                        ContextMenuSubContent {
                            for (target_id, target_label) in props.all_targets.iter() {
                                {
                                    let is_current = target_id == &col;
                                    let from_c = col.clone();
                                    let to_c = target_id.clone();
                                    let target_lbl = target_label.clone();
                                    rsx! {
                                        ContextMenuItem {
                                            key: "move-all-{target_id}",
                                            disabled: is_current,
                                            on_select: move |_| {
                                                state.move_column_tickets(&from_c, &to_c);
                                                toast.info("Tickets Moved", format!("All tickets moved to {target_lbl}"));
                                            },
                                            div { class: "flex items-center justify-between w-full",
                                                span { "{target_label}" }
                                                if is_current {
                                                    span { class: "text-[10px] text-muted-foreground font-mono ml-auto", "current" }
                                                }
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
                    on_select: move |_| props.on_clear_filters.call(()),
                    "Clear Column Filters"
                }
            }
        }
    }
}
