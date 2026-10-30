//! Calendar page — month calendar widget + rich agenda side panel.

use dioxus::prelude::*;

use crate::components::month_calendar::{is_event_on, CalEvent, EventColor};
use crate::components::ui::calendar::Calendar;
use crate::components::ui::item::{Item, ItemContent, ItemDescription, ItemTitle};

#[component]
pub fn CalendarPage() -> Element {
    let (year, month, day) = crate::state::get_real_current_date();
    let mut selected = use_signal(|| Some((year, month, day)));

    let events = vec![
        CalEvent {
            id: 1,
            title: "Sprint planning".to_string(),
            date: (year, month, day),
            end_date: None,
            time: Some("10:00 AM".to_string()),
            color: EventColor::Blue,
        },
        CalEvent {
            id: 2,
            title: "Design review".to_string(),
            date: (year, month, day),
            end_date: None,
            time: Some("02:30 PM".to_string()),
            color: EventColor::Purple,
        },
        CalEvent {
            id: 3,
            title: "QA freeze".to_string(),
            date: (year, month, day),
            end_date: Some((year, month, day.saturating_add(2).min(28))),
            time: Some("All day".to_string()),
            color: EventColor::Amber,
        },
        CalEvent {
            id: 4,
            title: "Client demo".to_string(),
            date: (year, month, day.saturating_add(3).min(28)),
            end_date: None,
            time: Some("11:00 AM".to_string()),
            color: EventColor::Green,
        },
    ];

    let selected_day = *selected.read();
    let selected_label = match selected_day {
        Some((y, m, d)) => format!("{y}-{m:02}-{d:02}"),
        None => "No date selected".to_string(),
    };

    let events_on_selected: Vec<&CalEvent> = match selected_day {
        Some(date) => events.iter().filter(|e| is_event_on(e, date)).collect(),
        None => Vec::new(),
    };

    let selected_rows: Vec<(u32, String, String, String)> = events_on_selected
        .iter()
        .map(|e| {
            (
                e.id,
                e.time.clone().unwrap_or_else(|| "All day".to_string()),
                e.color.dot_class().to_string(),
                e.title.clone(),
            )
        })
        .collect();

    let upcoming_rows: Vec<(u32, String, String, String)> = events
        .iter()
        .take(4)
        .map(|e| {
            (
                e.id,
                e.time.clone().unwrap_or_else(|| "All day".to_string()),
                e.title.clone(),
                e.color.dot_class().to_string(),
            )
        })
        .collect();

    rsx! {
        div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6 p-6 pb-24",

            // Page header
            div { class: "flex flex-wrap items-center justify-between gap-4 border-b border-border/50 pb-5",
                div {
                    div { class: "flex items-center gap-2.5",
                        h1 { class: "text-2xl font-bold tracking-tight text-foreground sm:text-3xl", "Calendar" }
                        span { class: "rounded-full bg-primary/15 px-2.5 py-0.5 text-xs font-semibold text-primary",
                            "{events.len()} events scheduled"
                        }
                    }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "View team events, sprint milestones, and release dates."
                    }
                }

                div { class: "flex items-center gap-2",
                    button {
                        r#type: "button",
                        class: "rounded-lg border border-border/80 bg-card px-3 py-1.5 text-xs font-semibold text-foreground shadow-2xs hover:bg-muted transition-colors cursor-pointer",
                        onclick: move |_| selected.set(Some((year, month, day))),
                        "Jump to Today"
                    }
                }
            }

            div { class: "grid gap-6 lg:grid-cols-[1fr_340px]",

                // Calendar widget
                div { class: "rounded-2xl border border-border/70 bg-card/80 p-5 shadow-xs backdrop-blur-xs",
                    Calendar {
                        events: events.clone(),
                        selected: Some(selected),
                        onselect: move |date| selected.set(Some(date)),
                    }
                }

                // Side panel
                aside { class: "flex flex-col gap-4",

                    // Selected Date Agenda
                    div { class: "rounded-2xl border border-border/70 bg-card p-5 shadow-xs",
                        div { class: "flex items-center justify-between mb-3 border-b border-border/40 pb-2.5",
                            div {
                                h2 { class: "text-xs font-bold uppercase tracking-wider text-muted-foreground",
                                    "Agenda for"
                                }
                                p { class: "font-mono text-sm font-bold text-foreground", "{selected_label}" }
                            }
                            if selected_day == Some((year, month, day)) {
                                span { class: "rounded-full bg-emerald-500/15 px-2 py-0.5 text-[10px] font-bold text-emerald-500",
                                    "TODAY"
                                }
                            }
                        }

                        if selected_day.is_some() {
                            if selected_rows.is_empty() {
                                div { class: "rounded-xl border border-dashed border-border/70 bg-muted/30 p-4 text-center text-xs text-muted-foreground",
                                    "No events scheduled on this day."
                                }
                            } else {
                                div { class: "flex flex-col gap-2.5",
                                    for (ev_id, ev_time, ev_color, ev_title) in selected_rows.iter() {
                                        div { key: "sel-{ev_id}",
                                            class: "flex items-center justify-between gap-3 rounded-xl border border-border/60 bg-muted/40 p-3 shadow-2xs transition-colors hover:bg-accent/40",
                                            div { class: "flex items-center gap-2.5 min-w-0",
                                                span { class: "size-2.5 shrink-0 rounded-full {ev_color} shadow-xs" }
                                                span { class: "truncate text-xs font-semibold text-foreground", "{ev_title}" }
                                            }
                                            span { class: "shrink-0 font-mono text-[11px] font-medium text-muted-foreground", "{ev_time}" }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { class: "text-xs text-muted-foreground",
                                "Select a day on the calendar to see its events."
                            }
                        }
                    }

                    // Upcoming Deadlines
                    div { class: "rounded-2xl border border-border/70 bg-card p-5 shadow-xs",
                        h2 { class: "mb-3 text-xs font-bold uppercase tracking-wider text-muted-foreground border-b border-border/40 pb-2.5",
                            "Upcoming Deadlines"
                        }
                        div { class: "flex flex-col gap-2",
                            for (ev_id, ev_time, ev_title, ev_color) in upcoming_rows.iter() {
                                Item { key: "upcoming-{ev_id}",
                                    class: "py-2.5 rounded-xl transition-all hover:bg-accent/40",
                                    ItemContent {
                                        div { class: "flex items-center gap-2",
                                            span { class: "size-2 shrink-0 rounded-full {ev_color}" }
                                            ItemTitle { class: "text-xs font-semibold", "{ev_title}" }
                                        }
                                        ItemDescription { class: "text-[11px] ml-4 font-mono", "{ev_time}" }
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
