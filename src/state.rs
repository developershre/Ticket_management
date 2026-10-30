//! Shared app state helpers.
//!
//! The UI kit (`src/components/ui/calendar.rs`, `date_range_picker.rs`)
//! calls [`get_real_current_date`] to seed calendars with today's date.

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current UTC date as `(year, month, day)`.
///
/// Uses `std` only (no chrono dependency) via Howard Hinnant's
/// "civil_from_days" algorithm.
pub fn get_real_current_date() -> (u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    civil_from_days((secs / 86_400) as i64)
}

/// Converts a count of days since 1970-01-01 into a civil (year, month, day).
fn civil_from_days(z: i64) -> (u32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    (y as u32, m as u32, d as u32)
}
