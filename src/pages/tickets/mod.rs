use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::data_table::DataTable;
use crate::components::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
use crate::components::ui::toast::ToastState;
use crate::icons::{PlusIcon, RefreshIcon};

pub mod components;
pub mod dialogs;
pub mod kanban;
pub mod table_view;
pub mod types;

// Re-export items for backwards compatibility across the project
#[allow(unused_imports)]
pub use components::{AdvancedFilterBar, AdvancedFilterState, AttachmentPicker, InsightsPanel};
pub use dialogs::{AddCustomViewDialog, CreateTicketDialog, EditTicketDialog, ViewTicketDialog};
pub use kanban::KanbanBoard;
#[allow(unused_imports)]
pub use table_view::{
    ticket_columns, ticket_filter_value, ticket_priority_filter_value, ticket_row_id,
    ticket_search_text,
};
pub use types::{
    assignee_color_classes, get_assignee_initials, initial_tickets, load_custom_kanban_views,
    load_hide_empty_views, load_hidden_views, save_custom_kanban_views, save_hidden_views,
    CustomKanbanView, KanbanViewDef, TicketContextState, TicketItem,
};

#[component]
pub fn Tickets() -> Element {
    let mut tickets = use_signal(initial_tickets);
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();
    let app_settings = try_use_context::<Signal<crate::state::AppSettings>>();
    use_effect(move || {
        if let Some(sync) = db_sync {
            let _ = sync();
            let new_list = initial_tickets();
            if *tickets.peek() != new_list {
                tickets.set(new_list);
            }
        }
    });

    let search_query = use_signal(String::new);
    let open_sheet_changes = try_use_context::<Signal<crate::state::OpenSheetChangesDialog>>();
    let selected_priority = use_signal(|| "All".to_string());
    let selected_status = use_signal(|| "All".to_string());
    let col_limits = use_signal(std::collections::HashMap::<String, usize>::new);

    let mut custom_views = use_signal(load_custom_kanban_views);
    let mut hidden_views = use_signal(load_hidden_views);
    let hide_empty_views = use_signal(load_hide_empty_views);

    let mut is_add_view_open = use_signal(|| false);
    let mut new_view_title = use_signal(String::new);
    let mut new_view_subtitle = use_signal(String::new);
    let new_view_color = use_signal(|| "bg-sky-500".to_string());

    let all_view_defs = use_memo(move || {
        let mut defs = vec![
            KanbanViewDef {
                id: "Pending".to_string(),
                title: "Pending".to_string(),
                subtitle: "New & open".to_string(),
                dot_color: "bg-warning".to_string(),
                is_custom: false,
            },
            KanbanViewDef {
                id: "InProgress".to_string(),
                title: "In Progress".to_string(),
                subtitle: "Active support".to_string(),
                dot_color: "bg-sky-500 animate-pulse".to_string(),
                is_custom: false,
            },
            KanbanViewDef {
                id: "Hold".to_string(),
                title: "On Hold".to_string(),
                subtitle: "Waiting / paused".to_string(),
                dot_color: "bg-orange-500".to_string(),
                is_custom: false,
            },
            KanbanViewDef {
                id: "Solved".to_string(),
                title: "Solved".to_string(),
                subtitle: "Solved, awaiting close".to_string(),
                dot_color: "bg-purple-500".to_string(),
                is_custom: false,
            },
            KanbanViewDef {
                id: "Completed".to_string(),
                title: "Solved (Closed)".to_string(),
                subtitle: "Resolved & closed".to_string(),
                dot_color: "bg-success".to_string(),
                is_custom: false,
            },
            KanbanViewDef {
                id: "Cancelled".to_string(),
                title: "Cancelled".to_string(),
                subtitle: "Cancelled / duplicate".to_string(),
                dot_color: "bg-slate-400".to_string(),
                is_custom: false,
            },
        ];
        for cv in custom_views.read().iter() {
            defs.push(KanbanViewDef {
                id: cv.id.clone(),
                title: cv.title.clone(),
                subtitle: cv.subtitle.clone(),
                dot_color: cv.dot_color.clone(),
                is_custom: true,
            });
        }
        defs
    });

    let grouped_columns = use_memo(move || {
        let q = search_query.read().trim().to_lowercase();
        let filter_prio = selected_priority.read().clone();
        let filter_stat = selected_status.read().clone();
        let defs = all_view_defs.read().clone();
        let mut map: std::collections::HashMap<String, Vec<TicketItem>> = std::collections::HashMap::new();
        for def in &defs {
            map.insert(def.id.clone(), Vec::new());
        }

        for t in tickets.read().iter() {
            if filter_prio != "All" && t.priority != filter_prio {
                continue;
            }
            if filter_stat != "All" && !t.status.eq_ignore_ascii_case(&filter_stat) {
                continue;
            }
            if !q.is_empty()
                && !t.id.to_lowercase().contains(&q)
                && !t.title.to_lowercase().contains(&q)
                && !t.category.to_lowercase().contains(&q)
                && !t.assignee.to_lowercase().contains(&q)
                && !t.my_activity.to_lowercase().contains(&q)
                && !t.activities.to_lowercase().contains(&q)
                && !t.stage.to_lowercase().contains(&q)
                && !crate::db::matches_emp_query(&t.employee_code, &t.emp_name(), &t.email_address, &q)
            {
                continue;
            }
            let st = if !t.kanban_state.is_empty() {
                t.kanban_state.as_str()
            } else if !t.stage.is_empty() {
                t.stage.as_str()
            } else {
                t.status.as_str()
            };

            let matched_custom = defs.iter().find(|d| d.is_custom && (d.id.eq_ignore_ascii_case(st) || d.title.eq_ignore_ascii_case(st)));
            if let Some(c) = matched_custom {
                if let Some(list) = map.get_mut(&c.id) {
                    list.push(t.clone());
                    continue;
                }
            }

            let key = match crate::sync::from_sheet_status(st) {
                "InProgress" => "InProgress",
                "Hold" => "Hold",
                "Solved" => "Solved",
                "Completed" => "Completed",
                "Cancelled" => "Cancelled",
                _ => "Pending",
            };
            if let Some(list) = map.get_mut(key) {
                list.push(t.clone());
            }
        }

        // Sort each column in descending order (highest/latest ticket first)
        for list in map.values_mut() {
            list.sort_by(|a, b| {
                let num_a = a.ticket_number();
                let num_b = b.ticket_number();
                num_b.cmp(&num_a).then_with(|| b.created_on.cmp(&a.created_on))
            });
        }

        map
    });

    let visible_view_defs = use_memo(move || {
        let defs = all_view_defs.read().clone();
        let hide_empty = *hide_empty_views.read();
        let hidden = hidden_views.read().clone();
        let groups = grouped_columns.read();
        let (hide_cancelled, hide_solved) = if let Some(s) = app_settings {
            let r = s.read();
            (r.kanban_hide_cancelled, r.kanban_hide_solved)
        } else {
            (false, false)
        };

        defs.into_iter()
            .filter(|def| {
                if hidden.contains(&def.id) {
                    return false;
                }
                if hide_cancelled && def.id == "Cancelled" {
                    return false;
                }
                if hide_solved && (def.id == "Completed" || def.id == "Solved") {
                    return false;
                }
                if hide_empty {
                    let count = groups.get(&def.id).map(|v| v.len()).unwrap_or(0);
                    if count == 0 {
                        return false;
                    }
                }
                true
            })
            .collect::<Vec<KanbanViewDef>>()
    });

    let empty_views_count = use_memo(move || {
        let defs = all_view_defs.read().clone();
        let hidden = hidden_views.read().clone();
        let groups = grouped_columns.read();

        defs.iter()
            .filter(|def| {
                if hidden.contains(&def.id) {
                    return false;
                }
                groups.get(&def.id).map(|v| v.len()).unwrap_or(0) == 0
            })
            .count()
    });

    let hidden_empty_names = use_memo(move || {
        let defs = all_view_defs.read().clone();
        let hidden = hidden_views.read().clone();
        let groups = grouped_columns.read();

        let mut names = Vec::new();
        for def in &defs {
            if !hidden.contains(&def.id) && groups.get(&def.id).map(|v| v.len()).unwrap_or(0) == 0 {
                names.push(def.title.clone());
            }
        }
        names.join(", ")
    });

    // View & Edit Ticket Modal signals
    let viewing_ticket = use_signal(|| Option::<TicketItem>::None);
    let editing_ticket = use_signal(|| Option::<TicketItem>::None);
    let is_view_open = use_signal(|| false);
    let is_edit_open = use_signal(|| false);

    let edit_emp_name = use_signal(String::new);
    let edit_employee_code = use_signal(String::new);
    let edit_email_address = use_signal(String::new);
    let edit_category = use_signal(|| "General".to_string());
    let edit_subject = use_signal(String::new);
    let edit_problem_description = use_signal(String::new);
    let edit_attachment_name = use_signal(String::new);
    let edit_attachment_url = use_signal(String::new);
    let edit_priority = use_signal(|| "Medium".to_string());
    let edit_assigned_to = use_signal(|| "Shreyansh".to_string());
    let edit_customer = use_signal(|| "Internal".to_string());
    let edit_time_spent = use_signal(|| 0.0f64);
    let edit_activities = use_signal(String::new);
    let edit_my_activity = use_signal(String::new);
    let edit_deadline = use_signal(String::new);
    let edit_sla_status = use_signal(|| "Within SLA".to_string());
    let edit_sla_deadline = use_signal(String::new);
    let edit_rating_avg_text = use_signal(String::new);
    let edit_kanban_state = use_signal(|| "Pending".to_string());
    let edit_stage = use_signal(|| "Pending".to_string());
    let edit_status = use_signal(|| "Pending".to_string());

    // Advanced Filter for IT Data Table
    let advanced_filter = AdvancedFilterState::new();

    let table_categories = use_memo(move || {
        let mut set = std::collections::BTreeSet::new();
        for t in tickets.read().iter() {
            let c = t.category.trim();
            if !c.is_empty() {
                set.insert(c.to_string());
            }
        }
        set.into_iter().collect::<Vec<_>>()
    });

    let table_assignees = use_memo(move || {
        let mut set = std::collections::BTreeSet::new();
        for t in tickets.read().iter() {
            let a = t.assigned_to.trim();
            if !a.is_empty() && !a.eq_ignore_ascii_case("unassigned") {
                set.insert(a.to_string());
            }
        }
        set.into_iter().collect::<Vec<_>>()
    });

    let table_stages = use_memo(move || {
        let mut set = std::collections::BTreeSet::new();
        for t in tickets.read().iter() {
            let s = t.stage.trim();
            if !s.is_empty() && s != "Triage" {
                set.insert(s.to_string());
            }
        }
        set.into_iter().collect::<Vec<_>>()
    });

    let filtered_table_tickets = use_memo({
        let filter = advanced_filter.clone();
        move || {
            let list = tickets.read().clone();
            let mut res = list.into_iter()
                .filter(|t| filter.matches(t))
                .collect::<Vec<_>>();
            res.sort_by(|a, b| {
                let num_a = a.ticket_number();
                let num_b = b.ticket_number();
                num_b.cmp(&num_a).then_with(|| b.created_on.cmp(&a.created_on))
            });
            res
        }
    });

    // Fetch latest tickets and employee directory in real-time on mount
    use_effect(move || {
        spawn(async move {
            let _ = crate::sync::pull_tickets_now().await;
            let _ = crate::sync::pull_employees_now().await;
        });
    });

    let is_create_open = use_signal(|| false);
    let create_emp_name = use_signal(String::new);
    let create_employee_code = use_signal(String::new);
    let create_email_address = use_signal(String::new);
    let create_category = use_signal(|| "Select category".to_string());
    let create_subject = use_signal(String::new);
    let create_problem_description = use_signal(String::new);
    let create_attachment_name = use_signal(String::new);
    let create_attachment_url = use_signal(String::new);
    let create_priority = use_signal(|| "Low".to_string());
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

    // Quick add ticket to column
    let quick_add_ticket = move |col: String| {
        let count = tickets.read().len() + 1;
        let fallback_id = format!("IT-{}", 200 + count);
        let col_disp = types::column_display_label(&col);
        let title = format!("New request ({col_disp})");
        let employee = crate::db::configured_employee_code();
        let email = crate::db::employee_email(&employee);
        let customer = crate::db::employee_name(&employee);
        let category = "General".to_string();
        let priority = "Medium".to_string();
        let assignee = "GR".to_string();
        let created_at = crate::state::format_local_now();

        if employee.is_empty() {
            toast.error(
                "No employee configured",
                "Configure your Default Employee Code in Settings before quick-adding tickets.",
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
        if let Ok(id) = db_id {
            crate::sync::enqueue_create(id);
        }
        let numeric_id = db_id.unwrap_or(200 + count as i64);

        tickets.write().insert(0, TicketItem::new(
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

    let mut handle_create_view = move |_| {
        let title = new_view_title.read().trim().to_string();
        if title.is_empty() {
            toast.error("Validation Error", "View title cannot be empty.");
            return;
        }
        let id = title.replace(' ', "_");
        let exists = all_view_defs
            .read()
            .iter()
            .any(|d| d.id.eq_ignore_ascii_case(&id) || d.title.eq_ignore_ascii_case(&title));
        if exists {
            toast.error("Duplicate View", "A view with this name already exists.");
            return;
        }
        let subtitle = new_view_subtitle.read().trim().to_string();
        let dot_color = new_view_color.read().clone();
        let cv = CustomKanbanView {
            id: id.clone(),
            title: title.clone(),
            subtitle: if subtitle.is_empty() {
                format!("Custom workflow: {title}")
            } else {
                subtitle
            },
            dot_color,
        };
        custom_views.with_mut(|views| {
            views.push(cv);
            save_custom_kanban_views(views);
        });
        new_view_title.set(String::new());
        new_view_subtitle.set(String::new());
        is_add_view_open.set(false);
        toast.success("View Created", format!("Added '{title}' view to Kanban board."));
    };

    let mut handle_delete_custom_view = move |del_id: String, del_name: String| {
        let mut list = tickets.write();
        let mut moved_count = 0;
        for t in list.iter_mut() {
            if t.kanban_state == del_id || t.stage == del_id {
                t.kanban_state = "Pending".to_string();
                t.stage = "Pending".to_string();
                t.status = "Pending".to_string();
                let num = t.ticket_id;
                let _ = crate::db::update_ticket_status(num, "Pending");
                crate::sync::enqueue_update(
                    num,
                    serde_json::json!({
                        "kanban_state": "Pending",
                        "stage": "Pending",
                        "status": "Pending",
                    }),
                );
                moved_count += 1;
            }
        }
        drop(list);

        custom_views.with_mut(|views| {
            views.retain(|v| v.id != del_id);
            save_custom_kanban_views(views);
        });

        hidden_views.with_mut(|set| {
            set.remove(&del_id);
            save_hidden_views(set);
        });

        if moved_count > 0 {
            toast.success(
                "View Deleted",
                format!("Deleted '{del_name}' view. {moved_count} ticket(s) moved to Pending."),
            );
        } else {
            toast.success("View Deleted", format!("Deleted '{del_name}' view."));
        }
    };

    let all_targets: Vec<(String, String)> = all_view_defs
        .read()
        .iter()
        .map(|d| (d.id.clone(), d.title.clone()))
        .collect();

    rsx! {
        div { class: "flex flex-col h-[calc(100vh-4rem)] max-h-[calc(100vh-4rem)] w-full gap-3 p-3 overflow-hidden",
            // Page Header
            div { class: "flex items-center justify-between gap-3 border-b border-border/40 shrink-0",
                div { class: "flex items-center gap-3 min-w-0 flex-1",
                    div { class: "flex flex-col min-w-0",
                        h1 { class: "text-lg font-bold tracking-tight text-foreground truncate", "Support Tickets" }
                        p { class: "text-xs text-muted-foreground truncate",
                            "{tickets.read().len()} total tickets across all stages"
                        }
                    }
                }
                div { class: "flex items-center gap-2 shrink-0",
                    Button {
                        variant: "outline",
                        size: "sm",
                        class: "h-8 px-3 text-xs gap-1.5 font-medium cursor-pointer shadow-2xs shrink-0",
                        title: "Fetch the latest tickets from Google Sheets now",
                        onclick: move |_| {
                            spawn(async move {
                                toast.info("Syncing", "Pulling latest tickets from Google Sheets…");
                                match crate::sync::pull_tickets_now().await {
                                    Ok(n) => {
                                        if n > 0 {
                                            toast.success("Sync Complete", format!("Fetched latest changes from Google Sheet ({n} updated)"));
                                        } else {
                                            toast.info("Up to Date", "All tickets and values are up to date with Google Sheet.");
                                        }
                                    }
                                    Err(e) => {
                                        toast.warning("Sync Status", format!("{e}"));
                                    }
                                }
                            });
                        },
                        RefreshIcon { class: "size-3.5 mr-0.5" }
                        "Refresh"
                    }
                    Button {
                        variant: "outline",
                        size: "sm",
                        class: "h-8 px-3 text-xs gap-1.5 font-medium cursor-pointer shadow-2xs shrink-0 text-foreground",
                        title: "View recent changes and live values captured during Google Sheet sync",
                        onclick: move |_| {
                            if let Some(mut sig) = open_sheet_changes {
                                sig.set(crate::state::OpenSheetChangesDialog(true));
                            }
                        },
                        RefreshIcon { class: "size-3.5 mr-0.5 text-primary" }
                        "Sheet Changes"
                    }
                    Button {
                        size: "sm",
                        class: "h-8 px-3 text-xs gap-1.5 font-medium cursor-pointer shadow-sm shrink-0",
                        onclick: move |_| state.open_create(None),
                        PlusIcon { class: "size-3.5" }
                        "New Ticket"
                    }
                }
            }

            // Tabs for Kanban vs IT Data Table
            Tabs {
                value: app_settings.map(|s| s.read().default_ticket_view.clone()).unwrap_or_else(|| "kanban".to_string()),
                class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden",
                div { class: "flex items-center justify-between pb-2 shrink-0 border-b border-border/30",
                    TabsList {
                        TabsTrigger { value: "kanban".to_string(), "Kanban Board" }
                        TabsTrigger { value: "table".to_string(), "IT Data Table" }
                    }
                }

                // Kanban View
                TabsContent {
                    value: "kanban".to_string(),
                    class: "flex flex-col flex-1 min-h-0 overflow-hidden pt-2",
                    KanbanBoard {
                        search_query,
                        selected_priority,
                        selected_status,
                        col_limits,
                        hidden_views,
                        hide_empty_views,
                        is_add_view_open,
                        all_view_defs,
                        grouped_columns,
                        visible_view_defs,
                        empty_views_count,
                        hidden_empty_names,
                        all_targets: all_targets.clone(),
                        on_delete_custom_view: move |(del_id, del_name)| handle_delete_custom_view(del_id, del_name),
                        on_quick_add: quick_add_ticket,
                    }
                }

                // IT Data Table View
                TabsContent {
                    value: "table".to_string(),
                    class: "flex flex-col flex-1 min-h-0 w-full overflow-hidden pt-2 gap-2.5",
                    AdvancedFilterBar {
                        filter: advanced_filter.clone(),
                        total_count: tickets.read().len(),
                        filtered_count: filtered_table_tickets.read().len(),
                        categories: table_categories.read().clone(),
                        assignees: table_assignees.read().clone(),
                        stages: table_stages.read().clone(),
                    }
                    DataTable {
                        data: filtered_table_tickets.read().clone(),
                        columns: ticket_columns(),
                        row_id: ticket_row_id,
                        search_text: ticket_search_text,
                        initial_sort: Some(("id".to_string(), false)),
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

            // Dialog Modals
            ViewTicketDialog {
                open: is_view_open,
                ticket: viewing_ticket,
            }

            CreateTicketDialog {
                open: is_create_open,
            }

            EditTicketDialog {
                open: is_edit_open,
            }

            AddCustomViewDialog {
                open: is_add_view_open,
                title: new_view_title,
                subtitle: new_view_subtitle,
                color: new_view_color,
                on_create: move |_| handle_create_view(()),
            }

            // Table context menu
            if table_ctx_open() {
                if let Some(t) = table_ctx_ticket() {
                    components::TableFloatingContextMenu {
                        ticket: t,
                        open: table_ctx_open,
                        x: table_ctx_x,
                        y: table_ctx_y,
                        on_view: move |item: TicketItem| state.open_view(item),
                        on_edit: move |item: TicketItem| state.open_edit(item),
                    }
                }
            }
        }
    }
}
