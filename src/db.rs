//! SQLite database management layer for BPSL IT Support.
//! Reads and writes directly to `management.db` adhering to `DATABASE.SQL`.

use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

static DB_PATH_CACHE: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// Get the path to `management.db`
pub fn db_path() -> PathBuf {
    // Test/CI isolation: point the whole app (UI + sync worker + test
    // harness) at a different database file. Checked first so
    // `--sync-test` scenarios never touch real user data.
    if let Some(p) = std::env::var_os("TMS_DB_PATH") {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    DB_PATH_CACHE.get_or_init(resolve_db_path).clone()
}

fn is_dir_writable(dir: &Path) -> bool {
    let test_file = dir.join(format!(".write_test_{}", std::process::id()));
    if std::fs::write(&test_file, b"").is_ok() {
        let _ = std::fs::remove_file(test_file);
        true
    } else {
        false
    }
}

fn resolve_db_path() -> PathBuf {
    // If installed in Program Files or a read-only system folder, always store user
    // data in %APPDATA%\BPSL IT Support\ to avoid permission errors on write/WAL.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let parent_str = parent.to_string_lossy().to_lowercase();
            let is_system_install = parent_str.contains("program files") || !is_dir_writable(parent);
            if is_system_install {
                if let Some(app_data) = std::env::var_os("APPDATA").or_else(|| std::env::var_os("LOCALAPPDATA")) {
                    let app_dir = PathBuf::from(app_data).join("BPSL IT Support");
                    let _ = std::fs::create_dir_all(&app_dir);
                    let target_db = app_dir.join("management.db");
                    if !target_db.exists() {
                        let seed_candidates = [
                            parent.join("management.db"),
                            parent.join("empty.db"),
                            parent.join("resources").join("management.db"),
                            parent.join("resources").join("empty.db"),
                            parent.join("../assets/database.db"),
                        ];
                        for seed in &seed_candidates {
                            if seed.exists() {
                                let _ = std::fs::copy(seed, &target_db);
                                break;
                            }
                        }
                    }
                    // In a system install, ALWAYS return target_db in AppData so SQLite can write
                    return target_db;
                }
            }
        }
    }

    // Portable / development mode: look right next to the executable first
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let next_to_exe = parent.join("management.db");
            if next_to_exe.exists() {
                return next_to_exe;
            }
            // If empty.db was bundled next to exe, copy it to management.db
            let empty_seed = parent.join("empty.db");
            if empty_seed.exists() && is_dir_writable(parent) {
                let _ = std::fs::copy(&empty_seed, &next_to_exe);
                if next_to_exe.exists() {
                    return next_to_exe;
                }
            }
            if let Some(up) = parent.parent() {
                let candidate2 = up.join("management.db");
                if candidate2.exists() {
                    return candidate2;
                }
            }
            // Working directory fallback if it already exists
            let local = Path::new("management.db");
            if local.exists() {
                return local.to_path_buf();
            }
            // If parent folder is writable, keep management.db directly alongside the executable
            if is_dir_writable(parent) {
                return next_to_exe;
            }
        }
    }

    // Working directory fallback
    let local = Path::new("management.db");
    if local.exists() {
        return local.to_path_buf();
    }
    PathBuf::from("management.db")
}

/// Directory for storing temporary ticket attachments before/during Apps Script sync.
pub fn temp_attachments_dir() -> PathBuf {
    let base = db_path()
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = base.join("temp_attachments");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Saves uploaded attachment bytes to the temporary attachments folder with a
/// collision-free timestamp prefix and returns the full local file path.
pub fn save_temp_attachment(file_name: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    let raw_name = Path::new(file_name)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment.bin");
    let clean_name: String = raw_name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let clean_name = clean_name.trim();
    let final_name = if clean_name.is_empty() {
        "attachment.bin"
    } else {
        clean_name
    };
    let ts_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let target = temp_attachments_dir().join(format!("{ts_ms}_{final_name}"));
    std::fs::write(&target, bytes)?;
    Ok(target)
}

/// Returns true if `path_str` points to an existing file inside `temp_attachments_dir()`.
pub fn is_temp_attachment_path(path_str: &str) -> bool {
    let trimmed = path_str.trim();
    if trimmed.is_empty() {
        return false;
    }
    let path = Path::new(trimmed);
    if !path.is_file() {
        return false;
    }
    let dir = temp_attachments_dir();
    if let (Ok(canon_file), Ok(canon_dir)) = (path.canonicalize(), dir.canonicalize()) {
        canon_file.starts_with(canon_dir)
    } else {
        path.starts_with(&dir)
    }
}

/// Deletes a temporary attachment file only if it lives inside `temp_attachments_dir()`.
pub fn remove_temp_attachment_if_managed(path_str: &str) {
    if is_temp_attachment_path(path_str) {
        let _ = std::fs::remove_file(path_str.trim());
    }
}

/// Guesses a MIME content type from a filename or path extension.
pub fn guess_mime_type(file_name: &str) -> &'static str {
    let lower = file_name.trim().to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else if lower.ends_with(".svg") {
        "image/svg+xml"
    } else if lower.ends_with(".bmp") {
        "image/bmp"
    } else if lower.ends_with(".pdf") {
        "application/pdf"
    } else if lower.ends_with(".txt") || lower.ends_with(".log") {
        "text/plain"
    } else if lower.ends_with(".csv") {
        "text/csv"
    } else if lower.ends_with(".json") {
        "application/json"
    } else {
        "application/octet-stream"
    }
}

