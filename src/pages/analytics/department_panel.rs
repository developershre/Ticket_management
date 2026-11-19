//! Department Ticket Demand & Operational Ecosystem Summary.
//!
//! Built with shadcn UI components (Card, Badge).

use dioxus::prelude::*;
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::icons::{AwardIcon, CheckCircle2Icon, UsersIcon};
use super::types::DepartmentStat;

#[derive(Props, Clone, PartialEq)]
pub struct DepartmentPanelProps {
    pub department_stats: Vec<DepartmentStat>,
    pub total_employees: usize,
    pub total_reminders: usize,
    pub total_notes: usize,
    pub sla_compliance_rate: f64,
}

#[component]
pub fn DepartmentPanel(props: DepartmentPanelProps) -> Element {
    let dept_stats = &props.department_stats;

    rsx! {
        div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6 pb-8",

            // 1. Department Distribution
            Card { class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-hidden",
                CardHeader { class: "flex flex-row items-center justify-between border-b border-border/60 p-5 pb-3",
                    div { class: "flex items-center gap-2.5",
                        div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary border border-primary/20",
                            UsersIcon { class: "size-4" }
                        }
                        div { class: "flex flex-col gap-0.5",
                            CardTitle { class: "text-sm font-bold text-foreground", "Top Requesting Departments" }
                            p { class: "text-[11px] text-muted-foreground", "Internal department ticket origins" }
                        }
                    }
                    Badge { variant: "secondary", class: "font-mono text-xs",
                        "Ticket Origins"
                    }
                }

                CardContent { class: "p-5 flex flex-col gap-3 pt-4",
                    if dept_stats.is_empty() {
                        div { class: "py-8 text-center text-xs text-muted-foreground",
                            "No department records available."
                        }
                    } else {
                        for dept in dept_stats.iter() {
                            div { key: "{dept.name}", class: "flex flex-col gap-1",
                                div { class: "flex items-center justify-between text-xs",
                                    span { class: "font-semibold text-foreground/90 truncate", "{dept.name}" }
                                    div { class: "flex items-center gap-2 shrink-0 font-mono text-[11px]",
                                        span { class: "font-bold text-foreground", "{dept.count}" }
                                        span { class: "text-muted-foreground", "({dept.percentage:.1}%)" }
                                    }
                                }
                                div { class: "h-2 w-full rounded-full bg-muted overflow-hidden",
                                    div {
                                        class: "h-full rounded-full bg-blue-600/80 transition-all duration-300",
                                        style: "width: {dept.percentage}%;",
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Operational Ecosystem Health & SLA Target
            Card { class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-hidden",
                CardHeader { class: "flex flex-row items-center justify-between border-b border-border/60 p-5 pb-3",
                    div { class: "flex items-center gap-2.5",
                        div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-500 border border-emerald-500/20",
                            CheckCircle2Icon { class: "size-4" }
                        }
                        div { class: "flex flex-col gap-0.5",
                            CardTitle { class: "text-sm font-bold text-foreground", "Operational Ecosystem" }
                            p { class: "text-[11px] text-muted-foreground", "Database & sync cluster health" }
                        }
                    }
                    Badge { variant: "secondary", class: "font-mono text-xs",
                        "SQLite & Sheets"
                    }
                }

                CardContent { class: "p-5 flex flex-col gap-4 justify-between flex-1 pt-4",
                    div { class: "grid grid-cols-2 gap-3",
                        div { class: "flex flex-col p-3 rounded-lg border border-border/80 bg-muted/20",
                            span { class: "text-[10px] font-bold uppercase text-muted-foreground", "Employee Directory" }
                            span { class: "text-xl font-black text-foreground mt-1", "{props.total_employees}" }
                            span { class: "text-[10px] text-muted-foreground", "Synced staff profiles" }
                        }
                        div { class: "flex flex-col p-3 rounded-lg border border-border/80 bg-muted/20",
                            span { class: "text-[10px] font-bold uppercase text-muted-foreground", "Active Reminders" }
                            span { class: "text-xl font-black text-foreground mt-1", "{props.total_reminders}" }
                            span { class: "text-[10px] text-muted-foreground", "Scheduled notifications" }
                        }
                        div { class: "flex flex-col p-3 rounded-lg border border-border/80 bg-muted/20",
                            span { class: "text-[10px] font-bold uppercase text-muted-foreground", "Knowledge Notes" }
                            span { class: "text-xl font-black text-foreground mt-1", "{props.total_notes}" }
                            span { class: "text-[10px] text-muted-foreground", "Technical knowledge base" }
                        }
                        div { class: "flex flex-col p-3 rounded-lg border border-border/80 bg-muted/20",
                            span { class: "text-[10px] font-bold uppercase text-muted-foreground", "Sync Engine Status" }
                            span { class: "text-xl font-black text-emerald-600 dark:text-emerald-400 mt-1", "Healthy" }
                            span { class: "text-[10px] text-muted-foreground", "Real-time bidirectional sync" }
                        }
                    }

                    div { class: "rounded-lg border border-primary/20 bg-primary/5 p-3 flex items-center justify-between text-xs",
                        div { class: "flex items-center gap-2",
                            AwardIcon { class: "size-4 text-primary" }
                            span { class: "font-semibold text-foreground", "Helpdesk SLA Objective" }
                        }
                        span { class: "font-mono font-bold text-primary", "Target 95.0% · Met: {props.sla_compliance_rate:.1}%" }
                    }
                }
            }
        }
    }
}
