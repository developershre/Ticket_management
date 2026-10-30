use dioxus::prelude::*;

use crate::components::ui::input::Input;
use crate::components::ui::switch::Switch;
use crate::icons::SearchIcon;

/// (title, when, tag, initially done)
const REMINDERS: &[(&str, &str, &str, bool)] = &[
    ("Review pull request #482", "Today · 2:00 PM", "Engineering", true),
    ("Team standup notes", "Today · 4:30 PM", "Meetings", false),
    ("Renew domain + SSL", "Tomorrow · 9:00 AM", "Ops", false),
    ("Send weekly report", "Friday · 5:00 PM", "Reporting", true),
    ("Plan sprint 24", "Mon · 10:00 AM", "Planning", false),
];

fn category_badge_style(tag: &str) -> &'static str {
    match tag {
        "Engineering" => "border-indigo-500/30 bg-indigo-500/10 text-indigo-500 dark:text-indigo-400",
        "Meetings" => "border-violet-500/30 bg-violet-500/10 text-violet-500 dark:text-violet-400",
        "Ops" => "border-amber-500/30 bg-amber-500/10 text-amber-500 dark:text-amber-400",
        "Reporting" => "border-sky-500/30 bg-sky-500/10 text-sky-500 dark:text-sky-400",
        _ => "border-emerald-500/30 bg-emerald-500/10 text-emerald-500 dark:text-emerald-400",
    }
}

