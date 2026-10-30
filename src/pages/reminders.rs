//! Reminders page — searchable list built on `Item`, `Switch`, `Badge`, `Input`.

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::input::Input;
use crate::components::ui::item::{
    Item, ItemActions, ItemContent, ItemDescription, ItemMedia, ItemTitle,
};
use crate::components::ui::switch::Switch;

/// (title, when, tag, initially done)
const REMINDERS: &[(&str, &str, &str, bool)] = &[
    ("Review pull request #482", "Today · 2:00 PM", "Engineering", true),
    ("Team standup notes", "Today · 4:30 PM", "Meetings", false),
    ("Renew domain + SSL", "Tomorrow · 9:00 AM", "Ops", false),
    ("Send weekly report", "Friday · 5:00 PM", "Reporting", true),
    ("Plan sprint 24", "Mon · 10:00 AM", "Planning", false),
];

#[component]
pub fn Reminders() -> Element {
    let mut query = use_signal(String::new);
    let q = query.read().to_lowercase();

    let filtered: Vec<_> = REMINDERS
        .iter()
        .filter(|(title, when, tag, _)| {
            q.is_empty()
                || title.to_lowercase().contains(&q)
                || when.to_lowercase().contains(&q)
                || tag.to_lowercase().contains(&q)
        })
        .collect();

    let total = REMINDERS.len();
    let shown = filtered.len();

    rsx! {
        div { class: "mx-auto flex w-full max-w-3xl flex-col gap-6 p-6 pb-24",

            // Page header
            div { class: "flex flex-wrap items-end justify-between gap-3",
                div {
                    h1 { class: "text-2xl font-semibold tracking-tight", "Reminders" }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Never miss a follow-up."
                    }
                }
                Badge { variant: "secondary", class: "tabular-nums", "{shown} of {total}" }
            }

            // Search
            Input {
                class: "w-full",
                placeholder: "Search reminders...".to_string(),
                value: query.read().clone(),
                oninput: move |e: FormEvent| query.set(e.value()),
            }

            // List
            if shown == 0 {
                div { class: "rounded-xl border bg-card p-10 text-center text-sm text-muted-foreground",
                    "No reminders match your search."
                }
            } else {
                div { class: "flex flex-col gap-2",
                    for (title, when, tag, done) in filtered {
                        ReminderRow {
                            key: "{title}",
                            title: (*title).to_string(),
                            when: (*when).to_string(),
                            tag: (*tag).to_string(),
                            done: *done,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ReminderRow(title: String, when: String, tag: String, done: bool) -> Element {
    let checked = use_signal(|| done);
    let is_done = *checked.read();

    rsx! {
        Item {
            active: is_done,
            div { class: "flex min-w-0 flex-1 items-center gap-3",
                ItemMedia {
                    Badge { variant: "outline", class: "text-[10px]", "{tag}" }
                }
                ItemContent {
                    ItemTitle {
                        class: if is_done { "line-through opacity-60".to_string() } else { String::new() },
                        "{title}"
                    }
                    ItemDescription { "{when}" }
                }
            }
            ItemActions {
                Switch { checked: Some(checked) }
            }
        }
    }
}
