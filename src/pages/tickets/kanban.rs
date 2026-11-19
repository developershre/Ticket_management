use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::dropdown_menu::{
    DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator,
    DropdownMenuTrigger,
};
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    CheckIcon, EyeIcon, EyeOffIcon, GripVerticalIcon, PanelRightIcon, PaperclipIcon, PencilIcon,
    PlusIcon, SearchIcon, TrashIcon,
};

use super::components::{ColumnContextMenu, TicketContextMenu};
use super::types::{
    column_display_label, kanban_grid_layout_class, priority_badge, save_hidden_views,
    save_hide_empty_views, status_badge_class, status_badge_variant, KanbanViewDef,
    TicketContextState, TicketItem,
};

#[derive(Props, Clone, PartialEq)]
pub struct KanbanBoardProps {
    pub search_query: Signal<String>,
    pub selected_priority: Signal<String>,
    pub selected_status: Signal<String>,
    pub col_limits: Signal<std::collections::HashMap<String, usize>>,
    pub hidden_views: Signal<std::collections::HashSet<String>>,
    pub hide_empty_views: Signal<bool>,
    pub is_add_view_open: Signal<bool>,
    pub all_view_defs: Memo<Vec<KanbanViewDef>>,
    pub grouped_columns: Memo<std::collections::HashMap<String, Vec<TicketItem>>>,
    pub visible_view_defs: Memo<Vec<KanbanViewDef>>,
    pub empty_views_count: Memo<usize>,
    pub hidden_empty_names: Memo<String>,
    pub all_targets: Vec<(String, String)>,
    pub on_delete_custom_view: EventHandler<(String, String)>,
    pub on_quick_add: EventHandler<String>,
}

