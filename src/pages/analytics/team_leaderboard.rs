//! IT Support Team & Assignee Leaderboard.
//!
//! Built with shadcn UI components (Card, Badge).

use dioxus::prelude::*;
use crate::components::ui::badge::Badge;
use crate::components::ui::card::{Card, CardContent, CardHeader, CardTitle};
use crate::icons::UsersIcon;
use crate::pages::tickets::{assignee_color_classes, get_assignee_initials};
use super::types::AssigneeStats;

#[derive(Props, Clone, PartialEq)]
pub struct TeamLeaderboardProps {
    pub assignee_stats: Vec<AssigneeStats>,
}

#[component]
pub fn TeamLeaderboard(props: TeamLeaderboardProps) -> Element {
    let stats = &props.assignee_stats;

    rsx! {
        Card { class: "flex flex-col justify-between transition-all duration-200 hover:shadow-sm overflow-hidden",
            CardHeader { class: "flex flex-row items-center justify-between border-b border-border/60 p-5 pb-3",
                div { class: "flex items-center gap-2.5",
                    div { class: "flex size-8 shrink-0 items-center justify-center rounded-lg bg-primary/10 text-primary border border-primary/20",
                        UsersIcon { class: "size-4" }
                    }
                    div { class: "flex flex-col gap-0.5",
                        CardTitle { class: "text-sm font-bold text-foreground", "IT Support Team & Assignee Workload" }
                        p { class: "text-[11px] text-muted-foreground", "Resolution rates, backlog, and logged effort" }
                    }
                }
                Badge { variant: "secondary", class: "font-mono text-xs",
                    "{stats.len()} specialists"
                }
            }

            CardContent { class: "p-5 pt-3 overflow-x-auto",
                table { class: "w-full text-left text-xs border-collapse",
                    thead { class: "border-b border-border/80 bg-muted/40 text-[11px] font-bold text-muted-foreground uppercase tracking-wider",
                        tr {
                            th { class: "py-2.5 px-3", "Assignee / Specialist" }
                            th { class: "py-2.5 px-3 text-center", "Total Assigned" }
                            th { class: "py-2.5 px-3 text-center", "Solved" }
                            th { class: "py-2.5 px-3 text-center", "In Progress" }
                            th { class: "py-2.5 px-3 text-center", "Pending / Hold" }
                            th { class: "py-2.5 px-3 text-center", "Resolution Rate" }
                            th { class: "py-2.5 px-3 text-right", "Logged Hours" }
                        }
                    }
                    tbody { class: "divide-y divide-border/40 font-sans",
                        if stats.is_empty() {
                            tr {
                                td { colspan: "7", class: "py-8 text-center text-muted-foreground",
                                    "No assignee records found for the active filter."
                                }
                            }
                        } else {
                            for agent in stats.iter() {
                                tr { key: "{agent.name}", class: "hover:bg-muted/30 transition-colors",
                                    td { class: "py-3 px-3 flex items-center gap-2.5 min-w-[160px]",
                                        span { class: "flex size-7 shrink-0 items-center justify-center rounded-full text-[10px] font-extrabold uppercase border shadow-2xs {assignee_color_classes(&agent.name)}",
                                            "{get_assignee_initials(&agent.name)}"
                                        }
                                        div { class: "flex flex-col min-w-0",
                                            span { class: "font-semibold text-foreground truncate", "{agent.name}" }
                                            if let Some(r) = agent.avg_rating {
                                                span { class: "text-[10px] text-amber-500 font-bold", "★ {r:.1} rating" }
                                            }
                                        }
                                    }
                                    td { class: "py-3 px-3 text-center font-bold font-mono text-foreground", "{agent.total}" }
                                    td { class: "py-3 px-3 text-center font-semibold font-mono text-emerald-600 dark:text-emerald-400", "{agent.solved}" }
                                    td { class: "py-3 px-3 text-center font-semibold font-mono text-blue-600 dark:text-blue-400", "{agent.in_progress}" }
                                    td { class: "py-3 px-3 text-center font-medium font-mono text-muted-foreground", "{agent.pending + agent.hold}" }
                                    td { class: "py-3 px-3 text-center",
                                        div { class: "flex items-center justify-center gap-2",
                                            span { class: "font-mono font-bold text-xs text-foreground", "{agent.resolution_rate:.1}%" }
                                            div { class: "w-14 h-1.5 rounded-full bg-muted overflow-hidden hidden sm:block",
                                                div {
                                                    class: "h-full rounded-full bg-emerald-500 transition-all duration-300",
                                                    style: "width: {agent.resolution_rate}%;",
                                                }
                                            }
                                        }
                                    }
                                    td { class: "py-3 px-3 text-right font-mono font-bold text-muted-foreground",
                                        "{agent.total_hours:.1}h"
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
