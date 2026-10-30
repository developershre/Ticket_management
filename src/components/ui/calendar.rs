use dioxus::prelude::*;

use super::{Tooltip, TooltipContent, TooltipTrigger};
use crate::components::month_calendar::{is_event_on, CalEvent, DateRange};

#[derive(Props, Clone, PartialEq)]
pub struct CalendarProps {
    #[props(default)]
    pub default_year: Option<u32>,

    #[props(default)]
    pub default_month: Option<u32>,

    #[props(default)]
    pub selected: Option<Signal<Option<(u32, u32, u32)>>>,

    #[props(default)]
    pub onselect: Option<EventHandler<(u32, u32, u32)>>,

    #[props(default)]
    pub events: Vec<CalEvent>,

    #[props(default)]
    pub class: String,

    /// Inclusive range highlighted across the grid (range-picker mode).
    #[props(default)]
    pub range: Option<DateRange>,
}

/// Visual style for days inside/outside the selected range, if any.
fn range_style(range: Option<DateRange>, date: (u32, u32, u32)) -> Option<&'static str> {
    let (start, end) = range?;
    if date == start || date == end {
        Some("bg-primary text-primary-foreground font-semibold")
    } else if date > start && date < end {
        Some("bg-primary/10 text-foreground")
    } else {
        None
    }
}

fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

// Sakamoto's algorithm: Day of week for 1st of month (0 = Sunday, 1 = Monday, ..., 6 = Saturday)
fn first_day_of_month(year: u32, month: u32) -> usize {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    ((y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + 1) % 7) as usize
}

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

const WEEKDAY_NAMES: [&str; 7] = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];

