//! Executive Summary KPI Cards.
//!
//! 6-card high-impact operational metrics overview. Built with shadcn Card & Badge components.

use crate::components::ui::badge::Badge;
use crate::components::ui::card::Card;
use crate::icons::{CheckCircle2Icon, ClockIcon, TicketIcon, TimerIcon, TrendingUpIcon};
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct KpiCardsProps {
    pub total_count: usize,
    pub solved_count: usize,
    pub in_progress_count: usize,
    pub pending_count: usize,
    pub hold_count: usize,
    pub cancelled_count: usize,
    pub total_hours: f64,
    pub resolution_rate: f64,
    pub sla_compliance_rate: f64,
    pub sla_breached_count: usize,
    pub total_employees: usize,
    pub total_reminders: usize,
}

#[component]
pub fn KpiCards(props: KpiCardsProps) -> Element {
    let open_queue = props.pending_count + props.in_progress_count + props.hold_count;

    let sla_subtext = if props.sla_breached_count == 0 {
        "Zero breached deadlines".to_string()
    } else {
        format!("{} breaches recorded", props.sla_breached_count)
    };

    let avg_effort_subtext = if props.solved_count > 0 {
        format!(
            "{:.2}h avg per resolution",
            props.total_hours / props.solved_count as f64
        )
    } else {
        "0.00h avg per resolution".to_string()
    };

    rsx! {
        div { class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6 gap-4",

            // 1. Total Volume
            Card { class: "relative overflow-hidden p-4 flex flex-col justify-between hover:border-blue-500/40 transition-all duration-200 shadow-xs hover:shadow-sm",
                div { class: "absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-blue-500 to-indigo-500" }
                div { class: "flex items-center justify-between text-muted-foreground",
                    span { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground/80", "Total Volume" }
                    div { class: "flex size-8 items-center justify-center rounded-lg bg-blue-500/10 text-blue-500 border border-blue-500/20",
                        TicketIcon { class: "size-4" }
                    }
                }
                div { class: "mt-3",
                    span { class: "text-3xl font-black tracking-tight text-foreground tabular-nums", "{props.total_count}" }
                    p { class: "text-[11px] text-muted-foreground mt-1 flex items-center gap-1",
                        span { class: "font-semibold text-emerald-600 dark:text-emerald-400", "{props.solved_count} closed" }
                        span { "·" }
                        span { "{open_queue} open" }
                    }
                }
                div { class: "mt-3.5 flex items-center justify-between text-[11px] text-muted-foreground pt-2 border-t border-border/40",
                    span { "Total Intake" }
                    Badge { variant: "secondary", class: "text-[10px] font-bold px-1.5 py-0", "Lifecycle" }
                }
            }

            // 2. Resolution Rate %
            Card { class: "relative overflow-hidden p-4 flex flex-col justify-between hover:border-emerald-500/40 transition-all duration-200 shadow-xs hover:shadow-sm",
                div { class: "absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-emerald-500 to-teal-500" }
                div { class: "flex items-center justify-between text-muted-foreground",
                    span { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground/80", "Resolution Rate" }
                    div { class: "flex size-8 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-500 border border-emerald-500/20",
                        CheckCircle2Icon { class: "size-4" }
                    }
                }
                div { class: "mt-3",
                    div { class: "flex items-baseline gap-1.5",
                        span { class: "text-3xl font-black tracking-tight text-emerald-600 dark:text-emerald-400 tabular-nums",
                            "{props.resolution_rate:.1}%"
                        }
                    }
                    p { class: "text-[11px] text-muted-foreground mt-1",
                        "{props.solved_count} of {props.total_count} resolved"
                    }
                }
                div { class: "mt-3.5 flex flex-col gap-1.5 pt-2 border-t border-border/40",
                    div { class: "h-1.5 w-full rounded-full bg-muted overflow-hidden",
                        div {
                            class: "h-full rounded-full bg-emerald-500 transition-all duration-500",
                            style: "width: {props.resolution_rate}%;",
                        }
                    }
                }
            }

            // 3. In Progress Queue
            Card { class: "relative overflow-hidden p-4 flex flex-col justify-between hover:border-sky-500/40 transition-all duration-200 shadow-xs hover:shadow-sm",
                div { class: "absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-sky-500 to-blue-500" }
                div { class: "flex items-center justify-between text-muted-foreground",
                    span { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground/80", "In Progress" }
                    div { class: "flex size-8 items-center justify-center rounded-lg bg-sky-500/10 text-sky-500 border border-sky-500/20",
                        ClockIcon { class: "size-4" }
                    }
                }
                div { class: "mt-3",
                    span { class: "text-3xl font-black tracking-tight text-blue-600 dark:text-blue-400 tabular-nums",
                        "{props.in_progress_count}"
                    }
                    p { class: "text-[11px] text-muted-foreground mt-1",
                        "Actively being worked on"
                    }
                }
                div { class: "mt-3.5 flex items-center justify-between text-[11px] text-muted-foreground pt-2 border-t border-border/40",
                    span { "{props.hold_count} on hold" }
                    Badge { variant: "info", class: "text-[10px] font-bold px-1.5 py-0", "Active" }
                }
            }

            // 4. Pending Queue
            Card { class: "relative overflow-hidden p-4 flex flex-col justify-between hover:border-amber-500/40 transition-all duration-200 shadow-xs hover:shadow-sm",
                div { class: "absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-amber-500 to-orange-500" }
                div { class: "flex items-center justify-between text-muted-foreground",
                    span { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground/80", "Pending Queue" }
                    div { class: "flex size-8 items-center justify-center rounded-lg bg-amber-500/10 text-amber-500 border border-amber-500/20",
                        TimerIcon { class: "size-4" }
                    }
                }
                div { class: "mt-3",
                    span { class: "text-3xl font-black tracking-tight text-amber-600 dark:text-amber-400 tabular-nums",
                        "{props.pending_count}"
                    }
                    p { class: "text-[11px] text-muted-foreground mt-1",
                        "Awaiting triage / assignment"
                    }
                }
                div { class: "mt-3.5 flex items-center justify-between text-[11px] text-muted-foreground pt-2 border-t border-border/40",
                    span { "Triage Queue" }
                    Badge { variant: "warning", class: "text-[10px] font-bold px-1.5 py-0", "Review" }
                }
            }

            // 5. SLA Adherence %
            Card { class: "relative overflow-hidden p-4 flex flex-col justify-between hover:border-emerald-500/40 transition-all duration-200 shadow-xs hover:shadow-sm",
                div { class: "absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-emerald-500 to-green-500" }
                div { class: "flex items-center justify-between text-muted-foreground",
                    span { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground/80", "SLA Adherence" }
                    div { class: "flex size-8 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-500 border border-emerald-500/20",
                        CheckCircle2Icon { class: "size-4" }
                    }
                }
                div { class: "mt-3",
                    span {
                        class: if props.sla_compliance_rate >= 95.0 {
                            "text-3xl font-black tracking-tight text-emerald-600 dark:text-emerald-400 tabular-nums"
                        } else {
                            "text-3xl font-black tracking-tight text-amber-600 dark:text-amber-400 tabular-nums"
                        },
                        "{props.sla_compliance_rate:.1}%"
                    }
                    p { class: "text-[11px] text-muted-foreground mt-1",
                        "{sla_subtext}"
                    }
                }
                div { class: "mt-3.5 flex flex-col gap-1.5 pt-2 border-t border-border/40",
                    div { class: "h-1.5 w-full rounded-full bg-muted overflow-hidden",
                        div {
                            class: if props.sla_compliance_rate >= 95.0 { "bg-emerald-500 h-full rounded-full" } else { "bg-amber-500 h-full rounded-full" },
                            style: "width: {props.sla_compliance_rate}%;",
                        }
                    }
                }
            }

            // 6. Total Effort Logged
            Card { class: "relative overflow-hidden p-4 flex flex-col justify-between hover:border-purple-500/40 transition-all duration-200 shadow-xs hover:shadow-sm",
                div { class: "absolute top-0 inset-x-0 h-1 bg-gradient-to-r from-purple-500 to-pink-500" }
                div { class: "flex items-center justify-between text-muted-foreground",
                    span { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground/80", "Effort Logged" }
                    div { class: "flex size-7 items-center justify-center rounded-lg bg-purple-500/10 text-purple-500 border border-purple-500/20",
                        TrendingUpIcon { class: "size-4" }
                    }
                }
                div { class: "mt-3",
                    span { class: "text-3xl font-black tracking-tight text-purple-600 dark:text-purple-400 tabular-nums",
                        "{props.total_hours:.1}h"
                    }
                    p { class: "text-[11px] text-muted-foreground mt-1",
                        "{avg_effort_subtext}"
                    }
                }
                div { class: "mt-3.5 flex items-center justify-between text-[11px] text-muted-foreground pt-2 border-t border-border/40",
                    span { "{props.total_employees} staff synced" }
                    Badge { variant: "purple", class: "text-[10px] font-bold px-1.5 py-0", "Hours" }
                }
            }
        }
    }
}
