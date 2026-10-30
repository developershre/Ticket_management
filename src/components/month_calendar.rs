//! Calendar domain types shared by the UI kit
//! (`calendar.rs` and `date_range_picker.rs`).

/// An inclusive date range: `((start_y, start_m, start_d), (end_y, end_m, end_d))`.
pub type DateRange = ((u32, u32, u32), (u32, u32, u32));

/// Accent color for an event dot on the calendar grid.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EventColor {
    Blue,
    Green,
    Amber,
    Red,
    Purple,
    Slate,
}

impl EventColor {
    /// Tailwind background class used for the small event dots.
    pub fn dot_class(&self) -> &'static str {
        match self {
            EventColor::Blue => "bg-blue-500",
            EventColor::Green => "bg-emerald-500",
            EventColor::Amber => "bg-amber-500",
            EventColor::Red => "bg-red-500",
            EventColor::Purple => "bg-violet-500",
            EventColor::Slate => "bg-zinc-400",
        }
    }
}

/// A single calendar event rendered as a dot (and tooltip entry) on a day.
#[derive(Clone, Debug, PartialEq)]
pub struct CalEvent {
    pub id: u32,
    pub title: String,
    /// Day the event starts on as `(year, month, day)`.
    pub date: (u32, u32, u32),
    /// Optional last day for multi-day events (inclusive).
    pub end_date: Option<(u32, u32, u32)>,
    /// Optional display time, e.g. `"14:30"`. `None` means all day.
    pub time: Option<String>,
    pub color: EventColor,
}

/// Whether `event` occurs on the given `(year, month, day)`.
pub fn is_event_on(event: &CalEvent, date: (u32, u32, u32)) -> bool {
    match event.end_date {
        // Tuples compare lexicographically, which is (year, month, day) order.
        Some(end) => date >= event.date && date <= end,
        None => date == event.date,
    }
}
