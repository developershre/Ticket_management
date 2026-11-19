//! Analytics data types and date calculation helpers.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeRange {
    AllTime,
    Today,
    Last7Days,
    Last30Days,
    ThisMonth,
    ThisYear,
}

impl TimeRange {
    pub fn label(&self) -> &'static str {
        match self {
            TimeRange::AllTime => "All Time",
            TimeRange::Today => "Today",
            TimeRange::Last7Days => "Last 7 Days",
            TimeRange::Last30Days => "Last 30 Days",
            TimeRange::ThisMonth => "This Month",
            TimeRange::ThisYear => "This Year",
        }
    }
}

/// Computes days between two YMD dates approximately for range comparison.
pub fn date_to_days(y: u32, m: u32, d: u32) -> u64 {
    (y as u64) * 365 + (m as u64) * 31 + (d as u64)
}

pub fn is_within_range(item_date: (u32, u32, u32), today: (u32, u32, u32), range: TimeRange) -> bool {
    match range {
        TimeRange::AllTime => true,
        TimeRange::Today => item_date == today,
        TimeRange::Last7Days => {
            let today_days = date_to_days(today.0, today.1, today.2);
            let item_days = date_to_days(item_date.0, item_date.1, item_date.2);
            item_days <= today_days && today_days.saturating_sub(item_days) <= 7
        }
        TimeRange::Last30Days => {
            let today_days = date_to_days(today.0, today.1, today.2);
            let item_days = date_to_days(item_date.0, item_date.1, item_date.2);
            item_days <= today_days && today_days.saturating_sub(item_days) <= 30
        }
        TimeRange::ThisMonth => item_date.0 == today.0 && item_date.1 == today.1,
        TimeRange::ThisYear => item_date.0 == today.0,
    }
}

/// Helper to shift a date by N days forwards or backwards.
pub fn add_days(mut y: u32, mut m: u32, mut d: u32, delta: i32) -> (u32, u32, u32) {
    if delta > 0 {
        for _ in 0..delta {
            d += 1;
            if d > crate::components::month_calendar::days_in_month(y, m) {
                d = 1;
                m += 1;
                if m > 12 {
                    m = 1;
                    y += 1;
                }
            }
        }
    } else if delta < 0 {
        for _ in 0..(-delta) {
            if d > 1 {
                d -= 1;
            } else {
                if m > 1 {
                    m -= 1;
                } else {
                    m = 12;
                    y -= 1;
                }
                d = crate::components::month_calendar::days_in_month(y, m);
            }
        }
    }
    (y, m, d)
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssigneeStats {
    pub name: String,
    pub total: usize,
    pub solved: usize,
    pub in_progress: usize,
    pub hold: usize,
    pub pending: usize,
    pub total_hours: f64,
    pub resolution_rate: f64,
    pub avg_rating: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CategoryStat {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
    pub color: &'static str,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DepartmentStat {
    pub name: String,
    pub count: usize,
    pub percentage: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MonthlyVolume {
    pub year: u32,
    pub month: u32,
    pub count: usize,
    pub label: String,
}
