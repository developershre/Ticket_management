//! Shared app state helpers.
//!
//! The UI kit (`src/components/ui/calendar.rs`, `date_range_picker.rs`)
//! calls [`get_real_current_date`] to seed calendars with today's date.

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current local date (Asia/Kolkata, UTC+05:30) as `(year, month, day)`.
///
/// Uses `std` only (no chrono dependency) via Howard Hinnant's
/// "civil_from_days" algorithm.
pub fn get_real_current_date() -> (u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let local_secs = secs + LOCAL_TZ_OFFSET_SECS;

    civil_from_days(local_secs.div_euclid(86_400))
}

/// Returns current local time in Asia/Kolkata (UTC+05:30) as `(year, month, day, hour, minute, second)`.
pub fn get_local_now_parts() -> (u32, u32, u32, u32, u32, u32) {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let local_secs = secs + LOCAL_TZ_OFFSET_SECS;
    let days = local_secs.div_euclid(86_400);
    let rem = local_secs.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    let h = (rem / 3600) as u32;
    let mi = ((rem % 3600) / 60) as u32;
    let s = (rem % 60) as u32;
    (y, m, d, h, mi, s)
}

/// Formats current local time in Asia/Kolkata (UTC+05:30) as `YYYY-MM-DD HH:MM:SS`.
pub fn format_local_now() -> String {
    let (y, m, d, h, mi, s) = get_local_now_parts();
    format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}")
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
    // Appearance & Theming
    pub sidebar_position: String,
    pub card_density: String,
    pub animations_enabled: bool,
    pub date_format: String,
    pub font_scale: String,
    pub font_family: String,
    pub accent_color: String,
    pub corner_radius: String,
    pub oled_dark: bool,

    // Workflow & Tickets
    pub default_landing_page: String,
    pub default_ticket_view: String,
    pub default_priority: String,
    pub default_category: String,
    pub default_assignee: String,
    pub kanban_hide_cancelled: bool,
    pub kanban_hide_solved: bool,

    // Notifications & Audio
    pub toast_duration_secs: u32,
    pub sound_enabled: bool,
    pub sound_volume: String,
    pub reminder_advance_mins: u32,
    pub notify_on_remote_pull: bool,
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

        let date_pattern = crate::db::sync_meta_get("ui_date_pattern").unwrap_or_else(|| "medium".to_string());
        set_date_display_style(&date_pattern);

        let sound_on = crate::db::sync_meta_get("ui_sound_enabled")
            .map(|v| v != "false")
            .unwrap_or(true);
        set_sound_enabled(sound_on);

        let sound_vol = crate::db::sync_meta_get("ui_sound_volume")
            .unwrap_or_else(|| "medium".to_string());
        set_sound_volume(&sound_vol);

        Self {
            sidebar_position: crate::db::sync_meta_get("ui_sidebar_position").unwrap_or_else(|| "left".to_string()),
            card_density: crate::db::sync_meta_get("ui_card_density").unwrap_or_else(|| "default".to_string()),
            animations_enabled: crate::db::sync_meta_get("ui_animations_enabled").map(|v| v != "false").unwrap_or(true),
            date_format: if is_24 { "24hr".to_string() } else { "12hr".to_string() },
            font_scale: crate::db::sync_meta_get("ui_font_scale").unwrap_or_else(|| "auto".to_string()),
            font_family: crate::db::sync_meta_get("ui_font_family").unwrap_or_else(|| "roboto".to_string()),
            accent_color: crate::db::sync_meta_get("ui_accent_color").unwrap_or_else(|| "default".to_string()),
            corner_radius: crate::db::sync_meta_get("ui_corner_radius").unwrap_or_else(|| "rounded".to_string()),
            oled_dark: crate::db::sync_meta_get("ui_oled_dark").map(|v| v == "true").unwrap_or(false),

            default_landing_page: crate::db::sync_meta_get("ui_default_landing_page").unwrap_or_else(|| "tickets".to_string()),
            default_ticket_view: crate::db::sync_meta_get("ui_default_ticket_view").unwrap_or_else(|| "kanban".to_string()),
            default_priority: crate::db::sync_meta_get("ui_default_priority").unwrap_or_else(|| "Medium".to_string()),
            default_category: crate::db::sync_meta_get("ui_default_category").unwrap_or_else(|| "General".to_string()),
            default_assignee: crate::db::sync_meta_get("ui_default_assignee").unwrap_or_else(|| "IT Support".to_string()),
            kanban_hide_cancelled: crate::db::sync_meta_get("ui_kanban_hide_cancelled").map(|v| v == "true").unwrap_or(false),
            kanban_hide_solved: crate::db::sync_meta_get("ui_kanban_hide_solved").map(|v| v == "true").unwrap_or(false),

            toast_duration_secs: crate::db::sync_meta_get("ui_toast_duration_secs")
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            sound_enabled: sound_on,
            sound_volume: sound_vol,
            reminder_advance_mins: crate::db::sync_meta_get("ui_reminder_advance_mins")
                .and_then(|v| v.parse().ok())
                .unwrap_or(15),
            notify_on_remote_pull: crate::db::sync_meta_get("sync_notify_on_pull")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}