/// Standard RFC 4648 base64 encoder (no extra crate dependency required).
pub fn encode_base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((triple >> 18) & 0x3F) as usize] as char);
        out.push(TABLE[((triple >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[((triple >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(triple & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

/// Updates only `attachment_url` for a local ticket (e.g. after uploading a temp file).
pub fn update_ticket_attachment_url(local_ticket_id: i64, new_url: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET attachment_url = ?1 WHERE ticket_id = ?2",
        params![new_url, local_ticket_id],
    )?;
    Ok(())
}

static WAL_INITIALIZED_PATH: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Open a connection with WAL mode enabled for smooth concurrency.
/// A busy timeout is set because the background sync worker and the UI
/// thread both write to the database.
pub fn get_connection() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    let needs_wal = WAL_INITIALIZED_PATH
        .lock()
        .map(|g| g.as_ref() != Some(&path))
        .unwrap_or(true);
    if needs_wal {
        conn.execute_batch(
            "PRAGMA journal_mode = WAL; \
             PRAGMA synchronous = NORMAL; \
             PRAGMA foreign_keys = ON; \
             PRAGMA temp_store = MEMORY; \
             PRAGMA cache_size = -8000;",
        )?;
        if let Ok(mut g) = WAL_INITIALIZED_PATH.lock() {
            *g = Some(path);
        }
    } else {
        conn.execute_batch(
            "PRAGMA synchronous = NORMAL; \
             PRAGMA foreign_keys = ON; \
             PRAGMA temp_store = MEMORY; \
             PRAGMA cache_size = -8000;",
        )?;
    }
    Ok(conn)
}

// ============================================================
// Data Models matching DATABASE.SQL
// ============================================================

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DbEmployee {
    pub emp_id: i64,
    /// Sheet employee code (numeric string, e.g. `"10081"`); may be empty
    /// when the sheet row has no code, and is not unique.
    pub emp_code: String,
    pub emp_name: String,
    /// Often empty — 293 of 324 sheet rows have no email.
    pub emp_email: String,
    pub department: String,
    pub division: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DbTicket {
    pub ticket_id: i64,
    pub employee_code: String,
    pub email_address: String,
    pub category: String,
    pub subject: String,
    pub problem_description: String,
    pub attachment_name: String,
    pub attachment_url: String,
    pub priority: String,
    pub assigned_to: String,
    pub customer: String,
    pub time_spent: f64,
    pub activities: String,
    pub my_activity: String,
    pub deadline: String,
    pub sla_status: String,
    pub sla_deadline: String,
    pub rating_avg_text: String,
    pub kanban_state: String, // "Pending" | "InProgress" | "Onhold" | "Completed"
    pub stage: String,
    pub status: String,
    pub created_on: String,
    pub last_updated_on: String,
    pub server_ticket_id: Option<String>,
    pub server_last_updated: Option<String>,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DbNote {
    pub id: i64,
    pub emp_code: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DbReminder {
    pub reminder_id: i64,
    pub employee_code: String,
    pub employee_name: String,
    pub email_address: String,
    pub service_category: String,
    pub reminder_message: String,
    pub send_at: String,
    pub status: String,
    pub email_sent_at: String,
    pub ticket_id: String,
    pub created_at: String,
    pub last_updated_at: String,
    /// Google Sheet `Reminder ID` of the row this local row maps to.
    /// Local-only bookkeeping — never part of an outbound API payload.
    pub server_reminder_id: String,
    /// Sheet `Last Updated At` of the mapped row (pull comparison).
    /// Local-only bookkeeping — never part of an outbound API payload.
    pub server_last_updated: String,
    pub cc_emails: String,
    pub repeat_frequency: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NewReminderInput {
    pub employee_code: String,
    pub employee_name: String,
    pub email_address: String,
    pub service_category: String,
    pub reminder_message: String,
    pub send_at: String,
    pub status: String,
    pub email_sent_at: String,
    pub ticket_id: String,
    pub cc_emails: String,
    pub repeat_frequency: String,
}

impl Default for NewReminderInput {
    fn default() -> Self {
        Self {
            // No baked-in employee: the Google Sheet is the directory of
            // record. Callers fill these from the employee picker / cache.
            employee_code: String::new(),
            employee_name: String::new(),
            email_address: String::new(),
            service_category: "General".to_string(),
            reminder_message: String::new(),
            send_at: "Today · 2:00 PM".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: String::new(),
            cc_emails: String::new(),
            repeat_frequency: "daily".to_string(),
        }
    }
}

// ============================================================
// Database Initialization & Seeding
// ============================================================

/// Ensures tables from `DATABASE.SQL` exist. No demo data is seeded:
/// tickets come from the Google Sheet (`tickets.list`) and employees from
/// `employees.list` via the background sync worker.
pub fn init_database() -> Result<()> {
    let conn = get_connection()?;

    // 1. Create tables if not exists (from DATABASE.SQL)
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS USERS(
            emp_id INTEGER PRIMARY KEY AUTOINCREMENT,
            emp_code TEXT,
            emp_name TEXT NOT NULL,
            emp_email TEXT,
            department TEXT,
            division TEXT,
            active BOOLEAN DEFAULT 1,
            is_deleted BOOLEAN DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS tickets (
            ticket_id INTEGER PRIMARY KEY AUTOINCREMENT,

            employee_code TEXT,
            email_address TEXT,

            category TEXT,
            subject TEXT NOT NULL,
            problem_description TEXT,

            attachment_name TEXT,
            attachment_url TEXT,

            priority TEXT DEFAULT 'Medium',
            assigned_to TEXT,
            customer TEXT,

            time_spent REAL DEFAULT 0,

            activities TEXT,
            my_activity TEXT,

            deadline DATETIME,
            sla_status TEXT,
            sla_deadline DATETIME,

            rating_avg_text TEXT,

            kanban_state TEXT,
            stage TEXT,
            status TEXT DEFAULT 'Open',

            created_on DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_updated_on DATETIME DEFAULT CURRENT_TIMESTAMP,

            server_ticket_id TEXT,
            server_last_updated TEXT
        );

        CREATE TABLE IF NOT EXISTS NOTES(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            emp_code TEXT,
            content TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS reminders (
            reminder_id INTEGER PRIMARY KEY AUTOINCREMENT,
            employee_code TEXT,
            employee_name TEXT,
            email_address TEXT,
            service_category TEXT,
            reminder_message TEXT NOT NULL,
            send_at TEXT,
            status TEXT DEFAULT 'Pending',
            email_sent_at TEXT,
            ticket_id TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            last_updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            cc_emails TEXT,
            repeat_frequency TEXT DEFAULT 'daily',
            server_reminder_id TEXT,
            server_last_updated TEXT
        );
        "#,
    )?;

    // 1b. Google Sheets ↔ SQLite sync layer. The mapping columns are added
    //     once (SQLite ALTER TABLE has no IF NOT EXISTS, so they are checked
    //     first); the queue/meta/conflict tables are idempotent.
    ensure_ticket_sync_columns(&conn)?;
    ensure_reminder_sync_columns(&conn)?;
    ensure_note_sync_columns(&conn)?;
    ensure_sync_queue_columns(&conn)?;
    // 1c. Employee cache schema (nullable/duplicate emails + division —
    //     the shape the sheet's employees.list actually returns).
    ensure_user_schema(&conn)?;
    sanitize_user_cache(&conn)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sync_queue (
            op_id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type TEXT NOT NULL DEFAULT 'ticket',
            local_ticket_id INTEGER NOT NULL,
            server_ticket_id TEXT,
            op_type TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL,
            retry_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'pending',
            last_error TEXT,
            next_attempt_at INTEGER
        );
        CREATE INDEX IF NOT EXISTS idx_sync_queue_due
            ON sync_queue(status, next_attempt_at);

        CREATE TABLE IF NOT EXISTS sync_meta (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS sync_conflicts (
            conflict_id INTEGER PRIMARY KEY AUTOINCREMENT,
            local_ticket_id INTEGER NOT NULL,
            server_ticket_id TEXT,
            op_kind TEXT NOT NULL,
            local_payload TEXT,
            server_payload TEXT,
            detected_at TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'unresolved',
            resolved_at DATETIME
        );
        CREATE INDEX IF NOT EXISTS idx_sync_conflicts
            ON sync_conflicts(status, local_ticket_id);
        "#,
    )?;

    // Crash recovery: operations interrupted mid-flight (application
    // restart during synchronization) go back to pending — an op is never
    // marked synced without server confirmation.
    conn.execute(
        "UPDATE sync_queue SET status = 'pending', last_error = 'interrupted by restart', \
         next_attempt_at = NULL WHERE status = 'in_flight'",
        [],
    )?;

    // Seed Google Sheet configuration in sync_meta if not already present.
    // If an existing .env is present, migrate its settings into the database.
    // If not, default to the built-in REST endpoint. This allows the application
    // to run 100% standalone from the single SQLite database.
    let existing_url: Option<String> = conn
        .query_row(
            "SELECT value FROM sync_meta WHERE key = 'system_api_base_url'",
            [],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();

    if existing_url.is_none() || existing_url.as_deref() == Some("") {
        let (dot_url, dot_key, dot_emp, dot_proto) = crate::api::ApiConfig::read_dotenv_values();
        let base_url = dot_url.unwrap_or_else(|| DEFAULT_REST_SHEET_URL.to_string());
        let _ = conn.execute(
            "INSERT INTO sync_meta (key, value) VALUES ('system_api_base_url', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![base_url],
        );
        if let Some(key) = dot_key {
            if !key.trim().is_empty() {
                let _ = conn.execute(
                    "INSERT INTO sync_meta (key, value) VALUES ('system_api_key', ?1) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![key],
                );
            }
        }
        if let Some(emp) = dot_emp {
            if !emp.trim().is_empty() {
                let _ = conn.execute(
                    "INSERT INTO sync_meta (key, value) VALUES ('system_employee_code', ?1) \
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![emp],
                );
            }
        }
        let proto = dot_proto.unwrap_or_else(|| "rest".to_string());
        let _ = conn.execute(
            "INSERT INTO sync_meta (key, value) VALUES ('system_api_protocol', ?1) \
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![proto],
        );
        let _ = conn.execute(
            "INSERT INTO sync_meta (key, value) VALUES ('active_sheet_source_id', 'rest') \
             ON CONFLICT(key) DO NOTHING",
            [],
        );
    }

    drop(conn);
    let _ = deduplicate_reminders_db();
    Ok(())
}

// ============================================================
// Tickets Queries & Operations
// ============================================================

/// Fetches all tickets from `tickets` table ordered by ticket_id.
pub fn get_all_tickets() -> Vec<DbTicket> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        r#"
        SELECT
            ticket_id,
            COALESCE(employee_code, ''),
            COALESCE(email_address, ''),
            COALESCE(category, 'General'),
            subject,
            COALESCE(problem_description, ''),
            COALESCE(attachment_name, ''),
            COALESCE(attachment_url, ''),
            COALESCE(priority, 'Medium'),
            COALESCE(assigned_to, 'IT'),
            COALESCE(customer, ''),
            COALESCE(time_spent, 0.0),
            COALESCE(activities, ''),
            COALESCE(my_activity, ''),
            COALESCE(deadline, ''),
            COALESCE(sla_status, 'Within SLA'),
            COALESCE(sla_deadline, ''),
            COALESCE(rating_avg_text, ''),
            COALESCE(kanban_state, 'Pending'),
            COALESCE(stage, 'Triage'),
            COALESCE(status, 'Open'),
            COALESCE(created_on, CURRENT_TIMESTAMP),
            COALESCE(last_updated_on, CURRENT_TIMESTAMP),
            server_ticket_id,
            server_last_updated
        FROM tickets
        ORDER BY ticket_id DESC
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok(DbTicket {
            ticket_id: row.get(0)?,
            employee_code: row.get(1)?,
            email_address: row.get(2)?,
            category: row.get(3)?,
            subject: row.get(4)?,
            problem_description: row.get(5)?,
            attachment_name: row.get(6)?,
            attachment_url: row.get(7)?,
            priority: row.get(8)?,
            assigned_to: row.get(9)?,
            customer: row.get(10)?,
            time_spent: row.get(11)?,
            activities: row.get(12)?,
            my_activity: row.get(13)?,
            deadline: row.get(14)?,
            sla_status: row.get(15)?,
            sla_deadline: row.get(16)?,
            rating_avg_text: row.get(17)?,
            kanban_state: row.get(18)?,
            stage: row.get(19)?,
            status: row.get(20)?,
            created_on: row.get(21)?,
            last_updated_on: row.get(22)?,
            server_ticket_id: row.get(23)?,
            server_last_updated: row.get(24)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Updates the kanban_state, status, and last_updated_on of a ticket in `tickets` table without overwriting stage.
pub fn update_ticket_status(ticket_id: i64, new_kanban_state: &str) -> Result<()> {
    let conn = get_connection()?;
    let new_status = if new_kanban_state.eq_ignore_ascii_case("Completed")
        || new_kanban_state.eq_ignore_ascii_case("Solved(closed)")
        || new_kanban_state.eq_ignore_ascii_case("Solved (closed)")
        || new_kanban_state.eq_ignore_ascii_case("Closed")
        || new_kanban_state.eq_ignore_ascii_case("Cancelled")
        || new_kanban_state.eq_ignore_ascii_case("Canceled")
    {
        "Closed"
    } else {
        "Open"
    };

    conn.execute(
        r#"
        UPDATE tickets
        SET kanban_state = ?1, status = ?2, last_updated_on = datetime('now', '+330 minutes')
        WHERE ticket_id = ?3
        "#,
        params![new_kanban_state, new_status, ticket_id],
    )?;
    Ok(())
}

/// Updates kanban_state, optional stage, and optional status of a ticket in `tickets` table.
pub fn update_ticket_kanban_and_stage(
    ticket_id: i64,
    new_kanban_state: &str,
    new_stage: Option<&str>,
    new_status: Option<&str>,
) -> Result<()> {
    let conn = get_connection()?;
    let status_str = match new_status {
        Some(s) => s.to_string(),
        None => {
            if new_kanban_state.eq_ignore_ascii_case("Completed")
                || new_kanban_state.eq_ignore_ascii_case("Solved(closed)")
                || new_kanban_state.eq_ignore_ascii_case("Solved (closed)")
                || new_kanban_state.eq_ignore_ascii_case("Closed")
                || new_kanban_state.eq_ignore_ascii_case("Cancelled")
                || new_kanban_state.eq_ignore_ascii_case("Canceled")
            {
                "Closed".to_string()
            } else {
                "Open".to_string()
            }
        }
    };

    if let Some(stg) = new_stage {
        conn.execute(
            r#"
            UPDATE tickets
            SET kanban_state = ?1, stage = ?2, status = ?3, last_updated_on = datetime('now', '+330 minutes')
            WHERE ticket_id = ?4
            "#,
            params![new_kanban_state, stg, status_str, ticket_id],
        )?;
    } else {
        conn.execute(
            r#"
            UPDATE tickets
            SET kanban_state = ?1, status = ?2, last_updated_on = datetime('now', '+330 minutes')
            WHERE ticket_id = ?3
            "#,
            params![new_kanban_state, status_str, ticket_id],
        )?;
    }
    Ok(())
}

/// Input parameters for creating a new ticket with all fields matching DATABASE.SQL.
#[derive(Clone, Debug, PartialEq)]
pub struct NewTicketInput {
    pub employee_code: String,
    pub email_address: String,
    pub category: String,
    pub subject: String,
    pub problem_description: String,
    pub attachment_name: String,
    pub attachment_url: String,
    pub priority: String,
    pub assigned_to: String,
    pub customer: String,
    pub time_spent: f64,
    pub activities: String,
    pub my_activity: String,
    pub deadline: String,
    pub sla_status: String,
    pub sla_deadline: String,
    pub rating_avg_text: String,
    pub kanban_state: String,
    pub stage: String,
    pub status: String,
}

impl Default for NewTicketInput {
    fn default() -> Self {
        Self {
            // No baked-in employee: the Google Sheet is the directory of
            // record. Callers fill `employee_code` (and the derived email /
            // customer) from the employee picker or the local sheet cache.
            employee_code: String::new(),
            email_address: String::new(),
            category: "General".to_string(),
            subject: String::new(),
            problem_description: String::new(),
            attachment_name: String::new(),
            attachment_url: String::new(),
            priority: "Medium".to_string(),
            assigned_to: "IT Support".to_string(),
            customer: String::new(),
            time_spent: 0.0,
            activities: "Ticket created".to_string(),
            my_activity: "Initial review".to_string(),
            deadline: String::new(),
            sla_status: "Within SLA".to_string(),
            sla_deadline: String::new(),
            rating_avg_text: "Unrated".to_string(),
            kanban_state: "Pending".to_string(),
            stage: "Triage".to_string(),
            status: "Open".to_string(),
        }
    }
}

/// Inserts a new ticket with all 20 non-generated fields into `tickets` table
/// and returns the generated `ticket_id`.
pub fn create_ticket_full(input: &NewTicketInput) -> Result<i64> {
    let conn = get_connection()?;
    let now_local = crate::state::format_local_now();
    conn.execute(
        r#"
        INSERT INTO tickets (
            employee_code, email_address, category, subject, problem_description,
            attachment_name, attachment_url, priority, assigned_to, customer,
            time_spent, activities, my_activity, deadline, sla_status, sla_deadline,
            rating_avg_text, kanban_state, stage, status, created_on, last_updated_on
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22
        )
        "#,
        params![
            input.employee_code,
            input.email_address,
            input.category,
            input.subject,
            input.problem_description,
            input.attachment_name,
            input.attachment_url,
            input.priority,
            input.assigned_to,
            input.customer,
            input.time_spent,
            input.activities,
            input.my_activity,
            input.deadline,
            input.sla_status,
            input.sla_deadline,
            input.rating_avg_text,
            input.kanban_state,
            input.stage,
            input.status,
            now_local,
            now_local,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Inserts a new ticket into `tickets` table and returns the generated `ticket_id`.
///
/// `email_address` / `customer` are filled from the local sheet cache for
/// `employee_code` (the `Employee` tab's `Mail` / `Name` columns) — never
/// from baked-in values. Empty when the sheet has no such data.
pub fn create_ticket(
    subject: &str,
    category: &str,
    priority: &str,
    employee_code: &str,
    assigned_to: &str,
    kanban_state: &str,
    description: &str,
) -> Result<i64> {
    let input = NewTicketInput {
        employee_code: employee_code.to_string(),
        email_address: employee_email(employee_code),
        customer: employee_name(employee_code),
        category: category.to_string(),
        subject: subject.to_string(),
        problem_description: description.to_string(),
        priority: priority.to_string(),
        assigned_to: assigned_to.to_string(),
        kanban_state: kanban_state.to_string(),
        stage: kanban_state.to_string(),
        status: "Open".to_string(),
        ..Default::default()
    };
    create_ticket_full(&input)
}

/// Deletes a ticket by `ticket_id`.
pub fn delete_ticket(ticket_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "DELETE FROM tickets WHERE ticket_id = ?1",
        params![ticket_id],
    )?;
    Ok(())
}

/// Updates the priority of a ticket.
pub fn update_ticket_priority(ticket_id: i64, new_priority: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET priority = ?1, last_updated_on = datetime('now', '+330 minutes') WHERE ticket_id = ?2",
        params![new_priority, ticket_id],
    )?;
    Ok(())
}

/// Updates the category of a ticket.
pub fn update_ticket_category(ticket_id: i64, new_category: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET category = ?1, last_updated_on = datetime('now', '+330 minutes') WHERE ticket_id = ?2",
        params![new_category, ticket_id],
    )?;
    Ok(())
}

/// Updates the assignee of a ticket.
pub fn update_ticket_assignee(ticket_id: i64, new_assignee: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET assigned_to = ?1, last_updated_on = datetime('now', '+330 minutes') WHERE ticket_id = ?2",
        params![new_assignee, ticket_id],
    )?;
    Ok(())
}

/// Updates all fields of a ticket in `tickets` table.
#[allow(clippy::too_many_arguments)]
pub fn update_ticket_full(
    ticket_id: i64,
    employee_code: &str,
    email_address: &str,
    category: &str,
    subject: &str,
    problem_description: &str,
    attachment_name: &str,
    attachment_url: &str,
    priority: &str,
    assigned_to: &str,
    customer: &str,
    time_spent: f64,
    activities: &str,
    my_activity: &str,
    deadline: &str,
    sla_status: &str,
    sla_deadline: &str,
    rating_avg_text: &str,
    kanban_state: &str,
    stage: &str,
    status: &str,
) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        UPDATE tickets
        SET employee_code = ?1,
            email_address = ?2,
            category = ?3,
            subject = ?4,
            problem_description = ?5,
            attachment_name = ?6,
            attachment_url = ?7,
            priority = ?8,
            assigned_to = ?9,
            customer = ?10,
            time_spent = ?11,
            activities = ?12,
            my_activity = ?13,
            deadline = ?14,
            sla_status = ?15,
            sla_deadline = ?16,
            rating_avg_text = ?17,
            kanban_state = ?18,
            stage = ?19,
            status = ?20,
            last_updated_on = datetime('now', '+330 minutes')
        WHERE ticket_id = ?21
        "#,
        params![
            employee_code,
            email_address,
            category,
            subject,
            problem_description,
            attachment_name,
            attachment_url,
            priority,
            assigned_to,
            customer,
            time_spent,
            activities,
            my_activity,
            deadline,
            sla_status,
            sla_deadline,
            rating_avg_text,
            kanban_state,
            stage,
            status,
            ticket_id
        ],
    )?;
    Ok(())
}

// ============================================================
// Notes & Reminders Queries & Operations
// ============================================================

/// Fetches all notes from `NOTES` table.
pub fn get_all_notes() -> Vec<DbNote> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, COALESCE(emp_code, ''), COALESCE(content, ''), COALESCE(created_at, CURRENT_TIMESTAMP) FROM NOTES ORDER BY id ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok(DbNote {
            id: row.get(0)?,
            emp_code: row.get(1)?,
            content: row.get(2)?,
            created_at: row.get(3).unwrap_or_default(),
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Inserts a new note into `NOTES` table and returns its id.
pub fn create_note(emp_code: &str, content: &str) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO NOTES (emp_code, content, created_at) VALUES (?1, ?2, CURRENT_TIMESTAMP)",
        params![emp_code, content],
    )?;
    Ok(conn.last_insert_rowid())
}

// ============================================================
// Employee directory helpers (read-only cache of the sheet)
// ============================================================

/// Employee code to stamp on locally-created rows:
/// `TMS_EMPLOYEE_CODE` (env → `.env` → Postman collection), else empty.
/// Nothing is invented — the API rejects employee codes missing from the
/// `Employee` sheet with `EMPLOYEE_NOT_FOUND`.
pub fn configured_employee_code() -> String {
    crate::api::ApiConfig::load()
        .ok()
        .and_then(|c| c.employee_code)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_default()
}

/// Employee name for `code` from the local sheet cache, else empty.
pub fn employee_name(code: &str) -> String {
    find_employee_by_code(code)
        .map(|e| e.emp_name)
        .unwrap_or_default()
}

/// Employee email for `code` from the local sheet cache, else empty.
/// The sheet has duplicate/blank emails, so this is a best-effort lookup
/// used only to pre-fill local rows — never as a cache key.
pub fn employee_email(code: &str) -> String {
    find_employee_by_code(code)
        .map(|e| e.emp_email)
        .unwrap_or_default()
}

/// First employee code present in the local sheet cache (test harness
/// fallback when `TMS_EMPLOYEE_CODE` is unset). Production code must not
/// guess an employee.
pub fn first_cached_employee_code() -> String {
    let Ok(conn) = get_connection() else {
        return String::new();
    };
    conn.query_row(
        "SELECT COALESCE(emp_code, '') FROM USERS \
         WHERE COALESCE(emp_code, '') <> '' ORDER BY emp_id ASC LIMIT 1",
        [],
        |r| r.get::<_, String>(0),
    )
    .unwrap_or_default()
}

/// Deletes a note by `id`.
pub fn delete_note(id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM NOTES WHERE id = ?1", params![id])?;
    Ok(())
}

/// Updates note content.
pub fn update_note(id: i64, content: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE NOTES SET content = ?1 WHERE id = ?2",
        params![content, id],
    )?;
    Ok(())
}

/// Updates note employee code.
pub fn update_note_emp_code(id: i64, emp_code: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE NOTES SET emp_code = ?1 WHERE id = ?2",
        params![emp_code, id],
    )?;
    Ok(())
}

// ============================================================
// Users Queries & Operations
// ============================================================

// ============================================================
// Employee cache (Google Sheets `employees.list` → USERS)
// ============================================================

/// Reads the cached employee directory, newest sheet row last.
///
/// Empty string is returned for missing email/division so the UI can
/// render a placeholder instead of "null".
pub fn get_all_employees() -> Vec<DbEmployee> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        r#"
        SELECT
            emp_id,
            COALESCE(emp_code, ''),
            emp_name,
            COALESCE(emp_email, ''),
            COALESCE(department, ''),
            COALESCE(division, '')
        FROM USERS
        WHERE is_deleted = 0
        ORDER BY emp_name COLLATE NOCASE ASC
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        let mut emp_email: String = row.get(3)?;
        let mut division: String = row.get(5)?;

        let is_placeholder = |s: &str| -> bool {
            let lower = s.trim().to_ascii_lowercase();
            lower.is_empty()
                || lower == "null"
                || lower == "undefined"
                || lower == "n/a"
                || lower == "na"
                || lower == "none"
                || lower == "nil"
                || lower == "-"
                || lower == "—"
        };

        if !emp_email.contains('@') && division.contains('@') {
            std::mem::swap(&mut emp_email, &mut division);
        }

        if !emp_email.contains('@') {
            if !is_placeholder(&emp_email) && (division.is_empty() || is_placeholder(&division)) {
                division = std::mem::take(&mut emp_email);
            } else {
                emp_email.clear();
            }
        } else if is_placeholder(&emp_email) {
            emp_email.clear();
        }

        if is_placeholder(&division) {
            division.clear();
        }

        Ok(DbEmployee {
            emp_id: row.get(0)?,
            emp_code: row.get(1)?,
            emp_name: row.get(2)?,
            emp_email,
            department: row.get(4)?,
            division,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Helper to extract clean employee code and name candidates from any input string.
/// E.g. "EMP-11061" -> ["EMP-11061", "11061"]
///      "11061 (John Doe)" -> ["11061 (John Doe)", "11061", "John Doe"]
pub fn extract_code_and_name_candidates(raw: &str) -> Vec<String> {
    let mut out = Vec::new();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return out;
    }
    out.push(trimmed.to_string());

    // Check for parentheses (e.g. "11061 (John Doe)" or "John Doe (11061)")
    if let (Some(open), Some(close)) = (trimmed.find('('), trimmed.rfind(')')) {
        if close > open {
            let outside = trimmed[..open].trim();
            let inside = trimmed[open + 1..close].trim();
            if !outside.is_empty() {
                out.push(outside.to_string());
            }
            if !inside.is_empty() {
                out.push(inside.to_string());
            }
        }
    }

    // Strip "emp-", "emp_", "emp " prefixes
    let lower = trimmed.to_lowercase();
    for prefix in &["emp-", "emp_", "emp ", "emp"] {
        if let Some(stripped) = lower.strip_prefix(prefix) {
            let s = stripped.trim();
            if !s.is_empty() {
                out.push(s.to_string());
            }
        }
    }

    // Extract continuous digits if any (e.g. "EMP-11061" -> "11061")
    let digits: String = trimmed.chars().filter(|c| c.is_ascii_digit()).collect();
    if !digits.is_empty() && digits != trimmed {
        out.push(digits);
    }

    out
}

type EmpCache = (PathBuf, std::collections::HashMap<String, DbEmployee>);
static EMP_LOOKUP_CACHE: std::sync::RwLock<Option<EmpCache>> = std::sync::RwLock::new(None);

pub fn invalidate_employee_lookup_cache() {
    if let Ok(mut guard) = EMP_LOOKUP_CACHE.write() {
        *guard = None;
    }
}

fn load_employee_cache_map(path: &Path) -> Option<std::collections::HashMap<String, DbEmployee>> {
    let mut map = std::collections::HashMap::new();
    for emp in get_all_employees() {
        let c = emp.emp_code.trim().to_lowercase();
        if !c.is_empty() {
            map.entry(c).or_insert_with(|| emp.clone());
        }
        let n = emp.emp_name.trim().to_lowercase();
        if !n.is_empty() {
            map.entry(n).or_insert_with(|| emp.clone());
        }
        let e = emp.emp_email.trim().to_lowercase();
        if !e.is_empty() {
            map.entry(e).or_insert_with(|| emp.clone());
        }
    }
    if let Ok(mut guard) = EMP_LOOKUP_CACHE.write() {
        *guard = Some((path.to_path_buf(), map.clone()));
    }
    Some(map)
}

/// Lookup a single employee by their employee code, name, or email with flexible matching.
pub fn find_employee_by_code(code: &str) -> Option<DbEmployee> {
    let clean = code.trim();
    if clean.is_empty() {
        return None;
    }
    let current_path = db_path();
    let candidates = extract_code_and_name_candidates(clean);

    if let Ok(guard) = EMP_LOOKUP_CACHE.read() {
        if let Some((ref cached_path, ref map)) = *guard {
            if cached_path == &current_path {
                for cand in &candidates {
                    if let Some(emp) = map.get(&cand.to_lowercase()) {
                        return Some(emp.clone());
                    }
                }
                return None;
            }
        }
    }

    if let Some(map) = load_employee_cache_map(&current_path) {
        for cand in &candidates {
            if let Some(emp) = map.get(&cand.to_lowercase()) {
                return Some(emp.clone());
            }
        }
        return None;
    }
    None
}

/// Checks if a ticket or reminder's employee fields match a user search query.
pub fn matches_emp_query(emp_code_field: &str, emp_name_field: &str, emp_email_field: &str, query: &str) -> bool {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return true;
    }

    // Direct match against fields
    if emp_code_field.to_lowercase().contains(&q)
        || emp_name_field.to_lowercase().contains(&q)
        || emp_email_field.to_lowercase().contains(&q)
    {
        return true;
    }

    // Digits comparison (e.g. searching "11061" matches "EMP-11061" or vice versa)
    let q_digits: String = q.chars().filter(|c| c.is_ascii_digit()).collect();
    let code_digits: String = emp_code_field.chars().filter(|c| c.is_ascii_digit()).collect();
    if !q_digits.is_empty()
        && !code_digits.is_empty()
        && (q_digits == code_digits || code_digits.contains(&q_digits) || q_digits.contains(&code_digits))
    {
        return true;
    }

    // Stripped EMP prefix comparison
    let clean_q = q.trim_start_matches("emp-")
        .trim_start_matches("emp_")
        .trim_start_matches("emp ")
        .trim_start_matches("emp")
        .trim();
    if !clean_q.is_empty() && emp_code_field.to_lowercase().contains(clean_q) {
        return true;
    }

    // Enriched check from DB USERS table
    if let Some(emp) = find_employee_by_code(emp_code_field).or_else(|| find_employee_by_code(emp_name_field)) {
        if emp.emp_code.to_lowercase().contains(&q)
            || emp.emp_name.to_lowercase().contains(&q)
            || emp.emp_email.to_lowercase().contains(&q)
            || emp.department.to_lowercase().contains(&q)
            || emp.division.to_lowercase().contains(&q)
        {
            return true;
        }
        if !q_digits.is_empty() {
            let emp_digits: String = emp.emp_code.chars().filter(|c| c.is_ascii_digit()).collect();
            if !emp_digits.is_empty() && (emp_digits == q_digits || emp_digits.contains(&q_digits) || q_digits.contains(&emp_digits)) {
                return true;
            }
        }
    }

    // Check if query itself is an employee in DB that matches the record
    if let Some(emp_from_q) = find_employee_by_code(&q) {
        if emp_from_q.emp_code.eq_ignore_ascii_case(emp_code_field)
            || emp_from_q.emp_name.eq_ignore_ascii_case(emp_name_field)
            || emp_from_q.emp_email.eq_ignore_ascii_case(emp_email_field)
        {
            return true;
        }
    }

    false
}

/// Number of cached employees (sidebar badge).
pub fn employee_count() -> usize {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row("SELECT COUNT(*) FROM USERS WHERE is_deleted = 0", [], |r| {
        r.get::<_, i64>(0)
    })
    .map(|n| n as usize)
    .unwrap_or(0)
}

/// Number of cached tickets (sidebar badge).
pub fn ticket_count() -> usize {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row("SELECT COUNT(*) FROM tickets", [], |r| r.get::<_, i64>(0))
        .map(|n| n as usize)
        .unwrap_or(0)
}

/// Number of notes (sidebar badge).
pub fn note_count() -> usize {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row("SELECT COUNT(*) FROM NOTES", [], |r| r.get::<_, i64>(0))
        .map(|n| n as usize)
        .unwrap_or(0)
}

/// Number of reminders (sidebar badge).
pub fn reminder_count() -> usize {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row("SELECT COUNT(*) FROM reminders", [], |r| r.get::<_, i64>(0))
        .map(|n| n as usize)
        .unwrap_or(0)
}

/// Replaces the whole employee cache with one `employees.list` result.
///
/// A full replace (in a single transaction) mirrors the sheet exactly —
/// employees removed upstream disappear locally too, and the non-unique
/// sheet codes never collide because rows are keyed by `emp_id`.
pub fn replace_employees(employees: &[crate::api::Employee]) -> Result<()> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM USERS", [])?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO USERS (emp_code, emp_name, emp_email, department, division, active, is_deleted) \
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 0)",
        )?;
        for e in employees {
            let mut emp = e.clone();
            crate::api::normalize_employee(&mut emp);
            let email = if emp.email.trim().is_empty() || !emp.email.contains('@') {
                None
            } else {
                Some(emp.email.trim().to_string())
            };
            let division = if emp.division.trim().is_empty() {
                None
            } else {
                Some(emp.division.trim().to_string())
            };
            stmt.execute(params![
                emp.employee_code.trim(),
                emp.employee_name.trim(),
                email,
                emp.department.trim(),
                division,
            ])?;
        }
    }
    tx.commit()?;
    invalidate_employee_lookup_cache();
    Ok(())
}

/// Distinct departments in the cache (filter dropdown), sorted.
pub fn employee_departments() -> Vec<String> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut stmt = match conn.prepare(
        "SELECT DISTINCT department FROM USERS \
         WHERE is_deleted = 0 AND department != '' \
         ORDER BY department COLLATE NOCASE ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map([], |r| r.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    // Bound to a local so the borrow of `stmt`/`conn` ends here rather
    // than extending into the function's tail expression.
    let out: Vec<String> = rows.filter_map(Result::ok).collect();
    out
}

/// Distinct divisions in the cache (filter dropdown), sorted.
pub fn employee_divisions() -> Vec<String> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut stmt = match conn.prepare(
        "SELECT DISTINCT division FROM USERS \
         WHERE is_deleted = 0 AND division != '' \
         ORDER BY division COLLATE NOCASE ASC",
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map([], |r| r.get::<_, String>(0)) {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    // Bound to a local so the borrow of `stmt`/`conn` ends here rather
    // than extending into the function's tail expression.
    let out: Vec<String> = rows.filter_map(Result::ok).collect();
    out
}

// ============================================================
// Reminders Queries & Operations
// ============================================================

/// Fetches all reminders from `reminders` table ordered by reminder_id.
pub fn get_all_reminders() -> Vec<DbReminder> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        r#"
        SELECT
            reminder_id,
            COALESCE(employee_code, ''),
            COALESCE(employee_name, ''),
            COALESCE(email_address, ''),
            COALESCE(service_category, 'General'),
            reminder_message,
            COALESCE(send_at, 'Today · 2:00 PM'),
            COALESCE(status, 'Pending'),
            COALESCE(email_sent_at, 'Not sent yet'),
            COALESCE(ticket_id, ''),
            COALESCE(created_at, CURRENT_TIMESTAMP),
            COALESCE(last_updated_at, CURRENT_TIMESTAMP),
            COALESCE(server_reminder_id, ''),
            COALESCE(server_last_updated, ''),
            COALESCE(cc_emails, ''),
            COALESCE(repeat_frequency, 'daily')
        FROM reminders
        WHERE COALESCE(status, '') != 'Cancelled'
        ORDER BY reminder_id ASC
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok(DbReminder {
            reminder_id: row.get(0)?,
            employee_code: row.get(1)?,
            employee_name: row.get(2)?,
            email_address: row.get(3)?,
            service_category: row.get(4)?,
            reminder_message: row.get(5)?,
            send_at: row.get(6)?,
            status: row.get(7)?,
            email_sent_at: row.get(8)?,
            ticket_id: row.get(9)?,
            created_at: row.get(10)?,
            last_updated_at: row.get(11)?,
            server_reminder_id: row.get(12)?,
            server_last_updated: row.get(13)?,
            cc_emails: row.get(14)?,
            repeat_frequency: row.get(15)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Creates a new reminder with all 14 sheet fields plus the local mapping
/// columns and returns the autogenerated `reminder_id`.
pub fn create_reminder_full(input: &NewReminderInput) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        INSERT INTO reminders (
            employee_code, employee_name, email_address, service_category,
            reminder_message, send_at, status, email_sent_at, ticket_id, cc_emails,
            repeat_frequency, created_at, last_updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
        )
        "#,
        params![
            input.employee_code,
            input.employee_name,
            input.email_address,
            input.service_category,
            input.reminder_message,
            input.send_at,
            input.status,
            input.email_sent_at,
            input.ticket_id,
            input.cc_emails,
            input.repeat_frequency,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Updates status of a reminder (e.g. "Pending", "Completed").
pub fn update_reminder_status(reminder_id: i64, status: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET status = ?1, last_updated_at = CURRENT_TIMESTAMP WHERE reminder_id = ?2",
        params![status, reminder_id],
    )?;
    Ok(())
}

/// Updates category of a reminder.
pub fn update_reminder_category(reminder_id: i64, service_category: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET service_category = ?1, last_updated_at = CURRENT_TIMESTAMP WHERE reminder_id = ?2",
        params![service_category, reminder_id],
    )?;
    Ok(())
}

/// Updates timing / send_at of a reminder.
pub fn update_reminder_send_at(reminder_id: i64, send_at: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET send_at = ?1, last_updated_at = CURRENT_TIMESTAMP WHERE reminder_id = ?2",
        params![send_at, reminder_id],
    )?;
    Ok(())
}

/// Updates repeat frequency of a reminder ("daily" or "once").
pub fn update_reminder_frequency(reminder_id: i64, repeat_frequency: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET repeat_frequency = ?1, last_updated_at = CURRENT_TIMESTAMP WHERE reminder_id = ?2",
        params![repeat_frequency, reminder_id],
    )?;
    Ok(())
}

/// Updates all fields of a reminder.
pub fn update_reminder_full(reminder_id: i64, input: &NewReminderInput) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        UPDATE reminders SET
            employee_code = ?1,
            employee_name = ?2,
            email_address = ?3,
            service_category = ?4,
            reminder_message = ?5,
            send_at = ?6,
            status = ?7,
            email_sent_at = ?8,
            ticket_id = ?9,
            cc_emails = ?10,
            repeat_frequency = ?11,
            last_updated_at = CURRENT_TIMESTAMP
        WHERE reminder_id = ?12
        "#,
        params![
            input.employee_code,
            input.employee_name,
            input.email_address,
            input.service_category,
            input.reminder_message,
            input.send_at,
            input.status,
            input.email_sent_at,
            input.ticket_id,
            input.cc_emails,
            input.repeat_frequency,
            reminder_id,
        ],
    )?;
    Ok(())
}

pub fn delete_server_reminder_db(server_reminder_id: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "DELETE FROM reminders WHERE server_reminder_id = ?1",
        params![server_reminder_id],
    )?;
    Ok(())
}

/// Deletes a reminder by reminder_id.
pub fn delete_reminder_db(reminder_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "DELETE FROM reminders WHERE reminder_id = ?1",
        params![reminder_id],
    )?;
    Ok(())
}

// ============================================================
// Sync layer: Google Sheets ↔ SQLite
//
// * `tickets.server_ticket_id` / `tickets.server_last_updated` map the
//   local INTEGER autoincrement id to the server's `TKT-…` id and store
//   the server row version used for conflict detection.
// * `sync_queue` is the persistent operation queue (ops survive restarts).
// * `sync_meta` holds sync timestamps (e.g. last pull).
// * `sync_conflicts` records unresolved local/server divergences.
// ============================================================

/// Adds the ticket↔server mapping columns once (ALTER TABLE has no
/// IF NOT EXISTS, so existing columns are checked first).
#[allow(clippy::let_and_return)]
fn ensure_ticket_sync_columns(conn: &Connection) -> Result<()> {
    let has_column = |name: &str| -> bool {
        let Ok(mut stmt) = conn.prepare("PRAGMA table_info(tickets)") else {
            return false;
        };
        let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
            return false;
        };
        // Bind to a local so the borrow of `stmt` ends inside the closure.
        let found = rows.filter_map(Result::ok).any(|c| c == name);
        found
    };

    if !has_column("server_ticket_id") {
        conn.execute("ALTER TABLE tickets ADD COLUMN server_ticket_id TEXT", [])?;
    }
    if !has_column("server_last_updated") {
        conn.execute(
            "ALTER TABLE tickets ADD COLUMN server_last_updated TEXT",
            [],
        )?;
    }
    Ok(())
}

#[allow(clippy::let_and_return)]
fn ensure_reminder_sync_columns(conn: &Connection) -> Result<()> {
    let has_column = |name: &str| -> bool {
        let Ok(mut stmt) = conn.prepare("PRAGMA table_info(reminders)") else {
            return false;
        };
        let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
            return false;
        };
        let found = rows.filter_map(Result::ok).any(|c| c == name);
        found
    };

    if !has_column("server_reminder_id") {
        conn.execute(
            "ALTER TABLE reminders ADD COLUMN server_reminder_id TEXT",
            [],
        )?;
    }
    if !has_column("server_last_updated") {
        conn.execute(
            "ALTER TABLE reminders ADD COLUMN server_last_updated TEXT",
            [],
        )?;
    }
    // Sheet column M (`CC Emails`) — part of the 13-column Reminders layout
    // in TMS_SHEET_HEADERS_README.md. Added so the local schema mirrors the
    // sheet; older databases are migrated in place.
    if !has_column("cc_emails") {
        conn.execute("ALTER TABLE reminders ADD COLUMN cc_emails TEXT", [])?;
    }
    // Sheet column N (`Repeat Frequency`): 14th column, 'daily' or 'once'. Treated as 'daily' when blank.
    if !has_column("repeat_frequency") {
        conn.execute("ALTER TABLE reminders ADD COLUMN repeat_frequency TEXT DEFAULT 'daily'", [])?;
    }
    Ok(())
}

