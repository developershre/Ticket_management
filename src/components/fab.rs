//! Floating action button with speed-dial quick actions.
//!
//! Replicating the exact same "Create Support Ticket", "Create Reminder",
//! and "Create Note" forms as found on their respective pages.
//! There is no separate or downgraded form.

use dioxus::prelude::*;

use crate::components::employee_search::EmployeeSearchInput;
use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::select::SimpleSelect;
use crate::components::ui::toast::ToastState;

// ============================================================
// Quick-add kinds
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuickKind {
    Ticket,
    Reminder,
    Note,
}

impl QuickKind {
    fn label(&self) -> &'static str {
        match self {
            QuickKind::Ticket => "New Ticket",
            QuickKind::Reminder => "New Reminder",
            QuickKind::Note => "New Note",
        }
    }

    fn icon(&self) -> Element {
        let paths = match self {
            // Ticket
            QuickKind::Ticket => rsx! {
                path { d: "M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" }
                path { d: "M13 5v2" }
                path { d: "M13 11v2" }
                path { d: "M13 17v2" }
            },
            // Bell
            QuickKind::Reminder => rsx! {
                path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
            },
            // File text
            QuickKind::Note => rsx! {
                path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
                path { d: "M14 2v4a2 2 0 0 0 2 2h4" }
                path { d: "M10 9H8" }
                path { d: "M16 13H8" }
                path { d: "M16 17H8" }
            },
        };

        rsx! {
            svg {
                class: "size-4.5",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                {paths}
            }
        }
    }
}

const ALL_KINDS: [QuickKind; 3] = [QuickKind::Ticket, QuickKind::Reminder, QuickKind::Note];

// ============================================================
// FAB Component
// ============================================================