static TIME_FORMAT_24HR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static DATE_DISPLAY_STYLE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0); // 0 = Medium, 1 = DMY, 2 = ISO, 3 = D-MMM-Y
static SOUND_ENABLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static SOUND_VOLUME_PERCENT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(15);

pub fn set_time_format_24hr(is_24hr: bool) {
    TIME_FORMAT_24HR.store(is_24hr, std::sync::atomic::Ordering::Relaxed);
}

pub fn is_time_format_24hr() -> bool {
    TIME_FORMAT_24HR.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_date_display_style(style: &str) {
    let val = match style.to_lowercase().as_str() {
        "dmy" | "indian" | "uk" => 1,
        "iso" => 2,
        "d-mmm-y" | "dmy_dash" => 3,
        _ => 0,
    };
    DATE_DISPLAY_STYLE.store(val, std::sync::atomic::Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn get_date_display_style() -> &'static str {
    match DATE_DISPLAY_STYLE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => "dmy",
        2 => "iso",
        3 => "d-mmm-y",
        _ => "medium",
    }
}

pub fn is_sound_enabled() -> bool {
    SOUND_ENABLED.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn set_sound_enabled(enabled: bool) {
    SOUND_ENABLED.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

pub fn get_sound_volume_float() -> f32 {
    let p = SOUND_VOLUME_PERCENT.load(std::sync::atomic::Ordering::Relaxed);
    (p as f32) / 100.0
}

pub fn set_sound_volume(vol: &str) {
    let percent = match vol.to_lowercase().as_str() {
        "mute" => 0,
        "low" => 8,
        "high" => 30,
        "max" => 50,
        _ => 15,
    };
    SOUND_VOLUME_PERCENT.store(percent, std::sync::atomic::Ordering::Relaxed);
}

/// Plays a pleasant synthesized sound effect via standard Web Audio API.
pub fn play_sound(sound_type: &str) {
    if !is_sound_enabled() {
        return;
    }
    let vol = get_sound_volume_float();
    let js = format!(
        r#"
        (function() {{
            try {{
                var ctx = new (window.AudioContext || window.webkitAudioContext)();
                if (ctx.state === 'suspended') {{ ctx.resume(); }}
                var osc = ctx.createOscillator();
                var gain = ctx.createGain();
                gain.gain.value = {vol};
                osc.connect(gain);
                gain.connect(ctx.destination);
                var t = '{sound_type}';
                if (t === 'chime' || t === 'success' || t === 'arpeggio') {{
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(523.25, ctx.currentTime);
                    osc.frequency.exponentialRampToValueAtTime(659.25, ctx.currentTime + 0.1);
                    osc.frequency.exponentialRampToValueAtTime(783.99, ctx.currentTime + 0.2);
                    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.4);
                    osc.start();
                    osc.stop(ctx.currentTime + 0.41);
                }} else if (t === 'ping' || t === 'bell') {{
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(880, ctx.currentTime);
                    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.35);
                    osc.start();
                    osc.stop(ctx.currentTime + 0.36);
                }} else if (t === 'marimba' || t === 'warm') {{
                    osc.type = 'triangle';
                    osc.frequency.setValueAtTime(440, ctx.currentTime);
                    osc.frequency.exponentialRampToValueAtTime(554.37, ctx.currentTime + 0.08);
                    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.25);
                    osc.start();
                    osc.stop(ctx.currentTime + 0.26);
                }} else if (t === 'digital') {{
                    osc.type = 'square';
                    osc.frequency.setValueAtTime(600, ctx.currentTime);
                    osc.frequency.setValueAtTime(900, ctx.currentTime + 0.06);
                    gain.gain.value = {vol} * 0.4;
                    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.18);
                    osc.start();
                    osc.stop(ctx.currentTime + 0.19);
                }} else if (t === 'alert' || t === 'warning') {{
                    osc.type = 'triangle';
                    osc.frequency.setValueAtTime(880, ctx.currentTime);
                    osc.frequency.setValueAtTime(440, ctx.currentTime + 0.12);
                    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.3);
                    osc.start();
                    osc.stop(ctx.currentTime + 0.31);
                }} else if (t === 'click' || t === 'tap') {{
                    osc.type = 'sine';
                    osc.frequency.setValueAtTime(1000, ctx.currentTime);
                    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.05);
                    osc.start();
                    osc.stop(ctx.currentTime + 0.06);
                }}
            }} catch(e) {{}}
        }})();
        "#
    );
    let _ = dioxus::prelude::document::eval(&js);
}