#[allow(clippy::let_and_return)]
fn ensure_note_sync_columns(conn: &Connection) -> Result<()> {
    let has_column = |name: &str| -> bool {
        let Ok(mut stmt) = conn.prepare("PRAGMA table_info(NOTES)") else {
            return false;
        };
        let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
            return false;
        };
        let cols: Vec<String> = rows.filter_map(Result::ok).collect();
        cols.iter().any(|c| c == name)
    };

    if !has_column("created_at") {
        let _ = conn.execute(
            "ALTER TABLE NOTES ADD COLUMN created_at DATETIME DEFAULT CURRENT_TIMESTAMP",
            [],
        );
    }
    Ok(())
}

#[allow(clippy::let_and_return)]
fn ensure_sync_queue_columns(conn: &Connection) -> Result<()> {
    let table_exists: bool = conn
        .query_row(
            "SELECT count(*) FROM sqlite_master WHERE type='table' AND name='sync_queue'",
            [],
            |r| r.get::<_, i64>(0),
        )
        .map(|c| c > 0)
        .unwrap_or(false);
    if !table_exists {
        return Ok(());
    }

    let has_column = |name: &str| -> bool {
        let Ok(mut stmt) = conn.prepare("PRAGMA table_info(sync_queue)") else {
            return false;
        };
        let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
            return false;
        };
        let found = rows.filter_map(Result::ok).any(|c| c == name);
        found
    };

    if !has_column("entity_type") {
        conn.execute(
            "ALTER TABLE sync_queue ADD COLUMN entity_type TEXT NOT NULL DEFAULT 'ticket'",
            [],
        )?;
    }
    Ok(())
}