#[component]
pub fn Fab() -> Element {
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();

    let mut menu_open = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut kind = use_signal(|| QuickKind::Ticket);

    // Refresh employee directory reactively when DB updates
    if let Some(sync) = db_sync {
        let _ = sync.read();
    }
    let directory_employees = crate::db::get_all_employees();

    // --------------------------------------------------------
    // Ticket signals (matching src/pages/tickets.rs)
    // --------------------------------------------------------
    let mut ticket_employee_code = use_signal(String::new);
    let mut ticket_employee_name = use_signal(String::new);
    let mut ticket_email_address = use_signal(String::new);
    let mut ticket_category = use_signal(|| "Select category".to_string());
    let mut ticket_priority = use_signal(|| "Low".to_string());
    let mut ticket_subject = use_signal(String::new);
    let mut ticket_problem_description = use_signal(String::new);

    // --------------------------------------------------------
    // Reminder signals (matching src/pages/reminders.rs)
    // --------------------------------------------------------
    let mut reminder_employee_code = use_signal(String::new);
    let mut reminder_employee_name = use_signal(String::new);
    let mut reminder_email_address = use_signal(String::new);
    let mut reminder_cc_emails = use_signal(String::new);
    let mut reminder_category = use_signal(|| "Select category".to_string());
    let mut reminder_priority = use_signal(|| "Low".to_string());
    let mut reminder_repeat_frequency = use_signal(|| "daily".to_string());
    let mut reminder_send_at = use_signal(String::new);
    let mut reminder_subject = use_signal(String::new);
    let mut reminder_problem_description = use_signal(String::new);

    // --------------------------------------------------------
    // Note signals (matching src/pages/notes.rs)
    // --------------------------------------------------------
    let mut note_title = use_signal(String::new);
    let mut note_details = use_signal(String::new);

    let mut open_kind = move |k: QuickKind| {
        kind.set(k);
        match k {
            QuickKind::Ticket => {
                ticket_employee_code.write().clear();
                ticket_employee_name.write().clear();
                ticket_email_address.write().clear();
                ticket_category.set("Select category".to_string());
                ticket_priority.set("Low".to_string());
                ticket_subject.write().clear();
                ticket_problem_description.write().clear();
            }
            QuickKind::Reminder => {
                reminder_employee_code.write().clear();
                reminder_employee_name.write().clear();
                reminder_email_address.write().clear();
                reminder_cc_emails.write().clear();
                reminder_category.set("Select category".to_string());
                reminder_priority.set("Low".to_string());
                reminder_repeat_frequency.set("daily".to_string());
                reminder_send_at.write().clear();
                reminder_subject.write().clear();
                reminder_problem_description.write().clear();
            }
            QuickKind::Note => {
                note_title.write().clear();
                note_details.write().clear();
            }
        }
        menu_open.set(false);
        dialog_open.set(true);
    };

    // Save ticket handler
    let mut save_ticket = move || {
        let emp_code = ticket_employee_code.read().trim().to_string();
        if emp_code.is_empty() {
            toast.error("Validation Error", "Please enter Employee Code");
            return;
        }

        // Always query database directly to get latest employee info
        let (name_val, email_val) = if let Some(u) = crate::db::find_employee_by_code(&emp_code) {
            let email = if !u.emp_email.is_empty() {
                u.emp_email
            } else {
                ticket_email_address.read().trim().to_string()
            };
            (u.emp_name, email)
        } else {
            (
                ticket_employee_name.read().trim().to_string(),
                ticket_email_address.read().trim().to_string(),
            )
        };

        if email_val.is_empty() {
            toast.error("Validation Error", "Please enter Email Address");
            return;
        }

        let cat_val = ticket_category.read().trim().to_string();
        if cat_val.is_empty() || cat_val == "Select category" {
            toast.error("Validation Error", "Please select a Category");
            return;
        }

        let subj_val = ticket_subject.read().trim().to_string();
        if subj_val.is_empty() {
            toast.error("Validation Error", "Please enter Subject");
            return;
        }

        let desc_val = ticket_problem_description.read().trim().to_string();
        if desc_val.is_empty() {
            toast.error("Validation Error", "Please describe the problem");
            return;
        }

        let emp_combined = if name_val.is_empty() {
            emp_code.clone()
        } else {
            format!("{} ({})", emp_code, name_val)
        };

        let prio_val = ticket_priority.read().clone();
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

        let input = crate::db::NewTicketInput {
            employee_code: emp_combined,
            email_address: email_val,
            category: cat_val,
            subject: subj_val,
            problem_description: desc_val,
            attachment_name: String::new(),
            attachment_url: String::new(),
            priority: prio_val,
            assigned_to: "IT Support".to_string(),
            customer: "Internal".to_string(),
            time_spent: 0.0,
            activities: "Ticket created".to_string(),
            my_activity: "Initial triage & review".to_string(),
            deadline: deadline_val,
            sla_status: "Within SLA".to_string(),
            sla_deadline: sla_dead_val,
            rating_avg_text: "Unrated".to_string(),
            kanban_state: "Pending".to_string(),
            stage: "Triage".to_string(),
            status: "Open".to_string(),
        };

        match crate::db::create_ticket_full(&input) {
            Ok(new_id) => {
                toast.success("Ticket Created", format!("IT-{new_id} created successfully"));
                crate::sync::enqueue_create(new_id);
                if let Some(mut sync) = db_sync {
                    sync.with_mut(|r| r.0 += 1);
                }
                dialog_open.set(false);
            }
            Err(e) => {
                toast.error("Database Error", format!("Failed to create ticket: {:?}", e));
            }
        }
    };

    // Save reminder handler
    let mut save_reminder = move || {
        let emp_code = reminder_employee_code.read().trim().to_string();
        if emp_code.is_empty() {
            toast.error("Validation Error", "Please enter Employee Code");
            return;
        }

        // Always query database directly to get latest employee info
        let (emp_name, email_val) = if let Some(u) = crate::db::find_employee_by_code(&emp_code) {
            let email = if !u.emp_email.is_empty() {
                u.emp_email
            } else {
                reminder_email_address.read().trim().to_string()
            };
            (u.emp_name, email)
        } else {
            (
                reminder_employee_name.read().trim().to_string(),
                reminder_email_address.read().trim().to_string(),
            )
        };

        if email_val.is_empty() {
            toast.error("Validation Error", "Please enter Email Address");
            return;
        }

        let cat_val = reminder_category.read().trim().to_string();
        if cat_val.is_empty() || cat_val == "Select category" {
            toast.error("Validation Error", "Please select a Category");
            return;
        }

        let subj_val = reminder_subject.read().trim().to_string();
        if subj_val.is_empty() {
            toast.error("Validation Error", "Please enter Subject");
            return;
        }

        let desc_val = reminder_problem_description.read().trim().to_string();
        if desc_val.is_empty() {
            toast.error("Validation Error", "Please describe the problem");
            return;
        }

        let reminder_msg = format!("{subj_val}: {desc_val}");
        let cc_val = reminder_cc_emails.read().trim().to_string();
        let send_at_val = reminder_send_at.read().trim().to_string();
        let final_send_at = if send_at_val.is_empty() {
            let now_str = crate::state::format_utc(crate::sync::now_epoch());
            crate::sync::parse_to_iso8601(&now_str)
        } else {
            crate::sync::parse_to_iso8601(&send_at_val)
        };
        let freq_val = reminder_repeat_frequency.read().trim().to_lowercase();
        let final_freq = if freq_val == "once" { "once" } else { "daily" };

        let input = crate::db::NewReminderInput {
            employee_code: emp_code,
            employee_name: emp_name,
            email_address: email_val,
            service_category: cat_val,
            reminder_message: reminder_msg,
            send_at: final_send_at,
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: String::new(),
            cc_emails: cc_val,
            repeat_frequency: final_freq.to_string(),
        };

        match crate::db::create_reminder_full(&input) {
            Ok(new_id) => {
                toast.success("Reminder Created", format!("REM-{new_id} scheduled"));
                crate::sync::enqueue_reminder_create(new_id);
                if let Some(mut sync) = db_sync {
                    sync.with_mut(|r| r.0 += 1);
                }
                dialog_open.set(false);
            }
            Err(e) => {
                toast.error("Database Error", format!("Failed to create reminder: {:?}", e));
            }
        }
    };

    // Save note handler
    let mut save_note = move || {
        let title = note_title.read().trim().to_string();
        if title.is_empty() {
            toast.error("Missing title", "Please enter a title for your note.");
            return;
        }
        let details = note_details.read().trim().to_string();
        let content = if details.is_empty() {
            title
        } else {
            format!("{title} — {details}")
        };

        match crate::db::create_note(&crate::db::configured_employee_code(), &content) {
            Ok(new_id) => {
                toast.success("Note Saved", format!("Note #{new_id} persisted to database."));
                if let Some(mut sync) = db_sync {
                    sync.with_mut(|r| r.0 += 1);
                }
                dialog_open.set(false);
            }
            Err(e) => {
                toast.error("Database Error", format!("Failed to insert note: {:?}", e));
            }
        }
    };

    rsx! {
        // ------------------------------------------------------------
        // Click-away backdrop for closing speed dial menu
        // ------------------------------------------------------------
        if menu_open() {
            div {
                class: "fixed inset-0 z-30 bg-background/40 backdrop-blur-xs transition-opacity duration-200",
                onclick: move |_| menu_open.set(false),
            }
        }

        // ------------------------------------------------------------
        // Speed-dial actions (New Ticket, New Reminder, New Note)
        // ------------------------------------------------------------
        if menu_open() {
            div {
                class: "fixed bottom-24 right-6 z-40 flex flex-col items-end gap-3 animate-in fade-in slide-in-from-bottom-3 duration-200",
                for (action_idx, kind_item) in ALL_KINDS.iter().enumerate() {
                    div {
                        key: "quick-action-{action_idx}",
                        class: "flex items-center gap-2.5 pr-1.5",
                        span { class: "rounded-lg border border-border/70 bg-card/95 px-3 py-1 text-xs font-semibold text-foreground shadow-md backdrop-blur-md select-none pointer-events-none",
                            "{kind_item.label()}"
                        }
                        button {
                            r#type: "button",
                            title: "{kind_item.label()}",
                            class: "flex size-11 items-center justify-center rounded-full border border-border/80 bg-card/95 text-foreground shadow-lg shadow-black/10 backdrop-blur-md transition-all duration-150 hover:scale-110 hover:border-primary/50 hover:bg-accent hover:text-primary cursor-pointer",
                            onclick: move |_| open_kind(*kind_item),
                            {kind_item.icon()}
                        }
                    }
                }
            }
        }

        // ------------------------------------------------------------
        // Main FAB button
        // ------------------------------------------------------------
        button {
            r#type: "button",
            title: "Quick actions",
            "aria-label": "Quick actions",
            class: "fixed bottom-6 right-6 z-40 flex size-14 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-xl shadow-primary/30 hover:shadow-2xl hover:shadow-primary/45 hover:scale-105 active:scale-95 transition-all duration-200 cursor-pointer border border-primary-foreground/15",
            onclick: move |_| {
                let next = !*menu_open.read();
                menu_open.set(next);
            },
            svg {
                class: if menu_open() { "size-6 rotate-45 transition-transform duration-300" } else { "size-6 transition-transform duration-300" },
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.5",
                stroke_linecap: "round",
                path { d: "M5 12h14" }
                path { d: "M12 5v14" }
            }
        }

        // ------------------------------------------------------------
        // Dialog Modal: 100% Identical to page forms
        // ------------------------------------------------------------
        if dialog_open() {
            Dialog {
                open: Some(dialog_open),

                // ====================================================
                // Form 1: New Ticket (identical to tickets.rs)
                // ====================================================
                if kind() == QuickKind::Ticket {
                    DialogContent {
                        class: "max-w-2xl sm:max-w-2xl w-full rounded-xl border border-border bg-card p-6 sm:p-8 shadow-2xl max-h-[92vh] overflow-y-auto",
                        DialogHeader {
                            class: "space-y-1 pb-3 text-left",
                            DialogTitle { class: "text-2xl font-bold tracking-tight text-foreground", "Create Support Ticket" }
                            DialogDescription { class: "text-sm text-muted-foreground",
                                "Enter your employee details and describe the issue."
                            }
                        }

                        div { class: "flex flex-col gap-4 py-2 text-sm",
                            // Row 0: Select Employee from Directory
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                    span { "Select Employee from Directory" }
                                    span { class: "text-[10px] text-muted-foreground font-normal",
                                        "{directory_employees.len()} employees loaded"
                                    }
                                }
                                EmployeeSearchInput {
                                    employee_code: ticket_employee_code,
                                    employee_name: ticket_employee_name,
                                    email_address: ticket_email_address,
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
                                        value: ticket_employee_code.read().clone(),
                                        oninput: move |e: FormEvent| {
                                            let val = e.value();
                                            ticket_employee_code.set(val.clone());
                                            let trimmed = val.trim();
                                            if !trimmed.is_empty() {
                                                if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                                                    ticket_employee_name.set(emp.emp_name);
                                                    ticket_email_address.set(emp.emp_email);
                                                }
                                            } else {
                                                ticket_employee_name.set(String::new());
                                                ticket_email_address.set(String::new());
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
                                        value: ticket_email_address.read().clone(),
                                        oninput: move |e: FormEvent| ticket_email_address.set(e.value()),
                                    }
                                }
                            }

                            // Row 2: Employee Name
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Employee Name" }
                                Input {
                                    class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                    placeholder: "Automatically filled from Employee sheet".to_string(),
                                    value: ticket_employee_name.read().clone(),
                                    oninput: move |e: FormEvent| ticket_employee_name.set(e.value()),
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
                                        value: ticket_category,
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
                                    label { class: "text-xs font-semibold text-foreground", "Priority" }
                                    SimpleSelect {
                                        value: ticket_priority,
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
                                    value: ticket_subject.read().clone(),
                                    oninput: move |e: FormEvent| ticket_subject.set(e.value()),
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
                                    value: "{ticket_problem_description}",
                                    oninput: move |e: FormEvent| ticket_problem_description.set(e.value()),
                                }
                            }
                        }

                        DialogFooter {
                            class: "flex items-center justify-start gap-3 pt-3",
                            button {
                                r#type: "button",
                                class: "inline-flex items-center justify-center rounded-md bg-blue-600 hover:bg-blue-700 hover:scale-[1.02] active:scale-[0.98] text-white font-medium px-6 py-2.5 text-sm shadow-xs transition-all duration-150 cursor-pointer",
                                onclick: move |_| save_ticket(),
                                "Submit Ticket"
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex items-center justify-center rounded-md border border-border bg-card hover:bg-muted active:scale-[0.98] text-foreground font-medium px-4 py-2.5 text-sm transition-all duration-150 cursor-pointer",
                                onclick: move |_| dialog_open.set(false),
                                "Cancel"
                            }
                        }
                    }
                }

                // ====================================================
                // Form 2: New Reminder (identical to reminders.rs)
                // ====================================================
                if kind() == QuickKind::Reminder {
                    DialogContent {
                        class: "max-w-2xl max-h-[92vh] overflow-y-auto p-6 sm:p-8 rounded-xl bg-card border border-border shadow-2xl",
                        DialogHeader {
                            class: "space-y-1 pb-3 text-left",
                            DialogTitle { class: "text-2xl font-bold tracking-tight text-foreground", "Create Reminder" }
                            DialogDescription { class: "text-sm text-muted-foreground",
                                "Enter recipient details and describe the reminder or issue."
                            }
                        }

                        div { class: "flex flex-col gap-4 py-2 text-sm",
                            // Row 0: Select Employee from Directory
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                    span { "Select Employee from Directory" }
                                    span { class: "text-[10px] text-muted-foreground font-normal",
                                        "{directory_employees.len()} employees loaded"
                                    }
                                }
                                EmployeeSearchInput {
                                    employee_code: reminder_employee_code,
                                    employee_name: reminder_employee_name,
                                    email_address: reminder_email_address,
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
                                        value: reminder_employee_code.read().clone(),
                                        oninput: move |e: FormEvent| {
                                            let val = e.value();
                                            reminder_employee_code.set(val.clone());
                                            let trimmed = val.trim();
                                            if !trimmed.is_empty() {
                                                if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                                                    reminder_employee_name.set(emp.emp_name);
                                                    reminder_email_address.set(emp.emp_email);
                                                }
                                            } else {
                                                reminder_employee_name.set(String::new());
                                                reminder_email_address.set(String::new());
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
                                        value: reminder_email_address.read().clone(),
                                        oninput: move |e: FormEvent| reminder_email_address.set(e.value()),
                                    }
                                }
                            }

                            // Row 2: Employee Name
                            div { class: "flex flex-col gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Employee Name" }
                                Input {
                                    class: "h-10 text-sm rounded-md border border-input bg-card px-3 focus:border-primary focus:ring-1 focus:ring-primary/20",
                                    placeholder: "Automatically filled from Employee sheet".to_string(),
                                    value: reminder_employee_name.read().clone(),
                                    oninput: move |e: FormEvent| reminder_employee_name.set(e.value()),
                                }
                            }

                            // Row 2.5: CC Recipients (Optional)
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
                                    value: reminder_cc_emails.read().clone(),
                                    oninput: move |e: FormEvent| reminder_cc_emails.set(e.value()),
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
                                        value: reminder_category,
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
                                    label { class: "text-xs font-semibold text-foreground", "Priority" }
                                    SimpleSelect {
                                        value: reminder_priority,
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
                                        value: reminder_repeat_frequency,
                                        placeholder: "Select frequency...".to_string(),
                                        options: vec![
                                            ("daily".to_string(), "Daily (every 24h)".to_string()),
                                            ("once".to_string(), "Once (one-time send)".to_string()),
                                        ],
                                    }
                                }
                                div { class: "flex flex-col gap-1.5",
                                    crate::components::ui::ClockDateTimePicker {
                                        value: reminder_send_at,
                                        label: "Schedule Date & Time".to_string(),
                                        placeholder: "Pick date & time (or leave empty)".to_string(),
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
                                    value: reminder_subject.read().clone(),
                                    oninput: move |e: FormEvent| reminder_subject.set(e.value()),
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
                                    value: "{reminder_problem_description}",
                                    oninput: move |e: FormEvent| reminder_problem_description.set(e.value()),
                                }
                            }
                        }

                        DialogFooter {
                            class: "flex items-center justify-start gap-3 pt-3",
                            button {
                                r#type: "button",
                                class: "inline-flex items-center justify-center rounded-md bg-blue-600 hover:bg-blue-700 hover:scale-[1.02] active:scale-[0.98] text-white font-medium px-6 py-2.5 text-sm shadow-xs transition-all duration-150 cursor-pointer",
                                onclick: move |_| save_reminder(),
                                "Submit Reminder"
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex items-center justify-center rounded-md border border-border bg-card hover:bg-muted active:scale-[0.98] text-foreground font-medium px-4 py-2.5 text-sm transition-all duration-150 cursor-pointer",
                                onclick: move |_| dialog_open.set(false),
                                "Cancel"
                            }
                        }
                    }
                }

                // ====================================================
                // Form 3: New Note (identical to notes.rs)
                // ====================================================
                if kind() == QuickKind::Note {
                    DialogContent {
                        class: "max-w-md w-full rounded-xl border border-border bg-card p-6 shadow-2xl",
                        DialogHeader {
                            DialogTitle { "Create New Note" }
                            DialogDescription { "Capture ideas, tasks, or reminders directly to SQLite." }
                        }

                        div { class: "grid gap-3 py-1",
                            div { class: "grid gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Title" }
                                Input {
                                    placeholder: "e.g. Plan Q4 server architecture review".to_string(),
                                    value: note_title.read().clone(),
                                    oninput: move |e: FormEvent| note_title.set(e.value()),
                                }
                            }
                            div { class: "grid gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Details (optional)" }
                                textarea {
                                    rows: "5",
                                    class: "flex w-full resize-none rounded-md border border-input bg-card px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                    placeholder: "Add supporting context, bullet points, or instructions...".to_string(),
                                    value: note_details.read().clone(),
                                    oninput: move |e: FormEvent| note_details.set(e.value()),
                                }
                            }
                        }

                        DialogFooter {
                            Button {
                                variant: "outline",
                                size: "sm",
                                onclick: move |_| dialog_open.set(false),
                                "Cancel"
                            }
                            Button {
                                variant: "default",
                                size: "sm",
                                onclick: move |_| save_note(),
                                "Create Note"
                            }
                        }
                    }
                }
            }
        }
    }
}