/// Applies appearance attributes to the DOM and persists to localStorage.
pub fn apply_appearance_to_dom(
    accent: &str,
    scale: &str,
    density: &str,
    radius: &str,
    font_fam: &str,
    oled: bool,
    animations: bool,
) {
    let high_contrast = crate::db::sync_meta_get("ui_high_contrast").map(|v| v == "true").unwrap_or(false);
    let glassmorphism = crate::db::sync_meta_get("ui_glassmorphism").map(|v| v != "false").unwrap_or(true);
    let js = format!(
        r#"
        (function() {{
            const root = document.documentElement;
            // Accent
            if ('{accent}' === 'default') {{
                root.removeAttribute('data-accent');
                localStorage.removeItem('bpsl-accent');
            }} else {{
                root.setAttribute('data-accent', '{accent}');
                localStorage.setItem('bpsl-accent', '{accent}');
            }}
            // Font Scale
            if ('{scale}' === 'auto') {{
                root.removeAttribute('data-font-scale');
                localStorage.removeItem('bpsl-font-scale');
            }} else {{
                root.setAttribute('data-font-scale', '{scale}');
                localStorage.setItem('bpsl-font-scale', '{scale}');
            }}
            // Density
            if ('{density}' === 'default') {{
                root.removeAttribute('data-card-density');
                localStorage.removeItem('bpsl-card-density');
            }} else {{
                root.setAttribute('data-card-density', '{density}');
                localStorage.setItem('bpsl-card-density', '{density}');
            }}
            // Radius
            if ('{radius}' === 'rounded') {{
                root.removeAttribute('data-radius');
                localStorage.removeItem('bpsl-radius');
            }} else {{
                root.setAttribute('data-radius', '{radius}');
                localStorage.setItem('bpsl-radius', '{radius}');
            }}
            // Font Family
            if ('{font_fam}' === 'roboto') {{
                root.removeAttribute('data-font-family');
                localStorage.removeItem('bpsl-font-family');
            }} else {{
                root.setAttribute('data-font-family', '{font_fam}');
                localStorage.setItem('bpsl-font-family', '{font_fam}');
            }}
            // OLED Pure Black
            if ({oled}) {{
                root.setAttribute('data-oled', 'true');
                localStorage.setItem('bpsl-oled', 'true');
            }} else {{
                root.removeAttribute('data-oled');
                localStorage.removeItem('bpsl-oled');
            }}
            // Animations
            if (!{animations}) {{
                root.setAttribute('data-animations', 'false');
                localStorage.setItem('bpsl-animations', 'false');
            }} else {{
                root.removeAttribute('data-animations');
                localStorage.removeItem('bpsl-animations');
            }}
            // High Contrast Borders
            if ({high_contrast}) {{
                root.setAttribute('data-high-contrast', 'true');
                localStorage.setItem('bpsl-high-contrast', 'true');
            }} else {{
                root.removeAttribute('data-high-contrast');
                localStorage.removeItem('bpsl-high-contrast');
            }}
            // Glassmorphism
            if (!{glassmorphism}) {{
                root.setAttribute('data-glassmorphism', 'false');
                localStorage.setItem('bpsl-glassmorphism', 'false');
            }} else {{
                root.removeAttribute('data-glassmorphism');
                localStorage.removeItem('bpsl-glassmorphism');
            }}
        }})();
        "#
    );
    let _ = dioxus::prelude::document::eval(&js);
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

/// Helper to parse datetime parts from flexible string formats (ISO 8601, SQL, 12-hour AM/PM,
/// Indian DD/MM/YYYY, etc.) and ensure local civil parts in Asia/Kolkata (UTC+05:30).
///
/// Rules:
/// 1. Timestamps without explicit timezone indicator (e.g. "2026-04-15 03:56 PM", "2026-10-31 16:32:00")
///    are local civil wall-clock time from Indian Google Sheets / users. They are retained as-is.
/// 2. Timestamps with explicit 'Z' or UTC offset (e.g. "2026-11-05T07:00:00.000Z") are converted from
///    UTC to local IST (+05:30).
/// 3. Timestamps with AM/PM (e.g. "10:21 AM", "03:56 PM", "12:14 PM") are properly parsed to 24-hour hours.
pub fn parse_datetime_flexible(s: &str) -> Option<(u32, u32, u32, u32, u32)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Split into date and time tokens
    let (date_str, time_str) = if let Some(t_idx) = s.find(['T', 't']) {
        (&s[..t_idx], s[t_idx + 1..].trim())
    } else {
        let mut tokens = s.split_whitespace();
        let first = tokens.next()?;
        let rest: Vec<&str> = tokens.collect();
        (first, if rest.is_empty() { "" } else { s[first.len()..].trim() })
    };

    // 1. Parse Date: supports YYYY-MM-DD, YYYY/MM/DD, DD-MM-YYYY, DD/MM/YYYY, MM/DD/YYYY
    let (y, m, d) = if date_str.contains('-') {
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() == 3 {
            if parts[0].len() == 4 {
                (parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?)
            } else if parts[2].len() == 4 {
                (parts[2].parse().ok()?, parts[1].parse().ok()?, parts[0].parse().ok()?)
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else if date_str.contains('/') {
        let parts: Vec<&str> = date_str.split('/').collect();
        if parts.len() == 3 {
            if parts[0].len() == 4 {
                (parts[0].parse().ok()?, parts[1].parse().ok()?, parts[2].parse().ok()?)
            } else if parts[2].len() == 4 {
                let p0: u32 = parts[0].parse().ok()?;
                let p1: u32 = parts[1].parse().ok()?;
                let yr: u32 = parts[2].parse().ok()?;
                if p0 > 12 && p1 <= 12 {
                    (yr, p1, p0)
                } else if p1 > 12 && p0 <= 12 {
                    (yr, p0, p1)
                } else {
                    // Default to DD/MM/YYYY
                    (yr, p1, p0)
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    } else {
        return None;
    };

    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }

    if time_str.is_empty() {
        return Some((y, m, d, 0, 0));
    }

    // 2. Parse Time
    let is_pm = time_str.contains("PM") || time_str.contains("pm");
    let is_am = time_str.contains("AM") || time_str.contains("am");

    // Remove AM/PM
    let clean_time = time_str
        .replace("PM", "")
        .replace("pm", "")
        .replace("AM", "")
        .replace("am", "");
    let mut clean_time = clean_time.trim();

    // Check timezone indicator
    let is_utc_z = clean_time.ends_with('Z') || clean_time.ends_with('z');
    if is_utc_z {
        clean_time = clean_time.trim_end_matches(['Z', 'z']).trim();
    }

    let (clean_time_core, offset_opt) = if is_utc_z {
        (clean_time, Some(0))
    } else if let Some(plus_idx) = clean_time.find('+') {
        let offset = parse_tz_offset(&clean_time[plus_idx..], 1);
        (clean_time[..plus_idx].trim(), offset)
    } else if let Some(minus_idx) = clean_time.find('-') {
        let offset = parse_tz_offset(&clean_time[minus_idx..], -1);
        (clean_time[..minus_idx].trim(), offset)
    } else {
        (clean_time, None)
    };

    let time_parts: Vec<&str> = clean_time_core.split(':').collect();
    if time_parts.is_empty() {
        return Some((y, m, d, 0, 0));
    }

    let mut h: u32 = time_parts[0].trim().parse().ok()?;
    let mi: u32 = if time_parts.len() >= 2 {
        time_parts[1].trim().parse().ok()?
    } else {
        0
    };
    let sec: u32 = if time_parts.len() >= 3 {
        time_parts[2]
            .trim()
            .split('.')
            .next()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0)
    } else {
        0
    };

    // Apply AM/PM adjustment
    if is_pm && h < 12 {
        h += 12;
    } else if is_am && h == 12 {
        h = 0;
    }

    match offset_opt {
        None => {
            // No explicit timezone -> local civil wall clock time in India (IST)
            Some((y, m, d, h, mi))
        }
        Some(offset_secs) if offset_secs == LOCAL_TZ_OFFSET_SECS => {
            // Already in local timezone (+05:30)
            Some((y, m, d, h, mi))
        }
        Some(offset_secs) => {
            // Explicit timezone provided (e.g. UTC 'Z' or other offset).
            // Convert to UTC epoch, then add local IST offset (+19,800s).
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
            Some((ly, lm, ld, lh, lmi))
        }
    }
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

/// Primary unified helper to arrange any date and time into a clean, human-readable display string
/// according to the user's 12-hour or 24-hour setting.
///
/// Handles:
/// - ISO 8601 strings (e.g. "2026-11-05T07:00:00.000Z" -> "Nov 05, 2026 · 12:30 PM")
/// - 12-hour AM/PM timestamps (e.g. "2026-04-15 03:56 PM" -> "Apr 15, 2026 · 03:56 PM")
/// - SQL timestamps (e.g. "2026-11-04 18:24:26" -> "Nov 04, 2026 · 06:24 PM")
/// - Indian date formats (e.g. "31/10/2026 16:32:00" -> "Oct 31, 2026 · 04:32 PM")
/// - Date-only strings (e.g. "2026-11-06" -> "Nov 06, 2026")
/// - Relative labels (e.g. "Today · 2:00 PM", "Tomorrow", "Just now" -> preserved)
/// - Empty / None strings -> returns empty string
pub fn arrange_date_time(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    // If it's already a relative word, return as-is
    if s.starts_with("Today")
        || s.starts_with("Tomorrow")
        || s.starts_with("Next Week")
        || s == "Not sent yet"
        || s == "Not sent"
        || s == "Just now"
    {
        return s.to_string();
    }

    if let Some((y, m, d, h, mi)) = parse_datetime_flexible(s) {
        arrange_date_time_parts(
            y,
            m,
            d,
            h,
            mi,
            s.len() >= 16 || s.contains('T') || s.contains(':'),
        )
    } else {
        s.to_string()
    }
}

/// Arranges civil date and time parts into a formatted string respecting 12-hour / 24-hour mode and date style.
pub fn arrange_date_time_parts(
    year: u32,
    month: u32,
    day: u32,
    hour24: u32,
    minute: u32,
    include_time: bool,
) -> String {
    let month_idx = (month.saturating_sub(1) as usize).min(11);
    let month_name = SHORT_MONTH_NAMES[month_idx];
    let date_str = match DATE_DISPLAY_STYLE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => format!("{:02}/{:02}/{:04}", day, month, year),
        2 => format!("{:04}-{:02}-{:02}", year, month, day),
        3 => format!("{:02}-{}-{:04}", day, month_name, year),
        _ => format!("{} {:02}, {:04}", month_name, day, year),
    };
    if include_time {
        if is_time_format_24hr() {
            format!("{} · {:02}:{:02}", date_str, hour24, minute)
        } else {
            let am_pm = if hour24 >= 12 { "PM" } else { "AM" };
            let h12 = match hour24 % 12 {
                0 => 12,
                other => other,
            };
            format!("{} · {:02}:{:02} {}", date_str, h12, minute, am_pm)
        }
    } else {
        date_str
    }
}

/// Arranges only the date part of a timestamp string (e.g. "Nov 12, 2026").
#[allow(dead_code)]
pub fn arrange_date_only(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some((y, m, d, _, _)) = parse_datetime_flexible(s) {
        let month_idx = (m.saturating_sub(1) as usize).min(11);
        format!("{} {:02}, {:04}", SHORT_MONTH_NAMES[month_idx], d, y)
    } else {
        s.to_string()
    }
}

/// Arranges only the time part of a timestamp string (e.g. "10:21 AM" or "10:21" in 24hr mode).
pub fn arrange_time_only(raw: &str) -> String {
    let s = raw.trim();
    if s.is_empty() {
        return String::new();
    }
    if let Some((_, _, _, h, mi)) = parse_datetime_flexible(s) {
        if is_time_format_24hr() {
            format!("{h:02}:{mi:02}")
        } else {
            let am_pm = if h >= 12 { "PM" } else { "AM" };
            let h12 = match h % 12 {
                0 => 12,
                other => other,
            };
            format!("{h12:02}:{mi:02} {am_pm}")
        }
    } else if let Some(pos) = s.find([' ', 'T']) {
        let rest = s[pos + 1..].trim();
        let clean = rest.trim_end_matches(['Z', 'z']);
        clean.to_string()
    } else {
        String::new()
    }
}

/// Formats any ISO 8601 or SQL timestamp for display, obeying the user's
/// 12-hour or 24-hour preference configured in Settings.
#[allow(dead_code)]
pub fn format_display_12hr(raw: &str) -> String {
    arrange_date_time(raw)
}

/// Formats any ISO 8601 or SQL timestamp for the UI according to current settings.
#[allow(dead_code)]
pub fn format_display_time(raw: &str) -> String {
    arrange_date_time(raw)
}