/// Migrates `USERS` to the employee-cache schema once.
///
/// The original table declared `emp_email TEXT UNIQUE NOT NULL` and
/// `emp_code INTEGER NOT NULL`, but `employees.list` returns 324 rows of
/// which 293 have no email, 4 emails are shared by several people and one
/// row has no code — so the old constraints reject the real data. The old
/// table is rebuilt (copying any existing rows) and `division` is added.
#[allow(clippy::let_and_return)]
fn ensure_user_schema(conn: &Connection) -> Result<()> {
    let has_column = |name: &str| -> bool {
        let Ok(mut stmt) = conn.prepare("PRAGMA table_info(USERS)") else {
            return false;
        };
        let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(1)) else {
            return false;
        };
        let found = rows.filter_map(Result::ok).any(|c| c == name);
        found
    };

    if has_column("division") {
        return Ok(()); // already migrated
    }

    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS USERS_new(
            emp_id INTEGER PRIMARY KEY AUTOINCREMENT,
            emp_code TEXT,
            emp_name TEXT NOT NULL,
            emp_email TEXT,
            department TEXT,
            division TEXT,
            active BOOLEAN DEFAULT 1,
            is_deleted BOOLEAN DEFAULT 0
        );
        INSERT INTO USERS_new (emp_id, emp_code, emp_name, emp_email, department, active, is_deleted)
            SELECT emp_id,
                   CASE WHEN typeof(emp_code) = 'integer' THEN CAST(emp_code AS TEXT)
                        ELSE emp_code END,
                   emp_name,
                   CASE WHEN emp_email = '' THEN NULL ELSE emp_email END,
                   department,
                   COALESCE(active, 1),
                   COALESCE(is_deleted, 0)
            FROM USERS;
        DROP TABLE USERS;
        ALTER TABLE USERS_new RENAME TO USERS;
        "#,
    )?;
    Ok(())
}

/// Cleans up any misplaced employee email / division data in the local USERS cache:
/// 1. Sheet alignment quirk: if an employee had no email, upstream column shifting
///    placed their division (e.g. "POWER PLANT DIVISION") into the email column while
///    leaving division empty. Moves the division text to `division` and clears `emp_email`.
/// 2. Any email string without '@' or placeholder text ("null", "undefined", "N/A", "—")
///    is converted to SQL NULL so the UI renders clean empty states and no unwanted
///    strings are used.
/// 3. Division placeholders ("null", "undefined", "—") are cleared to NULL.
fn sanitize_user_cache(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        -- 1. Where emp_email contains a division name and division is empty, preserve division
        UPDATE USERS SET
            division = TRIM(emp_email),
            emp_email = NULL
        WHERE emp_email IS NOT NULL
          AND emp_email NOT LIKE '%@%'
          AND TRIM(emp_email) != ''
          AND LOWER(TRIM(emp_email)) NOT IN ('null', 'undefined', 'n/a', 'na', 'none', 'nil', '-', '—')
          AND (division IS NULL OR TRIM(division) = '' OR division = '—' OR division = '-');

        -- 2. Set any remaining non-email or placeholder text in emp_email to NULL
        UPDATE USERS SET
            emp_email = NULL
        WHERE emp_email IS NOT NULL
          AND (emp_email NOT LIKE '%@%' OR LOWER(TRIM(emp_email)) IN ('null', 'undefined', 'n/a', 'na', 'none', 'nil', '-', '—'));

        -- 3. Clean up placeholder text in division
        UPDATE USERS SET
            division = NULL
        WHERE division IS NOT NULL
          AND (TRIM(division) = '' OR LOWER(TRIM(division)) IN ('null', 'undefined', 'n/a', 'na', 'none', 'nil', '-', '—'));
        "#,
    )?;
    Ok(())
}

/// One queued local mutation awaiting confirmation from the API.
#[derive(Clone, Debug)]
pub struct SyncOp {
    pub op_id: i64,
    pub local_ticket_id: i64,
    pub server_ticket_id: Option<String>,
    /// `CREATE` | `UPDATE` | `DELETE` | `SEND_NOW`
    pub op_type: String,
    pub payload: Value,
    pub created_at: String,
    pub retry_count: i64,
    /// `pending` | `in_flight` | `synced` | `failed` | `superseded`
    pub status: String,
    pub last_error: Option<String>,
    /// Epoch seconds before which the op must not be retried (backoff).
    pub next_attempt_at: Option<i64>,
    /// `ticket` | `reminder`
    pub entity_type: String,
}

const OP_COLUMNS: &str = "op_id, local_ticket_id, server_ticket_id, op_type, payload, \
                          created_at, retry_count, status, last_error, next_attempt_at, \
                          COALESCE(entity_type, 'ticket')";

