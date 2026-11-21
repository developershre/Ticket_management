use dioxus::prelude::*;

use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::select::SimpleSelect;
use crate::components::ui::toast::ToastState;

use super::super::components::AttachmentPicker;
use super::super::types::{TicketContextState, TicketItem};

#[derive(Props, Clone, PartialEq)]
pub struct CreateTicketDialogProps {
    pub open: Signal<bool>,
}

#[component]
pub fn CreateTicketDialog(mut props: CreateTicketDialogProps) -> Element {
    let mut state = use_context::<TicketContextState>();
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();

    let directory_employees = crate::db::get_all_employees();

    let mut create_emp_name = state.create_emp_name;
    let mut create_employee_code = state.create_employee_code;
    let mut create_email_address = state.create_email_address;
    let create_category = state.create_category;
    let create_priority = state.create_priority;
    let mut create_subject = state.create_subject;
    let mut create_problem_description = state.create_problem_description;
    let create_attachment_name = state.create_attachment_name;
    let create_attachment_url = state.create_attachment_url;
    let create_kanban_state = state.create_kanban_state;

    let mut is_submitting = use_signal(|| false);

    let is_sheet_configured = {
        let (url, _, _) = crate::db::get_system_api_config();
        url.map(|u| !u.trim().is_empty()).unwrap_or(false)
            || std::env::var("TMS_API_BASE_URL").map(|u| !u.trim().is_empty()).unwrap_or(false)
    };

    let mut save_create = move || {
        if *is_submitting.read() {
            return;
        }

        let emp_code = create_employee_code.read().trim().to_string();
        if emp_code.is_empty() {
            toast.error("Validation Error", "Please enter Employee Code");
            return;
        }

        let (name_val, email_val) = if let Some(emp) = crate::db::find_employee_by_code(&emp_code) {
            let email = if !emp.emp_email.is_empty() {
                emp.emp_email
            } else {
                create_email_address.read().trim().to_string()
            };
            (emp.emp_name, email)
        } else {
            (
                create_emp_name.read().trim().to_string(),
                create_email_address.read().trim().to_string(),
            )
        };

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

        let client = match crate::sync::get_client().or_else(|_| crate::sync::reload_client()) {
            Ok(c) => c,
            Err(_) => {
                toast.error(
                    "Google Sheet Disconnected",
                    "Tickets cannot be created offline. Please connect your Google Sheet Web App URL in Settings > API & Connection before creating tickets.",
                );
                return;
            }
        };

        let emp_combined = if name_val.is_empty() {
            emp_code.clone()
        } else {
            format!("{} ({})", emp_code, name_val)
        };

        let att_name_val = create_attachment_name.read().clone();
        let att_url_val = create_attachment_url.read().clone();
        let prio_val = create_priority.read().clone();
        let assigned_val = "IT Support".to_string();
        let cust_val = "Internal".to_string();
        let time_val = 0.0f64;
        let act_val = "Ticket created".to_string();
        let my_act_val = "Initial triage & review".to_string();

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
        let stage_raw = state.create_stage.read().clone();
        let stage_val = if stage_raw.trim().is_empty() {
            "Triage".to_string()
        } else {
            stage_raw.trim().to_string()
        };
        let status_val = if kanban_val == "Completed" || kanban_val == "Cancelled" {
            "Closed".to_string()
        } else {
            "Open".to_string()
        };

        let payload = serde_json::json!({
            "employee_code": emp_combined,
            "email_address": email_val,
            "category": cat_val,
            "subject": subj_val,
            "problem_description": desc_val,
            "attachment_name": att_name_val,
            "attachment_url": att_url_val,
            "priority": prio_val,
            "assigned_to": assigned_val,
            "customer": cust_val,
            "time_spent": time_val,
            "activities": act_val,
            "my_activity": my_act_val,
            "deadline": deadline_val,
            "sla_status": sla_stat_val,
            "sla_deadline": sla_dead_val,
            "rating_avg_text": rating_val,
            "kanban_state": kanban_val,
            "stage": stage_val,
            "status": status_val,
        });

        let sanitized = crate::sync::sanitize_ticket_payload(&client, &payload, true);

        is_submitting.set(true);

        spawn(async move {
            match client.tickets_create(&sanitized).await {
                Ok(server_ticket) => {
                    let local_id = crate::db::insert_server_ticket(&server_ticket).unwrap_or(0);
                    let new_ticket = TicketItem::from_server_ticket(&server_ticket, local_id);
                    state.tickets.write().insert(0, new_ticket);

                    if let Some(mut sync) = db_sync {
                        sync.with_mut(|r| r.0 += 1);
                    }

                    crate::notifications::notify_new_ticket(
                        &server_ticket.ticket_id,
                        &server_ticket.subject,
                        &server_ticket.assigned_to,
                        &server_ticket.priority,
                    );

                    toast.success(
                        "Ticket Created in Google Sheet",
                        format!("Ticket {} created and registered directly in Google Sheet.", server_ticket.ticket_id),
                    );

                    create_employee_code.set(String::new());
                    create_emp_name.set(String::new());
                    create_email_address.set(String::new());
                    create_subject.set(String::new());
                    create_problem_description.set(String::new());

                    is_submitting.set(false);
                    props.open.set(false);
                }
                Err(err) => {
                    is_submitting.set(false);
                    toast.error(
                        "Google Sheet Creation Failed",
                        format!(
                            "Could not create ticket in Google Sheet: {err}. Offline ticket creation is disabled. Please verify your connection.",
                        ),
                    );
                }
            }
        });
    };

    rsx! {
        Dialog { open: props.open,
            DialogContent { class: "max-w-2xl sm:max-w-2xl w-full rounded-xl border border-border bg-card p-6 sm:p-8 shadow-2xl max-h-[92vh] overflow-y-auto",
                DialogHeader {
                    class: "space-y-1 pb-3 text-left",
                    DialogTitle { class: "text-2xl font-bold tracking-tight text-foreground", "Create Support Ticket" }
                    DialogDescription { class: "text-sm text-muted-foreground",
                        "Enter ticket details. All tickets are validated and created directly in the connected Google Sheet."
                    }
                }

                if !is_sheet_configured {
                    div { class: "mb-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20 text-amber-500 text-xs flex items-center gap-2",
                        span { class: "font-semibold", "Google Sheet Connection Required:" }
                        span { "Offline ticket creation is disabled. Please connect your Google Sheet in Settings before submitting tickets." }
                    }
                } else {
                    div { class: "mb-2 px-3 py-1.5 rounded-md bg-emerald-500/10 border border-emerald-500/20 text-emerald-500 text-xs flex items-center gap-2",
                        span { class: "inline-block w-2 h-2 rounded-full bg-emerald-500 animate-pulse" }
                        span { class: "font-medium", "Google Sheet Connected · Direct Online Upload Active" }
                    }
                }

                div { class: "flex flex-col gap-4 py-2 text-sm",
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                            span { "Select Employee from Directory" }
                            span { class: "text-[10px] text-muted-foreground font-normal",
                                "{directory_employees.len()} employees loaded"
                            }
                        }
                        crate::components::employee_search::EmployeeSearchInput {
                            employee_code: create_employee_code,
                            employee_name: create_emp_name,
                            email_address: create_email_address,
                            placeholder: "Type code (e.g. 11061) or name (e.g. Chaitram)...".to_string(),
                        }
                    }

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
                                            create_email_address.set(emp.emp_email);
                                        }
                                    } else {
                                        create_emp_name.set(String::new());
                                        create_email_address.set(String::new());
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

                    AttachmentPicker {
                        attachment_name: create_attachment_name,
                        attachment_url: create_attachment_url,
                    }
                }

                DialogFooter {
                    class: "flex items-center justify-start gap-3 pt-3",
                    button {
                        r#type: "button",
                        disabled: *is_submitting.read(),
                        class: if *is_submitting.read() {
                            "inline-flex items-center justify-center rounded-md bg-blue-600/70 text-white font-medium px-6 py-2.5 text-sm cursor-not-allowed gap-2"
                        } else {
                            "inline-flex items-center justify-center rounded-md bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-2.5 text-sm shadow-xs transition-colors cursor-pointer"
                        },
                        onclick: move |_| save_create(),
                        if *is_submitting.read() {
                            span { class: "inline-block w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin mr-1.5" }
                            "Uploading to Google Sheet..."
                        }
                        if !*is_submitting.read() {
                            "Submit Ticket to Sheet"
                        }
                    }
                    button {
                        r#type: "button",
                        disabled: *is_submitting.read(),
                        class: "inline-flex items-center justify-center rounded-md border border-border bg-card hover:bg-muted text-foreground font-medium px-4 py-2.5 text-sm transition-colors cursor-pointer",
                        onclick: move |_| props.open.set(false),
                        "Cancel"
                    }
                }
            }
        }
    }
}
