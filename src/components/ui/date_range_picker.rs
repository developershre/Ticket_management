use dioxus::prelude::*;

use super::Calendar;
use crate::components::month_calendar::DateRange;
use crate::icons::CalendarIcon;

const SHORT_MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

fn format_range(range: DateRange) -> String {
    let (start, end) = range;
    if start == end {
        return format!(
            "{} {}, {}",
            SHORT_MONTHS[(start.1 - 1) as usize],
            start.2,
            start.0
        );
    }
    if start.0 == end.0 {
        format!(
            "{} {} \u{2013} {} {}, {}",
            SHORT_MONTHS[(start.1 - 1) as usize],
            start.2,
            SHORT_MONTHS[(end.1 - 1) as usize],
            end.2,
            end.0
        )
    } else {
        format!(
            "{} {}, {} \u{2013} {} {}, {}",
            SHORT_MONTHS[(start.1 - 1) as usize],
            start.2,
            start.0,
            SHORT_MONTHS[(end.1 - 1) as usize],
            end.2,
            end.0
        )
    }
}

/// Two-step selection: first click sets the start, second sets the end
/// (a click before the start restarts the range there).
fn select_day(mut range: Signal<Option<DateRange>>, mut pending: Signal<bool>, day: (u32, u32, u32)) {
    if !*pending.read() {
        range.set(Some((day, day)));
        pending.set(true);
    } else {
        let (start, _) = (*range.read()).unwrap_or((day, day));
        if day < start {
            range.set(Some((day, day)));
        } else {
            range.set(Some((start, day)));
            pending.set(false);
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DateRangePickerProps {
    pub range: Signal<Option<DateRange>>,

    #[props(default)]
    pub class: String,
}

/// Toolbar button that opens a two-month popover for picking a date range.
#[component]
pub fn DateRangePicker(props: DateRangePickerProps) -> Element {
    let mut open = use_signal(|| false);
    let mut pending = use_signal(|| false);
    let mut range = props.range;
    let class = props.class;

    let current = *range.read();
    let label = match current {
        Some(r) => format_range(r),
        None => "Date range".to_string(),
    };

    let today = crate::state::get_real_current_date();
    let (left_y, left_m) = match current {
        Some((s, _)) => (s.0, s.1),
        None => (today.0, today.1),
    };
    let (right_y, right_m) = if left_m == 12 {
        (left_y + 1, 1)
    } else {
        (left_y, left_m + 1)
    };

    let hint = if *pending.read() {
        "Choose an end date"
    } else {
        "Choose a start date"
    };

    rsx! {
        div { class: "relative inline-flex {class}",
            button {
                r#type: "button",
                class: "inline-flex max-w-64 cursor-pointer items-center gap-1.5 rounded-lg border border-border/40 px-2.5 py-1.5 text-sm text-foreground hover:bg-muted transition-colors",
                onclick: move |_| {
                    let next = !*open.read();
                    open.set(next);
                    if next {
                        pending.set(false);
                    }
                },
                CalendarIcon { class: "size-4 shrink-0 text-muted-foreground" }
                span { class: "truncate", "{label}" }
                svg {
                    class: "size-3.5 shrink-0",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "m6 9 6 6 6-6" }
                }
            }

            if *open.read() {
                // Click-away layer.
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| open.set(false),
                }

                div { class: "absolute right-0 top-full z-50 mt-2 w-max rounded-lg border border-border bg-popover p-3 text-popover-foreground shadow-lg",
                    div { class: "grid grid-cols-1 gap-4 sm:grid-cols-2",
                        Calendar {
                            default_year: Some(left_y),
                            default_month: Some(left_m),
                            range: current,
                            onselect: move |d| select_day(range, pending, d),
                        }
                        Calendar {
                            default_year: Some(right_y),
                            default_month: Some(right_m),
                            range: current,
                            onselect: move |d| select_day(range, pending, d),
                        }
                    }

                    div { class: "mt-3 flex items-center justify-between gap-4 border-t border-border/40 pt-2.5",
                        span { class: "text-xs text-muted-foreground", "{hint}" }
                        div { class: "flex items-center gap-2",
                            button {
                                r#type: "button",
                                class: "inline-flex h-7 cursor-pointer items-center rounded-md border border-border px-2.5 text-xs font-medium text-foreground hover:bg-muted transition-colors",
                                onclick: move |_| {
                                    range.set(None);
                                    pending.set(false);
                                },
                                "Clear"
                            }
                            button {
                                r#type: "button",
                                class: "inline-flex h-7 cursor-pointer items-center rounded-md bg-primary px-2.5 text-xs font-medium text-primary-foreground hover:opacity-90 transition-opacity",
                                onclick: move |_| open.set(false),
                                "Done"
                            }
                        }
                    }
                }
            }
        }
    }
}
