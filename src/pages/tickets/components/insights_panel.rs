use dioxus::prelude::*;
use crate::pages::tickets::types::TicketItem;
use crate::state::is_date_today;

#[derive(Props, Clone, PartialEq)]
pub struct InsightsPanelProps {
    pub tickets: Signal<Vec<TicketItem>>,
}

#[component]
pub fn InsightsPanel(props: InsightsPanelProps) -> Element {
    let tickets_read = props.tickets.read();

    let today_pending = tickets_read
        .iter()
        .filter(|t| {
            is_date_today(&t.created_on)
                && (t.kanban_state == "Pending" || t.stage.eq_ignore_ascii_case("Pending"))
        })
        .count();

    let today_in_progress = tickets_read
        .iter()
        .filter(|t| {
            is_date_today(&t.created_on)
                && (t.kanban_state == "InProgress"
                    || t.stage.eq_ignore_ascii_case("In Progress")
                    || t.stage.eq_ignore_ascii_case("InProgress"))
        })
        .count();

    let today_closed = tickets_read
        .iter()
        .filter(|t| {
            is_date_today(&t.last_updated_on)
                && (t.status.eq_ignore_ascii_case("Closed")
                    || t.kanban_state == "Completed"
                    || t.kanban_state == "Solved"
                    || t.stage.eq_ignore_ascii_case("Solved(closed)")
                    || t.stage.eq_ignore_ascii_case("Solved")
                    || t.stage.eq_ignore_ascii_case("Closed"))
        })
        .count();

    rsx! {
        div { class: "grid grid-cols-1 sm:grid-cols-3 gap-3 w-full shrink-0",
            // Today Pending Card
            div { class: "rounded-lg border border-amber-500/30 bg-card p-3 shadow-xs flex flex-col justify-between hover:border-amber-500/50 transition-colors",
                div { class: "flex items-center justify-between",
                    span { class: "text-[11px] font-bold uppercase tracking-wider text-amber-600 dark:text-amber-400",
                        "Today Pending"
                    }
                    span { class: "size-2 rounded-full bg-amber-500" }
                }
                div { class: "mt-1.5 flex items-baseline gap-2",
                    span { class: "text-2xl font-extrabold text-foreground tracking-tight font-mono",
                        "{today_pending}"
                    }
                    span { class: "text-[11px] text-muted-foreground", "Tickets created today" }
                }
            }

            // Today In Progress Card
            div { class: "rounded-lg border border-blue-500/30 bg-card p-3 shadow-xs flex flex-col justify-between hover:border-blue-500/50 transition-colors",
                div { class: "flex items-center justify-between",
                    span { class: "text-[11px] font-bold uppercase tracking-wider text-blue-600 dark:text-blue-400",
                        "Today In Progress"
                    }
                    span { class: "size-2 rounded-full bg-blue-500 animate-pulse" }
                }
                div { class: "mt-1.5 flex items-baseline gap-2",
                    span { class: "text-2xl font-extrabold text-foreground tracking-tight font-mono",
                        "{today_in_progress}"
                    }
                    span { class: "text-[11px] text-muted-foreground", "Tickets created today" }
                }
            }

            // Today Closed Card
            div { class: "rounded-lg border border-emerald-500/30 bg-card p-3 shadow-xs flex flex-col justify-between hover:border-emerald-500/50 transition-colors",
                div { class: "flex items-center justify-between",
                    span { class: "text-[11px] font-bold uppercase tracking-wider text-emerald-600 dark:text-emerald-400",
                        "Today Closed"
                    }
                    span { class: "size-2 rounded-full bg-emerald-500" }
                }
                div { class: "mt-1.5 flex items-baseline gap-2",
                    span { class: "text-2xl font-extrabold text-foreground tracking-tight font-mono",
                        "{today_closed}"
                    }
                    span { class: "text-[11px] text-muted-foreground", "Tickets closed today" }
                }
            }
        }
    }
}