fn row_to_op(row: &rusqlite::Row<'_>) -> rusqlite::Result<SyncOp> {
    let payload_str: String = row.get(4)?;
    Ok(SyncOp {
        op_id: row.get(0)?,
        local_ticket_id: row.get(1)?,
        server_ticket_id: row.get(2)?,
        op_type: row.get(3)?,
        payload: serde_json::from_str(&payload_str).unwrap_or(Value::Null),
        created_at: row.get(5)?,
        retry_count: row.get(6)?,
        status: row.get(7)?,
        last_error: row.get(8)?,
        next_attempt_at: row.get(9)?,
        entity_type: row.get(10)?,
    })
}

/// Shallow field merge — later values win per field (API updates are
/// partial field sets, so this is exactly the right granularity).
pub fn merge_json(base: &Value, patch: &Value) -> Value {
    let mut out = base.as_object().cloned().unwrap_or_default();
    if let Some(p) = patch.as_object() {
        for (k, v) in p {
            out.insert(k.clone(), v.clone());
        }
    }
    Value::Object(out)
}

fn active_ops(conn: &Connection, local_ticket_id: i64) -> Result<Vec<SyncOp>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {OP_COLUMNS} FROM sync_queue \
         WHERE local_ticket_id = ?1 AND status IN ('pending','failed','in_flight') \
         ORDER BY op_id"
    ))?;
    let rows = stmt.query_map(params![local_ticket_id], row_to_op)?;
    rows.collect::<Result<Vec<_>>>()
}

/// All non-terminal operations for a ticket (used for coalescing and tests).
pub fn active_ops_for_ticket(local_ticket_id: i64) -> Result<Vec<SyncOp>> {
    let conn = get_connection()?;
    active_ops(&conn, local_ticket_id)
}

fn insert_op(
    conn: &Connection,
    local_ticket_id: i64,
    server_ticket_id: Option<&str>,
    op_type: &str,
    payload: &Value,
    created_at: &str,
) -> Result<i64> {
    insert_entity_op(
        conn,
        "ticket",
        local_ticket_id,
        server_ticket_id,
        op_type,
        payload,
        created_at,
    )
}

fn insert_entity_op(
    conn: &Connection,
    entity_type: &str,
    local_id: i64,
    server_id: Option<&str>,
    op_type: &str,
    payload: &Value,
    created_at: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO sync_queue \
         (local_ticket_id, server_ticket_id, op_type, payload, created_at, status, entity_type) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6)",
        params![
            local_id,
            server_id,
            op_type,
            payload.to_string(),
            created_at,
            entity_type
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Queue a local UPDATE. Coalesces with any active operation for the same
/// ticket so the API never receives duplicate or conflicting requests for
/// one row (edits that arrive while an op is in flight are queued behind it).
pub fn enqueue_update(
    local_ticket_id: i64,
    server_ticket_id: Option<String>,
    fields: &Value,
    created_at: &str,
) -> Result<i64> {
    let conn = get_connection()?;

    // Fold into an existing pending/failed UPDATE.
    if let Some(op) = active_ops(&conn, local_ticket_id)?
        .into_iter()
        .find(|o| o.op_type == "UPDATE" && o.status != "in_flight")
    {
        let merged = merge_json(&op.payload, fields);
        conn.execute(
            "UPDATE sync_queue SET payload = ?1, status = 'pending', \
             next_attempt_at = NULL, last_error = NULL WHERE op_id = ?2",
            params![merged.to_string(), op.op_id],
        )?;
        return Ok(op.op_id);
    }

    // The ticket never reached the server: fold the edit into the CREATE.
    if let Some(op) = active_ops(&conn, local_ticket_id)?
        .into_iter()
        .find(|o| o.op_type == "CREATE" && o.status != "in_flight")
    {
        let merged = merge_json(&op.payload, fields);
        conn.execute(
            "UPDATE sync_queue SET payload = ?1 WHERE op_id = ?2",
            params![merged.to_string(), op.op_id],
        )?;
        return Ok(op.op_id);
    }

    // Otherwise queue behind any in-flight op.
    insert_op(
        &conn,
        local_ticket_id,
        server_ticket_id.as_deref(),
        "UPDATE",
        fields,
        created_at,
    )
}

/// Queue a local CREATE built from the current SQLite row.
pub fn enqueue_create(local_ticket_id: i64, payload: &Value, created_at: &str) -> Result<i64> {
    let conn = get_connection()?;
    let active = active_ops(&conn, local_ticket_id)?;

    // Re-creating a row supersedes a queued delete for it.
    for op in active.iter().filter(|o| o.op_type == "DELETE") {
        conn.execute(
            "UPDATE sync_queue SET status = 'superseded' WHERE op_id = ?1",
            params![op.op_id],
        )?;
    }

    if let Some(op) = active.into_iter().find(|o| o.op_type == "CREATE") {
        if op.status == "in_flight" {
            return Ok(op.op_id); // already being pushed
        }
        let merged = merge_json(&op.payload, payload);
        conn.execute(
            "UPDATE sync_queue SET payload = ?1, status = 'pending', \
             next_attempt_at = NULL, last_error = NULL WHERE op_id = ?2",
            params![merged.to_string(), op.op_id],
        )?;
        return Ok(op.op_id);
    }

    insert_op(&conn, local_ticket_id, None, "CREATE", payload, created_at)
}

/// Queue a local DELETE. Supersedes every other active op for the row
/// (their changes die with the ticket) and only reaches the API when the
/// row was previously synced — an unmapped row never existed server-side.
/// Returns the queued op id, or `None` when there is nothing to delete
/// remotely.
pub fn enqueue_delete(
    local_ticket_id: i64,
    server_ticket_id: Option<&str>,
    created_at: &str,
) -> Result<Option<i64>> {
    let conn = get_connection()?;
    let active = active_ops(&conn, local_ticket_id)?;

    for op in &active {
        conn.execute(
            "UPDATE sync_queue SET status = 'superseded', next_attempt_at = NULL WHERE op_id = ?1",
            params![op.op_id],
        )?;
    }

    let Some(server_id) = server_ticket_id else {
        // Never reached the server (no pending CREATE to clean up either —
        // it was superseded above): nothing to do remotely.
        return Ok(None);
    };

    Ok(Some(insert_op(
        &conn,
        local_ticket_id,
        Some(server_id),
        "DELETE",
        &Value::Null,
        created_at,
    )?))
}

/// Operations whose backoff has expired (or which never had one).
pub fn due_ops(now_epoch: i64, limit: i64) -> Result<Vec<SyncOp>> {
    let conn = get_connection()?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {OP_COLUMNS} FROM sync_queue \
         WHERE status IN ('pending','failed') \
           AND (next_attempt_at IS NULL OR next_attempt_at <= ?1) \
         ORDER BY op_id ASC LIMIT ?2"
    ))?;
    let rows = stmt.query_map(params![now_epoch, limit], row_to_op)?;
    rows.collect::<Result<Vec<_>>>()
}

pub fn mark_op_in_flight(op_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_queue SET status = 'in_flight' WHERE op_id = ?1",
        params![op_id],
    )?;
    Ok(())
}

/// Server confirmed the operation: flip to `synced` (retained as recent
/// history; the newest 50 synced rows are kept).
pub fn mark_op_synced(op_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_queue SET status = 'synced', last_error = NULL, next_attempt_at = NULL \
         WHERE op_id = ?1",
        params![op_id],
    )?;
    conn.execute(
        "DELETE FROM sync_queue WHERE status = 'synced' AND op_id NOT IN \
         (SELECT op_id FROM sync_queue WHERE status = 'synced' ORDER BY op_id DESC LIMIT 50)",
        [],
    )?;
    Ok(())
}

/// Attempt failed. Retryable failures go back to `pending` with a backoff
/// deadline; non-retryable ones become `failed` (retained — never
/// discarded — and re-checked slowly).
pub fn mark_op_error(op_id: i64, error: &str, retryable: bool, next_attempt_at: i64) -> Result<()> {
    let conn = get_connection()?;
    let status = if retryable { "pending" } else { "failed" };
    conn.execute(
        "UPDATE sync_queue SET retry_count = retry_count + 1, status = ?1, \
         last_error = ?2, next_attempt_at = ?3 WHERE op_id = ?4",
        params![status, error, next_attempt_at, op_id],
    )?;
    Ok(())
}

pub fn mark_op_superseded(op_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_queue SET status = 'superseded', next_attempt_at = NULL WHERE op_id = ?1",
        params![op_id],
    )?;
    Ok(())
}

pub fn cancel_queued_ops_for_entity(entity: &str, local_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_queue SET status = 'superseded', next_attempt_at = NULL \
         WHERE entity_type = ?1 AND local_ticket_id = ?2 AND status IN ('pending', 'failed')",
        params![entity, local_id],
    )?;
    Ok(())
}

/// Resets retryable failed operations in sync_queue to pending so they can retry.
/// Terminal failures (NOT_FOUND, CONFLICT, validation) are NEVER reset as transient errors.
pub fn reset_failed_sync_ops() -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_queue SET status = 'pending', next_attempt_at = NULL \
         WHERE status = 'failed' \
           AND last_error NOT LIKE '%NOT_FOUND%' \
           AND last_error NOT LIKE '%CONFLICT%' \
           AND last_error NOT LIKE '%Completed reminders cannot be updated%' \
           AND last_error NOT LIKE '%VALIDATION_ERROR%' \
           AND last_error NOT LIKE '%SCHEMA_MISMATCH%'",
        [],
    )?;
    Ok(())
}

/// Merge additional fields into an op's payload (used when collapsing
/// duplicate ops for the same ticket before execution).
pub fn merge_op_payload(op_id: i64, patch: &Value) -> Result<()> {
    let conn = get_connection()?;
    let current: Option<String> = conn
        .query_row(
            "SELECT payload FROM sync_queue WHERE op_id = ?1",
            params![op_id],
            |r| r.get(0),
        )
        .optional()?;
    let base: Value = current
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Object(Default::default()));
    let merged = merge_json(&base, patch);
    conn.execute(
        "UPDATE sync_queue SET payload = ?1 WHERE op_id = ?2",
        params![merged.to_string(), op_id],
    )?;
    Ok(())
}

