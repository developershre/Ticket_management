//! Tickets page — board + list views with KPI metrics, search & filters.

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::input::Input;
use crate::components::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};
use crate::icons::SearchIcon;

/// (id, title, priority, tag, assignee)
type Ticket = (&'static str, &'static str, &'static str, &'static str, &'static str);

/// (column name, tickets)
const COLUMNS: &[(&str, &[Ticket])] = &[
    (
        "Todo",
        &[
            ("TK-104", "Fix login redirect loop", "High", "Auth", "GR"),
            ("TK-108", "Update onboarding copy", "Low", "Content", "AL"),
            ("TK-111", "Audit settings page a11y", "Medium", "A11y", "JD"),
        ],
    ),
    (
        "In Progress",
        &[
            ("TK-101", "Design landing page", "Medium", "Design", "GR"),
            ("TK-115", "Refactor auth middleware", "High", "Security", "SL"),
        ],
    ),
    (
        "Done",
        &[
            ("TK-098", "Ship v1.2 release notes", "Medium", "Release", "AL"),
            ("TK-099", "Add system theme auto-detect", "Low", "Feature", "GR"),
        ],
    ),
];

fn priority_dot(priority: &str) -> &'static str {
    match priority {
        "High" => "bg-rose-500",
        "Medium" => "bg-amber-500",
        _ => "bg-blue-400",
    }
}

fn priority_badge(priority: &str) -> &'static str {
    match priority {
        "High" => "destructive",
        "Medium" => "secondary",
        _ => "outline",
    }
}

fn assignee_bg(initials: &str) -> &'static str {
    match initials {
        "GR" => "bg-indigo-600 text-white",
        "AL" => "bg-violet-600 text-white",
        "JD" => "bg-emerald-600 text-white",
        _ => "bg-sky-600 text-white",
    }
}

