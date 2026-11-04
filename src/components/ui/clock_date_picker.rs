//! Clock-based Date & Time Picker Component
//!
//! Features:
//! - Interactive Analog Clock UI with draggable / clickable handles for Hours and Minutes
//! - Mouse wheel scrollable date selector (scroll on Day, Month, or Year)
//! - Direct typing text inputs for date and time
//! - Quick presets: Today, Tomorrow, +2 Days, Next Week
//! - Formats directly into standard ISO 8601 timestamps (e.g. `2026-11-06T10:30:00+05:30`)

use dioxus::prelude::*;
use crate::icons::{CalendarIcon, CheckIcon, ClockIcon};

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const MONTH_FULL_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ClockMode {
    Hour,
    Minute,
}

fn days_in_month(y: u32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// Converts epoch seconds into civil date (year, month, day).
fn civil_from_epoch(epoch: i64) -> (u32, u32, u32) {
    let days = epoch.div_euclid(86_400) + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as u32, m as u32, d as u32)
}

/// Returns current local date and time parts in Asia/Kolkata (UTC+05:30).
pub fn now_local_parts() -> (u32, u32, u32, u32, u32) {
    let epoch = crate::sync::now_epoch() + 19_800;
    let (y, m, d) = civil_from_epoch(epoch);
    let secs = epoch.rem_euclid(86_400);
    let h = (secs / 3600) as u32;
    let mi = ((secs % 3600) / 60) as u32;
    (y, m, d, h, mi)
}

/// Parses a date-time string into (year, month, day, hour24, minute).
pub fn parse_datetime_parts(s: &str) -> Option<(u32, u32, u32, u32, u32)> {
    let s = s.trim();
    if s.len() >= 10 && s.as_bytes()[4] == b'-' && s.as_bytes()[7] == b'-' {
        let y: u32 = s[0..4].parse().ok()?;
        let m: u32 = s[5..7].parse().ok()?;
        let d: u32 = s[8..10].parse().ok()?;

        if s.len() >= 16 {
            let time_part = if s.contains('T') {
                s.split('T').nth(1)?
            } else if s.contains(' ') {
                s.split(' ').nth(1)?
            } else {
                return Some((y, m, d, 9, 0));
            };
            if time_part.len() >= 5 {
                let h: u32 = time_part[0..2].parse().ok()?;
                let mi: u32 = time_part[3..5].parse().ok()?;
                return Some((y, m, d, h, mi));
            }
        }
        return Some((y, m, d, 9, 0));
    }
    None
}

/// Formats civil parts into a standard ISO 8601 timestamp string.
pub fn format_iso8601(y: u32, m: u32, d: u32, h: u32, mi: u32) -> String {
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:00+05:30")
}

/// Formats civil parts into a human-friendly string.
pub fn format_human_datetime(y: u32, m: u32, d: u32, h: u32, mi: u32) -> String {
    let am_pm = if h >= 12 { "PM" } else { "AM" };
    let h12 = match h % 12 {
        0 => 12,
        other => other,
    };
    let month_idx = (m.saturating_sub(1) as usize).min(11);
    format!("{} {:02}, {:04} • {:02}:{:02} {}", MONTH_NAMES[month_idx], d, y, h12, mi, am_pm)
}