/// Operations that still need to reach the API.
pub fn pending_op_count() -> i64 {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_queue WHERE status IN ('pending','in_flight','failed')",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

pub fn ticket_has_pending_ops(local_ticket_id: i64) -> bool {
    let Ok(conn) = get_connection() else {
        return false;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_queue \
         WHERE local_ticket_id = ?1 AND status IN ('pending','in_flight','failed')",
        params![local_ticket_id],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Subjects of CREATE ops that have not been confirmed yet — used by the
/// pull phase to avoid inserting a duplicate local row for a ticket the
/// queue is about to adopt.
pub fn pending_create_subjects() -> Vec<String> {
    let Ok(conn) = get_connection() else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare(
        "SELECT payload FROM sync_queue \
         WHERE op_type = 'CREATE' AND status IN ('pending','in_flight','failed')",
    ) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) else {
        return Vec::new();
    };
    rows.filter_map(Result::ok)
        .filter_map(|s| serde_json::from_str::<Value>(&s).ok())
        .filter_map(|v| {
            v.get("subject")
                .and_then(|s| s.as_str())
                .map(str::to_string)
        })
        .collect()
}

/// True when a server ticket has a queued DELETE — the pull phase must
/// not re-insert it locally while the delete is pending.
pub fn server_id_has_active_delete(server_ticket_id: &str) -> bool {
    let Ok(conn) = get_connection() else {
        return false;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_queue \
         WHERE server_ticket_id = ?1 AND op_type = 'DELETE' \
           AND status IN ('pending','in_flight','failed')",
        params![server_ticket_id],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Most recent error among active ops (surfaces in the sync status pill
/// while an op waits out its backoff).
pub fn latest_active_error() -> Option<String> {
    let conn = get_connection().ok()?;
    conn.query_row(
        "SELECT last_error FROM sync_queue \
         WHERE status IN ('pending','failed') AND last_error IS NOT NULL \
         ORDER BY op_id DESC LIMIT 1",
        [],
        |r| r.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

/// Most recent error among terminal failed ops.
pub fn latest_failed_error() -> Option<String> {
    let conn = get_connection().ok()?;
    conn.query_row(
        "SELECT last_error FROM sync_queue \
         WHERE status = 'failed' AND last_error IS NOT NULL \
         ORDER BY op_id DESC LIMIT 1",
        [],
        |r| r.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

/// Count of permanently failed ops in sync_queue.
pub fn failed_op_count() -> i64 {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_queue WHERE status = 'failed'",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// Clears last_error from all sync queue entries and resets active sync error metadata.
pub fn clear_sync_errors() -> Result<()> {
    let conn = get_connection()?;
    conn.execute("UPDATE sync_queue SET last_error = NULL WHERE last_error IS NOT NULL", [])?;
    let _ = sync_meta_set("last_sync_error", "");
    Ok(())
}

// ---- sync metadata ----

pub fn sync_meta_get(key: &str) -> Option<String> {
    let conn = get_connection().ok()?;
    conn.query_row(
        "SELECT value FROM sync_meta WHERE key = ?1",
        params![key],
        |r| r.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

pub fn sync_meta_set(key: &str, value: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO sync_meta (key, value) VALUES (?1, ?2) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Retrieves user-configured custom API settings stored locally in SQLite `sync_meta`.
pub fn get_system_api_config() -> (Option<String>, Option<String>, Option<String>) {
    let url = sync_meta_get("system_api_base_url").filter(|s| !s.trim().is_empty());
    let key = sync_meta_get("system_api_key").filter(|s| !s.trim().is_empty());
    let emp = sync_meta_get("system_employee_code").filter(|s| !s.trim().is_empty());
    (url, key, emp)
}

/// Saves user-configured custom API settings locally in SQLite `sync_meta`
/// and in `%APPDATA%\BPSL IT Support\.env`.
pub fn save_system_api_config(base_url: &str, api_key: &str, employee_code: &str) -> Result<()> {
    sync_meta_set("system_api_base_url", base_url.trim())?;
    sync_meta_set("system_api_key", api_key.trim())?;
    sync_meta_set("system_employee_code", employee_code.trim())?;

    if let Some(app_data) = std::env::var_os("APPDATA").or_else(|| std::env::var_os("LOCALAPPDATA")) {
        let app_dir = std::path::PathBuf::from(app_data).join("BPSL IT Support");
        let _ = std::fs::create_dir_all(&app_dir);
        let env_path = app_dir.join(".env");
        let proto = get_system_api_protocol().unwrap_or_else(|| {
            if api_key.trim().is_empty() {
                "rest".to_string()
            } else {
                "v1".to_string()
            }
        });
        let env_content = format!(
            "# BPSL IT Support - System API Configuration\nTMS_API_BASE_URL=\"{}\"\nTMS_API_KEY=\"{}\"\nTMS_EMPLOYEE_CODE=\"{}\"\nTMS_API_PROTOCOL=\"{}\"\n",
            base_url.trim(),
            api_key.trim(),
            employee_code.trim(),
            proto
        );
        let _ = std::fs::write(env_path, env_content);
    }
    Ok(())
}

/// Returns the user-configured API protocol override (`"rest"`, `"v1"`, or `"auto"`), if any.
pub fn get_system_api_protocol() -> Option<String> {
    sync_meta_get("system_api_protocol").filter(|s| !s.trim().is_empty())
}

/// Persists the user-configured API protocol (`"auto"`, `"rest"`, or `"v1"`).
pub fn save_system_api_protocol(protocol: &str) -> Result<()> {
    sync_meta_set("system_api_protocol", protocol.trim())
}

/// Clears user-configured custom API settings from SQLite `sync_meta`.
pub fn clear_system_api_config() -> Result<()> {
    sync_meta_set("system_api_base_url", "")?;
    sync_meta_set("system_api_key", "")?;
    sync_meta_set("system_employee_code", "")?;
    sync_meta_set("system_api_protocol", "")?;

    if let Some(app_data) = std::env::var_os("APPDATA").or_else(|| std::env::var_os("LOCALAPPDATA")) {
        let env_path = std::path::PathBuf::from(app_data).join("BPSL IT Support").join(".env");
        if env_path.exists() {
            let _ = std::fs::remove_file(env_path);
        }
    }
    Ok(())
}

pub const DEFAULT_REST_SHEET_URL: &str = "https://script.google.com/macros/s/AKfycbw32a4pqW9KjOFZebB8dE8IwTcStKR9drNWlliLD4pve4moL8H6Il88SO8BrpvtOdY7/exec";
pub const DEFAULT_V1_SHEET_URL: &str = "https://script.google.com/macros/s/AKfycbx6TTCZF1WfcFPpJWOqGoMXKhXq-wxJprGtCIKfAHZnm9wGmSgBuRCpiTWUu4519oyV/exec";
pub const DEFAULT_V1_SHEET_KEY: &str = "032fd9719ace4011bf0e7dab1dbed3363ecff4adbaf84b328539038cdc19c8389bd8d285cfa2400d8b89c874485e2315";

/// Configured Google Sheet data source (URL, API key, protocol, display name).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SheetSource {
    pub id: String,
    pub name: String,
    pub base_url: String,
    pub api_key: String,
    pub protocol: String,
}

impl SheetSource {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        protocol: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            base_url: base_url.into(),
            api_key: api_key.into(),
            protocol: protocol.into(),
        }
    }
}

/// Retrieves all configured Google Sheet sources from SQLite `sync_meta`.
/// If no custom sources have been saved, seeds with the default Primary (REST) and Secondary (V1) sources.
pub fn get_all_sheet_sources() -> Vec<SheetSource> {
    if let Some(json_str) = sync_meta_get("custom_sheet_sources") {
        if let Ok(sources) = serde_json::from_str::<Vec<SheetSource>>(&json_str) {
            if !sources.is_empty() {
                return sources;
            }
        }
    }

    // Seed directly from sync_meta keys or defaults to prevent any recursion
    let rest_name = sync_meta_get("sheet_rest_name")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Primary Sheet".to_string());
    let rest_url = sync_meta_get("sheet_rest_url")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_REST_SHEET_URL.to_string());
    let rest_key = sync_meta_get("sheet_rest_key").unwrap_or_default();

    let v1_name = sync_meta_get("sheet_v1_name")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "Secondary Sheet".to_string());
    let v1_url = sync_meta_get("sheet_v1_url")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_V1_SHEET_URL.to_string());
    let v1_key = sync_meta_get("sheet_v1_key")
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_V1_SHEET_KEY.to_string());

    let initial = vec![
        SheetSource {
            id: "rest".to_string(),
            name: rest_name,
            base_url: rest_url,
            api_key: rest_key,
            protocol: "rest".to_string(),
        },
        SheetSource {
            id: "v1".to_string(),
            name: v1_name,
            base_url: v1_url,
            api_key: v1_key,
            protocol: "v1".to_string(),
        },
    ];
    let _ = save_all_sheet_sources(&initial);
    initial
}

/// Saves the list of all Google Sheet sources in SQLite `sync_meta`.
pub fn save_all_sheet_sources(sources: &[SheetSource]) -> Result<()> {
    if let Ok(json_str) = serde_json::to_string(sources) {
        sync_meta_set("custom_sheet_sources", &json_str)?;
    }
    Ok(())
}

/// Returns the currently active Google Sheet source ID (e.g. `"rest"`, `"v1"`, or a custom ID).
pub fn get_active_sheet_source_id() -> String {
    if let Some(id) = sync_meta_get("active_sheet_source_id") {
        let t = id.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    if let Some(legacy) = sync_meta_get("active_sheet_profile") {
        let t = legacy.trim();
        if !t.is_empty() {
            return t.to_string();
        }
    }
    "rest".to_string()
}

/// Returns the active `SheetSource` instance.
pub fn get_active_sheet_source() -> SheetSource {
    let active_id = get_active_sheet_source_id();
    let sources = get_all_sheet_sources();
    sources
        .into_iter()
        .find(|s| s.id == active_id)
        .unwrap_or_else(|| SheetSource {
            id: "rest".to_string(),
            name: "Primary Sheet".to_string(),
            base_url: DEFAULT_REST_SHEET_URL.to_string(),
            api_key: String::new(),
            protocol: "rest".to_string(),
        })
}

/// Saves or updates a `SheetSource`. If its ID already exists, it is updated in-place;
/// otherwise, it is appended to the list of available sources.
pub fn save_sheet_source(source: &SheetSource) -> Result<()> {
    let mut sources = get_all_sheet_sources();
    if let Some(pos) = sources.iter().position(|s| s.id == source.id) {
        sources[pos] = source.clone();
    } else {
        sources.push(source.clone());
    }
    save_all_sheet_sources(&sources)?;

    // Backwards compatibility with legacy keys
    if source.id == "rest" {
        let _ = save_sheet_profile_name("rest", &source.name);
        let _ = sync_meta_set("sheet_rest_url", &source.base_url);
        let _ = sync_meta_set("sheet_rest_key", &source.api_key);
    } else if source.id == "v1" {
        let _ = save_sheet_profile_name("v1", &source.name);
        let _ = sync_meta_set("sheet_v1_url", &source.base_url);
        let _ = sync_meta_set("sheet_v1_key", &source.api_key);
    }

    if get_active_sheet_source_id() == source.id {
        let emp = sync_meta_get("system_employee_code").unwrap_or_default();
        save_system_api_config(&source.base_url, &source.api_key, &emp)?;
        save_system_api_protocol(&source.protocol)?;
    }
    Ok(())
}

/// Deletes a sheet source by ID. Returns an error if attempting to delete the only remaining source.
pub fn delete_sheet_source(id: &str) -> Result<()> {
    let mut sources = get_all_sheet_sources();
    if sources.len() <= 1 {
        return Ok(());
    }
    let was_active = get_active_sheet_source_id() == id;
    sources.retain(|s| s.id != id);
    save_all_sheet_sources(&sources)?;

    if was_active {
        if let Some(first) = sources.first() {
            let _ = switch_sheet_profile(&first.id);
        }
    }
    Ok(())
}

/// Returns the active Google Sheet profile ID.
pub fn get_active_sheet_profile() -> String {
    get_active_sheet_source_id()
}

/// Returns the user-assigned display name for a given sheet profile/source ID.
pub fn get_sheet_profile_name(profile: &str) -> String {
    if let Some(json_str) = sync_meta_get("custom_sheet_sources") {
        if let Ok(sources) = serde_json::from_str::<Vec<SheetSource>>(&json_str) {
            if let Some(src) = sources.iter().find(|s| s.id == profile) {
                return src.name.clone();
            }
        }
    }
    let key = if profile == "v1" { "sheet_v1_name" } else { "sheet_rest_name" };
    sync_meta_get(key)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            if profile == "v1" {
                "Secondary Sheet".to_string()
            } else {
                "Primary Sheet".to_string()
            }
        })
}

/// Saves the user-assigned display name for the sheet profile (`"rest"` or `"v1"`).
pub fn save_sheet_profile_name(profile: &str, name: &str) -> Result<()> {
    let key = if profile == "v1" { "sheet_v1_name" } else { "sheet_rest_name" };
    sync_meta_set(key, name.trim())
}

/// Returns `(base_url, api_key)` for the given sheet profile ID.
pub fn get_sheet_profile_config(profile: &str) -> (String, String) {
    if let Some(json_str) = sync_meta_get("custom_sheet_sources") {
        if let Ok(sources) = serde_json::from_str::<Vec<SheetSource>>(&json_str) {
            if let Some(src) = sources.iter().find(|s| s.id == profile) {
                return (src.base_url.clone(), src.api_key.clone());
            }
        }
    }
    if profile == "v1" {
        let url = sync_meta_get("sheet_v1_url")
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_V1_SHEET_URL.to_string());
        let key = sync_meta_get("sheet_v1_key")
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_V1_SHEET_KEY.to_string());
        (url, key)
    } else {
        let url = sync_meta_get("sheet_rest_url")
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_REST_SHEET_URL.to_string());
        let key = sync_meta_get("sheet_rest_key")
            .unwrap_or_default();
        (url, key)
    }
}

/// Saves `(name, base_url, api_key)` for the given sheet profile.
pub fn save_sheet_profile_config(profile: &str, name: &str, base_url: &str, api_key: &str) -> Result<()> {
    let proto = if profile == "v1" { "v1" } else { "rest" };
    let source = SheetSource {
        id: profile.to_string(),
        name: name.to_string(),
        base_url: base_url.to_string(),
        api_key: api_key.to_string(),
        protocol: proto.to_string(),
    };
    save_sheet_source(&source)?;
    sync_meta_set("active_sheet_source_id", profile)?;
    sync_meta_set("active_sheet_profile", profile)?;
    save_system_api_protocol(proto)?;
    let emp = sync_meta_get("system_employee_code").unwrap_or_default();
    save_system_api_config(base_url, if profile == "v1" { api_key } else { "" }, &emp)?;
    Ok(())
}

/// Switches the active Google Sheet source by ID, updates active API
/// credentials, clears previously synced remote cache so the new sheet's data loads cleanly,
/// and returns `(base_url, api_key, protocol)`.
pub fn switch_sheet_profile(profile: &str) -> Result<(String, String, String)> {
    let sources = get_all_sheet_sources();
    let source = sources
        .iter()
        .find(|s| s.id == profile)
        .cloned()
        .unwrap_or_else(|| {
            if profile == "v1" {
                sources.get(1).cloned().unwrap_or_else(|| sources[0].clone())
            } else {
                sources[0].clone()
            }
        });

    sync_meta_set("active_sheet_source_id", &source.id)?;
    sync_meta_set("active_sheet_profile", &source.id)?;
    save_system_api_protocol(&source.protocol)?;
    let emp = sync_meta_get("system_employee_code").unwrap_or_default();
    save_system_api_config(&source.base_url, &source.api_key, &emp)?;

    let conn = get_connection()?;
    conn.execute("DELETE FROM tickets WHERE COALESCE(server_ticket_id, '') != ''", [])?;
    conn.execute("DELETE FROM USERS", [])?;
    invalidate_employee_lookup_cache();

    sync_meta_set("last_pull_at", "0")?;
    sync_meta_set("last_employee_pull_at", "0")?;
    sync_meta_set("last_reminder_pull_at", "0")?;

    Ok((source.base_url, source.api_key, source.protocol))
}

// ---- ticket ↔ server mapping ----

/// Finds the local SQLite `ticket_id` for a ticket identified by either its server ID (e.g. "TC-0529"),
/// prefixed ID (e.g. "IT-123"), or raw numeric string.
pub fn find_ticket_local_id(id_str: &str) -> Option<i64> {
    let conn = get_connection().ok()?;
    let trimmed = id_str.trim();
    if trimmed.is_empty() {
        return None;
    }
    // 1. Exact match on server_ticket_id (e.g. "TC-0529")
    if let Ok(id) = conn.query_row(
        "SELECT ticket_id FROM tickets WHERE server_ticket_id = ?1 LIMIT 1",
        params![trimmed],
        |r| r.get::<_, i64>(0),
    ) {
        return Some(id);
    }
    // 2. Strip "IT-" or "it-" prefix
    if let Some(rest) = trimmed.strip_prefix("IT-").or_else(|| trimmed.strip_prefix("it-")) {
        if let Ok(n) = rest.parse::<i64>() {
            if let Ok(id) = conn.query_row(
                "SELECT ticket_id FROM tickets WHERE ticket_id = ?1 LIMIT 1",
                params![n],
                |r| r.get::<_, i64>(0),
            ) {
                return Some(id);
            }
        }
    }
    // 3. Raw integer
    if let Ok(n) = trimmed.parse::<i64>() {
        if let Ok(id) = conn.query_row(
            "SELECT ticket_id FROM tickets WHERE ticket_id = ?1 LIMIT 1",
            params![n],
            |r| r.get::<_, i64>(0),
        ) {
            return Some(id);
        }
    }
    None
}

/// True when a ticket has an active CREATE op in the queue.
pub fn ticket_has_pending_create(local_ticket_id: i64) -> bool {
    let Ok(conn) = get_connection() else {
        return false;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_queue \
         WHERE local_ticket_id = ?1 AND op_type = 'CREATE' \
           AND status IN ('pending','in_flight','failed')",
        params![local_ticket_id],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0) > 0
}

/// Server id of a local ticket (`None` when it never reached the API).
pub fn server_ticket_id(local_ticket_id: i64) -> Option<String> {
    let conn = get_connection().ok()?;
    conn.query_row(
        "SELECT server_ticket_id FROM tickets WHERE ticket_id = ?1",
        params![local_ticket_id],
        |r| r.get::<_, Option<String>>(0),
    )
    .optional()
    .ok()
    .flatten()
    .flatten()
    .filter(|s| !s.trim().is_empty())
}

/// Records the mapping after a confirmed CREATE/UPDATE and stores the
/// server row version (`last_updated_on`) used for conflict detection.
pub fn set_ticket_mapping(
    local_ticket_id: i64,
    server_ticket_id: &str,
    server_last_updated: &str,
) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET server_ticket_id = ?1, server_last_updated = ?2 \
         WHERE ticket_id = ?3",
        params![server_ticket_id, server_last_updated, local_ticket_id],
    )?;
    Ok(())
}

