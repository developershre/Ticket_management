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

/// Formats a Unix timestamp (seconds) as `YYYY-MM-DD HH:MM:SS` in UTC.
///
/// Used by the sync layer for queue `created_at`, conflict `detected_at`
/// and "last successful sync" display values.
pub fn format_utc(epoch_secs: i64) -> String {
    let days = epoch_secs.div_euclid(86_400);
    let secs = epoch_secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let h = secs / 3600;
    let mi = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}")
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

/// Context signal wrapper to open or close the settings modal from anywhere in the app.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub struct OpenSettingsDialog(pub bool);

/// Global application appearance and behavior settings.
#[derive(Clone, PartialEq, Debug)]
pub struct AppSettings {
    pub sidebar_position: String,
    pub card_density: String,
    pub animations_enabled: bool,
    pub date_format: String,
    pub font_scale: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self::load_from_db()
    }
}

impl AppSettings {
    pub fn load_from_db() -> Self {
        let is_24 = crate::db::sync_meta_get("ui_date_format")
            .map(|v| v == "24hr")
            .unwrap_or(false);
        set_time_format_24hr(is_24);

        Self {
            sidebar_position: crate::db::sync_meta_get("ui_sidebar_position").unwrap_or_else(|| "left".to_string()),
            card_density: crate::db::sync_meta_get("ui_card_density").unwrap_or_else(|| "default".to_string()),
            animations_enabled: crate::db::sync_meta_get("ui_animations_enabled").map(|v| v != "false").unwrap_or(true),
            date_format: if is_24 { "24hr".to_string() } else { "12hr".to_string() },
            font_scale: crate::db::sync_meta_get("ui_font_scale").unwrap_or_else(|| "auto".to_string()),
        }
    }
}

static TIME_FORMAT_24HR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_time_format_24hr(is_24hr: bool) {
    TIME_FORMAT_24HR.store(is_24hr, std::sync::atomic::Ordering::Relaxed);
}

pub fn is_time_format_24hr() -> bool {
    TIME_FORMAT_24HR.load(std::sync::atomic::Ordering::Relaxed)
}

/// Global reactive revision counter to signal when SQLite database tables
/// (like NOTES or tickets) have been modified by FAB or dialogs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AppDbRevision(pub u64);

const SHORT_MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Application standard local timezone offset (Asia/Kolkata = UTC+05:30).
pub const LOCAL_TZ_OFFSET_SECS: i64 = 19_800; // 5 hours 30 minutes

