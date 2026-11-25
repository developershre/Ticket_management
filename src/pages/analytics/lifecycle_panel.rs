//! Ticket Lifecycle & Priority Severity Panels.
//!
//! Built with shadcn UI components (Card, Badge).

use dioxus::prelude::*;
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::icons::{AlertTriangleIcon, CheckCircle2Icon, PieChartIcon};

#[derive(Props, Clone, PartialEq)]
pub struct LifecyclePanelProps {
    pub total_count: usize,
    pub solved_count: usize,
    pub in_progress_count: usize,
    pub pending_count: usize,
    pub hold_count: usize,
    pub cancelled_count: usize,
    pub crit_count: usize,
    pub high_count: usize,
    pub med_count: usize,
    pub low_count: usize,
    pub sla_met_count: usize,
    pub sla_breached_count: usize,
    pub sla_compliance_rate: f64,
}

#[component]
pub fn LifecyclePanel(props: LifecyclePanelProps) -> Element {
    let total = props.total_count.max(1) as f64;

    let sla_summary_text = if props.total_count == 0 {
        "No SLA records evaluated".to_string()
    } else {
        format!("{} met · {} breached", props.sla_met_count, props.sla_breached_count)
    };

    rsx! {
        div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",

            // 1. Status Breakdown Box
            Card { class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-hidden",
                CardHeader { class: "flex flex-row items-center justify-between border-b border-border/60 p-5 pb-3",
                    div { class: "flex items-center gap-2.5",
                        div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary border border-primary/20",
                            PieChartIcon { class: "size-4" }
                        }
                        div { class: "flex flex-col gap-0.5",
                            CardTitle { class: "text-sm font-bold text-foreground", "Ticket Lifecycle Distribution" }
                            p { class: "text-[11px] text-muted-foreground", "Progress state from intake to closure" }
                        }
                    }
                    Badge { variant: "secondary", class: "font-mono text-xs",
                        "{props.total_count} total"
                    }
                }

                CardContent { class: "p-5 flex flex-col gap-4",
                    // Stacked visual bar
                    if props.total_count == 0 {
                        div { class: "h-3.5 w-full rounded-md bg-muted/60 border border-border/40 flex items-center justify-center text-[10px] text-muted-foreground font-medium",
                            "No tickets in selected period"
                        }
                    } else {
                        div { class: "h-3.5 w-full rounded-md bg-muted overflow-hidden flex shadow-inner",
                            if props.solved_count > 0 {
                                div {
                                    class: "bg-emerald-500 h-full transition-all duration-300",
                                    style: "width: {(props.solved_count as f64 / total) * 100.0}%;",
                                    title: "Solved: {props.solved_count}",
                                }
                            }
                            if props.in_progress_count > 0 {
                                div {
                                    class: "bg-blue-500 h-full transition-all duration-300",
                                    style: "width: {(props.in_progress_count as f64 / total) * 100.0}%;",
                                    title: "In Progress: {props.in_progress_count}",
                                }
                            }
                            if props.hold_count > 0 {
                                div {
                                    class: "bg-amber-500 h-full transition-all duration-300",
                                    style: "width: {(props.hold_count as f64 / total) * 100.0}%;",
                                    title: "On Hold: {props.hold_count}",
                                }
                            }
                            if props.pending_count > 0 {
                                div {
                                    class: "bg-sky-500 h-full transition-all duration-300",
                                    style: "width: {(props.pending_count as f64 / total) * 100.0}%;",
                                    title: "Pending: {props.pending_count}",
                                }
                            }
                            if props.cancelled_count > 0 {
                                div {
                                    class: "bg-rose-500 h-full transition-all duration-300",
                                    style: "width: {(props.cancelled_count as f64 / total) * 100.0}%;",
                                    title: "Cancelled: {props.cancelled_count}",
                                }
                            }
                        }
                    }

                    // Status metric items
                    div { class: "grid grid-cols-2 sm:grid-cols-5 gap-2.5 pt-1",
                        div { class: "flex flex-col p-2.5 rounded-lg border border-emerald-500/25 bg-emerald-500/5",
                            div { class: "flex items-center gap-1.5",
                                span { class: "size-2 rounded-full bg-emerald-500" }
                                span { class: "text-[11px] font-semibold text-emerald-600 dark:text-emerald-400", "Solved" }
                            }
                            span { class: "text-lg font-black text-foreground mt-1", "{props.solved_count}" }
                            span { class: "text-[10px] text-muted-foreground",
                                "{(props.solved_count as f64 / total * 100.0):.1}%"
                            }
                        }

                        div { class: "flex flex-col p-2.5 rounded-lg border border-blue-500/25 bg-blue-500/5",
                            div { class: "flex items-center gap-1.5",
                                span { class: "size-2 rounded-full bg-blue-500" }
                                span { class: "text-[11px] font-semibold text-blue-600 dark:text-blue-400", "In Progress" }
                            }
                            span { class: "text-lg font-black text-foreground mt-1", "{props.in_progress_count}" }
                            span { class: "text-[10px] text-muted-foreground",
                                "{(props.in_progress_count as f64 / total * 100.0):.1}%"
                            }
                        }

                        div { class: "flex flex-col p-2.5 rounded-lg border border-sky-500/25 bg-sky-500/5",
                            div { class: "flex items-center gap-1.5",
                                span { class: "size-2 rounded-full bg-sky-500" }
                                span { class: "text-[11px] font-semibold text-sky-600 dark:text-sky-400", "Pending" }
                            }
                            span { class: "text-lg font-black text-foreground mt-1", "{props.pending_count}" }
                            span { class: "text-[10px] text-muted-foreground",
                                "{(props.pending_count as f64 / total * 100.0):.1}%"
                            }
                        }

                        div { class: "flex flex-col p-2.5 rounded-lg border border-amber-500/25 bg-amber-500/5",
                            div { class: "flex items-center gap-1.5",
                                span { class: "size-2 rounded-full bg-amber-500" }
                                span { class: "text-[11px] font-semibold text-amber-600 dark:text-amber-400", "On Hold" }
                            }
                            span { class: "text-lg font-black text-foreground mt-1", "{props.hold_count}" }
                            span { class: "text-[10px] text-muted-foreground",
                                "{(props.hold_count as f64 / total * 100.0):.1}%"
                            }
                        }

                        div { class: "flex flex-col p-2.5 rounded-lg border border-border bg-muted/30",
                            div { class: "flex items-center gap-1.5",
                                span { class: "size-2 rounded-full bg-muted-foreground" }
                                span { class: "text-[11px] font-semibold text-muted-foreground", "Cancelled" }
                            }
                            span { class: "text-lg font-black text-foreground mt-1", "{props.cancelled_count}" }
                            span { class: "text-[10px] text-muted-foreground",
                                "{(props.cancelled_count as f64 / total * 100.0):.1}%"
                            }
                        }
                    }
                }
            }

            // 2. Priority Breakdown Box
            Card { class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-hidden",
                CardHeader { class: "flex flex-row items-center justify-between border-b border-border/60 p-5 pb-3",
                    div { class: "flex items-center gap-2.5",
                        div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-amber-500/10 text-amber-500 border border-amber-500/20",
                            AlertTriangleIcon { class: "size-4" }
                        }
                        div { class: "flex flex-col gap-0.5",
                            CardTitle { class: "text-sm font-bold text-foreground", "Priority Severity Breakdown" }
                            p { class: "text-[11px] text-muted-foreground", "Urgency tiers and SLA health" }
                        }
                    }
                    if props.crit_count > 0 {
                        Badge { variant: "destructive", class: "text-[10px] font-bold animate-pulse",
                            "{props.crit_count} Critical Tickets"
                        }
                    }
                }

                CardContent { class: "p-5 flex flex-col gap-4 pt-4",
                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2.5",
                        div { class: "flex flex-col p-3 rounded-lg border border-rose-500/25 bg-rose-500/5",
                            span { class: "text-[10px] font-extrabold uppercase tracking-wider text-rose-600 dark:text-rose-400", "Critical" }
                            span { class: "text-2xl font-black text-foreground mt-1", "{props.crit_count}" }
                            div { class: "mt-2 h-1.5 w-full rounded-full bg-rose-500/20 overflow-hidden",
                                div { class: "bg-rose-500 h-full rounded-full transition-all duration-300", style: "width: {(props.crit_count as f64 / total * 100.0)}%;" }
                            }
                        }

                        div { class: "flex flex-col p-3 rounded-lg border border-amber-500/25 bg-amber-500/5",
                            span { class: "text-[10px] font-extrabold uppercase tracking-wider text-amber-600 dark:text-amber-400", "High" }
                            span { class: "text-2xl font-black text-foreground mt-1", "{props.high_count}" }
                            div { class: "mt-2 h-1.5 w-full rounded-full bg-amber-500/20 overflow-hidden",
                                div { class: "bg-amber-500 h-full rounded-full transition-all duration-300", style: "width: {(props.high_count as f64 / total * 100.0)}%;" }
                            }
                        }

                        div { class: "flex flex-col p-3 rounded-lg border border-blue-500/25 bg-blue-500/5",
                            span { class: "text-[10px] font-extrabold uppercase tracking-wider text-blue-600 dark:text-blue-400", "Medium" }
                            span { class: "text-2xl font-black text-foreground mt-1", "{props.med_count}" }
                            div { class: "mt-2 h-1.5 w-full rounded-full bg-blue-500/20 overflow-hidden",
                                div { class: "bg-blue-500 h-full rounded-full transition-all duration-300", style: "width: {(props.med_count as f64 / total * 100.0)}%;" }
                            }
                        }

                        div { class: "flex flex-col p-3 rounded-lg border border-border bg-muted/30",
                            span { class: "text-[10px] font-extrabold uppercase tracking-wider text-muted-foreground", "Low" }
                            span { class: "text-2xl font-black text-foreground mt-1", "{props.low_count}" }
                            div { class: "mt-2 h-1.5 w-full rounded-full bg-muted overflow-hidden",
                                div { class: "bg-muted-foreground h-full rounded-full transition-all duration-300", style: "width: {(props.low_count as f64 / total * 100.0)}%;" }
                            }
                        }
                    }

                    // SLA Health Pill banner
                    div { class: "mt-1 flex items-center justify-between rounded-lg border border-border/80 bg-muted/20 p-3 text-xs",
                        div { class: "flex items-center gap-2",
                            CheckCircle2Icon {
                                class: if props.total_count == 0 { "size-4 text-muted-foreground" } else { "size-4 text-emerald-500" }
                            }
                            span { class: "font-semibold text-foreground", "SLA Adherence Compliance" }
                        }
                        div { class: "flex items-center gap-3",
                            span { class: "text-muted-foreground",
                                "{sla_summary_text}"
                            }
                            Badge {
                                variant: if props.total_count == 0 { "secondary" } else if props.sla_compliance_rate >= 95.0 { "success" } else { "warning" },
                                class: "font-mono font-bold",
                                "{props.sla_compliance_rate:.1}%"
                            }
                        }
                    }
                }
            }
        }
    }
}
