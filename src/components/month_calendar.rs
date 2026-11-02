//! Calendar domain types shared by the UI kit
//! (`calendar.rs`, `date_range_picker.rs`, and `calendar_page.rs`).

#![allow(dead_code)]

/// An inclusive date range: `((start_y, start_m, start_d), (end_y, end_m, end_d))`.
pub type DateRange = ((u32, u32, u32), (u32, u32, u32));

/// Accent color for an event dot or card on the calendar grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventColor {
    Blue,
    Green,
    Amber,
    Red,
    Purple,
    Slate,
}

impl EventColor {
    /// Tailwind background class used for small event dots.
    pub fn dot_class(&self) -> &'static str {
        match self {
            EventColor::Blue => "bg-info",
            EventColor::Green => "bg-success",
            EventColor::Amber => "bg-warning",
            EventColor::Red => "bg-destructive",
            EventColor::Purple => "bg-purple-500",
            EventColor::Slate => "bg-muted-foreground",
        }
    }

    /// Color styling for event pill / card in calendar grid: (bg_class, text_class, border_class).
    pub fn pill_classes(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            EventColor::Blue => (
                "bg-info/12 hover:bg-info/22",
                "text-info",
                "border-info/35",
            ),
            EventColor::Green => (
                "bg-success/12 hover:bg-success/22",
                "text-success",
                "border-success/35",
            ),
            EventColor::Amber => (
                "bg-warning/15 hover:bg-warning/25",
                "text-warning-foreground font-medium",
                "border-warning/40",
            ),
            EventColor::Red => (
                "bg-destructive/12 hover:bg-destructive/22",
                "text-destructive",
                "border-destructive/35",
            ),
            EventColor::Purple => (
                "bg-purple-500/15 hover:bg-purple-500/25",
                "text-purple-400",
                "border-purple-500/35",
            ),
            EventColor::Slate => (
                "bg-muted hover:bg-muted/80",
                "text-muted-foreground",
                "border-border",
            ),
        }
    }

    /// Accent left border class for cards and agenda items.
    pub fn border_accent_class(&self) -> &'static str {
        match self {
            EventColor::Blue => "border-l-info",
            EventColor::Green => "border-l-success",
            EventColor::Amber => "border-l-warning",
            EventColor::Red => "border-l-destructive",
            EventColor::Purple => "border-l-purple-500",
            EventColor::Slate => "border-l-muted-foreground",
        }
    }

    /// Badge class for category tag.
    pub fn badge_class(&self) -> &'static str {
        match self {
            EventColor::Blue => "border-info/30 bg-info/15 text-info",
            EventColor::Green => "border-success/30 bg-success/15 text-success",
            EventColor::Amber => "border-warning/30 bg-warning/20 text-warning-foreground",
            EventColor::Red => "border-destructive/30 bg-destructive/15 text-destructive",
            EventColor::Purple => "border-purple-500/30 bg-purple-500/15 text-purple-400",
            EventColor::Slate => "border-border bg-muted text-muted-foreground",
        }
    }

    /// Label name for the color.
    pub fn name(&self) -> &'static str {
        match self {
            EventColor::Blue => "Blue",
            EventColor::Green => "Green",
            EventColor::Amber => "Amber",
            EventColor::Red => "Red",
            EventColor::Purple => "Purple",
            EventColor::Slate => "Slate",
        }
    }
}

/// A calendar event with full rich details, participants, and schedule times.
#[derive(Clone, Debug, PartialEq)]
pub struct CalEvent {
    pub id: u32,
    pub title: String,
    /// Day the event starts on as `(year, month, day)`.
    pub date: (u32, u32, u32),
    /// Optional last day for multi-day events (inclusive).
    pub end_date: Option<(u32, u32, u32)>,
    /// Display start time, e.g. `"10:00 AM"`. `None` means all day.
    pub time: Option<String>,
    /// Optional display end time, e.g. `"11:30 AM"`.
    pub end_time: Option<String>,
    pub color: EventColor,
    pub category: String,
    pub description: String,
    pub location: Option<String>,
    pub attendees: Vec<String>,
    pub priority: String,
}

impl CalEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u32,
        title: impl Into<String>,
        date: (u32, u32, u32),
        time: Option<&str>,
        end_time: Option<&str>,
        color: EventColor,
        category: impl Into<String>,
        description: impl Into<String>,
        location: Option<&str>,
        attendees: &[&str],
        priority: impl Into<String>,
    ) -> Self {
        Self {
            id,
            title: title.into(),
            date,
            end_date: None,
            time: time.map(|s| s.to_string()),
            end_time: end_time.map(|s| s.to_string()),
            color,
            category: category.into(),
            description: description.into(),
            location: location.map(|s| s.to_string()),
            attendees: attendees.iter().map(|s| s.to_string()).collect(),
            priority: priority.into(),
        }
    }

    /// Formatted time span string (e.g. "10:00 AM - 11:30 AM" or "All day").
    pub fn time_display(&self) -> String {
        match (&self.time, &self.end_time) {
            (Some(s), Some(e)) => format!("{s} – {e}"),
            (Some(s), None) => s.clone(),
            (None, _) => "All day".to_string(),
        }
    }
}

/// Whether `event` occurs on the given `(year, month, day)`.
pub fn is_event_on(event: &CalEvent, date: (u32, u32, u32)) -> bool {
    match event.end_date {
        Some(end) => date >= event.date && date <= end,
        None => date == event.date,
    }
}

pub fn is_leap_year(year: u32) -> bool {
    (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400)
}

pub fn days_in_month(year: u32, month: u32) -> u32 {
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

/// Sakamoto's algorithm: Day of week for 1st of month (0 = Sunday, 1 = Monday, ..., 6 = Saturday)
pub fn first_day_of_month(year: u32, month: u32) -> usize {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    ((y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + 1) % 7) as usize
}

/// Day of week for any date (0 = Sunday, 1 = Monday, ..., 6 = Saturday)
pub fn day_of_week(year: u32, month: u32, day: u32) -> usize {
    let t = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let mut y = year;
    if month < 3 {
        y -= 1;
    }
    ((y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day) % 7) as usize
}

pub const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June", "July", "August", "September",
    "October", "November", "December",
];

pub const SHORT_MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub const WEEKDAY_NAMES: [&str; 7] = [
    "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday",
];

pub const WEEKDAY_SHORT_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

pub fn format_date_pretty(year: u32, month: u32, day: u32) -> String {
    let dow = day_of_week(year, month, day);
    let day_name = WEEKDAY_NAMES[dow];
    let month_name = if (1..=12).contains(&month) {
        SHORT_MONTH_NAMES[(month - 1) as usize]
    } else {
        "?"
    };
    format!("{day_name}, {month_name} {day}, {year}")
}