pub fn set_server_last_updated(local_ticket_id: i64, server_last_updated: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET server_last_updated = ?1 WHERE ticket_id = ?2",
        params![server_last_updated, local_ticket_id],
    )?;
    Ok(())
}

pub struct MappedTicket {
    pub local_id: i64,
    pub server_id: String,
    pub server_last_updated: String,
}

/// Every local ticket that has a server counterpart.
pub fn mapped_tickets() -> Vec<MappedTicket> {
    let Ok(conn) = get_connection() else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare(
        "SELECT ticket_id, server_ticket_id, COALESCE(server_last_updated, '') \
         FROM tickets WHERE server_ticket_id IS NOT NULL AND server_ticket_id != ''",
    ) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |r| {
        Ok(MappedTicket {
            local_id: r.get(0)?,
            server_id: r.get(1)?,
            server_last_updated: r.get(2)?,
        })
    }) else {
        return Vec::new();
    };
    rows.filter_map(Result::ok).collect()
}

// ---- server row ⇄ local row ----

/// Builds the API field set (`tickets.create` / `tickets.update` names)
/// from a local row. `activities` maps to the server's `activities_text`.
pub fn ticket_create_payload(local_ticket_id: i64) -> Option<Value> {
    let conn = get_connection().ok()?;
    let mut stmt = conn
        .prepare(
            "SELECT employee_code, email_address, category, subject, problem_description, \
                    attachment_name, attachment_url, priority, assigned_to, customer, \
                    time_spent, activities, my_activity, deadline, sla_status, sla_deadline, \
                    rating_avg_text, kanban_state, stage, status \
             FROM tickets WHERE ticket_id = ?1",
        )
        .ok()?;

    let value = stmt
        .query_row(params![local_ticket_id], |r| {
            let mut map = serde_json::Map::new();
            let emp_code: String = r.get(0)?;
            let email: String = r.get(1)?;
            let cat: String = r.get(2)?;
            let subj: String = r.get(3)?;
            let desc: String = r.get(4)?;
            let att_name: String = r.get(5)?;
            let att_url: String = r.get(6)?;
            let prio: String = r.get(7)?;
            let assigned: String = r.get(8)?;
            let cust: String = r.get(9)?;
            let time_spent: f64 = r.get(10)?;
            let activities: String = r.get(11)?;
            let my_act: String = r.get(12)?;
            let deadline: String = r.get(13)?;
            let sla_status: String = r.get(14)?;
            let sla_deadline: String = r.get(15)?;
            let rating: String = r.get(16)?;
            let kanban: String = r.get(17)?;
            let stage: String = r.get(18)?;
            let status: String = r.get(19)?;

            map.insert("employee_code".to_string(), json!(emp_code));
            map.insert("subject".to_string(), json!(subj));
            if !email.is_empty() {
                map.insert("email_address".to_string(), json!(email));
            }
            if !cat.is_empty() {
                map.insert("category".to_string(), json!(cat));
            }
            if !desc.is_empty() {
                map.insert("problem_description".to_string(), json!(desc));
            }
            if !prio.is_empty() {
                map.insert("priority".to_string(), json!(prio));
            }
            if !assigned.is_empty() {
                map.insert("assigned_to".to_string(), json!(assigned));
            }
            if !cust.is_empty() {
                map.insert("customer".to_string(), json!(cust));
            }
            if !status.is_empty() {
                map.insert("status".to_string(), json!(status));
            }
            if !stage.is_empty() {
                map.insert("stage".to_string(), json!(stage));
            }
            if !kanban.is_empty() {
                map.insert("kanban_state".to_string(), json!(kanban));
            }
            if !att_name.is_empty() {
                map.insert("attachment_name".to_string(), json!(att_name));
            }
            if !att_url.is_empty() {
                map.insert("attachment_url".to_string(), json!(att_url));
            }
            if time_spent > 0.0 {
                map.insert("time_spent".to_string(), json!(time_spent));
            }
            if !activities.is_empty() {
                map.insert("activities".to_string(), json!(activities));
                map.insert("activities_text".to_string(), json!(activities));
            }
            if !my_act.is_empty() {
                map.insert("my_activity".to_string(), json!(my_act));
            }
            if !deadline.is_empty() {
                map.insert("deadline".to_string(), json!(deadline));
            }
            if !sla_status.is_empty() {
                map.insert("sla_status".to_string(), json!(sla_status));
            }
            if !sla_deadline.is_empty() {
                map.insert("sla_deadline".to_string(), json!(sla_deadline));
            }
            if !rating.is_empty() {
                map.insert("rating_avg_text".to_string(), json!(rating));
            }

            Ok(Value::Object(map))
        })
        .optional()
        .ok()
        .flatten()?;

    Some(value)
}

/// Inserts a server ticket row into the local cache and returns the new
/// local id.
pub fn insert_server_ticket(t: &crate::api::ServerTicket) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        INSERT INTO tickets (
            employee_code, email_address, category, subject, problem_description,
            attachment_name, attachment_url, priority, assigned_to, customer,
            time_spent, activities, my_activity, deadline, sla_status, sla_deadline,
            rating_avg_text, kanban_state, stage, status, created_on, last_updated_on,
            server_ticket_id, server_last_updated
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15,
            ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24
        )
        "#,
        params![
            t.employee_code,
            t.email_address,
            t.category,
            t.subject,
            t.problem_description,
            t.attachment_name,
            t.attachment_url,
            t.priority,
            t.assigned_to,
            t.customer,
            t.time_spent,
            t.activities,
            t.my_activity,
            t.deadline,
            t.sla_status,
            t.sla_deadline,
            t.rating_avg_text,
            t.kanban_state,
            t.stage,
            t.status,
            t.created_on,
            t.last_updated_on,
            t.ticket_id,
            t.last_updated_on,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Overwrites a mapped local row with the server version (only called
/// when the local row has no pending operations).
pub fn apply_server_ticket(local_ticket_id: i64, t: &crate::api::ServerTicket) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        UPDATE tickets SET
            employee_code = ?1, email_address = ?2, category = ?3, subject = ?4,
            problem_description = ?5, attachment_name = ?6, attachment_url = ?7,
            priority = ?8, assigned_to = ?9, customer = ?10, time_spent = ?11,
            activities = ?12, my_activity = ?13, deadline = ?14, sla_status = ?15,
            sla_deadline = ?16, rating_avg_text = ?17, kanban_state = ?18,
            stage = ?19, status = ?20, created_on = ?21, last_updated_on = ?22,
            server_ticket_id = ?23, server_last_updated = ?24
        WHERE ticket_id = ?25
        "#,
        params![
            t.employee_code,
            t.email_address,
            t.category,
            t.subject,
            t.problem_description,
            t.attachment_name,
            t.attachment_url,
            t.priority,
            t.assigned_to,
            t.customer,
            t.time_spent,
            t.activities,
            t.my_activity,
            t.deadline,
            t.sla_status,
            t.sla_deadline,
            t.rating_avg_text,
            t.kanban_state,
            t.stage,
            t.status,
            t.created_on,
            t.last_updated_on,
            t.ticket_id,
            t.last_updated_on,
            local_ticket_id,
        ],
    )?;
    Ok(())
}

// ---- conflicts ----

/// Records an unresolved local/server divergence. At most one unresolved
/// conflict per local ticket is kept (re-detections are no-ops).
#[allow(clippy::too_many_arguments)]
pub fn record_conflict(
    local_ticket_id: i64,
    server_ticket_id: Option<&str>,
    op_kind: &str,
    local_payload: Option<&Value>,
    server_payload: Option<&Value>,
    detected_at: &str,
) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO sync_conflicts \
         (local_ticket_id, server_ticket_id, op_kind, local_payload, server_payload, detected_at, status) \
         SELECT ?1, ?2, ?3, ?4, ?5, ?6, 'unresolved' \
         WHERE NOT EXISTS ( \
             SELECT 1 FROM sync_conflicts \
             WHERE local_ticket_id = ?1 AND status = 'unresolved')",
        params![
            local_ticket_id,
            server_ticket_id,
            op_kind,
            local_payload.map(|v| v.to_string()),
            server_payload.map(|v| v.to_string()),
            detected_at,
        ],
    )?;
    Ok(())
}

pub fn unresolved_conflict_count() -> i64 {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_conflicts WHERE status = 'unresolved'",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// True while a local/server divergence is still recorded for this
/// ticket. The pull phase uses this to force a reconciliation pass even
/// when `server_last_updated` matches (a successful local push refreshes
/// that timestamp, which would otherwise mask the remote half of the
/// conflict).
pub fn has_unresolved_conflict(local_ticket_id: i64) -> bool {
    let Ok(conn) = get_connection() else {
        return false;
    };
    conn.query_row(
        "SELECT COUNT(*) FROM sync_conflicts \
         WHERE local_ticket_id = ?1 AND status = 'unresolved'",
        params![local_ticket_id],
        |r| r.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Called after the server version of a ticket was applied locally with
/// no pending operations — both sides' changes are now on the server, so
/// the recorded divergence is settled.
pub fn resolve_conflicts_for_ticket(local_ticket_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_conflicts SET status = 'resolved', resolved_at = CURRENT_TIMESTAMP \
         WHERE local_ticket_id = ?1 AND status = 'unresolved'",
        params![local_ticket_id],
    )?;
    Ok(())
}

// ============================================================
// Reminder sync helpers
// ============================================================

pub struct MappedReminder {
    pub local_id: i64,
    pub server_id: String,
    pub server_last_updated: String,
}

pub fn mapped_reminders() -> Vec<MappedReminder> {
    let Ok(conn) = get_connection() else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare(
        "SELECT reminder_id, server_reminder_id, COALESCE(server_last_updated, '') \
         FROM reminders WHERE server_reminder_id IS NOT NULL AND server_reminder_id != ''",
    ) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |r| {
        Ok(MappedReminder {
            local_id: r.get(0)?,
            server_id: r.get(1)?,
            server_last_updated: r.get(2)?,
        })
    }) else {
        return Vec::new();
    };
    rows.filter_map(Result::ok).collect()
}

pub fn server_reminder_id(local_reminder_id: i64) -> Option<String> {
    let conn = get_connection().ok()?;
    conn.query_row(
        "SELECT server_reminder_id FROM reminders WHERE reminder_id = ?1",
        params![local_reminder_id],
        |r| r.get::<_, Option<String>>(0),
    )
    .optional()
    .ok()
    .flatten()
    .flatten()
}

pub fn set_reminder_mapping(
    local_reminder_id: i64,
    server_reminder_id: &str,
    server_last_updated: &str,
) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET server_reminder_id = ?1, server_last_updated = ?2 \
         WHERE reminder_id = ?3",
        params![server_reminder_id, server_last_updated, local_reminder_id],
    )?;
    conn.execute(
        "UPDATE sync_queue SET server_ticket_id = ?1 \
         WHERE entity_type = 'reminder' AND local_ticket_id = ?2 AND (server_ticket_id IS NULL OR server_ticket_id = '')",
        params![server_reminder_id, local_reminder_id],
    )?;
    Ok(())
}

pub fn find_unmapped_or_duplicate_reminder(r: &crate::api::ServerReminder) -> Option<i64> {
    let conn = get_connection().ok()?;
    // 1. Direct match by server_reminder_id
    let srv_id = r.reminder_id.trim();
    if !srv_id.is_empty() {
        if let Ok(id) = conn.query_row(
            "SELECT reminder_id FROM reminders WHERE server_reminder_id = ?1 LIMIT 1",
            params![srv_id],
            |row| row.get::<_, i64>(0),
        ) {
            return Some(id);
        }
    }

    // 2. Data match: compare employee_code and reminder_message on unmapped row
    let clean_code = r.employee_code.trim();
    let clean_msg = r.reminder_message.trim();
    if !clean_code.is_empty() && !clean_msg.is_empty() {
        if let Ok(id) = conn.query_row(
            "SELECT reminder_id FROM reminders \
             WHERE (server_reminder_id IS NULL OR server_reminder_id = '') \
               AND TRIM(employee_code) = ?1 AND TRIM(reminder_message) = ?2 \
             ORDER BY reminder_id ASC LIMIT 1",
            params![clean_code, clean_msg],
            |row| row.get::<_, i64>(0),
        ) {
            return Some(id);
        }
    }

    None
}