#[component]
pub fn KanbanBoard(props: KanbanBoardProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();

    let mut search_query = props.search_query;
    let mut selected_priority = props.selected_priority;
    let filter_prio = selected_priority.read().clone();
    let mut selected_status = props.selected_status;
    let filter_status = selected_status.read().clone();
    let mut col_limits = props.col_limits;
    let mut hidden_views = props.hidden_views;
    let mut hide_empty_views = props.hide_empty_views;
    let mut is_add_view_open = props.is_add_view_open;

    let all_view_defs = props.all_view_defs;
    let grouped_columns = props.grouped_columns;
    let visible_view_defs = props.visible_view_defs;
    let empty_views_count = props.empty_views_count;
    let hidden_empty_names = props.hidden_empty_names;
    let all_targets = props.all_targets.clone();

    // Drag-and-drop reactive state
    let mut dragged_id = use_signal(|| Option::<String>::None);
    let mut drag_over_col = use_signal(|| Option::<String>::None);
    let mut drag_over_card = use_signal(|| Option::<String>::None);

    let mut clear_drag_state = move || {
        dragged_id.set(None);
        drag_over_col.set(None);
        drag_over_card.set(None);
    };

    let mut move_to_column_end = move |src_id: String, target_col: String| {
        let (sheet_kanban, sheet_status) = match target_col.as_str() {
            "Pending" => ("Pending", "Open"),
            "InProgress" => ("In Progress", "Open"),
            "Hold" | "Onhold" => ("Hold", "Open"),
            "Solved" => ("Solved", "Open"),
            "Completed" => ("Solved(closed)", "Closed"),
            "Cancelled" => ("Cancelled", "Closed"),
            _ => (target_col.as_str(), "Open"),
        };

        let mut list = state.tickets.write();
        if let Some(pos) = list.iter().position(|t| t.id == src_id) {
            let mut item = list.remove(pos);
            let num = item.ticket_id;
            let old_kanban = item.kanban_state.clone();
            item.status = sheet_status.to_string();
            item.kanban_state = target_col.clone();
            // stage is preserved
            list.push(item.clone());

            let _ = crate::db::update_ticket_status(num, &target_col);
            let mut payload = serde_json::json!({
                "kanban_state": sheet_kanban,
                "status": sheet_status,
            });
            if !item.stage.is_empty() {
                payload["stage"] = serde_json::json!(item.stage.clone());
            }
            crate::sync::enqueue_update_with_server_id(num, Some(src_id.clone()), payload);

            if old_kanban != target_col {
                let label = column_display_label(&target_col);
                toast.success("Status Updated", format!("{src_id} moved to {label}"));
            }
        }
    };

    let mut move_card_before = move |src_id: String, target_card_id: String, target_col: String| {
        if src_id == target_card_id {
            return;
        }
        let (sheet_kanban, sheet_status) = match target_col.as_str() {
            "Pending" => ("Pending", "Open"),
            "InProgress" => ("In Progress", "Open"),
            "Hold" | "Onhold" => ("Hold", "Open"),
            "Solved" => ("Solved", "Open"),
            "Completed" => ("Solved(closed)", "Closed"),
            "Cancelled" => ("Cancelled", "Closed"),
            _ => (target_col.as_str(), "Open"),
        };

        let mut list = state.tickets.write();
        if let Some(src_pos) = list.iter().position(|t| t.id == src_id) {
            let mut item = list.remove(src_pos);
            let num = item.ticket_id;
            let old_kanban = item.kanban_state.clone();
            item.status = sheet_status.to_string();
            item.kanban_state = target_col.clone();
            // stage is preserved
            if let Some(target_pos) = list.iter().position(|t| t.id == target_card_id) {
                list.insert(target_pos, item.clone());
            } else {
                list.push(item.clone());
            }

            let _ = crate::db::update_ticket_status(num, &target_col);
            let mut payload = serde_json::json!({
                "kanban_state": sheet_kanban,
                "status": sheet_status,
            });
            if !item.stage.is_empty() {
                payload["stage"] = serde_json::json!(item.stage.clone());
            }
            crate::sync::enqueue_update_with_server_id(num, Some(src_id.clone()), payload);

            if old_kanban != target_col {
                let label = column_display_label(&target_col);
                toast.success("Moved & Reordered", format!("{src_id} moved to {label}"));
            } else {
                toast.info("Ticket Reordered", format!("{src_id} repositioned"));
            }
        }
    };

    rsx! {
        div { class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden gap-3",

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

                div { class: "flex flex-wrap items-center gap-2",
                    div { class: "flex items-center gap-1.5",
                        span { class: "text-xs font-medium text-muted-foreground mr-1",
                            "Status:"
                        }
                        for st in ["All", "Open", "Closed"] {
                            {
                                let is_sel = filter_status == st;
                                let btn_variant = if is_sel { "default" } else { "outline" };
                                rsx! {
                                    Button {
                                        variant: "{btn_variant}",
                                        size: "sm",
                                        key: "st-{st}",
                                        class: "h-7 px-2.5 text-xs",
                                        onclick: move |_| selected_status.set(st.to_string()),
                                        "{st}"
                                    }
                                }
                            }
                        }
                    }

                    div { class: "h-4 w-px bg-border/60 mx-1 hidden sm:block" }

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

                    div { class: "h-4 w-px bg-border/60 mx-1 hidden sm:block" }

                    // Toggle button to hide/show empty views
                    Button {
                        variant: if *hide_empty_views.read() { "secondary" } else { "outline" },
                        size: "sm",
                        class: "h-7 px-2.5 text-xs gap-1.5 cursor-pointer font-medium border-border/80",
                        onclick: move |_| {
                            hide_empty_views.with_mut(|b| {
                                *b = !*b;
                                save_hide_empty_views(*b);
                            });
                        },
                        if *hide_empty_views.read() {
                            EyeOffIcon { class: "size-3.5 text-primary" }
                            span { "Hiding Empty" }
                            if *empty_views_count.read() > 0 {
                                Badge {
                                    variant: "secondary",
                                    class: "ml-0.5 px-1 py-0 text-[10px] font-mono bg-primary/15 text-primary border-0",
                                    "{empty_views_count}"
                                }
                            }
                        } else {
                            EyeIcon { class: "size-3.5 text-muted-foreground" }
                            span { "Hide Empty" }
                            if *empty_views_count.read() > 0 {
                                span { class: "text-[10px] text-muted-foreground/70 font-mono", "({empty_views_count})" }
                            }
                        }
                    }

                    // Views Visibility Dropdown Menu
                    DropdownMenu {
                        DropdownMenuTrigger {
                            class: "inline-flex items-center gap-1.5 h-7 px-2.5 text-xs font-medium rounded-md border border-border/80 bg-background hover:bg-muted/80 text-foreground cursor-pointer transition-colors shadow-2xs",
                            PanelRightIcon { class: "size-3.5 text-muted-foreground" }
                            span { "Views" }
                            span { class: "text-[10px] font-mono text-muted-foreground ml-0.5",
                                "{visible_view_defs.read().len()}/{all_view_defs.read().len()}"
                            }
                        }
                        DropdownMenuContent {
                            class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md p-1",
                            align: "end".to_string(),
                            DropdownMenuLabel {
                                div { class: "flex items-center justify-between text-xs font-semibold px-2 py-1",
                                    span { "Kanban Views" }
                                    span { class: "text-[10px] text-muted-foreground font-normal",
                                        "{visible_view_defs.read().len()} active"
                                    }
                                }
                            }
                            DropdownMenuSeparator {}
                            for def in all_view_defs.read().iter() {
                                {
                                    let is_hidden = hidden_views.read().contains(&def.id);
                                    let def_id = def.id.clone();
                                    let def_title = def.title.clone();
                                    let is_cust = def.is_custom;
                                    let title_cls = if is_hidden { "text-muted-foreground line-through" } else { "text-foreground" };
                                    let box_cls = if !is_hidden { "size-3.5 rounded bg-primary text-primary-foreground flex items-center justify-center shrink-0" } else { "size-3.5 rounded border border-muted-foreground/40 shrink-0" };
                                    rsx! {
                                        div {
                                            key: "view-toggle-{def_id}",
                                            class: "flex items-center justify-between px-2 py-1.5 text-xs rounded hover:bg-accent/40 cursor-pointer select-none transition-colors",
                                            onclick: move |_| {
                                                hidden_views.with_mut(|set| {
                                                    if set.contains(&def_id) {
                                                        set.remove(&def_id);
                                                    } else {
                                                        set.insert(def_id.clone());
                                                    }
                                                    save_hidden_views(set);
                                                });
                                            },
                                            div { class: "flex items-center gap-2 min-w-0",
                                                div {
                                                    class: "{box_cls}",
                                                    if !is_hidden {
                                                        CheckIcon { class: "size-2.5 stroke-[3]" }
                                                    }
                                                }
                                                span { class: "truncate font-medium {title_cls}",
                                                    "{def_title}"
                                                }
                                                if is_cust {
                                                    span { class: "text-[9px] px-1 py-0.2 rounded bg-primary/10 text-primary font-mono shrink-0", "custom" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            DropdownMenuSeparator {}
                            DropdownMenuItem {
                                onclick: move |_| is_add_view_open.set(true),
                                PlusIcon { class: "size-3.5 text-primary mr-1.5" }
                                "Add New View..."
                            }
                            if !hidden_views.read().is_empty() {
                                DropdownMenuItem {
                                    onclick: move |_| {
                                        hidden_views.with_mut(|set| {
                                            set.clear();
                                            save_hidden_views(set);
                                        });
                                    },
                                    EyeIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                                    "Show All Views"
                                }
                            }
                        }
                    }

                    // Add New View Button
                    Button {
                        variant: "outline",
                        size: "sm",
                        class: "h-7 px-2.5 text-xs gap-1.5 cursor-pointer font-medium border-dashed border-primary/50 text-primary hover:bg-primary/10",
                        onclick: move |_| is_add_view_open.set(true),
                        PlusIcon { class: "size-3.5" }
                        "Add View"
                    }
                }
            }

            // Notice banner when empty views are hidden
            if *hide_empty_views.read() && *empty_views_count.read() > 0 {
                div { class: "shrink-0 flex items-center justify-between px-3 py-1.5 rounded-md bg-muted/40 border border-border/50 text-xs text-muted-foreground",
                    div { class: "flex items-center gap-2",
                        EyeOffIcon { class: "size-3.5 text-primary shrink-0" }
                        span {
                            span { class: "font-semibold text-foreground", "{empty_views_count} empty view(s) hidden " }
                            span { class: "text-muted-foreground/80", "({hidden_empty_names}) to maximize your screen space." }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "cursor-pointer text-[11px] font-semibold text-primary hover:underline ml-2 shrink-0",
                        onclick: move |_| {
                            hide_empty_views.set(false);
                            save_hide_empty_views(false);
                        },
                        "Show All Columns"
                    }
                }
            }

            // Dynamic Kanban Columns
            if visible_view_defs.read().is_empty() {
                div { class: "flex flex-col items-center justify-center p-8 text-center rounded-lg border border-dashed border-border/80 bg-card/50 flex-1 my-auto max-w-md mx-auto",
                    EyeOffIcon { class: "size-10 text-muted-foreground/50 mb-3" }
                    h3 { class: "text-sm font-bold text-foreground", "All Kanban Views Hidden" }
                    p { class: "text-xs text-muted-foreground mt-1 mb-4",
                        "All views are currently hidden or have no tickets. Turn off empty views hiding or restore views from the Views menu."
                    }
                    div { class: "flex items-center gap-2",
                        Button {
                            variant: "outline",
                            size: "sm",
                            class: "h-7 px-3 text-xs",
                            onclick: move |_| {
                                hide_empty_views.set(false);
                                save_hide_empty_views(false);
                                hidden_views.with_mut(|s| { s.clear(); save_hidden_views(s); });
                            },
                            "Reset All Views"
                        }
                        Button {
                            variant: "default",
                            size: "sm",
                            class: "h-7 px-3 text-xs gap-1",
                            onclick: move |_| is_add_view_open.set(true),
                            PlusIcon { class: "size-3.5" }
                            "Add View"
                        }
                    }
                }
            } else {
                div { class: "{kanban_grid_layout_class(visible_view_defs.read().len())} w-full flex-1 min-h-0",
                    for col_def in visible_view_defs.read().clone() {
                        {
                            let column = col_def.id.clone();
                            let col_display = col_def.title.clone();
                            let col_sub = col_def.subtitle.clone();
                            let col_dot = col_def.dot_color.clone();
                            let is_custom = col_def.is_custom;

                            let grouped_guard = grouped_columns.read();
                            let column_tickets = grouped_guard.get(&column).cloned().unwrap_or_default();
                            let visible_limit = col_limits.read().get(&column).copied().unwrap_or(25);
                            let total_col_count = column_tickets.len();
                            let remaining_col_count = total_col_count.saturating_sub(visible_limit);
                            let is_col_over = drag_over_col.read().as_deref() == Some(&column);
                            let col_highlight = if is_col_over {
                                "ring-1 ring-primary border-primary bg-primary/5"
                            } else {
                                ""
                            };
                            let col_hide = column.clone();
                            let col_del_id = column.clone();
                            let col_del_name = col_display.clone();
                            let col_over1 = column.clone();
                            let col_over2 = column.clone();
                            let col_drop1 = column.clone();
                            let col_enter2 = column.clone();
                            let col_over3 = column.clone();
                            let col_drop2 = column.clone();
                            let flex_col_cls = if visible_view_defs.read().len() == 1 {
                                "w-full max-w-3xl flex-1 shrink-0"
                            } else {
                                "min-w-[275px] sm:min-w-[295px] flex-1 shrink-0"
                            };

                            rsx! {
                                ColumnContextMenu {
                                    column: column.clone(),
                                    title: col_display.clone(),
                                    is_custom,
                                    all_targets: all_targets.clone(),
                                    on_quick_add: {
                                        let h = props.on_quick_add;
                                        move |col| h.call(col)
                                    },
                                    on_filter_column: move |col| search_query.set(col),
                                    on_clear_filters: move |_| {
                                        search_query.set(String::new());
                                        selected_priority.set("All".to_string());
                                    },
                                    on_hide_column: move |col| {
                                        hidden_views.with_mut(|set| {
                                            set.insert(col);
                                            save_hidden_views(set);
                                        });
                                    },
                                    on_delete_column: {
                                        let h = props.on_delete_custom_view;
                                        move |_| h.call((col_del_id.clone(), col_del_name.clone()))
                                    },

                                    div {
                                        key: "board-col-{column}",
                                        class: "{flex_col_cls} flex flex-col h-full min-h-0 rounded-md border border-border/70 bg-card/75 p-3 transition-colors duration-150 {col_highlight}",
                                        ondragenter: move |e: DragEvent| {
                                            e.prevent_default();
                                            if drag_over_col.read().as_deref() != Some(&col_over1) {
                                                drag_over_col.set(Some(col_over1.clone()));
                                            }
                                        },
                                        ondragover: move |e: DragEvent| {
                                            e.prevent_default();
                                            if drag_over_col.read().as_deref() != Some(&col_over2) {
                                                drag_over_col.set(Some(col_over2.clone()));
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
                                                move_to_column_end(s, col_drop1.clone());
                                            }
                                            clear_drag_state();
                                        },

                                        div { class: "flex items-center justify-between px-1 border-b border-border/40 pb-2.5",
                                            div { class: "flex flex-col gap-0.5 min-w-0 pr-1",
                                                div { class: "flex items-center gap-2",
                                                    span { class: "size-2 rounded-full shrink-0 {col_dot}" }
                                                    h2 { class: "text-xs font-bold uppercase tracking-wider text-foreground truncate",
                                                        "{col_display}"
                                                    }
                                                    if is_custom {
                                                        span { class: "text-[9px] px-1 py-0.2 rounded bg-primary/10 text-primary font-mono uppercase font-semibold shrink-0", "custom" }
                                                    }
                                                }
                                                span { class: "text-[11px] text-muted-foreground truncate", "{col_sub}" }
                                            }
                                            div { class: "flex items-center gap-1.5 shrink-0",
                                                Badge {
                                                    variant: "outline",
                                                    class: "text-[10px] font-mono tabular-nums text-muted-foreground",
                                                    "{total_col_count}"
                                                }
                                                button {
                                                    r#type: "button",
                                                    title: "Hide '{col_display}' view",
                                                    class: "cursor-pointer rounded p-1 text-muted-foreground/60 hover:text-foreground hover:bg-muted/80 transition-colors",
                                                    onclick: {
                                                        let col_to_hide = col_hide.clone();
                                                        move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            hidden_views.with_mut(|set| {
                                                                set.insert(col_to_hide.clone());
                                                                save_hidden_views(set);
                                                            });
                                                        }
                                                    },
                                                    EyeOffIcon { class: "size-3" }
                                                }
                                                if is_custom {
                                                    button {
                                                        r#type: "button",
                                                        title: "Delete '{col_display}' custom view",
                                                        class: "cursor-pointer rounded p-1 text-muted-foreground/60 hover:text-destructive hover:bg-destructive/10 transition-colors",
                                                        onclick: {
                                                            let d_id = column.clone();
                                                            let d_name = col_display.clone();
                                                            let h = props.on_delete_custom_view;
                                                            move |e: MouseEvent| {
                                                                e.stop_propagation();
                                                                h.call((d_id.clone(), d_name.clone()));
                                                            }
                                                        },
                                                        TrashIcon { class: "size-3" }
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            class: "flex flex-col gap-2 min-h-0 flex-1 py-4 overflow-y-auto pr-1",
                                            ondragenter: move |e: DragEvent| {
                                                e.prevent_default();
                                                if drag_over_col.read().as_deref() != Some(&col_enter2) {
                                                    drag_over_col.set(Some(col_enter2.clone()));
                                                }
                                            },
                                            ondragover: move |e: DragEvent| {
                                                e.prevent_default();
                                                if drag_over_col.read().as_deref() != Some(&col_over3) {
                                                    drag_over_col.set(Some(col_over3.clone()));
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
                                                    move_to_column_end(s, col_drop2.clone());
                                                }
                                                clear_drag_state();
                                            },

                                            for ticket in column_tickets.iter().take(visible_limit) {
                                                {
                                                    let id = ticket.id.clone();
                                                    let title = ticket.title.clone();
                                                    let priority = ticket.priority.clone();
                                                    let category = ticket.category.clone();
                                                    let deadline = ticket.deadline.clone();
                                                    let disp_due = crate::state::format_display_12hr(&deadline);

                                                    let is_dragged = dragged_id.read().as_deref() == Some(&id);
                                                    let is_card_over = drag_over_card.read().as_deref() == Some(&id) && !is_dragged;

                                                    let col_stripe = match ticket.kanban_state.as_str() {
                                                        "Hold" | "Onhold" => "border-l-4 border-l-amber-500",
                                                        "Cancelled" => "border-l-4 border-l-rose-500",
                                                        "Completed" | "Solved" => "border-l-4 border-l-emerald-500",
                                                        "InProgress" => "border-l-4 border-l-sky-500",
                                                        "Pending" => "border-l-4 border-l-blue-500",
                                                        _ => "border-l-4 border-l-primary/60",
                                                    };

                                                    let card_cls = if is_dragged {
                                                        "opacity-40 scale-[0.99] border-dashed border-primary ring-1 ring-primary/40 bg-muted/40 cursor-grabbing rounded-md border p-3 flex flex-col gap-2 transition-all duration-200"
                                                    } else {
                                                        "group flex cursor-grab active:cursor-grabbing flex-col gap-2 rounded-md border border-border/70 bg-card p-3 transition-all duration-200 ease-out hover:border-primary/50 hover:bg-accent/15 hover:-translate-y-0.5 hover:shadow-md shadow-xs"
                                                    };

                                                    let id_for_start = id.clone();
                                                    let id_for_enter = id.clone();
                                                    let id_for_over = id.clone();
                                                    let id_for_drop = id.clone();
                                                    let col_for_card_enter = column.clone();
                                                    let col_for_card_over = column.clone();
                                                    let col_for_card_drop = column.clone();

                                                    rsx! {
                                                        div {
                                                            key: "board-card-wrap-{id}",
                                                            class: "flex flex-col gap-1.5 ",
                                                            ondragenter: move |e: DragEvent| {
                                                                e.prevent_default();
                                                                if drag_over_card.read().as_deref() != Some(&id_for_enter) {
                                                                    drag_over_card.set(Some(id_for_enter.clone()));
                                                                }
                                                                if drag_over_col.read().as_deref() != Some(&col_for_card_enter) {
                                                                    drag_over_col.set(Some(col_for_card_enter.clone()));
                                                                }
                                                            },
                                                            ondragover: move |e: DragEvent| {
                                                                e.prevent_default();
                                                                if drag_over_card.read().as_deref() != Some(&id_for_over) {
                                                                    drag_over_card.set(Some(id_for_over.clone()));
                                                                }
                                                                if drag_over_col.read().as_deref() != Some(&col_for_card_over) {
                                                                    drag_over_col.set(Some(col_for_card_over.clone()));
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
                                                                    move_card_before(s, id_for_drop.clone(), col_for_card_drop.clone());
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
                                                                    class: "select-none {card_cls} {col_stripe}",
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
                                                                            div { class: "flex items-center gap-1.5 shrink-0 min-w-0",
                                                                                GripVerticalIcon { class: "size-3.5 shrink-0 text-muted-foreground/40 group-hover:text-primary transition-colors cursor-grab" }
                                                                                span { class: "font-mono text-xs font-bold text-primary tracking-wide shrink-0 whitespace-nowrap group-hover:underline transition-colors",
                                                                                    "{id}"
                                                                                }
                                                                            }
                                                                            div { class: "flex items-center gap-1 shrink-0 ml-auto",
                                                                                Badge {
                                                                                    variant: "{status_badge_variant(&ticket.kanban_state)}",
                                                                                    class: "text-[10px] font-semibold uppercase tracking-wider shrink-0 {status_badge_class(&ticket.kanban_state)}",
                                                                                    "{column_display_label(&ticket.kanban_state)}"
                                                                                }
                                                                                div { class: "flex items-center gap-0.5 pointer-events-auto opacity-0 group-hover:opacity-100 transition-opacity ml-1 border-l border-border/40 pl-1",
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

                                                                        p { class: "text-sm font-semibold leading-snug text-foreground group-hover:text-primary transition-colors line-clamp-2",
                                                                            "{title}"
                                                                        }

                                                                        div { class: "flex flex-col gap-0.5 text-[11px] text-muted-foreground pt-0.5",
                                                                            div { class: "flex items-center gap-1 text-[11px] min-w-0",
                                                                                span { class: "text-muted-foreground/80 shrink-0", "Customer:" }
                                                                                span { class: "font-medium text-foreground truncate", "{ticket.emp_name()}" }
                                                                            }
                                                                            {
                                                                                let dept = ticket.department_display();
                                                                                let div_s = ticket.division_display();
                                                                                let show_div = !div_s.is_empty() && div_s != "N/A";
                                                                                rsx! {
                                                                                    if !dept.is_empty() || show_div {
                                                                                        div { class: "flex items-center gap-1 text-[10px] text-muted-foreground/80 truncate",
                                                                                            span { class: "shrink-0", "Department:" }
                                                                                            span { class: "truncate", "{dept}" }
                                                                                            if show_div {
                                                                                                span { class: "shrink-0", "|" }
                                                                                                span { class: "truncate", "Division: {div_s}" }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                            if !ticket.created_on.is_empty() {
                                                                                div { class: "flex items-center gap-1 text-[10px] text-muted-foreground/70",
                                                                                    span { class: "shrink-0", "Created:" }
                                                                                    span { class: "font-mono", "{crate::state::format_display_12hr(&ticket.created_on)}" }
                                                                                }
                                                                            }
                                                                            if !ticket.last_updated_on.is_empty() {
                                                                                div { class: "flex items-center gap-1 text-[10px] text-muted-foreground/70",
                                                                                    span { class: "shrink-0", "Updated:" }
                                                                                    span { class: "font-mono", "{crate::state::format_display_12hr(&ticket.last_updated_on)}" }
                                                                                }
                                                                            }
                                                                        }

                                                                        if !ticket.stage.is_empty() && ticket.stage != "Triage" && ticket.stage != ticket.kanban_state {
                                                                            div { class: "flex items-center gap-1",
                                                                                span {
                                                                                    class: "inline-flex items-center rounded bg-secondary/80 border border-border/60 px-1.5 py-0.5 text-[10px] font-mono font-medium text-foreground/80 truncate max-w-full",
                                                                                    title: "Action / Stage: {ticket.stage}",
                                                                                    "{ticket.stage}"
                                                                                }
                                                                            }
                                                                        }

                                                                        if !deadline.is_empty() {
                                                                            div { class: "flex items-center justify-between text-[10px] text-muted-foreground/80 pt-1 border-t border-border/30",
                                                                                span { class: "font-mono text-warning font-medium", title: "{deadline}", "Due: {disp_due}" }
                                                                            }
                                                                        }
                                                                    }

                                                                    div { class: "mt-1 flex items-center justify-between pt-2 border-t border-border/40 text-xs gap-2",
                                                                        div { class: "flex items-center gap-1.5 pointer-events-auto min-w-0 flex-wrap",
                                                                            span { class: "rounded bg-muted/80 px-2 py-0.5 text-[10px] font-medium text-foreground truncate max-w-[130px] border border-border/50",
                                                                                "{ticket.assignee_name()}"
                                                                            }
                                                                            if !category.is_empty() {
                                                                                span { class: "rounded bg-muted/50 px-1.5 py-0.5 text-[9px] font-medium text-muted-foreground truncate max-w-[100px]",
                                                                                    "{category}"
                                                                                }
                                                                            }
                                                                            if !ticket.rating_avg_text.is_empty() && ticket.rating_avg_text != "Unrated" {
                                                                                span {
                                                                                    class: "inline-flex items-center gap-0.5 text-[9px] font-semibold text-amber-500 bg-amber-500/10 border border-amber-500/20 rounded px-1.5 py-0.2 shrink-0",
                                                                                    "★ {ticket.rating_avg_text}"
                                                                                }
                                                                            }
                                                                            if !ticket.attachment_name.is_empty() || !ticket.attachment_url.is_empty() {
                                                                                span {
                                                                                    class: "inline-flex items-center text-muted-foreground hover:text-primary transition-colors cursor-pointer shrink-0",
                                                                                    title: if !ticket.attachment_name.is_empty() { "{ticket.attachment_name}" } else { "{ticket.attachment_url}" },
                                                                                    PaperclipIcon { class: "size-3 text-muted-foreground/80" }
                                                                                }
                                                                            }
                                                                        }
                                                                        Badge {
                                                                            variant: "{priority_badge(&priority)}",
                                                                            class: "text-[10px] font-semibold px-2 py-0.5 shrink-0 whitespace-nowrap shadow-2xs",
                                                                            "{priority}"
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            if remaining_col_count > 0 {
                                                div { class: "flex items-center justify-between gap-2 rounded-md border border-border/60 bg-muted/30 px-2.5 py-2 text-[11px]",
                                                    span { class: "text-muted-foreground font-medium",
                                                        "Showing {visible_limit} of {total_col_count}"
                                                    }
                                                    div { class: "flex items-center gap-1.5",
                                                        button {
                                                            r#type: "button",
                                                            class: "cursor-pointer rounded bg-primary/10 px-2 py-1 font-semibold text-primary hover:bg-primary/20 transition-colors",
                                                            onclick: {
                                                                let col_k = column.clone();
                                                                move |_| {
                                                                    col_limits.with_mut(|m| {
                                                                        let cur = m.get(&col_k).copied().unwrap_or(25);
                                                                        m.insert(col_k.clone(), cur + 25);
                                                                    });
                                                                }
                                                            },
                                                            "Load 25 more"
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "cursor-pointer rounded border border-border/70 px-2 py-1 font-medium text-muted-foreground hover:text-foreground hover:bg-background transition-colors",
                                                            onclick: {
                                                                let col_k = column.clone();
                                                                move |_| {
                                                                    col_limits.with_mut(|m| {
                                                                        m.insert(col_k.clone(), total_col_count);
                                                                    });
                                                                }
                                                            },
                                                            "Show all ({remaining_col_count})"
                                                        }
                                                    }
                                                }
                                            }

                                            div {
                                                class: if is_col_over { "flex items-center justify-center rounded-md border-2 border-dashed border-primary bg-primary/10 py-2.5 text-xs font-semibold text-primary transition-all animate-pulse" } else { "flex items-center justify-center rounded-md border border-dashed border-border/40 py-2 text-[11px] text-muted-foreground/50 transition-all hover:border-border hover:text-muted-foreground hover:bg-accent/20 cursor-pointer" },
                                                onclick: {
                                                    let col_c = column.clone();
                                                    let h = props.on_quick_add;
                                                    move |_| h.call(col_c.clone())
                                                },
                                                ondragenter: {
                                                    let col_b = column.clone();
                                                    move |e: DragEvent| {
                                                        e.prevent_default();
                                                        if drag_over_col.read().as_deref() != Some(&col_b) {
                                                            drag_over_col.set(Some(col_b.clone()));
                                                        }
                                                        drag_over_card.set(None);
                                                    }
                                                },
                                                ondragover: {
                                                    let col_b = column.clone();
                                                    move |e: DragEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        if drag_over_col.read().as_deref() != Some(&col_b) {
                                                            drag_over_col.set(Some(col_b.clone()));
                                                        }
                                                        drag_over_card.set(None);
                                                    }
                                                },
                                                ondrop: {
                                                    let col_b = column.clone();
                                                    move |e: DragEvent| {
                                                        e.prevent_default();
                                                        e.stop_propagation();
                                                        let src = e
                                                            .data()
                                                            .data_transfer()
                                                            .get_as_text()
                                                            .filter(|s| !s.is_empty())
                                                            .or_else(|| dragged_id.read().clone());
                                                        if let Some(src) = src {
                                                            move_to_column_end(src, col_b.clone());
                                                        }
                                                        clear_drag_state();
                                                    }
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
        }
    }
}