#[component]
pub fn Calendar(props: CalendarProps) -> Element {
    let real_now = crate::state::get_real_current_date();
    let initial_y = props.default_year.unwrap_or(real_now.0);
    let initial_m = props.default_month.unwrap_or(real_now.1);

    let mut current_year = use_signal(move || initial_y);
    let mut current_month = use_signal(move || initial_m);

    // In range mode there is no single "selected" day; the range highlights instead.
    let range = props.range;
    let range_mode = range.is_some();
    let managed_selection = props.selected.is_some();
    let onselect = props.onselect;

    let local_selected = use_signal(move || if range_mode { None } else { Some(real_now) });
    let mut selected = props.selected.unwrap_or(local_selected);

    let year = current_year();
    let month = current_month();

    let total_days = days_in_month(year, month);
    let start_weekday = first_day_of_month(year, month);

    // Days from previous month to fill the first row
    let prev_month = if month == 1 { 12 } else { month - 1 };
    let prev_year = if month == 1 { year - 1 } else { year };
    let prev_days_count = days_in_month(prev_year, prev_month);

    let next_month = if month == 12 { 1 } else { month + 1 };
    let next_year = if month == 12 { year + 1 } else { year };

    let total_cells = start_weekday as u32 + total_days;
    let grid_cells = if total_cells <= 35 { 35 } else { 42 };
    let trailing_count = grid_cells - total_cells;

    let class = format!(
        "w-full flex flex-col gap-4 select-none {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "calendar",
            class: class,

            // Header: Month Year + Prev/Next buttons
            div {
                class: "flex items-center justify-between px-1",

                // Previous Month Button
                button {
                    r#type: "button",
                    class: "size-8 rounded-lg bg-secondary/80 hover:bg-secondary text-foreground flex items-center justify-center transition-colors cursor-pointer border border-border/40 shadow-xs",
                    onclick: move |_| {
                        let m = current_month();
                        let y = current_year();
                        if m == 1 {
                            current_month.set(12);
                            current_year.set(y - 1);
                        } else {
                            current_month.set(m - 1);
                        }
                    },
                    svg {
                        class: "size-4",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        path { d: "m15 18-6-6 6-6" }
                    }
                    span { class: "sr-only", "Previous month" }
                }

                // Month & Year Label
                div {
                    class: "text-sm font-semibold tracking-tight text-foreground select-none",
                    "{MONTH_NAMES[(month - 1) as usize]} {year}"
                }

                // Next Month Button
                button {
                    r#type: "button",
                    class: "size-8 rounded-lg bg-secondary/80 hover:bg-secondary text-foreground flex items-center justify-center transition-colors cursor-pointer border border-border/40 shadow-xs",
                    onclick: move |_| {
                        let m = current_month();
                        let y = current_year();
                        if m == 12 {
                            current_month.set(1);
                            current_year.set(y + 1);
                        } else {
                            current_month.set(m + 1);
                        }
                    },
                    svg {
                        class: "size-4",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        path { d: "m9 18 6-6-6-6" }
                    }
                    span { class: "sr-only", "Next month" }
                }
            }

            // Weekday Headers: Su Mo Tu We Th Fr Sa
            div {
                class: "grid grid-cols-7 gap-1 text-center text-xs font-medium text-muted-foreground pt-1",
                for day_name in WEEKDAY_NAMES {
                    div {
                        key: "{day_name}",
                        class: "select-none",
                        "{day_name}"
                    }
                }
            }

            // Days Grid
            div {
                class: "grid grid-cols-7 gap-y-2 gap-x-1 text-center text-sm",

                // Previous month padding days
                for i in 0..start_weekday {
                    {
                        let day_num = prev_days_count - (start_weekday as u32 - 1) + (i as u32);
                        let day_date = (prev_year, prev_month, day_num);
                        let day_class = match range_style(range, day_date) {
                            Some(extra) => format!(
                                "size-8 mx-auto flex items-center justify-center text-xs select-none rounded-lg transition-colors cursor-pointer {extra}"
                            ),
                            None => "size-8 mx-auto flex items-center justify-center text-xs text-muted-foreground/35 select-none font-normal rounded-lg hover:bg-muted/40 cursor-pointer".to_string(),
                        };
                        rsx! {
                            button {
                                key: "prev-{day_num}",
                                r#type: "button",
                                class: day_class,
                                onclick: move |_| {
                                    if managed_selection {
                                        selected.set(Some(day_date));
                                    }
                                    if let Some(handler) = &onselect {
                                        handler.call(day_date);
                                    }
                                },
                                "{day_num}"
                            }
                        }
                    }
                }

                // Current month days
                for day in 1..=total_days {
                    {
                        let is_selected = match *selected.read() {
                            Some((y, m, d)) => y == year && m == month && d == day,
                            None => false,
                        };

                        let is_today = real_now.0 == year && real_now.1 == month && real_now.2 == day;

                        let day_date = (year, month, day);
                        let day_events: Vec<CalEvent> = props
                            .events
                            .iter()
                            .filter(|e| is_event_on(e, day_date))
                            .cloned()
                            .collect();

                        let day_class = match range_style(range, day_date) {
                            Some(extra) => format!(
                                "size-8 mx-auto flex flex-col items-center justify-center gap-1 rounded-lg text-xs select-none transition-colors cursor-pointer {extra}"
                            ),
                            None => {
                                if is_selected {
                                    "size-8 mx-auto flex flex-col items-center justify-center gap-1 rounded-lg bg-primary text-primary-foreground font-semibold text-xs shadow-xs transition-colors cursor-pointer".to_string()
                                } else if is_today {
                                    "size-8 mx-auto flex flex-col items-center justify-center gap-1 rounded-lg border border-primary/50 text-foreground font-semibold text-xs hover:bg-muted transition-colors cursor-pointer".to_string()
                                } else {
                                    "size-8 mx-auto flex flex-col items-center justify-center gap-1 rounded-lg text-foreground hover:bg-muted text-xs font-normal transition-colors cursor-pointer".to_string()
                                }
                            }
                        };

                        let on_day_click = {
                            let mut sel = selected;
                            let handler = onselect;
                            let managed = managed_selection;
                            move |_| {
                                if managed {
                                    sel.set(Some(day_date));
                                }
                                if let Some(h) = handler {
                                    h.call(day_date);
                                }
                            }
                        };

                        rsx! {
                            if day_events.is_empty() {
                                button {
                                    key: "current-{day}",
                                    r#type: "button",
                                    class: day_class,
                                    onclick: on_day_click,
                                    "{day}"
                                }
                            } else {
                                Tooltip {
                                    key: "current-{day}",
                                    TooltipTrigger { class: "w-full justify-center",
                                        button {
                                            r#type: "button",
                                            class: day_class,
                                            onclick: on_day_click,
                                            span { class: "leading-none", "{day}" }
                                            span { class: "flex gap-0.5",
                                                for ev in day_events.iter().take(3) {
                                                    span {
                                                        key: "{ev.id}",
                                                        class: "size-1 rounded-full {ev.color.dot_class()}",
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    TooltipContent { side: "top",
                                        div { class: "flex flex-col gap-1",
                                            for ev in day_events.iter() {
                                                {
                                                    let label_time = ev.time.as_deref().unwrap_or("All day");
                                                    rsx! {
                                                        div {
                                                            key: "{ev.id}",
                                                            class: "flex items-center gap-1.5",
                                                            span {
                                                                class: "size-1.5 shrink-0 rounded-full {ev.color.dot_class()}",
                                                            }
                                                            span { class: "font-medium", "{label_time}" }
                                                            span { class: "opacity-80", "{ev.title}" }
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

                // Next month trailing days
                for next_d in 1..=trailing_count {
                    {
                        let day_date = (next_year, next_month, next_d);
                        let day_class = match range_style(range, day_date) {
                            Some(extra) => format!(
                                "size-8 mx-auto flex items-center justify-center text-xs select-none rounded-lg transition-colors cursor-pointer {extra}"
                            ),
                            None => "size-8 mx-auto flex items-center justify-center text-xs text-muted-foreground/35 select-none font-normal rounded-lg hover:bg-muted/40 cursor-pointer".to_string(),
                        };
                        rsx! {
                            button {
                                key: "next-{next_d}",
                                r#type: "button",
                                class: day_class,
                                onclick: move |_| {
                                    if managed_selection {
                                        selected.set(Some(day_date));
                                    }
                                    if let Some(handler) = &onselect {
                                        handler.call(day_date);
                                    }
                                },
                                "{next_d}"
                            }
                        }
                    }
                }
            }
        }
    }
}