/// Deduplicates reminders table in SQLite by comparing data.
/// Merges redundant duplicate rows so only one canonical row and card remains.
pub fn deduplicate_reminders_db() -> Result<usize> {
    let mut conn = get_connection()?;
    let tx = conn.transaction()?;
    let mut deleted_count = 0usize;

    // 1. Merge unmapped rows into matching server-mapped rows
    // e.g. Row B has server_reminder_id == '' and matches Row A which has server_reminder_id != ''
    {
        let mut stmt = tx.prepare(
            "SELECT b.reminder_id, a.reminder_id, a.server_reminder_id, a.server_last_updated, a.email_sent_at, a.status \
             FROM reminders b \
             JOIN reminders a ON TRIM(b.employee_code) = TRIM(a.employee_code) \
                             AND TRIM(b.reminder_message) = TRIM(a.reminder_message) \
                             AND b.reminder_id != a.reminder_id \
             WHERE (b.server_reminder_id IS NULL OR b.server_reminder_id = '') \
               AND (a.server_reminder_id IS NOT NULL AND a.server_reminder_id != '')"
        )?;

        let pairs: Vec<(i64, i64, String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            })?
            .filter_map(Result::ok)
            .collect();

        for (unmapped_id, mapped_id, srv_id, srv_updated, email_sent, status) in pairs {
            // Update unmapped row with server fields
            tx.execute(
                "UPDATE reminders SET server_reminder_id = ?1, server_last_updated = ?2, \
                 email_sent_at = ?3, status = ?4, last_updated_at = CURRENT_TIMESTAMP \
                 WHERE reminder_id = ?5",
                params![srv_id, srv_updated, email_sent, status, unmapped_id],
            )?;
            // Point any queued ops for mapped_id to unmapped_id
            tx.execute(
                "UPDATE sync_queue SET local_ticket_id = ?1 WHERE entity_type = 'reminder' AND local_ticket_id = ?2",
                params![unmapped_id, mapped_id],
            )?;
            // Delete the duplicate mapped row
            tx.execute("DELETE FROM reminders WHERE reminder_id = ?1", params![mapped_id])?;
            deleted_count += 1;
        }
    }

    // 2. Remove identical duplicates with same non-empty server_reminder_id
    {
        let mut stmt = tx.prepare(
            "SELECT reminder_id FROM reminders r1 \
             WHERE server_reminder_id IS NOT NULL AND server_reminder_id != '' \
               AND reminder_id > (SELECT MIN(r2.reminder_id) FROM reminders r2 WHERE r2.server_reminder_id = r1.server_reminder_id)"
        )?;
        let duplicate_ids: Vec<i64> = stmt
            .query_map([], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();

        for dup_id in duplicate_ids {
            tx.execute("DELETE FROM reminders WHERE reminder_id = ?1", params![dup_id])?;
            deleted_count += 1;
        }
    }

    tx.commit()?;
    Ok(deleted_count)
}

pub fn is_reminder_completed(reminder_id: i64) -> bool {
    let Ok(conn) = get_connection() else {
        return false;
    };
    conn.query_row(
        "SELECT status FROM reminders WHERE reminder_id = ?1",
        params![reminder_id],
        |r| r.get::<_, String>(0),
    )
    .map(|s| s.eq_ignore_ascii_case("completed"))
    .unwrap_or(false)
}

pub fn update_reminder_ticket_id(reminder_id: i64, ticket_id: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET ticket_id = ?1, last_updated_at = CURRENT_TIMESTAMP WHERE reminder_id = ?2",
        params![ticket_id, reminder_id],
    )?;
    Ok(())
}

pub fn cancel_queued_updates_for_reminder(local_reminder_id: i64) -> Result<usize> {
    let conn = get_connection()?;
    let count = conn.execute(
        "UPDATE sync_queue SET status = 'superseded', next_attempt_at = NULL, \
         last_error = 'Reminder already completed on server' \
         WHERE entity_type = 'reminder' AND local_ticket_id = ?1 AND op_type = 'UPDATE' \
           AND status IN ('pending', 'in_flight', 'failed')",
        params![local_reminder_id],
    )?;
    Ok(count)
}

pub fn mark_reminder_server_not_found(local_reminder_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET server_reminder_id = '', last_updated_at = CURRENT_TIMESTAMP \
         WHERE reminder_id = ?1",
        params![local_reminder_id],
    )?;
    conn.execute(
        "UPDATE sync_queue SET status = 'superseded', next_attempt_at = NULL, \
         last_error = 'Reminder not found on server' \
         WHERE entity_type = 'reminder' AND local_ticket_id = ?1 \
           AND status IN ('pending', 'in_flight', 'failed')",
        params![local_reminder_id],
    )?;
    Ok(())
}

pub fn set_reminder_sent(reminder_id: i64, sent_at: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET status = 'Sent', email_sent_at = ?1, last_updated_at = CURRENT_TIMESTAMP \
         WHERE reminder_id = ?2",
        params![sent_at, reminder_id],
    )?;
    Ok(())
}

pub fn set_server_reminder_sent(server_reminder_id: &str, sent_at: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE reminders SET status = 'Sent', email_sent_at = ?1, last_updated_at = CURRENT_TIMESTAMP \
         WHERE server_reminder_id = ?2",
        params![sent_at, server_reminder_id],
    )?;
    Ok(())
}

pub fn insert_server_reminder(r: &crate::api::ServerReminder) -> Result<i64> {
    let conn = get_connection()?;
    let normalized_send_at = crate::state::to_ist_iso8601(&r.send_at);
    conn.execute(
        r#"
        INSERT INTO reminders (
            employee_code, employee_name, email_address, service_category,
            reminder_message, send_at, status, email_sent_at, ticket_id,
            created_at, last_updated_at, server_reminder_id, server_last_updated,
            cc_emails, repeat_frequency
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
        )
        "#,
        params![
            r.employee_code,
            r.employee_name,
            r.email_address,
            r.service_category,
            r.reminder_message,
            normalized_send_at,
            r.status,
            r.email_sent_at,
            r.ticket_id,
            r.created_at,
            r.last_updated_at,
            r.reminder_id,
            r.last_updated_at,
            r.cc_emails,
            r.repeat_frequency,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn apply_server_reminder(local_reminder_id: i64, r: &crate::api::ServerReminder) -> Result<()> {
    let conn = get_connection()?;
    let normalized_send_at = crate::state::to_ist_iso8601(&r.send_at);
    conn.execute(
        r#"
        UPDATE reminders SET
            employee_code = ?1, employee_name = ?2, email_address = ?3,
            service_category = ?4, reminder_message = ?5, send_at = ?6,
            status = ?7, email_sent_at = ?8, ticket_id = ?9,
            created_at = ?10, last_updated_at = ?11,
            server_reminder_id = ?12, server_last_updated = ?13,
            cc_emails = ?14, repeat_frequency = ?15
        WHERE reminder_id = ?16
        "#,
        params![
            r.employee_code,
            r.employee_name,
            r.email_address,
            r.service_category,
            r.reminder_message,
            normalized_send_at,
            r.status,
            r.email_sent_at,
            r.ticket_id,
            r.created_at,
            r.last_updated_at,
            r.reminder_id,
            r.last_updated_at,
            r.cc_emails,
            r.repeat_frequency,
            local_reminder_id,
        ],
    )?;
    Ok(())
}

/// Builds the `reminders.create` payload from a local row.
///
/// Fields the documented API accepts:
/// `employee_code`, `service_category`, `reminder_message`, `send_at`,
/// `repeat_frequency` (required/default) plus `cc_emails` when the row carries one.
pub fn reminder_create_payload(local_reminder_id: i64) -> Option<Value> {
    let conn = get_connection().ok()?;
    let mut stmt = conn
        .prepare(
            "SELECT employee_code, service_category, reminder_message, send_at, \
                    COALESCE(cc_emails, ''), COALESCE(repeat_frequency, 'daily') \
             FROM reminders WHERE reminder_id = ?1",
        )
        .ok()?;

    let value = stmt
        .query_row(params![local_reminder_id], |r| {
            let emp_code: String = r.get(0)?;
            let cat: String = r.get(1)?;
            let msg: String = r.get(2)?;
            let send_at: String = r.get(3)?;
            let cc: String = r.get(4)?;
            let freq: String = r.get(5)?;
            let send_at_iso = crate::sync::parse_to_iso8601(&send_at);

            let mut map = serde_json::Map::new();
            map.insert("employee_code".to_string(), json!(emp_code));
            map.insert("service_category".to_string(), json!(cat));
            map.insert("reminder_message".to_string(), json!(msg));
            map.insert("send_at".to_string(), json!(send_at_iso));
            map.insert("repeat_frequency".to_string(), json!(freq));
            if !cc.trim().is_empty() {
                map.insert("cc_emails".to_string(), json!(cc));
            }

            Ok(Value::Object(map))
        })
        .optional()
        .ok()
        .flatten()?;

    Some(value)
}

pub fn enqueue_reminder_create(
    local_reminder_id: i64,
    payload: &Value,
    created_at: &str,
) -> Result<i64> {
    let conn = get_connection()?;
    insert_entity_op(
        &conn,
        "reminder",
        local_reminder_id,
        None,
        "CREATE",
        payload,
        created_at,
    )
}

pub fn enqueue_reminder_update(
    local_reminder_id: i64,
    server_reminder_id: Option<String>,
    fields: &Value,
    created_at: &str,
) -> Result<i64> {
    let conn = get_connection()?;
    insert_entity_op(
        &conn,
        "reminder",
        local_reminder_id,
        server_reminder_id.as_deref(),
        "UPDATE",
        fields,
        created_at,
    )
}

pub fn enqueue_reminder_send_now(
    local_reminder_id: i64,
    server_reminder_id: &str,
    force: bool,
    created_at: &str,
) -> Result<i64> {
    let conn = get_connection()?;
    let payload = json!({ "force": force });
    insert_entity_op(
        &conn,
        "reminder",
        local_reminder_id,
        Some(server_reminder_id),
        "SEND_NOW",
        &payload,
        created_at,
    )
}

pub fn enqueue_reminder_delete(
    local_reminder_id: i64,
    server_reminder_id: Option<String>,
    created_at: &str,
) -> Result<i64> {
    let conn = get_connection()?;
    insert_entity_op(
        &conn,
        "reminder",
        local_reminder_id,
        server_reminder_id.as_deref(),
        "DELETE",
        &json!({}),
        created_at,
    )
}

// ============================================================
// Employee DB helpers (for local cache modifications)
// ============================================================

pub fn create_employee_db(emp: &crate::api::Employee) -> Result<i64> {
    let mut normalized = emp.clone();
    crate::api::normalize_employee(&mut normalized);
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO USERS (emp_code, emp_name, emp_email, department, division, active, is_deleted) \
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 0)",
        params![
            normalized.employee_code.trim(),
            normalized.employee_name.trim(),
            if normalized.email.trim().is_empty() || !normalized.email.contains('@') {
                None
            } else {
                Some(normalized.email.trim().to_string())
            },
            normalized.department.trim(),
            if normalized.division.trim().is_empty() {
                None
            } else {
                Some(normalized.division.trim().to_string())
            },
        ],
    )?;
    invalidate_employee_lookup_cache();
    Ok(conn.last_insert_rowid())
}

pub fn update_employee_db(
    emp_code: &str,
    emp_name: &str,
    emp_email: &str,
    department: &str,
    division: &str,
) -> Result<()> {
    let email_trimmed = emp_email.trim();
    let email = if email_trimmed.is_empty() || !email_trimmed.contains('@') {
        None
    } else {
        Some(email_trimmed)
    };
    let div_trimmed = division.trim();
    let div = if div_trimmed.is_empty() {
        None
    } else {
        Some(div_trimmed)
    };
    let conn = get_connection()?;
    conn.execute(
        "UPDATE USERS SET emp_name = ?1, emp_email = ?2, department = ?3, division = ?4 \
         WHERE emp_code = ?5 AND is_deleted = 0",
        params![
            emp_name.trim(),
            email,
            department.trim(),
            div,
            emp_code.trim(),
        ],
    )?;
    invalidate_employee_lookup_cache();
    Ok(())
}

pub fn delete_employee_db(emp_code: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE USERS SET is_deleted = 1 WHERE emp_code = ?1",
        params![emp_code.trim()],
    )?;
    invalidate_employee_lookup_cache();
    Ok(())
}

/// Compacts and optimizes the SQLite database file on disk.
pub fn optimize_database() -> Result<()> {
    let conn = get_connection()?;
    conn.execute_batch("VACUUM; PRAGMA optimize;")?;
    Ok(())
}

/// Purges resolved conflicts from the sync_conflicts table.
pub fn purge_resolved_conflicts() -> Result<usize> {
    let conn = get_connection()?;
    let affected = conn.execute("DELETE FROM sync_conflicts WHERE resolved_at IS NOT NULL", [])?;
    Ok(affected)
}

/// Returns the file size of the active database in bytes.
pub fn get_database_file_size() -> u64 {
    std::fs::metadata(db_path()).map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheet_sources_no_recursion() {
        let sources = get_all_sheet_sources();
        assert!(!sources.is_empty());
        let active_id = get_active_sheet_source_id();
        assert!(!active_id.is_empty());
        let name = get_sheet_profile_name(&sources[0].id);
        assert!(!name.is_empty());
        let (url, _) = get_sheet_profile_config(&sources[0].id);
        assert!(!url.is_empty());
    }
}
