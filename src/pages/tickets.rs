//! Tickets page — board + list views built on the UI kit's `Tabs` + `Badge`.

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::tabs::{Tabs, TabsContent, TabsList, TabsTrigger};

/// (id, title, priority)
type Ticket = (&'static str, &'static str, &'static str);

/// (column name, tickets)
const COLUMNS: &[(&str, &[Ticket])] = &[
    (
        "Todo",
        &[
            ("TK-104", "Fix login redirect loop", "High"),
            ("TK-108", "Update onboarding copy", "Low"),
            ("TK-111", "Audit settings page a11y", "Medium"),
        ],
    ),
    (
        "In Progress",
        &[
            ("TK-101", "Design landing page", "Medium"),
            ("TK-115", "Refactor auth middleware", "High"),
        ],
    ),
    (
        "Done",
        &[
            ("TK-098", "Ship v1.2 release notes", "Medium"),
            ("TK-099", "Add dark mode toggle", "Low"),
        ],
    ),
];

fn priority_badge(priority: &str) -> &'static str {
    match priority {
        "High" => "destructive",
        "Medium" => "secondary",
        _ => "outline",
    }
}

#[component]
pub fn Tickets() -> Element {
    let open_count: usize = COLUMNS
        .iter()
        .filter(|(name, _)| *name != "Done")
        .map(|(_, tickets)| tickets.len())
        .sum();

    rsx! {
        div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6 p-6 pb-24",

            // Page header
            div { class: "flex flex-wrap items-end justify-between gap-3",
                div {
                    h1 { class: "text-2xl font-semibold tracking-tight", "Tickets" }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Plan, track and ship your team's work."
                    }
                }
                Badge { variant: "secondary", class: "tabular-nums", "{open_count} open" }
            }

            Tabs { value: "board".to_string(),
                TabsList {
                    TabsTrigger { value: "board".to_string(), "Board" }
                    TabsTrigger { value: "list".to_string(), "List" }
                }

                // --------------------------------------------------------
                // Board view
                // --------------------------------------------------------
                TabsContent { value: "board".to_string(),
                    div { class: "grid gap-4 md:grid-cols-3",
                        for (column, tickets) in COLUMNS {
                            div { key: "board-col-{column}",
                                class: "flex flex-col gap-3 rounded-xl border bg-card/60 p-3",
                                div { class: "flex items-center justify-between px-1",
                                    h2 { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                                        "{column}"
                                    }
                                    Badge { variant: "outline", class: "text-[10px] tabular-nums",
                                        "{tickets.len()}"
                                    }
                                }
                                div { class: "flex flex-col gap-2",
                                    for (id, title, priority) in tickets.iter() {
                                        div { key: "board-{id}",
                                            class: "flex cursor-pointer flex-col gap-2 rounded-lg border bg-card p-3 shadow-xs transition-colors hover:bg-accent/40",
                                            div { class: "flex items-center justify-between",
                                                span { class: "text-[11px] font-medium text-muted-foreground",
                                                    "{id}"
                                                }
                                                Badge { variant: "{priority_badge(priority)}", class: "text-[10px]",
                                                    "{priority}"
                                                }
                                            }
                                            p { class: "text-sm font-medium leading-snug", "{title}" }
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
                    div { class: "flex flex-col gap-2",
                        for (column, tickets) in COLUMNS {
                            for (id, title, priority) in tickets.iter() {
                                div { key: "list-{id}",
                                    class: "flex cursor-pointer items-center justify-between gap-4 rounded-lg border bg-card px-4 py-3 shadow-xs transition-colors hover:bg-accent/40",
                                    div { class: "flex min-w-0 items-center gap-3",
                                        span { class: "shrink-0 text-xs text-muted-foreground", "{id}" }
                                        span { class: "truncate text-sm font-medium", "{title}" }
                                    }
                                    div { class: "flex shrink-0 items-center gap-2",
                                        Badge { variant: "outline", class: "text-[10px]", "{column}" }
                                        Badge { variant: "{priority_badge(priority)}", class: "text-[10px]",
                                            "{priority}"
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