#[component]
pub fn Tickets() -> Element {
    let mut search_query = use_signal(String::new);
    let mut selected_priority = use_signal(|| "All".to_string());

    let q = search_query.read().to_lowercase();
    let filter_prio = selected_priority.read().clone();

    let open_count: usize = COLUMNS
        .iter()
        .filter(|(name, _)| *name != "Done")
        .map(|(_, tickets)| tickets.len())
        .sum();

    let in_progress_count = COLUMNS
        .iter()
        .find(|(name, _)| *name == "In Progress")
        .map(|(_, t)| t.len())
        .unwrap_or(0);

    let done_count = COLUMNS
        .iter()
        .find(|(name, _)| *name == "Done")
        .map(|(_, t)| t.len())
        .unwrap_or(0);

    let high_prio_count: usize = COLUMNS
        .iter()
        .flat_map(|(_, tickets)| tickets.iter())
        .filter(|(_, _, p, _, _)| *p == "High")
        .count();

    rsx! {
        div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6 p-6 pb-24",

            // Page header with actions
            div { class: "flex flex-wrap items-center justify-between gap-4 border-b border-border/50 pb-5",
                div {
                    div { class: "flex items-center gap-2.5",
                        h1 { class: "text-2xl font-bold tracking-tight text-foreground sm:text-3xl", "Tickets" }
                        span { class: "rounded-full bg-primary/15 px-2.5 py-0.5 text-xs font-semibold text-primary",
                            "{open_count} open"
                        }
                    }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Plan, track, and ship high-priority sprint deliverables."
                    }
                }
            }

            // Stat KPI Metrics
            div { class: "grid grid-cols-2 gap-3 sm:grid-cols-4",
                // Total open
                div { class: "flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-3.5 shadow-2xs transition-all hover:border-primary/40",
                    span { class: "text-xs font-medium text-muted-foreground", "Open Tickets" }
                    div { class: "flex items-baseline justify-between",
                        span { class: "text-2xl font-bold tracking-tight text-foreground", "{open_count}" }
                        span { class: "text-[11px] font-semibold text-indigo-500", "Sprint 24" }
                    }
                }
                // In progress
                div { class: "flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-3.5 shadow-2xs transition-all hover:border-primary/40",
                    span { class: "text-xs font-medium text-muted-foreground", "In Progress" }
                    div { class: "flex items-baseline justify-between",
                        span { class: "text-2xl font-bold tracking-tight text-amber-500", "{in_progress_count}" }
                        span { class: "text-[11px] font-semibold text-amber-500/80", "Active" }
                    }
                }
                // Completed
                div { class: "flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-3.5 shadow-2xs transition-all hover:border-primary/40",
                    span { class: "text-xs font-medium text-muted-foreground", "Completed" }
                    div { class: "flex items-baseline justify-between",
                        span { class: "text-2xl font-bold tracking-tight text-emerald-500", "{done_count}" }
                        span { class: "text-[11px] font-semibold text-emerald-500", "100% on time" }
                    }
                }
                // High priority
                div { class: "flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-3.5 shadow-2xs transition-all hover:border-primary/40",
                    span { class: "text-xs font-medium text-muted-foreground", "Urgent / High" }
                    div { class: "flex items-baseline justify-between",
                        span { class: "text-2xl font-bold tracking-tight text-rose-500", "{high_prio_count}" }
                        span { class: "text-[11px] font-semibold text-rose-500/80", "Attention" }
                    }
                }
            }

            // Search & Filter toolbar
            div { class: "flex flex-wrap items-center justify-between gap-3",
                div { class: "relative flex flex-1 min-w-[200px] max-w-sm items-center",
                    div { class: "pointer-events-none absolute left-3 flex items-center text-muted-foreground",
                        SearchIcon { class: "size-4" }
                    }
                    Input {
                        class: "pl-9 text-xs h-9 w-full",
                        placeholder: "Filter tickets by title, tag, ID...".to_string(),
                        value: search_query.read().clone(),
                        oninput: move |e: FormEvent| search_query.set(e.value()),
                    }
                }

                div { class: "flex items-center gap-1.5",
                    span { class: "text-xs font-medium text-muted-foreground mr-1", "Priority:" }
                    for prio in ["All", "High", "Medium", "Low"] {
                        {
                            let is_sel = filter_prio == prio;
                            let sel_cls = if is_sel {
                                "bg-primary text-primary-foreground font-semibold shadow-xs"
                            } else {
                                "bg-muted/70 text-muted-foreground hover:text-foreground hover:bg-muted"
                            };
                            rsx! {
                                button {
                                    r#type: "button",
                                    key: "prio-{prio}",
                                    class: "rounded-lg px-2.5 py-1 text-xs transition-colors cursor-pointer {sel_cls}",
                                    onclick: move |_| selected_priority.set(prio.to_string()),
                                    "{prio}"
                                }
                            }
                        }
                    }
                }
            }

            Tabs { value: "board".to_string(),
                TabsList {
                    TabsTrigger { value: "board".to_string(), "Kanban Board" }
                    TabsTrigger { value: "list".to_string(), "Table List" }
                }

                // --------------------------------------------------------
                // Board view
                // --------------------------------------------------------
                TabsContent { value: "board".to_string(),
                    div { class: "grid gap-4 md:grid-cols-3",
                        for (column, tickets) in COLUMNS {
                            {
                                let (col_dot, col_border) = match *column {
                                    "Todo" => ("bg-blue-500", "border-blue-500/20"),
                                    "In Progress" => ("bg-amber-500 animate-pulse", "border-amber-500/20"),
                                    _ => ("bg-emerald-500", "border-emerald-500/20"),
                                };

                                let filtered_tickets: Vec<_> = tickets
                                    .iter()
                                    .filter(|(id, title, priority, tag, assignee)| {
                                        let match_q = q.is_empty()
                                            || id.to_lowercase().contains(&q)
                                            || title.to_lowercase().contains(&q)
                                            || tag.to_lowercase().contains(&q)
                                            || assignee.to_lowercase().contains(&q);
                                        let match_prio = filter_prio == "All" || *priority == filter_prio;
                                        match_q && match_prio
                                    })
                                    .collect();

                                rsx! {
                                    div { key: "board-col-{column}",
                                        class: "flex flex-col gap-3 rounded-2xl border {col_border} bg-card/60 p-3.5 backdrop-blur-xs",
                                        div { class: "flex items-center justify-between px-1",
                                            div { class: "flex items-center gap-2",
                                                span { class: "size-2 rounded-full {col_dot}" }
                                                h2 { class: "text-xs font-bold uppercase tracking-wider text-foreground",
                                                    "{column}"
                                                }
                                            }
                                            Badge { variant: "outline", class: "text-[10px] font-mono tabular-nums",
                                                "{filtered_tickets.len()}"
                                            }
                                        }

                                        div { class: "flex flex-col gap-2.5",
                                            for (id, title, priority, tag, assignee) in filtered_tickets {
                                                div { key: "board-{id}",
                                                    class: "group flex cursor-pointer flex-col gap-2.5 rounded-xl border border-border/70 bg-card p-3.5 shadow-xs transition-all duration-200 hover:-translate-y-0.5 hover:border-primary/50 hover:shadow-md",
                                                    div { class: "flex items-center justify-between",
                                                        span { class: "font-mono text-[11px] font-bold text-muted-foreground group-hover:text-primary transition-colors",
                                                            "{id}"
                                                        }
                                                        div { class: "flex items-center gap-1.5",
                                                            span { class: "size-1.5 rounded-full {priority_dot(priority)}" }
                                                            Badge { variant: "{priority_badge(priority)}", class: "text-[10px] px-1.5 py-0.2",
                                                                "{priority}"
                                                            }
                                                        }
                                                    }
                                                    p { class: "text-sm font-semibold leading-snug text-foreground group-hover:text-primary/95 transition-colors",
                                                        "{title}"
                                                    }
                                                    div { class: "mt-1 flex items-center justify-between pt-2 border-t border-border/40 text-xs",
                                                        span { class: "rounded-md bg-muted/80 px-2 py-0.5 text-[10px] font-medium text-muted-foreground",
                                                            "{tag}"
                                                        }
                                                        div {
                                                            class: "flex size-5.5 items-center justify-center rounded-full text-[10px] font-bold {assignee_bg(assignee)}",
                                                            "{assignee}"
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
                }

                // --------------------------------------------------------
                // List view
                // --------------------------------------------------------
                TabsContent { value: "list".to_string(),
                    div { class: "flex flex-col gap-2 rounded-2xl border border-border/70 bg-card/60 p-2",
                        for (column, tickets) in COLUMNS {
                            for (id, title, priority, tag, assignee) in tickets.iter() {
                                {
                                    let match_q = q.is_empty()
                                        || id.to_lowercase().contains(&q)
                                        || title.to_lowercase().contains(&q)
                                        || tag.to_lowercase().contains(&q)
                                        || assignee.to_lowercase().contains(&q);
                                    let match_prio = filter_prio == "All" || *priority == filter_prio;

                                    if match_q && match_prio {
                                        rsx! {
                                            div { key: "list-{id}",
                                                class: "group flex cursor-pointer items-center justify-between gap-4 rounded-xl border border-border/60 bg-card px-4 py-3 shadow-2xs transition-all hover:-translate-y-0.2 hover:border-primary/40 hover:bg-accent/40",
                                                div { class: "flex min-w-0 items-center gap-3",
                                                    span { class: "size-2 rounded-full {priority_dot(priority)}" }
                                                    span { class: "font-mono shrink-0 text-xs font-semibold text-muted-foreground", "{id}" }
                                                    span { class: "truncate text-sm font-medium text-foreground group-hover:text-primary transition-colors", "{title}" }
                                                }
                                                div { class: "flex shrink-0 items-center gap-2",
                                                    span { class: "rounded-md bg-muted px-2 py-0.5 text-[10px] font-medium text-muted-foreground",
                                                        "{tag}"
                                                    }
                                                    Badge { variant: "outline", class: "text-[10px]", "{column}" }
                                                    Badge { variant: "{priority_badge(priority)}", class: "text-[10px]",
                                                        "{priority}"
                                                    }
                                                    div {
                                                        class: "flex size-5.5 items-center justify-center rounded-full text-[10px] font-bold {assignee_bg(assignee)}",
                                                        "{assignee}"
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        rsx! {}
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