fn get_wheel_delta_y(e: &WheelEvent) -> f64 {
    match e.data().delta() {
        dioxus::html::geometry::WheelDelta::Pixels(p) => p.y,
        dioxus::html::geometry::WheelDelta::Lines(l) => l.y as f64 * 20.0,
        dioxus::html::geometry::WheelDelta::Pages(p) => p.y as f64 * 100.0,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ClockDateTimePickerProps {
    /// Reactive signal holding the formatted timestamp string
    pub value: Signal<String>,
    /// Optional placeholder text
    #[props(default = "Select date & time (e.g. 2026-11-06T10:30:00+05:30)".to_string())]
    pub placeholder: String,
    /// Optional label
    #[props(default = String::new())]
    pub label: String,
    /// Optional disabled state
    #[props(default = false)]
    pub disabled: bool,
}

#[component]
pub fn ClockDateTimePicker(props: ClockDateTimePickerProps) -> Element {
    let mut is_open = use_signal(|| false);
    let mut clock_mode = use_signal(|| ClockMode::Hour);
    let mut is_dragging = use_signal(|| false);

    // Initial state derived from props or current time
    let (init_y, init_m, init_d, init_h, init_mi) = {
        let current_val = props.value.read().clone();
        parse_datetime_parts(&current_val).unwrap_or_else(now_local_parts)
    };

    let mut year = use_signal(|| init_y);
    let mut month = use_signal(|| init_m);
    let mut day = use_signal(|| init_d);
    let mut hour24 = use_signal(|| init_h);
    let mut minute = use_signal(|| init_mi);

    // Direct typed date input buffer
    let mut date_type_input = use_signal(|| format!("{:04}-{:02}-{:02}", init_y, init_m, init_d));

    // When dialog opens, sync state from value
    let open_picker = {
        let val_sig = props.value;
        move |_| {
            if props.disabled {
                return;
            }
            let raw = val_sig.read().clone();
            let (y, m, d, h, mi) = parse_datetime_parts(&raw).unwrap_or_else(now_local_parts);
            year.set(y);
            month.set(m);
            day.set(d);
            hour24.set(h);
            minute.set(mi);
            date_type_input.set(format!("{:04}-{:02}-{:02}", y, m, d));
            clock_mode.set(ClockMode::Hour);
            is_open.set(true);
        }
    };

    // Calculate 12-hour format and AM/PM
    let current_h24 = *hour24.read();
    let is_pm = current_h24 >= 12;
    let hour12 = match current_h24 % 12 {
        0 => 12,
        other => other,
    };
    let current_minute = *minute.read();

    // Toggle AM / PM
    let mut toggle_meridiem = move |to_pm: bool| {
        let h = *hour24.read();
        let was_pm = h >= 12;
        if to_pm && !was_pm {
            hour24.set(h + 12);
        } else if !to_pm && was_pm {
            hour24.set(h - 12);
        }
    };

    // Clock angle calculation from mouse position
    let mut handle_clock_pointer = move |client_x: f64, client_y: f64, rect_left: f64, rect_top: f64, rect_width: f64, rect_height: f64| {
        let rel_x = client_x - rect_left;
        let rel_y = client_y - rect_top;
        let cx = rect_width / 2.0;
        let cy = rect_height / 2.0;
        let dx = rel_x - cx;
        let dy = rel_y - cy;

        // Angle in degrees where top (12 o'clock) is 0 deg
        let mut deg = dx.atan2(-dy).to_degrees();
        if deg < 0.0 {
            deg += 360.0;
        }

        match *clock_mode.read() {
            ClockMode::Hour => {
                let mut h = (deg / 30.0).round() as u32;
                if h == 0 {
                    h = 12;
                }
                let pm = *hour24.read() >= 12;
                let new_h24 = if pm {
                    if h == 12 { 12 } else { h + 12 }
                } else {
                    if h == 12 { 0 } else { h }
                };
                hour24.set(new_h24);
            }
            ClockMode::Minute => {
                let mut m = (deg / 6.0).round() as u32;
                if m >= 60 {
                    m = 0;
                }
                minute.set(m);
            }
        }
    };

    // Wheel scrolling helpers
    let mut scroll_day = move |delta_y: f64| {
        let max_d = days_in_month(*year.read(), *month.read());
        let cur = *day.read();
        if delta_y < 0.0 {
            if cur >= max_d { day.set(1); } else { day.set(cur + 1); }
        } else if delta_y > 0.0 {
            if cur <= 1 { day.set(max_d); } else { day.set(cur - 1); }
        }
        date_type_input.set(format!("{:04}-{:02}-{:02}", *year.read(), *month.read(), *day.read()));
    };

    let mut scroll_month = move |delta_y: f64| {
        let cur = *month.read();
        if delta_y < 0.0 {
            if cur >= 12 { month.set(1); } else { month.set(cur + 1); }
        } else if delta_y > 0.0 {
            if cur <= 1 { month.set(12); } else { month.set(cur - 1); }
        }
        let max_d = days_in_month(*year.read(), *month.read());
        if *day.read() > max_d {
            day.set(max_d);
        }
        date_type_input.set(format!("{:04}-{:02}-{:02}", *year.read(), *month.read(), *day.read()));
    };

    let mut scroll_year = move |delta_y: f64| {
        let cur = *year.read();
        if delta_y < 0.0 {
            year.set(cur + 1);
        } else if delta_y > 0.0 && cur > 2020 {
            year.set(cur - 1);
        }
        let max_d = days_in_month(*year.read(), *month.read());
        if *day.read() > max_d {
            day.set(max_d);
        }
        date_type_input.set(format!("{:04}-{:02}-{:02}", *year.read(), *month.read(), *day.read()));
    };

    let mut scroll_hour = move |delta_y: f64| {
        let cur = *hour24.read();
        if delta_y < 0.0 {
            hour24.set((cur + 1) % 24);
        } else if delta_y > 0.0 {
            hour24.set(if cur == 0 { 23 } else { cur - 1 });
        }
    };

    let mut scroll_minute = move |delta_y: f64| {
        let cur = *minute.read();
        if delta_y < 0.0 {
            minute.set((cur + 1) % 60);
        } else if delta_y > 0.0 {
            minute.set(if cur == 0 { 59 } else { cur - 1 });
        }
    };

    // Apply & Save
    let mut mut_val_prop = props.value;
    let mut apply_datetime = move || {
        let iso = format_iso8601(*year.read(), *month.read(), *day.read(), *hour24.read(), *minute.read());
        mut_val_prop.set(iso);
        is_open.set(false);
    };

    let mut clear_datetime = move || {
        mut_val_prop.set(String::new());
        is_open.set(false);
    };

    let mut set_to_now = move || {
        let (y, m, d, h, mi) = now_local_parts();
        year.set(y);
        month.set(m);
        day.set(d);
        hour24.set(h);
        minute.set(mi);
        date_type_input.set(format!("{y:04}-{m:02}-{d:02}"));
    };

    let mut set_preset_days = move |days_ahead: i64| {
        let epoch = crate::sync::now_epoch() + 19_800 + (days_ahead * 86_400);
        let (y, m, d) = civil_from_epoch(epoch);
        year.set(y);
        month.set(m);
        day.set(d);
        date_type_input.set(format!("{y:04}-{m:02}-{d:02}"));
    };

    // Calculate hand coordinates for SVG
    let mode = *clock_mode.read();
    let hand_angle_deg = match mode {
        ClockMode::Hour => (hour12 as f64) * 30.0,
        ClockMode::Minute => (current_minute as f64) * 6.0,
    };
    let hand_rad = hand_angle_deg.to_radians();
    let hand_length = match mode {
        ClockMode::Hour => 56.0,
        ClockMode::Minute => 68.0,
    };
    let hand_tip_x = 100.0 + hand_length * hand_rad.sin();
    let hand_tip_y = 100.0 - hand_length * hand_rad.cos();

    // Trigger display text
    let current_raw_val = props.value.read().clone();
    let display_preview = if current_raw_val.trim().is_empty() {
        String::new()
    } else if let Some((y, m, d, h, mi)) = parse_datetime_parts(&current_raw_val) {
        format_human_datetime(y, m, d, h, mi)
    } else {
        current_raw_val.clone()
    };

    let has_val = !current_raw_val.trim().is_empty();

    rsx! {
        div { class: "flex flex-col gap-1.5 w-full",
            if !props.label.is_empty() {
                label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                    span { "{props.label}" }
                    span { class: "text-[10px] text-muted-foreground font-normal", "(Click to pick date & time)" }
                }
            }

            // TRIGGER BUTTON / INPUT
            div {
                class: format!(
                    "group relative flex items-center justify-between h-10 w-full rounded-md border border-input bg-card px-3 text-sm text-foreground shadow-2xs transition-colors hover:border-primary/80 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/20 {}",
                    if props.disabled { "opacity-50 cursor-not-allowed" } else { "cursor-pointer" }
                ),
                onclick: open_picker,

                div { class: "flex items-center gap-2.5 truncate",
                    ClockIcon { class: "size-4 text-primary shrink-0 transition-transform group-hover:scale-110" }
                    if has_val {
                        span { class: "font-medium text-foreground tracking-tight truncate",
                            "{display_preview}"
                        }
                    } else {
                        span { class: "text-muted-foreground/75 italic truncate text-xs",
                            "{props.placeholder}"
                        }
                    }
                }

                div { class: "flex items-center gap-1.5 shrink-0 ml-2",
                    if has_val {
                        button {
                            r#type: "button",
                            title: "Clear selected date and time",
                            class: "p-1 rounded text-muted-foreground/70 hover:text-foreground hover:bg-muted/80 transition-colors cursor-pointer",
                            onclick: move |e: MouseEvent| {
                                e.stop_propagation();
                                mut_val_prop.set(String::new());
                            },
                            "×"
                        }
                    }
                    span { class: "rounded bg-muted/80 px-2 py-0.5 text-[10px] font-mono text-muted-foreground font-semibold uppercase",
                        "Set Time"
                    }
                }
            }

            // CLOCK & DATE PICKER MODAL POPUP
            if *is_open.read() {
                div {
                    class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in-0 duration-150",
                    onclick: move |_| is_open.set(false),

                    div {
                        class: "relative w-full max-w-md rounded-xl border border-border/80 bg-popover p-5 text-popover-foreground shadow-2xl animate-in zoom-in-95 duration-150 flex flex-col gap-4 max-h-[92vh] overflow-y-auto",
                        onclick: move |e: MouseEvent| e.stop_propagation(),

                        // MODAL HEADER
                        div { class: "flex items-start justify-between border-b border-border/60 pb-3",
                            div { class: "flex flex-col gap-0.5",
                                div { class: "flex items-center gap-2",
                                    div { class: "flex size-7 items-center justify-center rounded bg-primary/10 text-primary",
                                        ClockIcon { class: "size-4" }
                                    }
                                    h3 { class: "font-bold text-sm text-foreground", "Clock & Date Scheduler" }
                                }
                                span { class: "text-[11px] text-muted-foreground",
                                    "Scroll mouse on date/time pills or move clock hands"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "p-1 rounded text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer text-base font-bold",
                                onclick: move |_| is_open.set(false),
                                "×"
                            }
                        }

                        // 1. DATE SELECTOR SECTION (MOUSE SCROLL + DIRECT TYPING)
                        div { class: "flex flex-col gap-2 rounded-lg bg-muted/30 border border-border/50 p-3",
                            div { class: "flex items-center justify-between",
                                span { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-wider flex items-center gap-1",
                                    CalendarIcon { class: "size-3 text-primary" }
                                    "1. Date Selection (Scroll Wheel or Type)"
                                }
                                span { class: "text-[10px] text-muted-foreground font-mono",
                                    "{MONTH_FULL_NAMES[(*month.read() as usize - 1).min(11)]} {year}"
                                }
                            }

                            // Interactive Mouse-Scrollable Date Pills
                            div { class: "grid grid-cols-3 gap-2 py-0.5",
                                // Day Scroll Pill
                                div {
                                    class: "flex flex-col items-center justify-center rounded-md border border-border/80 bg-background/90 py-1.5 px-2 select-none cursor-ns-resize hover:border-primary/80 transition-colors shadow-2xs",
                                    title: "Hover and scroll mouse wheel up/down to change day",
                                    onwheel: move |e: WheelEvent| {
                                        scroll_day(get_wheel_delta_y(&e));
                                    },
                                    span { class: "text-[9px] uppercase font-bold text-muted-foreground tracking-wider", "Day (Scroll)" }
                                    span { class: "text-lg font-mono font-extrabold text-foreground tracking-tight leading-none mt-1",
                                        "{day:02}"
                                    }
                                    div { class: "flex items-center gap-2 text-[10px] text-muted-foreground/60 mt-1",
                                        span { "▲" }
                                        span { "▼" }
                                    }
                                }

                                // Month Scroll Pill
                                div {
                                    class: "flex flex-col items-center justify-center rounded-md border border-border/80 bg-background/90 py-1.5 px-2 select-none cursor-ns-resize hover:border-primary/80 transition-colors shadow-2xs",
                                    title: "Hover and scroll mouse wheel up/down to change month",
                                    onwheel: move |e: WheelEvent| {
                                        scroll_month(get_wheel_delta_y(&e));
                                    },
                                    span { class: "text-[9px] uppercase font-bold text-muted-foreground tracking-wider", "Month (Scroll)" }
                                    span { class: "text-lg font-mono font-extrabold text-primary tracking-tight leading-none mt-1",
                                        "{MONTH_NAMES[(*month.read() as usize - 1).min(11)]}"
                                    }
                                    div { class: "flex items-center gap-2 text-[10px] text-muted-foreground/60 mt-1",
                                        span { "▲" }
                                        span { "▼" }
                                    }
                                }

                                // Year Scroll Pill
                                div {
                                    class: "flex flex-col items-center justify-center rounded-md border border-border/80 bg-background/90 py-1.5 px-2 select-none cursor-ns-resize hover:border-primary/80 transition-colors shadow-2xs",
                                    title: "Hover and scroll mouse wheel up/down to change year",
                                    onwheel: move |e: WheelEvent| {
                                        scroll_year(get_wheel_delta_y(&e));
                                    },
                                    span { class: "text-[9px] uppercase font-bold text-muted-foreground tracking-wider", "Year (Scroll)" }
                                    span { class: "text-lg font-mono font-extrabold text-foreground tracking-tight leading-none mt-1",
                                        "{year}"
                                    }
                                    div { class: "flex items-center gap-2 text-[10px] text-muted-foreground/60 mt-1",
                                        span { "▲" }
                                        span { "▼" }
                                    }
                                }
                            }

                            // Quick Date Presets & Direct Date Typing
                            div { class: "flex items-center justify-between gap-1.5 pt-1 flex-wrap",
                                div { class: "flex items-center gap-1 flex-wrap",
                                    button {
                                        r#type: "button",
                                        class: "rounded px-2 py-0.5 text-[10px] font-medium border border-border/80 bg-card hover:border-primary hover:text-primary transition-colors cursor-pointer",
                                        onclick: move |_| set_preset_days(0),
                                        "Today"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "rounded px-2 py-0.5 text-[10px] font-medium border border-border/80 bg-card hover:border-primary hover:text-primary transition-colors cursor-pointer",
                                        onclick: move |_| set_preset_days(1),
                                        "Tomorrow"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "rounded px-2 py-0.5 text-[10px] font-medium border border-border/80 bg-card hover:border-primary hover:text-primary transition-colors cursor-pointer",
                                        onclick: move |_| set_preset_days(2),
                                        "+2 Days"
                                    }
                                    button {
                                        r#type: "button",
                                        class: "rounded px-2 py-0.5 text-[10px] font-medium border border-border/80 bg-card hover:border-primary hover:text-primary transition-colors cursor-pointer",
                                        onclick: move |_| set_preset_days(7),
                                        "+1 Week"
                                    }
                                }

                                // Direct typed date input
                                div { class: "flex items-center gap-1 ml-auto",
                                    span { class: "text-[10px] text-muted-foreground font-mono", "Type:" }
                                    input {
                                        r#type: "text",
                                        placeholder: "YYYY-MM-DD",
                                        value: date_type_input.read().clone(),
                                        class: "h-6 w-24 rounded border border-border bg-background px-1.5 font-mono text-[11px] text-foreground focus:border-primary focus:outline-none",
                                        oninput: move |e: FormEvent| {
                                            let val = e.value();
                                            date_type_input.set(val.clone());
                                            if val.len() == 10 && val.as_bytes()[4] == b'-' && val.as_bytes()[7] == b'-' {
                                                if let (Ok(y), Ok(m), Ok(d)) = (val[0..4].parse::<u32>(), val[5..7].parse::<u32>(), val[8..10].parse::<u32>()) {
                                                    if (1..=12).contains(&m) && (1..=31).contains(&d) {
                                                        year.set(y);
                                                        month.set(m);
                                                        day.set(d);
                                                    }
                                                }
                                            }
                                        },
                                    }
                                }
                            }
                        }

                        // 2. ANALOG CLOCK & DIGITAL TIME SECTION
                        div { class: "flex flex-col items-center gap-3 rounded-lg bg-muted/20 border border-border/50 p-4",
                            // Digital Readout & Meridiem Toggle
                            div { class: "flex items-center justify-between w-full px-2",
                                span { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-wider",
                                    "2. Clock Time (Handle Drag or Scroll)"
                                }

                                // Interactive Digital Time Display
                                div { class: "flex items-center gap-1.5",
                                    // Hour Box
                                    button {
                                        r#type: "button",
                                        class: format!(
                                            "rounded-md px-2.5 py-1 font-mono text-base font-bold transition-all cursor-pointer border {}",
                                            if mode == ClockMode::Hour {
                                                "bg-primary text-primary-foreground border-primary shadow-xs scale-105"
                                            } else {
                                                "bg-background/80 text-foreground border-border/80 hover:border-primary/50"
                                            }
                                        ),
                                        title: "Click to set Hour or scroll wheel",
                                        onwheel: move |e: WheelEvent| {
                                            scroll_hour(get_wheel_delta_y(&e));
                                        },
                                        onclick: move |_| clock_mode.set(ClockMode::Hour),
                                        "{hour12:02}"
                                    }

                                    span { class: "font-mono font-bold text-muted-foreground animate-pulse text-base", ":" }

                                    // Minute Box
                                    button {
                                        r#type: "button",
                                        class: format!(
                                            "rounded-md px-2.5 py-1 font-mono text-base font-bold transition-all cursor-pointer border {}",
                                            if mode == ClockMode::Minute {
                                                "bg-primary text-primary-foreground border-primary shadow-xs scale-105"
                                            } else {
                                                "bg-background/80 text-foreground border-border/80 hover:border-primary/50"
                                            }
                                        ),
                                        title: "Click to set Minute or scroll wheel",
                                        onwheel: move |e: WheelEvent| {
                                            scroll_minute(get_wheel_delta_y(&e));
                                        },
                                        onclick: move |_| clock_mode.set(ClockMode::Minute),
                                        "{current_minute:02}"
                                    }

                                    // AM / PM Toggle
                                    div { class: "flex rounded-md border border-border/80 bg-background/90 p-0.5 ml-1 text-xs font-semibold",
                                        button {
                                            r#type: "button",
                                            class: format!(
                                                "rounded px-2 py-0.5 transition-colors cursor-pointer {}",
                                                if !is_pm { "bg-primary text-primary-foreground" } else { "text-muted-foreground hover:text-foreground" }
                                            ),
                                            onclick: move |_| toggle_meridiem(false),
                                            "AM"
                                        }
                                        button {
                                            r#type: "button",
                                            class: format!(
                                                "rounded px-2 py-0.5 transition-colors cursor-pointer {}",
                                                if is_pm { "bg-primary text-primary-foreground" } else { "text-muted-foreground hover:text-foreground" }
                                            ),
                                            onclick: move |_| toggle_meridiem(true),
                                            "PM"
                                        }
                                    }
                                }
                            }

                            // Interactive Analog Clock Dial
                            div {
                                class: "relative size-52 select-none rounded-full bg-card border-2 border-border/80 shadow-inner flex items-center justify-center cursor-pointer hover:border-primary/60 transition-colors",
                                title: if mode == ClockMode::Hour { "Click or drag handle to select hour" } else { "Click or drag handle to select minute" },
                                onwheel: move |e: WheelEvent| {
                                    match *clock_mode.read() {
                                        ClockMode::Hour => scroll_hour(get_wheel_delta_y(&e)),
                                        ClockMode::Minute => scroll_minute(get_wheel_delta_y(&e)),
                                    }
                                },
                                onmousedown: move |_| is_dragging.set(true),
                                onmouseup: move |_| {
                                    if *is_dragging.read() {
                                        is_dragging.set(false);
                                        // Auto advance from hour to minute mode
                                        if *clock_mode.read() == ClockMode::Hour {
                                            clock_mode.set(ClockMode::Minute);
                                        }
                                    }
                                },
                                onclick: move |e: MouseEvent| {
                                    // Direct click sets handle
                                    let data = e.data();
                                    handle_clock_pointer(
                                        data.client_coordinates().x,
                                        data.client_coordinates().y,
                                        data.client_coordinates().x - data.element_coordinates().x,
                                        data.client_coordinates().y - data.element_coordinates().y,
                                        208.0,
                                        208.0,
                                    );
                                    if *clock_mode.read() == ClockMode::Hour {
                                        clock_mode.set(ClockMode::Minute);
                                    }
                                },
                                onmousemove: move |e: MouseEvent| {
                                    if *is_dragging.read() {
                                        let data = e.data();
                                        handle_clock_pointer(
                                            data.client_coordinates().x,
                                            data.client_coordinates().y,
                                            data.client_coordinates().x - data.element_coordinates().x,
                                            data.client_coordinates().y - data.element_coordinates().y,
                                            208.0,
                                            208.0,
                                        );
                                    }
                                },

                                svg {
                                    view_box: "0 0 200 200",
                                    class: "size-full",

                                    // Clock dial background circle
                                    circle {
                                        cx: "100",
                                        cy: "100",
                                        r: "96",
                                        fill: "none",
                                        stroke: "currentColor",
                                        stroke_width: "1",
                                        stroke_opacity: "0.1",
                                    }

                                    // Minute tick marks around rim
                                    for i in 0..60 {
                                        {
                                            let is_major = i % 5 == 0;
                                            let rad = (i as f64 * 6.0).to_radians();
                                            let r1 = if is_major { 88.0 } else { 92.0 };
                                            let r2 = 95.0;
                                            let x1 = 100.0 + r1 * rad.sin();
                                            let y1 = 100.0 - r1 * rad.cos();
                                            let x2 = 100.0 + r2 * rad.sin();
                                            let y2 = 100.0 - r2 * rad.cos();
                                            rsx! {
                                                line {
                                                    key: "tick-{i}",
                                                    x1: "{x1}",
                                                    y1: "{y1}",
                                                    x2: "{x2}",
                                                    y2: "{y2}",
                                                    stroke: "currentColor",
                                                    stroke_width: if is_major { "1.5" } else { "0.8" },
                                                    stroke_opacity: if is_major { "0.4" } else { "0.15" },
                                                }
                                            }
                                        }
                                    }

                                    // Radially positioned Numbers
                                    if mode == ClockMode::Hour {
                                        for h in 1..=12 {
                                            {
                                                let is_selected = h == hour12;
                                                let rad = (h as f64 * 30.0).to_radians();
                                                let x = 100.0 + 72.0 * rad.sin();
                                                let y = 100.0 - 72.0 * rad.cos();
                                                rsx! {
                                                    g {
                                                        key: "num-h-{h}",
                                                        if is_selected {
                                                            circle {
                                                                cx: "{x}",
                                                                cy: "{y}",
                                                                r: "13",
                                                                class: "fill-primary text-primary shadow-md",
                                                            }
                                                        }
                                                        text {
                                                            x: "{x}",
                                                            y: "{y + 4.0}",
                                                            text_anchor: "middle",
                                                            class: format!(
                                                                "font-mono font-bold text-xs select-none pointer-events-none {}",
                                                                if is_selected { "fill-primary-foreground font-extrabold" } else { "fill-foreground/80" }
                                                            ),
                                                            "{h}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        for m in (0..60).step_by(5) {
                                            {
                                                let is_selected = m == current_minute;
                                                let rad = (m as f64 * 6.0).to_radians();
                                                let x = 100.0 + 72.0 * rad.sin();
                                                let y = 100.0 - 72.0 * rad.cos();
                                                rsx! {
                                                    g {
                                                        key: "num-m-{m}",
                                                        if is_selected {
                                                            circle {
                                                                cx: "{x}",
                                                                cy: "{y}",
                                                                r: "13",
                                                                class: "fill-primary text-primary shadow-md",
                                                            }
                                                        }
                                                        text {
                                                            x: "{x}",
                                                            y: "{y + 4.0}",
                                                            text_anchor: "middle",
                                                            class: format!(
                                                                "font-mono font-bold text-[10px] select-none pointer-events-none {}",
                                                                if is_selected { "fill-primary-foreground font-extrabold" } else { "fill-foreground/80" }
                                                            ),
                                                            "{m:02}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // CLOCK HAND (POINTER TO HANDLE)
                                    line {
                                        x1: "100",
                                        y1: "100",
                                        x2: "{hand_tip_x}",
                                        y2: "{hand_tip_y}",
                                        stroke: "currentColor",
                                        class: "text-primary stroke-primary",
                                        stroke_width: if mode == ClockMode::Hour { "3.5" } else { "2.5" },
                                        stroke_linecap: "round",
                                    }

                                    // Tip of hand selector knob
                                    circle {
                                        cx: "{hand_tip_x}",
                                        cy: "{hand_tip_y}",
                                        r: if mode == ClockMode::Hour { "4" } else { "3.5" },
                                        class: "fill-primary",
                                    }

                                    // Center Pivot Pin
                                    circle {
                                        cx: "100",
                                        cy: "100",
                                        r: "5",
                                        class: "fill-primary ring-2 ring-background",
                                    }
                                }
                            }

                            // Mode Switch Pill Buttons
                            div { class: "flex items-center gap-2 text-xs",
                                button {
                                    r#type: "button",
                                    class: format!(
                                        "rounded-full px-3 py-1 font-semibold text-xs border transition-colors cursor-pointer {}",
                                        if mode == ClockMode::Hour { "bg-primary text-primary-foreground border-primary" } else { "bg-background text-muted-foreground border-border hover:text-foreground" }
                                    ),
                                    onclick: move |_| clock_mode.set(ClockMode::Hour),
                                    "Set Hour Hand"
                                }
                                button {
                                    r#type: "button",
                                    class: format!(
                                        "rounded-full px-3 py-1 font-semibold text-xs border transition-colors cursor-pointer {}",
                                        if mode == ClockMode::Minute { "bg-primary text-primary-foreground border-primary" } else { "bg-background text-muted-foreground border-border hover:text-foreground" }
                                    ),
                                    onclick: move |_| clock_mode.set(ClockMode::Minute),
                                    "Set Minute Hand"
                                }
                            }
                        }

                        // PREVIEW SUMMARY BANNER
                        div { class: "flex items-center justify-between rounded-md bg-primary/10 border border-primary/20 px-3 py-2",
                            div { class: "flex flex-col gap-0.5 min-w-0",
                                span { class: "text-[9px] font-bold text-primary uppercase tracking-wider", "Scheduled Result (ISO 8601)" }
                                span { class: "font-mono font-bold text-xs text-foreground truncate",
                                    "{format_human_datetime(*year.read(), *month.read(), *day.read(), *hour24.read(), *minute.read())}"
                                }
                            }
                            span { class: "font-mono text-[10px] text-muted-foreground shrink-0 ml-2 bg-background/60 px-1.5 py-0.5 rounded border border-border/40",
                                "{hour24:02}:{minute:02}"
                            }
                        }

                        // MODAL ACTIONS FOOTER
                        div { class: "flex items-center justify-between gap-2 pt-2 border-t border-border/50",
                            div { class: "flex items-center gap-1.5",
                                button {
                                    r#type: "button",
                                    class: "rounded-md border border-border/80 bg-card px-2.5 py-1.5 text-xs font-medium text-foreground hover:bg-muted transition-colors cursor-pointer",
                                    onclick: move |_| set_to_now(),
                                    "Now"
                                }
                                button {
                                    r#type: "button",
                                    class: "rounded-md border border-border/80 bg-card px-2.5 py-1.5 text-xs font-medium text-muted-foreground hover:text-destructive hover:border-destructive transition-colors cursor-pointer",
                                    onclick: move |_| clear_datetime(),
                                    "Clear"
                                }
                            }

                            div { class: "flex items-center gap-2",
                                button {
                                    r#type: "button",
                                    class: "rounded-md border border-border/80 bg-card px-3 py-1.5 text-xs font-medium text-foreground hover:bg-muted transition-colors cursor-pointer",
                                    onclick: move |_| is_open.set(false),
                                    "Cancel"
                                }
                                button {
                                    r#type: "button",
                                    class: "inline-flex items-center gap-1.5 rounded-md bg-primary text-primary-foreground px-4 py-1.5 text-xs font-bold hover:bg-primary/90 shadow-xs transition-colors cursor-pointer",
                                    onclick: move |_| apply_datetime(),
                                    CheckIcon { class: "size-3.5" }
                                    "Set Schedule"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
