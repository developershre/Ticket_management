use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::icons::{ExternalLinkIcon, FileTextIcon, PaperclipIcon, PencilIcon};

use super::super::components::get_image_preview_url;
use super::super::types::{
    assignee_color_classes, column_display_label, get_assignee_initials, priority_badge,
    sla_badge_class, status_badge_class, status_badge_variant, TicketContextState, TicketItem,
};

#[derive(Props, Clone, PartialEq)]
pub struct ViewTicketDialogProps {
    pub open: Signal<bool>,
    pub ticket: Signal<Option<TicketItem>>,
}

#[component]
pub fn ViewTicketDialog(mut props: ViewTicketDialogProps) -> Element {
    let mut state = use_context::<TicketContextState>();

    rsx! {
        Dialog { open: props.open,
            DialogContent { class: "max-w-4xl sm:max-w-4xl w-full rounded-md border border-border/80 bg-card p-6 sm:p-7 shadow-2xl max-h-[88vh] overflow-y-auto",
                if let Some(ticket) = (props.ticket)() {
                    {
                        let t_for_edit = ticket.clone();
                        let status_lbl = column_display_label(&ticket.kanban_state);
                        let status_badge_cls = status_badge_class(&ticket.kanban_state);
                        let prio_variant = priority_badge(&ticket.priority);
                        let sla_cls = sla_badge_class(&ticket.sla_status);
                        let initial_letter = ticket.emp_name().chars().next().unwrap_or('U');
                        let disp_deadline = if ticket.deadline.is_empty() { "None set".to_string() } else { crate::state::format_display_12hr(&ticket.deadline) };
                        let disp_sla_deadline = if ticket.sla_deadline.is_empty() { "None set".to_string() } else { crate::state::format_display_12hr(&ticket.sla_deadline) };

                        rsx! {
                            DialogHeader {
                                div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-border/50 pb-3",
                                    div { class: "flex items-center gap-2",
                                        span { class: "font-mono text-xs font-bold text-foreground bg-muted px-2.5 py-0.5 rounded border border-border/80",
                                            "{ticket.id}"
                                        }
                                        Badge {
                                            variant: "{prio_variant}",
                                            class: "text-[10px] font-semibold px-2 py-0.5",
                                            "{ticket.priority}"
                                        }
                                        Badge {
                                            variant: "{status_badge_variant(&ticket.kanban_state)}",
                                            class: "text-[10px] font-semibold {status_badge_cls}",
                                            "{status_lbl}"
                                        }
                                    }
                                    if !ticket.sla_status.is_empty() {
                                        span { class: "rounded px-2 py-0.5 text-[10px] font-semibold border {sla_cls}",
                                            "SLA: {ticket.sla_status}"
                                        }
                                    }
                                }

                                DialogTitle { class: "text-lg font-bold text-foreground mt-3 leading-snug", "{ticket.subject}" }
                                {
                                    let disp_c = crate::state::format_display_12hr(&ticket.created_on);
                                    let disp_u = crate::state::format_display_12hr(&ticket.last_updated_on);
                                    rsx! {
                                        DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                            "Reported on {disp_c} · Last updated: {disp_u}"
                                        }
                                    }
                                }
                            }

                            div { class: "flex flex-col gap-4 py-3 text-xs",
                                div { class: "flex flex-col gap-1.5 rounded-md bg-muted/30 border border-border/40 p-3",
                                    span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider",
                                        "Created By (Requester)"
                                    }
                                    div { class: "flex items-center justify-between gap-2 flex-wrap",
                                        div { class: "flex items-center gap-2.5",
                                            div { class: "flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-foreground font-bold text-xs border border-border/80",
                                                "{initial_letter}"
                                            }
                                            div { class: "flex flex-col min-w-0",
                                                span { class: "font-semibold text-foreground text-xs", "{ticket.emp_name()}" }
                                                span { class: "font-mono text-[11px] text-muted-foreground",
                                                    "{ticket.emp_code_only()}"
                                                }
                                            }
                                        }
                                        div { class: "flex items-center gap-1.5 text-xs text-muted-foreground",
                                            span { class: "font-mono text-foreground/80 bg-background/60 px-2 py-0.5 rounded border border-border/50",
                                                "{ticket.email_address}"
                                            }
                                        }
                                    }
                                }

                                div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Assigned To"
                                        }
                                        div { class: "flex items-center gap-1.5 mt-0.5",
                                            div {
                                                class: "flex size-5.5 shrink-0 items-center justify-center rounded-full text-[9px] font-extrabold uppercase border shadow-2xs {assignee_color_classes(&ticket.assigned_to)}",
                                                "{get_assignee_initials(&ticket.assigned_to)}"
                                            }
                                            span { class: "font-semibold text-foreground text-xs truncate", "{ticket.assignee_name()}" }
                                        }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "({ticket.assigned_to})" }
                                    }
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Stage / State"
                                        }
                                        span { class: "font-semibold text-foreground text-xs", "{status_lbl}" }
                                        span { class: "text-[10px] text-muted-foreground", "{ticket.stage}" }
                                    }
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Category"
                                        }
                                        span { class: "font-semibold text-foreground text-xs", "{ticket.category}" }
                                        span { class: "text-[10px] text-muted-foreground", "IT Support" }
                                    }
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Customer / Dept"
                                        }
                                        span { class: "font-semibold text-foreground text-xs truncate",
                                            if ticket.customer.is_empty() {
                                                "Internal"
                                            } else {
                                                "{ticket.customer}"
                                            }
                                        }
                                        span { class: "text-[10px] text-muted-foreground", "Client Scope" }
                                    }
                                }

                                div { class: "flex flex-col gap-1.5",
                                    span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider",
                                        "Problem Description"
                                    }
                                    div { class: "rounded-md bg-muted/40 p-3 text-xs leading-relaxed text-foreground border border-border/40 font-mono whitespace-pre-wrap",
                                        if ticket.problem_description.is_empty() {
                                            "No detailed description provided."
                                        } else {
                                            "{ticket.problem_description}"
                                        }
                                    }
                                }

                                div { class: "flex flex-col gap-1.5",
                                    span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5",
                                        PaperclipIcon { class: "size-3.5 text-primary" }
                                        "Attached File"
                                    }
                                    if !ticket.attachment_name.is_empty() || !ticket.attachment_url.is_empty() {
                                        {
                                            let att_name = ticket.attachment_name.clone();
                                            let att_url = ticket.attachment_url.clone();
                                            let img_preview = get_image_preview_url(&att_name, &att_url);
                                            rsx! {
                                                div { class: "flex flex-col gap-2 rounded-md bg-muted/30 border border-border/60 p-3",
                                                    div { class: "flex items-center justify-between",
                                                        div { class: "flex items-center gap-2.5 min-w-0",
                                                            div { class: "flex size-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary",
                                                                FileTextIcon { class: "size-4" }
                                                            }
                                                            div { class: "flex flex-col min-w-0",
                                                                span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                                                    "{att_name}"
                                                                }
                                                                span { class: "text-[10px] text-muted-foreground font-mono truncate",
                                                                    "{att_url}"
                                                                }
                                                            }
                                                        }
                                                        div { class: "flex items-center gap-2 shrink-0",
                                                            span { class: "text-[10px] bg-primary/10 text-primary px-2 py-0.5 rounded font-medium",
                                                                "Attached"
                                                            }
                                                        }
                                                    }
                                                    if let Some(preview_src) = img_preview {
                                                        div { class: "mt-1 flex flex-col items-center justify-center rounded-md border border-border/70 bg-card p-2 shadow-xs",
                                                            img {
                                                                src: "{preview_src}",
                                                                alt: "{att_name}",
                                                                class: "max-h-72 max-w-full rounded object-contain",
                                                                loading: "lazy",
                                                            }
                                                            if !att_url.is_empty() {
                                                                a {
                                                                    href: "{att_url}",
                                                                    target: "_blank",
                                                                    class: "mt-2 inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                                                    ExternalLinkIcon { class: "size-3" }
                                                                    "Open full image in browser"
                                                                }
                                                            }
                                                        }
                                                    } else if !att_url.is_empty() {
                                                        a {
                                                            href: "{att_url}",
                                                            target: "_blank",
                                                            class: "inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                                            ExternalLinkIcon { class: "size-3" }
                                                            "Open attachment link"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        div { class: "rounded-md bg-muted/20 border border-border/40 p-2.5 text-xs text-muted-foreground flex items-center gap-2",
                                            PaperclipIcon { class: "size-3.5 text-muted-foreground/60" }
                                            "No file attached to this ticket."
                                        }
                                    }
                                }

                                div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Target Deadline"
                                        }
                                        span { class: "font-mono font-semibold text-foreground text-xs truncate", title: "{ticket.deadline}",
                                            "{disp_deadline}"
                                        }
                                        span { class: "text-[10px] text-muted-foreground", "Resolution Target" }
                                    }
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "SLA Deadline"
                                        }
                                        span { class: "font-mono font-semibold text-foreground text-xs truncate", title: "{ticket.sla_deadline}",
                                            "{disp_sla_deadline}"
                                        }
                                        span { class: "text-[10px] text-muted-foreground", "SLA Breach Target" }
                                    }
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Time Spent"
                                        }
                                        span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                            "{ticket.time_spent:.1} hrs"
                                        }
                                        span { class: "text-[10px] text-muted-foreground", "Logged Work" }
                                    }
                                    div { class: "flex flex-col gap-0.5 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                            "Rating Avg"
                                        }
                                        span { class: "font-semibold text-foreground text-xs truncate",
                                            if ticket.rating_avg_text.is_empty() {
                                                "Unrated"
                                            } else {
                                                "{ticket.rating_avg_text}"
                                            }
                                        }
                                        span { class: "text-[10px] text-muted-foreground", "Client Feedback" }
                                    }
                                }

                                if !ticket.activities.is_empty() || !ticket.my_activity.is_empty() {
                                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2.5",
                                        if !ticket.activities.is_empty() {
                                            div { class: "flex flex-col gap-1 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                                span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                    "Logged Activities"
                                                }
                                                span { class: "text-xs text-foreground/90 leading-relaxed", "{ticket.activities}" }
                                            }
                                        }
                                        if !ticket.my_activity.is_empty() {
                                            div { class: "flex flex-col gap-1 rounded-md bg-muted/20 p-2.5 border border-border/40",
                                                span { class: "text-[9px] font-semibold text-muted-foreground uppercase tracking-wider",
                                                    "My Next Activity"
                                                }
                                                span { class: "text-xs text-foreground/90 leading-relaxed", "{ticket.my_activity}" }
                                            }
                                        }
                                    }
                                }
                            }

                            DialogFooter { class: "flex items-center justify-between gap-2 pt-3 border-t border-border/50",
                                Button {
                                    variant: "outline",
                                    size: "sm",
                                    onclick: move |_| props.open.set(false),
                                    "Close"
                                }
                                Button {
                                    variant: "default",
                                    size: "sm",
                                    class: "gap-1.5",
                                    onclick: move |_| state.open_edit(t_for_edit.clone()),
                                    PencilIcon { class: "size-3.5" }
                                    "Modify Ticket"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
