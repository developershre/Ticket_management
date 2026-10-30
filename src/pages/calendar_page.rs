//! Calendar page — the kit's `Calendar` widget plus an events side panel.

use dioxus::prelude::*;

use crate::components::month_calendar::{is_event_on, CalEvent, EventColor};
use crate::components::ui::calendar::Calendar;
use crate::components::ui::item::{Item, ItemContent, ItemDescription, ItemTitle};

#[component]
pub fn CalendarPage() -> Element {
    let (year, month, day) = crate::state::get_real_current_date();
    let mut selected = use_signal(|| None::<(u32, u32, u32)>);

    let events = vec![
        CalEvent {
            id: 1,
            title: "Sprint planning".to_string(),
            date: (year, month, day),
            end_date: None,
            time: Some("10:00".to_string()),
            color: EventColor::Blue,
        },
        CalEvent {
            id: 2,
            title: "Design review".to_string(),
            date: (year, month, day),
            end_date: None,
            time: Some("14:30".to_string()),
            color: EventColor::Purple,
        },
        CalEvent {
            id: 3,
            title: "QA freeze".to_string(),
            date: (year, month, day),
            end_date: Some((year, month, day.saturating_add(2).min(28))),
            time: None,
            color: EventColor::Amber,
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

    // Precomputed rows so loop elements can carry proper keys.
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

    let upcoming_rows: Vec<(u32, String, String)> = events
        .iter()
        .take(3)
        .map(|e| {
            (
                e.id,
                e.time.clone().unwrap_or_else(|| "All day".to_string()),
                e.title.clone(),
            )
        })
        .collect();

    rsx! {
        div { class: "mx-auto flex w-full max-w-6xl flex-col gap-6 p-6 pb-24",

            // Page header
            div {
                h1 { class: "text-2xl font-semibold tracking-tight", "Calendar" }
                p { class: "mt-1 text-sm text-muted-foreground",
                    "Your schedule at a glance."
                }
            }

            div { class: "grid gap-6 lg:grid-cols-[1fr_320px]",

                // Calendar widget
                div { class: "rounded-xl border bg-card p-4",
                    Calendar {
                        events: events.clone(),
                        selected: Some(selected),
                        onselect: move |date| selected.set(Some(date)),
                    }
                }

                // Side panel
                aside { class: "flex flex-col gap-4",

                    div { class: "rounded-xl border bg-card p-4",
                        h2 { class: "mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                            "Selected date"
                        }
                        p { class: "text-sm font-medium", "{selected_label}" }
                        if selected_day.is_some() {
                            p { class: "mt-1 text-xs text-muted-foreground",
                                "{events_on_selected.len()} event(s)"
                            }
                            div { class: "mt-3 flex flex-col gap-2",
                                for (ev_id, ev_time, ev_color, ev_title) in selected_rows.iter() {
                                    div { key: "sel-{ev_id}",
                                        class: "flex items-center gap-2 text-sm",
                                        span { class: "size-2 shrink-0 rounded-full {ev_color}" }
                                        span { class: "text-xs text-muted-foreground", "{ev_time}" }
                                        span { class: "truncate font-medium", "{ev_title}" }
                                    }
                                }
                            }
                        } else {
                            p { class: "mt-1 text-xs text-muted-foreground",
                                "Pick a day on the calendar to see its events."
                            }
                        }
                    }

                    div { class: "rounded-xl border bg-card p-4",
                        h2 { class: "mb-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                            "Upcoming"
                        }
                        div { class: "flex flex-col gap-2",
                            for (ev_id, ev_time, ev_title) in upcoming_rows.iter() {
                                Item { key: "upcoming-{ev_id}",
                                    class: "py-2.5",
                                    ItemContent {
                                        ItemTitle { class: "text-sm", "{ev_title}" }
                                        ItemDescription { "{ev_time}" }
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
