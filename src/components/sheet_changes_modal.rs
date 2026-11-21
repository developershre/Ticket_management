use dioxus::prelude::*;
use crate::components::ui::button::Button;
use crate::components::ui::dialog::{Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use crate::icons::RefreshIcon;

#[derive(Props, Clone, PartialEq)]
pub struct SheetChangesModalProps {
    pub on_close: EventHandler<()>,
}

#[component]
pub fn SheetChangesModal(props: SheetChangesModalProps) -> Element {
    let mut search_query = use_signal(String::new);
    let mut expanded_change_id = use_signal(|| Option::<i64>::None);
    let mut refresh_rev = use_signal(|| 0);
    let mut is_open = use_signal(|| true);

    let changes = use_memo(move || {
        let _ = refresh_rev.read();
        crate::db::get_recent_sheet_changes(100)
    });

    let q = search_query.read().trim().to_lowercase();
    let filtered_changes: Vec<crate::db::SheetChangeRecord> = changes
        .read()
        .iter()
        .filter(|c| {
            if q.is_empty() {
                return true;
            }
            c.entity_id.to_lowercase().contains(&q)
                || c.entity_title.to_lowercase().contains(&q)
                || c.field_name.to_lowercase().contains(&q)
                || c.old_value.to_lowercase().contains(&q)
                || c.new_value.to_lowercase().contains(&q)
                || c.pulled_at.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    let total_count = changes.read().len();

    let on_close = props.on_close;

    rsx! {
        Dialog {
            open: is_open,
            DialogContent {
                class: "max-w-4xl sm:max-w-4xl w-full rounded-2xl border border-border bg-card p-6 shadow-2xl max-h-[90vh] flex flex-col gap-4 overflow-hidden",
                DialogHeader { class: "flex items-center justify-between pb-2 border-b border-border/50 shrink-0",
                    div { class: "flex flex-col gap-0.5",
                        div { class: "flex items-center gap-2",
                            DialogTitle { class: "text-lg font-bold tracking-tight text-foreground flex items-center gap-2",
                                RefreshIcon { class: "size-4 text-primary" }
                                "Recent Google Sheet Changes"
                            }
                            span { class: "text-[10px] font-mono font-bold px-2 py-0.5 rounded-full bg-primary/10 text-primary border border-primary/20",
                                "{total_count} changes"
                            }
                        }
                        DialogDescription { class: "text-xs text-muted-foreground",
                            "Live field diffs, stage activities, and values captured during Google Sheets synchronization."
                        }
                    }
                }

                // Controls row: Search filter & Clear History
                div { class: "flex items-center justify-between gap-3 shrink-0",
                    div { class: "relative flex-1 max-w-sm",
                        input {
                            r#type: "text",
                            class: "h-8 w-full rounded-md border border-input bg-background px-3 text-xs text-foreground placeholder:text-muted-foreground focus:outline-none focus:ring-1 focus:ring-primary",
                            placeholder: "Search by ticket ID, field, or value...",
                            value: search_query.read().clone(),
                            oninput: move |e: FormEvent| search_query.set(e.value()),
                        }
                    }
                    div { class: "flex items-center gap-2",
                        Button {
                            variant: "outline",
                            size: "sm",
                            class: "h-8 px-2.5 text-xs text-muted-foreground hover:text-destructive cursor-pointer",
                            onclick: move |_| {
                                let _ = crate::db::clear_sheet_changes();
                                refresh_rev.with_mut(|r| *r += 1);
                            },
                            "Clear History"
                        }
                    }
                }

                // Changes list
                div { class: "flex-1 min-h-[300px] overflow-y-auto pr-1 flex flex-col gap-2.5",
                    if filtered_changes.is_empty() {
                        div { class: "py-12 text-center text-xs text-muted-foreground bg-muted/10 rounded-xl border border-dashed border-border flex flex-col items-center justify-center gap-1.5",
                            span { class: "font-semibold text-foreground/80", "No remote changes found" }
                            span { "Any remote updates pulled from your Google Sheet will be recorded here with before and after values." }
                        }
                    } else {
                        for item in filtered_changes.iter() {
                            {
                                let cid = item.change_id;
                                let is_expanded = *expanded_change_id.read() == Some(cid);
                                let local_num = crate::pages::tickets::types::parse_ticket_num(&item.entity_id);
                                let full_ticket = local_num.and_then(|num| crate::db::get_ticket_by_id(num).ok().flatten());

                                rsx! {
                                    div {
                                        key: "{cid}",
                                        class: "rounded-xl border border-border/70 bg-card/80 p-3 shadow-2xs flex flex-col gap-2 transition-all hover:border-primary/40",
                                        
                                        // Main Change Row matching user's Image 2
                                        div { class: "flex flex-wrap items-center justify-between gap-2 text-xs",
                                            div { class: "flex flex-wrap items-center gap-2 min-w-0 flex-1",
                                                span { class: "size-5 rounded-full bg-muted flex items-center justify-center shrink-0 text-muted-foreground",
                                                    RefreshIcon { class: "size-3 text-muted-foreground" }
                                                }
                                                span { class: "font-mono font-bold text-foreground shrink-0 text-[11px] bg-muted/50 px-1.5 py-0.5 rounded",
                                                    "{item.entity_id}"
                                                }
                                                span { class: "font-medium text-muted-foreground truncate max-w-[150px] text-[11px]",
                                                    "{item.entity_title}"
                                                }
                                                span { class: "rounded bg-muted px-2 py-0.5 text-[10px] font-semibold text-muted-foreground shrink-0",
                                                    "{item.field_name}"
                                                }
                                                
                                                // Value Transition
                                                div { class: "inline-flex items-center gap-1.5 shrink-0",
                                                    span { class: "line-through text-rose-500 bg-rose-500/10 px-2 py-0.5 rounded text-[11px] font-mono",
                                                        if item.old_value.is_empty() { "—" } else { "{item.old_value}" }
                                                    }
                                                    span { class: "text-muted-foreground font-bold", "→" }
                                                    span { class: "text-emerald-500 bg-emerald-500/10 px-2 py-0.5 rounded text-[11px] font-semibold font-mono",
                                                        if item.new_value.is_empty() { "—" } else { "{item.new_value}" }
                                                    }
                                                }
                                            }

                                            div { class: "flex items-center gap-2 shrink-0 ml-auto",
                                                span { class: "text-[10px] font-mono text-muted-foreground",
                                                    "({item.pulled_at})"
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: "px-2.5 py-1 rounded bg-slate-900 dark:bg-slate-100 text-white dark:text-slate-950 font-bold text-[11px] cursor-pointer hover:opacity-90 shadow-2xs transition-opacity",
                                                    onclick: move |_| {
                                                        expanded_change_id.with_mut(|opt| {
                                                            if *opt == Some(cid) {
                                                                *opt = None;
                                                            } else {
                                                                *opt = Some(cid);
                                                            }
                                                        });
                                                    },
                                                    if is_expanded { "Hide Values" } else { "View All Values" }
                                                }
                                            }
                                        }

                                        // Expanded All Values Drawer
                                        if is_expanded {
                                            div { class: "mt-2 pt-2 border-t border-border/40 grid grid-cols-2 sm:grid-cols-4 gap-2 text-[11px] bg-muted/20 p-2.5 rounded-lg",
                                                if let Some(ref t) = full_ticket {
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Stage / Activity" }
                                                        span { class: "font-mono font-bold text-foreground", if t.stage.is_empty() { "N/A" } else { "{t.stage}" } }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "My Activity" }
                                                        span { class: "font-mono font-bold text-foreground", if t.my_activity.is_empty() { "N/A" } else { "{t.my_activity}" } }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Status" }
                                                        span { class: "font-mono font-bold text-foreground", "{t.status}" }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Kanban Column" }
                                                        span { class: "font-mono font-bold text-foreground", "{t.kanban_state}" }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Priority" }
                                                        span { class: "font-mono font-bold text-foreground", "{t.priority}" }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Assigned To" }
                                                        span { class: "font-mono font-bold text-foreground", if t.assigned_to.is_empty() { "Unassigned" } else { "{t.assigned_to}" } }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Requester" }
                                                        span { class: "font-mono font-bold text-foreground", "{t.employee_code}" }
                                                    }
                                                    div { class: "flex flex-col gap-0.5",
                                                        span { class: "text-[9px] font-semibold text-muted-foreground uppercase", "Last Updated" }
                                                        span { class: "font-mono font-bold text-foreground", "{t.last_updated_on}" }
                                                    }
                                                } else {
                                                    div { class: "col-span-full text-xs text-muted-foreground italic",
                                                        "Ticket details no longer found in local database."
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

                DialogFooter { class: "flex items-center justify-end gap-2 pt-3 border-t border-border/50 shrink-0",
                    Button {
                        variant: "outline",
                        size: "sm",
                        onclick: move |_| {
                            is_open.set(false);
                            on_close.call(());
                        },
                        "Close"
                    }
                }
            }
        }
    }
}
