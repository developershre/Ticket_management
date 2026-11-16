use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;

use super::super::components::{AttachmentPicker, MultiActivitySelector};
use super::super::types::TicketContextState;

#[derive(Props, Clone, PartialEq)]
pub struct EditTicketDialogProps {
    pub open: Signal<bool>,
}

#[component]
pub fn EditTicketDialog(mut props: EditTicketDialogProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();

    let directory_employees = crate::db::get_all_employees();

    let mut edit_subject = state.edit_subject;
    let mut edit_employee_code = state.edit_employee_code;
    let mut edit_emp_name = state.edit_emp_name;
    let mut edit_email_address = state.edit_email_address;
    let mut edit_category = state.edit_category;
    let mut edit_priority = state.edit_priority;
    let mut edit_assigned_to = state.edit_assigned_to;
    let mut edit_kanban_state = state.edit_kanban_state;
    let mut edit_stage = state.edit_stage;
    let mut edit_status = state.edit_status;
    let mut edit_problem_description = state.edit_problem_description;
    let edit_attachment_name = state.edit_attachment_name;
    let edit_attachment_url = state.edit_attachment_url;
    let edit_my_activity = state.edit_my_activity;
    let edit_customer = state.edit_customer;
    let edit_time_spent = state.edit_time_spent;
    let edit_activities = state.edit_activities;
    let edit_deadline = state.edit_deadline;
    let edit_sla_status = state.edit_sla_status;
    let edit_sla_deadline = state.edit_sla_deadline;
    let edit_rating_avg_text = state.edit_rating_avg_text;

    let mut save_edit = move || {
        let current_ticket = (state.editing_ticket)();
        if let Some(mut current) = current_ticket {
            let subj_val = edit_subject.read().trim().to_string();
            if subj_val.is_empty() {
                toast.error("Validation Error", "Ticket subject cannot be empty");
                return;
            }

            let code_val = edit_employee_code.read().trim().to_string();
            let (name_val, email_val) = if let Some(emp) = crate::db::find_employee_by_code(&code_val) {
                let email = if !emp.emp_email.is_empty() {
                    emp.emp_email
                } else {
                    edit_email_address.read().trim().to_string()
                };
                (emp.emp_name, email)
            } else {
                (
                    edit_emp_name.read().trim().to_string(),
                    edit_email_address.read().trim().to_string(),
                )
            };

            let emp_combined = if name_val.is_empty() {
                code_val.clone()
            } else {
                format!("{} ({})", code_val, name_val)
            };

            let cat_val = edit_category.read().trim().to_string();
            let desc_val = edit_problem_description.read().trim().to_string();
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

            let kanban_val = edit_kanban_state.read().clone();
            let stage_val = kanban_val.clone();
            let status_val = kanban_val.clone();

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
                if current.kanban_state != kanban_val || current.stage != stage_val || current.status != status_val {
                    let sheet_kanban = match kanban_val.as_str() {
                        "InProgress" => "In Progress",
                        "Hold" => "Hold",
                        "Solved" => "Solved",
                        "Completed" => "Solved(closed)",
                        "Cancelled" => "Cancelled",
                        other => other,
                    };
                    api_fields.insert(
                        "kanban_state".to_string(),
                        serde_json::Value::String(sheet_kanban.to_string()),
                    );
                    if !stage_val.is_empty() {
                        api_fields.insert(
                            "stage".to_string(),
                            serde_json::Value::String(stage_val.clone()),
                        );
                    }
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
            current.status = status_val.clone();
            current.stage = stage_val.clone();
            current.last_updated_on = "Just now".to_string();

            let mut list = state.tickets.write();
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

            if !api_fields.is_empty() {
                crate::sync::enqueue_update(num, serde_json::Value::Object(api_fields));
            }

            toast.success(
                "Ticket Modified",
                format!("{} modified successfully", current.id),
            );
            props.open.set(false);
            state.editing_ticket.set(None);
        }
    };

    rsx! {
        Dialog { open: props.open,
            DialogContent { class: "max-w-4xl sm:max-w-4xl w-full rounded-md border border-border/80 bg-card p-6 sm:p-7 shadow-2xl max-h-[88vh] overflow-y-auto",
                if let Some(ticket) = (state.editing_ticket)() {
                    {
                        let t_id = ticket.id.clone();
                        rsx! {
                            DialogHeader {
                                div { class: "flex items-center gap-2",
                                    span { class: "font-mono text-xs font-semibold text-foreground bg-muted border border-border/70 px-2.5 py-0.5 rounded",
                                        "{t_id}"
                                    }
                                    DialogTitle { class: "text-lg font-bold text-foreground", "Modify Ticket" }
                                }
                                DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                    "Update ticket details, requester information, assignment, and attachments."
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
                                            "{directory_employees.len()} employees loaded"
                                        }
                                    }
                                    crate::components::employee_search::EmployeeSearchInput {
                                        employee_code: edit_employee_code,
                                        employee_name: edit_emp_name,
                                        email_address: edit_email_address,
                                        placeholder: "Type code (e.g. 11061) or name (e.g. Chaitram)...".to_string(),
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
                                                oninput: move |e: FormEvent| {
                                                    let val = e.value();
                                                    edit_employee_code.set(val.clone());
                                                    let trimmed = val.trim();
                                                    if !trimmed.is_empty() {
                                                        if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                                                            edit_emp_name.set(emp.emp_name);
                                                            edit_email_address.set(emp.emp_email);
                                                        }
                                                    } else {
                                                        edit_emp_name.set(String::new());
                                                        edit_email_address.set(String::new());
                                                    }
                                                },
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

                                div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
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
                                        label { class: "text-[11px] font-semibold text-foreground", "Kanban Column" }
                                        select {
                                            class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                            value: "{edit_kanban_state}",
                                            onchange: move |e: FormEvent| {
                                                let val = e.value();
                                                edit_kanban_state.set(val.clone());
                                                let st = if val == "Completed" || val == "Cancelled" { "Closed" } else { "Open" };
                                                edit_status.set(st.to_string());
                                            },
                                            option { value: "Pending", "Pending" }
                                            option { value: "InProgress", "In Progress" }
                                            option { value: "Hold", "On Hold" }
                                            option { value: "Solved", "Solved" }
                                            option { value: "Completed", "Solved (Closed)" }
                                            option { value: "Cancelled", "Cancelled" }
                                        }
                                    }
                                }

                                div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                                    div { class: "flex flex-col gap-1",
                                        label { class: "text-[11px] font-semibold text-foreground", "Action / Stage Tag" }
                                        input {
                                            class: "h-9 rounded-md border border-border/80 bg-card px-2.5 text-xs text-foreground focus:border-primary focus:outline-none font-mono",
                                            placeholder: "e.g. WB,SALE ORDER,AMD, Backend...",
                                            value: "{edit_stage}",
                                            oninput: move |e: FormEvent| edit_stage.set(e.value()),
                                        }
                                    }
                                    div { class: "flex flex-col gap-1",
                                        label { class: "text-[11px] font-semibold text-foreground", "Status (Lifecycle)" }
                                        select {
                                            class: "h-9 rounded-md border border-border/80 bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                            value: "{edit_status}",
                                            onchange: move |e: FormEvent| edit_status.set(e.value()),
                                            option { value: "Open", "Open" }
                                            option { value: "Closed", "Closed" }
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

                                AttachmentPicker {
                                    attachment_name: edit_attachment_name,
                                    attachment_url: edit_attachment_url,
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
                                        props.open.set(false);
                                        state.editing_ticket.set(None);
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
    }
}
