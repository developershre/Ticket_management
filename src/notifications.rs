//! Native Windows desktop notifications for BPSL IT Support.
//!
//! Provides OS-level toast/balloon notifications in the Windows Action Center
//! for incoming tickets, status updates, reminder due dates, and SLA deadlines.
//! Runs completely in the background without blocking the UI thread.

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Icon type for native notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationIcon {
    Info,
    Warning,
    Error,
}

impl NotificationIcon {
    pub fn as_str(&self) -> &'static str {
        match self {
            NotificationIcon::Info => "Info",
            NotificationIcon::Warning => "Warning",
            NotificationIcon::Error => "Error",
        }
    }
}

// In-memory deduplication cache: keeps track of recent notification keys
// so identical notifications are never spammed repeatedly.
static RECENT_NOTIFICATIONS: Mutex<Option<(Instant, HashSet<String>)>> = Mutex::new(None);

fn should_notify(key: &str) -> bool {
    let mut guard = match RECENT_NOTIFICATIONS.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    let now = Instant::now();
    let (last_reset, set) = guard.get_or_insert_with(|| (now, HashSet::new()));
    // Prune cache every 2 hours to prevent unbounded memory growth
    if now.duration_since(*last_reset) > Duration::from_secs(7200) {
        set.clear();
        *last_reset = now;
    }
    set.insert(key.to_string())
}

/// Checks whether Windows desktop notifications are enabled in user settings.
pub fn is_windows_notifications_enabled() -> bool {
    crate::db::sync_meta_get("ui_windows_notifications")
        .map(|v| v != "false")
        .unwrap_or(true)
}

/// Simple RFC 4648 standard base64 encoder to avoid external crate dependencies.
fn to_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = if chunk.len() > 1 { chunk[1] } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] } else { 0 };

        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 3) << 4) | (b1 >> 4)) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[(((b1 & 15) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(b2 & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Dispatches an OS notification to Windows. Runs asynchronously on a separate thread.
pub fn send_windows_notification(title: &str, body: &str, icon: NotificationIcon) {
    if !is_windows_notifications_enabled() {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        let t_b64 = to_base64(title.as_bytes());
        let b_b64 = to_base64(body.as_bytes());
        let icon_str = icon.as_str();

        std::thread::spawn(move || {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;

            let script = format!(
                r#"
                Add-Type -AssemblyName System.Windows.Forms
                $t = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String('{t_b64}'))
                $b = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String('{b_b64}'))
                $n = New-Object System.Windows.Forms.NotifyIcon
                $n.Icon = [System.Drawing.SystemIcons]::Information
                $n.BalloonTipTitle = $t
                $n.BalloonTipText = $b
                $n.BalloonTipIcon = [System.Windows.Forms.ToolTipIcon]::{icon_str}
                $n.Visible = $True
                $n.ShowBalloonTip(5000)
                Start-Sleep -Milliseconds 450
                $n.Dispose()
                "#
            );

            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", &script])
                .creation_flags(CREATE_NO_WINDOW)
                .output();
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        println!("[desktop-notification] {title}: {body}");
    }
}

/// Helper: Notify when a single new ticket is detected from remote sync or created.
pub fn notify_new_ticket(ticket_id: &str, subject: &str, employee: &str, priority: &str) {
    let key = format!("ticket_new_{ticket_id}");
    if !should_notify(&key) {
        return;
    }
    let title = format!("New Ticket #{ticket_id}");
    let body = if !employee.trim().is_empty() {
        format!("{subject}\nFrom: {employee} · Priority: {priority}")
    } else {
        format!("{subject}\nPriority: {priority}")
    };
    send_windows_notification(&title, &body, NotificationIcon::Info);
}

/// Helper: Notify when multiple new tickets are fetched in a single sync batch.
pub fn notify_multiple_new_tickets(count: usize, latest_ticket_id: &str, latest_subject: &str) {
    let key = format!("tickets_batch_{count}_{latest_ticket_id}");
    if !should_notify(&key) {
        return;
    }
    let title = format!("{count} New Tickets Arrived");
    let body = format!("Latest #{latest_ticket_id}: {latest_subject}");
    send_windows_notification(&title, &body, NotificationIcon::Info);
}

/// Helper: Notify when a ticket's status has been modified on Google Sheets.
pub fn notify_ticket_status_change(ticket_id: &str, subject: &str, new_status: &str) {
    let key = format!("ticket_status_{ticket_id}_{new_status}");
    if !should_notify(&key) {
        return;
    }
    let title = format!("Ticket #{ticket_id} Updated");
    let body = format!("{subject}\nStatus changed to: {new_status}");
    send_windows_notification(&title, &body, NotificationIcon::Info);
}

/// Helper: Notify when a reminder is due.
pub fn notify_reminder_due(reminder_id: &str, message: &str, employee: &str, send_at: &str) {
    let key = format!("reminder_due_{reminder_id}");
    if !should_notify(&key) {
        return;
    }
    let title = "BPSL IT Support · Reminder Due";
    let body = if !employee.trim().is_empty() {
        format!("{message}\nFor: {employee} · Due: {send_at}")
    } else {
        format!("{message}\nDue: {send_at}")
    };
    send_windows_notification(title, &body, NotificationIcon::Warning);
}

/// Helper: Notify when a new reminder is added via remote sheet sync.
pub fn notify_new_reminder(reminder_id: &str, message: &str, employee: &str) {
    let key = format!("reminder_new_{reminder_id}");
    if !should_notify(&key) {
        return;
    }
    let title = "New Reminder Scheduled";
    let body = if !employee.trim().is_empty() {
        format!("{message}\nAssigned to: {employee}")
    } else {
        message.to_string()
    };
    send_windows_notification(title, &body, NotificationIcon::Info);
}

/// Helper: Notify on approaching SLA deadlines for open tickets.
pub fn notify_sla_warning(ticket_id: &str, subject: &str, deadline: &str) {
    let key = format!("sla_warn_{ticket_id}");
    if !should_notify(&key) {
        return;
    }
    let title = format!("SLA Deadline Alert · #{ticket_id}");
    let body = format!("{subject}\nApproaching deadline: {deadline}");
    send_windows_notification(&title, &body, NotificationIcon::Warning);
}

/// Helper: Sends a test notification to verify Windows Notification Center integration.
pub fn test_notification() {
    send_windows_notification(
        "BPSL IT Support · Notifications Active",
        "Windows desktop alerts are operational. You will receive notifications for new tickets, due reminders, and status changes.",
        NotificationIcon::Info,
    );
}

/// Helper to parse flexible datetime strings into epoch seconds for time comparisons.
pub fn parse_timestamp_to_epoch(raw: &str) -> Option<i64> {
    if let Some((y, m, d, h, mi)) = crate::state::parse_datetime_flexible(raw) {
        let civil_epoch = crate::state::epoch_from_civil(y, m, d)
            + (h as i64 * 3600)
            + (mi as i64 * 60);
        // local IST is +05:30 (19800s ahead of UTC)
        Some(civil_epoch - crate::state::LOCAL_TZ_OFFSET_SECS)
    } else {
        None
    }
}
