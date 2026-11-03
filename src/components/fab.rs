//! Floating action button with a speed-dial menu.
//!
//! The main FAB expands into quick actions (note / ticket / reminder). Picking
//! one opens a quick-add dialog; saving fires a toast via the `ToastProvider`
//! higher in the tree.

use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::icons::{BellIcon, ClockIcon, FileTextIcon, PaperclipIcon, UploadCloudIcon, XIcon};

// ============================================================
// Quick-add kinds
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuickKind {
    Note,
    Ticket,
    Reminder,
}

impl QuickKind {
    fn label(&self) -> &'static str {
        match self {
            QuickKind::Note => "Note",
            QuickKind::Ticket => "Ticket",
            QuickKind::Reminder => "Reminder",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            QuickKind::Note => "New note",
            QuickKind::Ticket => "New ticket",
            QuickKind::Reminder => "New reminder",
        }
    }

    fn placeholder(&self) -> &'static str {
        match self {
            QuickKind::Note => "Note title, e.g. \"Ideas for the v2 launch\"",
            QuickKind::Ticket => "IT ticket subject, e.g. \"VPN gateway disconnects every 30m\"",
            QuickKind::Reminder => "Reminder title, e.g. \"Call the dentist\"",
        }
    }

    fn icon(&self) -> Element {
        let paths = match self {
            // File text
            QuickKind::Note => rsx! {
                path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
                path { d: "M14 2v4a2 2 0 0 0 2 2h4" }
                path { d: "M10 9H8" }
                path { d: "M16 13H8" }
                path { d: "M16 17H8" }
            },
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

const ALL_KINDS: [QuickKind; 3] = [QuickKind::Note, QuickKind::Ticket, QuickKind::Reminder];

// ============================================================
// FAB
// ============================================================

#[component]
pub fn Fab() -> Element {
    // ToastProvider (AppShell) must be an ancestor.
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();

    let mut menu_open = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut kind = use_signal(|| QuickKind::Note);
    let mut title_text = use_signal(String::new);
    let mut details = use_signal(String::new);
    let mut ticket_category = use_signal(|| "General".to_string());
    let mut ticket_priority = use_signal(|| "Medium".to_string());
    let mut ticket_attachment_name = use_signal(String::new);
    let mut ticket_attachment_url = use_signal(String::new);

    // Reminder 12-field signals
    let mut reminder_emp_code = use_signal(String::new);
    let mut reminder_emp_name = use_signal(String::new);
    let mut reminder_email = use_signal(String::new);
    let mut reminder_category = use_signal(|| "General".to_string());
    let mut reminder_send_at = use_signal(|| "Today · 2:00 PM".to_string());
    let mut reminder_status = use_signal(|| "Pending".to_string());
    let mut reminder_email_sent_at = use_signal(|| "Not sent yet".to_string());
    let mut reminder_ticket_id = use_signal(String::new);

    // Bumped every time the dialog opens so inputs remount empty.
    let mut generation = use_signal(|| 0u32);

    let mut open_kind = move |k: QuickKind| {
        kind.set(k);
        title_text.write().clear();
        details.write().clear();
        ticket_category.set("General".to_string());
        ticket_priority.set("Medium".to_string());
        ticket_attachment_name.write().clear();
        ticket_attachment_url.write().clear();

        reminder_emp_code.write().clear();
        reminder_emp_name.write().clear();
        reminder_email.write().clear();
        reminder_category.set("General".to_string());
        reminder_send_at.set("Today · 2:00 PM".to_string());
        reminder_status.set("Pending".to_string());
        reminder_email_sent_at.set("Not sent yet".to_string());
        reminder_ticket_id.write().clear();

        *generation.write() += 1;
        menu_open.set(false);
        dialog_open.set(true);
    };

    rsx! {
        // ------------------------------------------------------------
        // Click-away layer that closes the speed dial
        // ------------------------------------------------------------
        if menu_open() {
            div {
                class: "fixed inset-0 z-30 bg-background/40 backdrop-blur-xs transition-opacity duration-200",
                onclick: move |_| menu_open.set(false),
            }
        }

        // ------------------------------------------------------------
        // Speed-dial actions
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
                            title: "New {kind_item.label()}",
                            class: "flex size-11 items-center justify-center rounded-full border border-border/80 bg-card/95 text-foreground shadow-lg shadow-black/10 backdrop-blur-md transition-all duration-150 hover:scale-110 hover:border-primary/50 hover:bg-accent hover:text-primary cursor-pointer",
                            onclick: move |_| open_kind(*kind_item),
                            {kind_item.icon()}
                        }
                    }
                }
            }
        }

        // ------------------------------------------------------------
        // Main FAB
        // ------------------------------------------------------------
        button {
            r#type: "button",
            title: "Quick add",
            "aria-label": "Quick add",
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
        // Quick-add dialog
        // ------------------------------------------------------------
        Dialog {
            open: Some(dialog_open),
            DialogContent {
                DialogHeader {
                    DialogTitle { "{kind().title()}" }
                    DialogDescription { "Capture it now — you can view, edit, and organize it in the workspace." }
                }

                div { class: "grid gap-3 pt-1",
                    div { class: "grid gap-1.5",
                        label { class: "text-xs font-semibold text-foreground", "Title" }
                        Input {
                            key: "title-{generation()}",
                            placeholder: kind().placeholder().to_string(),
                            value: title_text.read().clone(),
                            oninput: move |e: FormEvent| title_text.set(e.value()),
                        }
                    }

                    if kind() == QuickKind::Ticket {
                        div { class: "grid grid-cols-2 gap-2.5",
                            div { class: "grid gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Category" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-2.5 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                    value: "{ticket_category}",
                                    onchange: move |e: FormEvent| ticket_category.set(e.value()),
                                    option { value: "Network", "Network" }
                                    option { value: "Hardware", "Hardware" }
                                    option { value: "Access", "Access" }
                                    option { value: "Security", "Security" }
                                    option { value: "Software", "Software" }
                                    option { value: "Infrastructure", "Infrastructure" }
                                    option { value: "General", "General" }
                                }
                            }
                            div { class: "grid gap-1.5",
                                label { class: "text-xs font-semibold text-foreground", "Priority" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-2.5 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                    value: "{ticket_priority}",
                                    onchange: move |e: FormEvent| ticket_priority.set(e.value()),
                                    option { value: "High", "High" }
                                    option { value: "Medium", "Medium" }
                                    option { value: "Low", "Low" }
                                    option { value: "Critical", "Critical" }
                                }
                            }
                        }
                    }

                    div { class: "grid gap-1.5",
                        label { class: "text-xs font-semibold text-foreground", "Details" }
                        textarea {
                            key: "details-{generation()}",
                            rows: "3",
                            class: "flex w-full resize-none rounded-md border border-input bg-card px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                            placeholder: "Optional details...".to_string(),
                            oninput: move |e: FormEvent| details.set(e.value()),
                        }
                    }

                    if kind() == QuickKind::Ticket {
                        div { class: "rounded-md border border-border/70 bg-muted/20 p-2.5 flex flex-col gap-2",
                            div { class: "flex items-center justify-between",
                                span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5",
                                    PaperclipIcon { class: "size-3.5 text-primary" }
                                    "Attachment Option"
                                }
                                if !ticket_attachment_name.read().is_empty() {
                                    button {
                                        r#type: "button",
                                        class: "text-[11px] text-destructive hover:underline flex items-center gap-1 cursor-pointer",
                                        onclick: move |_| {
                                            ticket_attachment_name.set(String::new());
                                            ticket_attachment_url.set(String::new());
                                        },
                                        XIcon { class: "size-3" }
                                        "Clear"
                                    }
                                }
                            }

                            if ticket_attachment_name.read().is_empty() {
                                div { class: "flex flex-col gap-2",
                                    label { class: "flex items-center justify-center gap-2 border border-dashed border-border/80 hover:border-primary/60 bg-card/60 hover:bg-card p-2.5 rounded cursor-pointer transition-colors group",
                                        input {
                                            r#type: "file",
                                            class: "hidden",
                                            onchange: move |e: FormEvent| {
                                                let raw = e.value();
                                                let clean = raw.split(['/', '\\']).last().unwrap_or(&raw).to_string();
                                                if !clean.is_empty() {
                                                    ticket_attachment_name.set(clean.clone());
                                                    ticket_attachment_url.set(format!("/attachments/{}", clean));
                                                }
                                            }
                                        }
                                        UploadCloudIcon { class: "size-4 text-muted-foreground group-hover:text-primary transition-colors" }
                                        span { class: "text-xs font-medium text-foreground group-hover:text-primary transition-colors", "Click to insert file attachment" }
                                    }
                                    div { class: "grid grid-cols-2 gap-2",
                                        Input {
                                            class: "text-xs h-8",
                                            placeholder: "Or filename...".to_string(),
                                            value: ticket_attachment_name.read().clone(),
                                            oninput: move |e: FormEvent| ticket_attachment_name.set(e.value()),
                                        }
                                        Input {
                                            class: "text-xs h-8",
                                            placeholder: "Or attachment URL...".to_string(),
                                            value: ticket_attachment_url.read().clone(),
                                            oninput: move |e: FormEvent| ticket_attachment_url.set(e.value()),
                                        }
                                    }
                                }
                            } else {
                                div { class: "flex items-center justify-between p-2 rounded bg-primary/10 border border-primary/20",
                                    div { class: "flex items-center gap-2 min-w-0",
                                        FileTextIcon { class: "size-3.5 text-primary shrink-0" }
                                        span { class: "font-mono text-xs font-semibold text-foreground truncate", "{ticket_attachment_name}" }
                                    }
                                    span { class: "text-[10px] text-primary font-medium", "Ready" }
                                }
                            }
                        }
                    }
                    if kind() == QuickKind::Reminder {
                        div { class: "rounded-md border border-border/70 bg-muted/20 p-3 flex flex-col gap-2.5",
                            div { class: "flex items-center justify-between",
                                span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5",
                                    BellIcon { class: "size-3.5 text-primary" }
                                    "Reminder Fields"
                                }
                                span { class: "text-[10px] font-mono text-primary bg-primary/10 px-2 py-0.5 rounded",
                                    "ID: Auto (REM-*)"
                                }
                            }

                            // Recipient details
                            div { class: "grid grid-cols-1 sm:grid-cols-3 gap-2",
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Employee Code" }
                                    Input {
                                        class: "text-xs h-8",
                                        placeholder: "e.g. 10001".to_string(),
                                        value: reminder_emp_code.read().clone(),
                                        oninput: move |e: FormEvent| reminder_emp_code.set(e.value()),
                                    }
                                }
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Employee Name" }
                                    Input {
                                        class: "text-xs h-8",
                                        placeholder: "e.g. Employee Name".to_string(),
                                        value: reminder_emp_name.read().clone(),
                                        oninput: move |e: FormEvent| reminder_emp_name.set(e.value()),
                                    }
                                }
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Email Address" }
                                    Input {
                                        class: "text-xs h-8",
                                        placeholder: "e.g. employee@bpsl.net".to_string(),
                                        value: reminder_email.read().clone(),
                                        oninput: move |e: FormEvent| reminder_email.set(e.value()),
                                    }
                                }
                            }

                            // Service Category, Status & Ticket ID
                            div { class: "grid grid-cols-1 sm:grid-cols-3 gap-2",
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Service / Category" }
                                    select {
                                        class: "h-8 rounded-md border border-input bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                        value: "{reminder_category}",
                                        onchange: move |e: FormEvent| reminder_category.set(e.value()),
                                        option { value: "Engineering", "Engineering" }
                                        option { value: "Meetings", "Meetings" }
                                        option { value: "Ops", "Ops" }
                                        option { value: "Reporting", "Reporting" }
                                        option { value: "Planning", "Planning" }
                                        option { value: "Infrastructure", "Infrastructure" }
                                        option { value: "General", "General" }
                                    }
                                }
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Status" }
                                    select {
                                        class: "h-8 rounded-md border border-input bg-card px-2 text-xs text-foreground focus:border-primary focus:outline-none cursor-pointer",
                                        value: "{reminder_status}",
                                        onchange: move |e: FormEvent| reminder_status.set(e.value()),
                                        option { value: "Pending", "Pending" }
                                        option { value: "Sent", "Sent" }
                                        option { value: "Completed", "Completed" }
                                        option { value: "Cancelled", "Cancelled" }
                                    }
                                }
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Ticket ID (optional)" }
                                    Input {
                                        class: "text-xs h-8 font-mono",
                                        placeholder: "e.g. IT-1".to_string(),
                                        value: reminder_ticket_id.read().clone(),
                                        oninput: move |e: FormEvent| reminder_ticket_id.set(e.value()),
                                    }
                                }
                            }

                            // Schedule Timing & Email Delivery
                            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2",
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground flex items-center gap-1",
                                        ClockIcon { class: "size-3 text-muted-foreground" }
                                        "Send At"
                                    }
                                    Input {
                                        class: "text-xs h-8",
                                        placeholder: "e.g. Today · 2:00 PM".to_string(),
                                        value: reminder_send_at.read().clone(),
                                        oninput: move |e: FormEvent| reminder_send_at.set(e.value()),
                                    }
                                    div { class: "flex flex-wrap gap-1 pt-0.5",
                                        for preset in ["Today · 2:00 PM", "Tomorrow · 9:00 AM", "Friday · 5:00 PM", "Mon · 10:00 AM"] {
                                            {
                                                let preset_str = preset.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{preset}",
                                                        r#type: "button",
                                                        class: "text-[10px] px-1.5 py-0.5 rounded border border-border/70 hover:bg-muted/80 text-muted-foreground hover:text-foreground cursor-pointer transition-colors",
                                                        onclick: move |_| reminder_send_at.set(preset_str.clone()),
                                                        "{preset}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                div { class: "grid gap-1",
                                    label { class: "text-[11px] font-medium text-foreground", "Email Sent At" }
                                    Input {
                                        class: "text-xs h-8",
                                        placeholder: "Not sent yet".to_string(),
                                        value: reminder_email_sent_at.read().clone(),
                                        oninput: move |e: FormEvent| reminder_email_sent_at.set(e.value()),
                                    }
                                    span { class: "text-[10px] text-muted-foreground", "Created At and Last Updated At are populated automatically." }
                                }
                            }
                        }
                    }
                }

                DialogFooter {
                    Button {
                        variant: "outline",
                        onclick: move |_| dialog_open.set(false),
                        "Cancel"
                    }
                    Button {
                        variant: "default",
                        onclick: move |_| {
                            let title = title_text.read().trim().to_string();
                            if title.is_empty() {
                                toast.error("Missing title", format!("Give your {} a title first.", kind().label().to_lowercase()));
                                return;
                            }
                            let detail = details.read().trim().to_string();
                            let current_kind = kind();

                            match current_kind {
                                QuickKind::Ticket => {
                                    let input = crate::db::NewTicketInput {
                                        employee_code: "11135".to_string(),
                                        email_address: "it.support@bpsl.net".to_string(),
                                        category: ticket_category.read().clone(),
                                        subject: title.clone(),
                                        problem_description: detail.clone(),
                                        attachment_name: ticket_attachment_name.read().clone(),
                                        attachment_url: ticket_attachment_url.read().clone(),
                                        priority: ticket_priority.read().clone(),
                                        assigned_to: "IT Support".to_string(),
                                        customer: "Internal".to_string(),
                                        kanban_state: "Pending".to_string(),
                                        stage: "Triage".to_string(),
                                        status: "Open".to_string(),
                                        ..Default::default()
                                    };
                                    match crate::db::create_ticket_full(&input) {
                                        Ok(new_id) => {
                                            toast.success("Ticket Saved to DB", format!("IT-{new_id}: \"{title}\" persisted to management.db"));
                                            // Queue the create for background sync.
                                            crate::sync::enqueue_create(new_id);
                                            if let Some(mut sync) = db_sync {
                                                sync.with_mut(|r| r.0 += 1);
                                            }
                                        }
                                        Err(e) => {
                                            toast.error("Database Error", format!("Failed to save ticket: {:?}", e));
                                        }
                                    }
                                }
                                QuickKind::Reminder => {
                                    let msg = if detail.is_empty() {
                                        title.clone()
                                    } else {
                                        format!("{title} — {detail}")
                                    };
                                    let input = crate::db::NewReminderInput {
                                        employee_code: reminder_emp_code.read().clone(),
                                        employee_name: reminder_emp_name.read().clone(),
                                        email_address: reminder_email.read().clone(),
                                        service_category: reminder_category.read().clone(),
                                        reminder_message: msg,
                                        send_at: crate::sync::parse_to_iso8601(&reminder_send_at.read()),
                                        status: reminder_status.read().clone(),
                                        email_sent_at: reminder_email_sent_at.read().clone(),
                                        ticket_id: reminder_ticket_id.read().clone(),
                                    };
                                    match crate::db::create_reminder_full(&input) {
                                        Ok(new_id) => {
                                            toast.success("Reminder Saved to DB", format!("REM-{new_id}: \"{title}\" scheduled"));
                                            crate::sync::enqueue_reminder_create(new_id);
                                            if let Some(mut sync) = db_sync {
                                                sync.with_mut(|r| r.0 += 1);
                                            }
                                        }
                                        Err(e) => {
                                            toast.error("Database Error", format!("Failed to save reminder: {:?}", e));
                                        }
                                    }
                                }
                                QuickKind::Note => {
                                    let content = if detail.is_empty() {
                                        title.clone()
                                    } else {
                                        format!("{title} — {detail}")
                                    };
                                    match crate::db::create_note("EMP-1001", &content) {
                                        Ok(new_id) => {
                                            let label = current_kind.label();
                                            toast.success("Saved to DB", format!("{label} #{new_id} persisted to management.db"));
                                            if let Some(mut sync) = db_sync {
                                                sync.with_mut(|r| r.0 += 1);
                                            }
                                        }
                                        Err(e) => {
                                            toast.error("Database Error", format!("Failed to save: {:?}", e));
                                        }
                                    }
                                }
                            }
                            dialog_open.set(false);
                        },
                        "Create"
                    }
                }
            }
        }
    }
}