/// Converts civil date parts (year, month, day) into epoch seconds at 00:00:00 UTC.
pub fn epoch_from_civil(year: u32, month: u32, day: u32) -> i64 {
    let y = year as i64 - if month <= 2 { 1 } else { 0 };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let m = month as i64 + if month > 2 { -3 } else { 9 };
    let doy = (153 * m + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days * 86_400
}

/// Converts epoch seconds into civil date (year, month, day).
pub fn civil_from_epoch(epoch: i64) -> (u32, u32, u32) {
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

fn parse_tz_offset(s: &str, sign: i64) -> Option<i64> {
    let clean = s.trim_start_matches(['+', '-']);
    if clean.len() >= 5 && clean.as_bytes()[2] == b':' {
        let oh: i64 = clean[0..2].parse().ok()?;
        let om: i64 = clean[3..5].parse().ok()?;
        Some(sign * (oh * 3600 + om * 60))
    } else if clean.len() >= 4 {
        let oh: i64 = clean[0..2].parse().ok()?;
        let om: i64 = clean[2..4].parse().ok()?;
        Some(sign * (oh * 3600 + om * 60))
    } else {
        None
    }
}

/// Helper to parse datetime parts from flexible string formats (ISO 8601, SQL, etc.)
/// and convert any UTC timestamp (ending in 'Z' or offset) to local civil parts in Asia/Kolkata (UTC+05:30).
pub fn parse_datetime_flexible(s: &str) -> Option<(u32, u32, u32, u32, u32)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
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
                return Some((y, m, d, 0, 0));
            };
            if time_part.len() >= 5 {
                let h: u32 = time_part[0..2].parse().ok()?;
                let mi: u32 = time_part[3..5].parse().ok()?;
                let sec: u32 = if time_part.len() >= 8 && time_part.as_bytes()[5] == b':' {
                    time_part[6..8].parse().unwrap_or(0)
                } else {
                    0
                };

                // Check for explicit timezone indicator
                let is_utc_z = time_part.ends_with('Z') || time_part.ends_with('z');
                let offset_opt = if let Some(plus_idx) = time_part.find('+') {
                    parse_tz_offset(&time_part[plus_idx..], 1)
                } else if let Some(minus_idx) = time_part[2..].find('-').map(|i| i + 2) {
                    parse_tz_offset(&time_part[minus_idx..], -1)
                } else if is_utc_z {
                    Some(0)
                } else {
                    None
                };

                match offset_opt {
                    Some(offset_secs) if offset_secs == LOCAL_TZ_OFFSET_SECS => {
                        // Already in local timezone (+05:30)
                        return Some((y, m, d, h, mi));
                    }
                    Some(offset_secs) => {
                        // Convert to UTC epoch, then add local offset (+19,800s)
                        let civil_epoch = epoch_from_civil(y, m, d)
                            + (h as i64 * 3600)
                            + (mi as i64 * 60)
                            + (sec as i64);
                        let utc_epoch = civil_epoch - offset_secs;
                        let local_epoch = utc_epoch + LOCAL_TZ_OFFSET_SECS;

                        let (ly, lm, ld) = civil_from_epoch(local_epoch);
                        let lsecs = local_epoch.rem_euclid(86_400);
                        let lh = (lsecs / 3600) as u32;
                        let lmi = ((lsecs % 3600) / 60) as u32;
                        return Some((ly, lm, ld, lh, lmi));
                    }
                    None => {
                        // No timezone indicator provided -> default to UTC since SQLite
                        // CURRENT_TIMESTAMP and Google Sheets/Apps Script server values are in UTC.
                        let civil_epoch = epoch_from_civil(y, m, d)
                            + (h as i64 * 3600)
                            + (mi as i64 * 60)
                            + (sec as i64);
                        let local_epoch = civil_epoch + LOCAL_TZ_OFFSET_SECS;

                        let (ly, lm, ld) = civil_from_epoch(local_epoch);
                        let lsecs = local_epoch.rem_euclid(86_400);
                        let lh = (lsecs / 3600) as u32;
                        let lmi = ((lsecs % 3600) / 60) as u32;
                        return Some((ly, lm, ld, lh, lmi));
                    }
                }
            }
        }
        return Some((y, m, d, 0, 0));
    }
    None
}

/// Normalizes any timestamp string into standard ISO 8601 format with explicit +05:30 offset.
/// Example: "2026-11-05T07:00:00.000Z" -> "2026-11-05T12:30:00+05:30"
/// Example: "2026-11-05 12:30:00"      -> "2026-11-05T12:30:00+05:30"
/// Example: "2026-11-05"               -> "2026-11-05T09:00:00+05:30"
pub fn to_ist_iso8601(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if let Some((y, m, d, h, mi)) = parse_datetime_flexible(trimmed) {
        format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:00+05:30")
    } else {
        trimmed.to_string()
    }
}

/// Formats any ISO 8601 or SQL timestamp for display, obeying the user's
/// 12-hour or 24-hour preference configured in Settings.
pub fn format_display_12hr(raw: &str) -> String {
    format_display_time(raw)
}

/// Formats any ISO 8601 or SQL timestamp for the UI according to current settings.
/// Examples:
/// - In 12-hour mode: "2026-11-06T14:30:00+05:30" -> "Nov 06, 2026 · 02:30 PM"
/// - In 24-hour mode: "2026-11-06T14:30:00+05:30" -> "Nov 06, 2026 · 14:30"
pub fn format_display_time(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    // If it's already a relative word, return as-is
    if s.starts_with("Today") || s.starts_with("Tomorrow") || s.starts_with("Next Week") || s == "Not sent yet" || s == "Not sent" {
        return s.to_string();
    }

    if let Some((y, m, d, h, mi)) = parse_datetime_flexible(s) {
        let month_idx = (m.saturating_sub(1) as usize).min(11);
        let month_name = SHORT_MONTH_NAMES[month_idx];
        if s.len() >= 16 || s.contains('T') || s.contains(':') {
            if is_time_format_24hr() {
                format!("{} {:02}, {:04} · {:02}:{:02}", month_name, d, y, h, mi)
            } else {
                let am_pm = if h >= 12 { "PM" } else { "AM" };
                let h12 = match h % 12 {
                    0 => 12,
                    other => other,
                };
                format!("{} {:02}, {:04} · {:02}:{:02} {}", month_name, d, y, h12, mi, am_pm)
            }
        } else {
            format!("{} {:02}, {:04}", month_name, d, y)
        }
    } else {
        s.to_string()
    }
}