#[component]
pub fn Reminders() -> Element {
    let mut query = use_signal(String::new);
    let mut category_filter = use_signal(|| "All".to_string());

    let q = query.read().to_lowercase();
    let cat = category_filter.read().clone();

    let filtered: Vec<_> = REMINDERS
        .iter()
        .filter(|(title, when, tag, _)| {
            let match_q = q.is_empty()
                || title.to_lowercase().contains(&q)
                || when.to_lowercase().contains(&q)
                || tag.to_lowercase().contains(&q);
            let match_cat = cat == "All" || *tag == cat;
            match_q && match_cat
        })
        .collect();

    let total = REMINDERS.len();
    let initial_done_count = REMINDERS.iter().filter(|(_, _, _, done)| *done).count();
    let percent_done = (initial_done_count * 100) / total;

    rsx! {
        div { class: "mx-auto flex w-full max-w-4xl flex-col gap-6 p-6 pb-24",

            // Page header
            div { class: "flex flex-wrap items-center justify-between gap-4 border-b border-border/50 pb-5",
                div {
                    div { class: "flex items-center gap-2.5",
                        h1 { class: "text-2xl font-bold tracking-tight text-foreground sm:text-3xl", "Reminders" }
                        span { class: "rounded-full bg-primary/15 px-2.5 py-0.5 text-xs font-semibold text-primary",
                            "{total} items"
                        }
                    }
                    p { class: "mt-1 text-sm text-muted-foreground",
                        "Stay on top of deadlines and follow-ups."
                    }
                }
            }

            // Progress Banner
            div { class: "flex flex-col gap-2.5 rounded-2xl border border-border/70 bg-card/80 p-4 shadow-xs",
                div { class: "flex items-center justify-between text-xs",
                    span { class: "font-semibold text-foreground", "Completion Progress" }
                    span { class: "font-mono font-bold text-primary", "{percent_done}% completed" }
                }
                div { class: "h-2 w-full overflow-hidden rounded-full bg-muted",
                    div {
                        class: "h-full rounded-full bg-gradient-to-r from-indigo-500 via-indigo-600 to-violet-500 transition-all duration-500",
                        style: "width: {percent_done}%;",
                    }
                }
                div { class: "flex items-center justify-between text-[11px] text-muted-foreground",
                    span { "{initial_done_count} completed" }
                    span { "{total - initial_done_count} remaining" }
                }
            }

            // Search & Category Chips toolbar
            div { class: "flex flex-wrap items-center justify-between gap-3",
                div { class: "relative flex flex-1 min-w-[200px] max-w-sm items-center",
                    div { class: "pointer-events-none absolute left-3 flex items-center text-muted-foreground",
                        SearchIcon { class: "size-4" }
                    }
                    Input {
                        class: "pl-9 text-xs h-9 w-full",
                        placeholder: "Search reminders...".to_string(),
                        value: query.read().clone(),
                        oninput: move |e: FormEvent| query.set(e.value()),
                    }
                }

                // Category Chips
                div { class: "flex flex-wrap items-center gap-1.5",
                    for tag_name in ["All", "Engineering", "Meetings", "Ops", "Planning", "Reporting"] {
                        {
                            let is_sel = cat == tag_name;
                            let sel_cls = if is_sel {
                                "bg-primary text-primary-foreground font-semibold shadow-xs"
                            } else {
                                "bg-muted/70 text-muted-foreground hover:text-foreground hover:bg-muted"
                            };
                            rsx! {
                                button {
                                    r#type: "button",
                                    key: "cat-{tag_name}",
                                    class: "rounded-lg px-2.5 py-1 text-xs transition-colors cursor-pointer {sel_cls}",
                                    onclick: move |_| category_filter.set(tag_name.to_string()),
                                    "{tag_name}"
                                }
                            }
                        }
                    }
                }
            }

            // List of Reminders
            if filtered.is_empty() {
                div { class: "flex flex-col items-center justify-center rounded-2xl border border-dashed border-border p-12 text-center",
                    div { class: "flex size-12 items-center justify-center rounded-full bg-muted text-muted-foreground mb-3",
                        SearchIcon { class: "size-6" }
                    }
                    h3 { class: "text-sm font-semibold text-foreground", "No reminders found" }
                    p { class: "mt-1 text-xs text-muted-foreground max-w-xs",
                        "No reminders match your current search or category filter. Try clearing your filter."
                    }
                }
            } else {
                div { class: "flex flex-col gap-2.5",
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
    let mut checked = use_signal(|| done);
    let is_done = *checked.read();

    let border_cls = if is_done {
        "border-border/40 bg-card/40 opacity-75"
    } else {
        "border-border/80 bg-card hover:border-primary/50 hover:shadow-xs"
    };

    let check_btn_cls = if is_done {
        "border-primary bg-primary text-primary-foreground"
    } else {
        "border-border bg-background hover:border-primary/60"
    };

    let title_cls = if is_done {
        "line-through text-muted-foreground"
    } else {
        "text-foreground group-hover:text-primary"
    };

    rsx! {
        div {
            class: "group flex items-center justify-between gap-4 rounded-xl border p-3.5 shadow-2xs transition-all duration-200 {border_cls}",
            div { class: "flex min-w-0 flex-1 items-center gap-3.5",
                button {
                    r#type: "button",
                    title: if is_done { "Mark incomplete" } else { "Mark complete" },
                    class: "flex size-5.5 shrink-0 items-center justify-center rounded-md border transition-all cursor-pointer {check_btn_cls}",
                    onclick: move |_| {
                        let cur = *checked.read();
                        checked.set(!cur);
                    },
                    if is_done {
                        svg {
                            class: "size-3.5",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "3",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            path { d: "M20 6 9 17l-5-5" }
                        }
                    }
                }
                div { class: "flex min-w-0 flex-col gap-0.5",
                    span {
                        class: "truncate text-sm font-semibold transition-all {title_cls}",
                        "{title}"
                    }
                    div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                        svg {
                            class: "size-3.5",
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            circle { cx: "12", cy: "12", r: "10" }
                            polyline { points: "12 6 12 12 16 14" }
                        }
                        span { "{when}" }
                    }
                }
            }

            div { class: "flex shrink-0 items-center gap-3",
                span { class: "rounded-md border px-2 py-0.5 text-[10px] font-semibold {category_badge_style(&tag)}",
                    "{tag}"
                }
                Switch { checked: Some(checked) }
            }
        }
    }
}
