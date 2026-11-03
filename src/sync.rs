//! Background synchronization: Google Sheets API (source of truth)
//! ↔ SQLite cache + persistent operation queue.
//!
//! Design:
//! * **Single worker thread**, independent of the Dioxus UI lifecycle —
//!   one cycle at a time, never overlapping.
//! * Wakes **immediately** after local changes ([`kick`]), otherwise runs
//!   every [`CYCLE_SECS`] seconds (20 s). Between cycles the thread
//!   sleeps on a condvar — no CPU usage.
//! * **Push phase**: executes queued operations oldest-first, coalescing
//!   duplicate ops per ticket into a single API request.
//! * **Pull phase**: fetches `tickets.list`, validates rows, diffs them
//!   against SQLite (insert / apply / delete) and records conflicts when
//!   the server changed while local operations are still pending.
//! * Retryable failures use exponential backoff with jitter and honor
//!   `Retry-After`; non-retryable failures are retained as `failed` (never
//!   discarded) and re-checked slowly.
//! * The UI subscribes with [`subscribe_status`] and is woken only when
//!   status actually changes (no polling).
//!
//! Platform note: this is best-effort background execution. The OS may
//! suspend the process (suspend-to-RAM, etc.) — sync resumes on the next
//! wake-up; no guaranteed two-minute cadence is claimed.

use futures_channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Condvar, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::api::{ApiClient, ApiConfig, ApiError, ServerReminder, ServerTicket};
use crate::db::{self, SyncOp};

/// Periodic sync interval (seconds). Reduced to 20s for high-responsiveness.
pub const CYCLE_SECS: u64 = 20;
/// Debounce window: wait 60ms after a local change before updating all data in Google Sheets.
pub const DEBOUNCE_MS: u64 = 60;
/// Minimum gap between two pull phases (local edits trigger extra cycles;
/// pulls are rate-limited so rapid edits don't spam `tickets.list`).
const MIN_PULL_GAP_SECS: i64 = 5;
/// Maximum operations executed per cycle.
const OPS_PER_CYCLE: i64 = 50;
/// `tickets.list` page size (API max is 200).
const PAGE_LIMIT: u64 = 200;
/// Safety bound on pagination.
const MAX_PAGES: u64 = 50;

/// Backoff: base * 2^retry (full jitter in the upper half), capped.
const BACKOFF_BASE_SECS: u64 = 5;
const BACKOFF_CAP_SECS: u64 = 900;
/// Non-retryable failures are re-checked every 10 minutes (retained, never
/// discarded — this lets a fixed API key or employee code self-heal).
const NON_RETRYABLE_DELAY_SECS: u64 = 600;

// ============================================================
// Status shared with the UI
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SyncState {
    /// API not configured (missing URL/key).
    Disabled,
    /// No problems; may still have queued changes waiting for backoff.
    Idle,
    /// A cycle is running right now.
    Syncing,
    /// The last cycle (or an operation) failed.
    Error,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SyncStatus {
    pub state: SyncState,
    pub detail: String,
    /// Operations not yet confirmed by the API.
    pub pending_ops: i64,
    /// Unresolved local/server conflicts.
    pub conflicts: i64,
    pub last_success_at: Option<String>,
    /// Bumped whenever a pull changed local ticket data — the UI uses this
    /// to bump `AppDbRevision` and reload from SQLite.
    pub revision: u64,
}

impl Default for SyncStatus {
    fn default() -> Self {
        SyncStatus {
            state: SyncState::Disabled,
            detail: "Starting sync service…".to_string(),
            pending_ops: 0,
            conflicts: 0,
            last_success_at: None,
            revision: 0,
        }
    }
}

static STATUS: Mutex<Option<SyncStatus>> = Mutex::new(None);
static SUBSCRIBERS: Mutex<Vec<UnboundedSender<SyncStatus>>> = Mutex::new(Vec::new());

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // A panic elsewhere must not permanently disable sync.
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn with_status<R>(f: impl FnOnce(&mut SyncStatus) -> R) -> R {
    let result = {
        let mut guard = lock(&STATUS);
        let status = guard.get_or_insert_with(SyncStatus::default);
        f(status)
    };
    notify_subscribers();
    result
}

fn notify_subscribers() {
    let snapshot = snapshot();
    let mut subs = lock(&SUBSCRIBERS);
    subs.retain(|tx| tx.unbounded_send(snapshot.clone()).is_ok());
}

/// Current status snapshot (cheap clone, safe from any thread).
pub fn snapshot() -> SyncStatus {
    lock(&STATUS).clone().unwrap_or_default()
}

/// Subscribe to status changes. The receiver immediately yields the
/// current snapshot and then one message per change — the UI task simply
/// awaits it (no timers, no polling).
pub fn subscribe_status() -> UnboundedReceiver<SyncStatus> {
    let (tx, rx) = unbounded();
    let _ = tx.unbounded_send(snapshot());
    lock(&SUBSCRIBERS).push(tx);
    rx
}

// ============================================================
// Worker thread
// ============================================================

static WAKE: Mutex<bool> = Mutex::new(false);
static WAKE_CV: Condvar = Condvar::new();
static STARTED: OnceLock<()> = OnceLock::new();
static CLIENT: OnceLock<ApiClient> = OnceLock::new();

/// Wake the worker for an immediate sync cycle (called after every local
/// ticket mutation). Safe from any thread; coalesces — multiple kicks
/// before the worker wakes result in a single extra cycle.
pub fn kick() {
    let mut wake = lock(&WAKE);
    *wake = true;
    WAKE_CV.notify_one();
}

/// Start the single background worker (idempotent). Call once at startup,
/// before the UI launches — sync must not depend on any screen.
pub fn start_worker() {
    if STARTED.set(()).is_err() {
        return;
    }
    match std::thread::Builder::new()
        .name("tms-sync".to_string())
        .spawn(worker_loop)
    {
        Ok(_) => {}
        Err(e) => {
            with_status(|s| {
                s.state = SyncState::Error;
                s.detail = format!("Failed to start sync thread: {e}");
            });
        }
    }
}

fn worker_loop() {
    let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => {
            with_status(|s| {
                s.state = SyncState::Error;
                s.detail = format!("Failed to start sync runtime: {e}");
            });
            return;
        }
    };

    // Immediate startup cycle: loads queued ops from previous runs and
    // pulls server changes as soon as the app opens.
    rt.block_on(run_cycle());

    let mut deadline = Instant::now() + Duration::from_secs(CYCLE_SECS);
    loop {
        let was_kicked = {
            let mut wake = lock(&WAKE);
            while !*wake {
                let now = Instant::now();
                if now >= deadline {
                    break;
                }
                let (guard, _) = WAKE_CV
                    .wait_timeout(wake, deadline - now)
                    .unwrap_or_else(|e| e.into_inner());
                wake = guard;
            }
            let kicked = *wake;
            *wake = false;
            kicked
        };

        // When a change is made in the app, wait 60ms to coalesce and then update all data in Google Sheets
        if was_kicked {
            let debounce = Duration::from_millis(DEBOUNCE_MS);
            let mut debounce_deadline = Instant::now() + debounce;
            loop {
                let mut wake = lock(&WAKE);
                let now = Instant::now();
                if now >= debounce_deadline {
                    *wake = false;
                    break;
                }
                let (guard, timeout_result) = WAKE_CV
                    .wait_timeout(wake, debounce_deadline - now)
                    .unwrap_or_else(|e| e.into_inner());
                wake = guard;
                if timeout_result.timed_out() {
                    *wake = false;
                    break;
                }
                if *wake {
                    *wake = false;
                    debounce_deadline = Instant::now() + debounce;
                }
            }
        }

        deadline = Instant::now() + Duration::from_secs(CYCLE_SECS);
        rt.block_on(run_cycle());
    }
}

/// Runs one synchronous cycle (used by `--sync-once` and the test
/// harness; the normal path is the worker thread).
pub fn run_cycle_blocking() {
    let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() else {
        return;
    };
    rt.block_on(run_cycle());
}

pub fn get_client() -> Result<&'static ApiClient, String> {
    ensure_client()
}

fn ensure_client() -> Result<&'static ApiClient, String> {
    if let Some(client) = CLIENT.get() {
        return Ok(client);
    }
    // Configuration problems are retried next cycle (e.g. the collection
    // file appearing later); successes are cached forever.
    let cfg = ApiConfig::load()?;
    let client = ApiClient::new(cfg)?;
    Ok(CLIENT.get_or_init(|| client))
}

pub fn now_epoch() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

// ============================================================
// One sync cycle
// ============================================================

pub async fn run_cycle() {
    let client = match ensure_client() {
        Ok(c) => c,
        Err(reason) => {
            with_status(|s| {
                s.state = SyncState::Disabled;
                s.detail = reason;
                s.pending_ops = db::pending_op_count();
                s.conflicts = db::unresolved_conflict_count();
            });
            return;
        }
    };

    with_status(|s| {
        s.state = SyncState::Syncing;
        s.detail = "Syncing with Google Sheets…".to_string();
    });

    let mut cycle_error: Option<String> = None;

    // ---- Push: confirm local changes with the API ----
    let now = now_epoch();
    loop {
        let due = db::due_ops(now, OPS_PER_CYCLE).unwrap_or_default();
        if due.is_empty() {
            break;
        }
        let merged = merge_due_ops(due);
        if merged.is_empty() {
            break;
        }
        for op in merged {
            let _ = db::mark_op_in_flight(op.op_id);
            match execute_op(client, &op).await {
                Ok(ExecOutcome::Synced) => {
                    let _ = db::mark_op_synced(op.op_id);
                }
                Ok(ExecOutcome::Superseded) => {
                    let _ = db::mark_op_superseded(op.op_id);
                }
                Err(e) => {
                    let is_schema_mismatch = matches!(&e, ApiError::Api { code, .. } if code == "SCHEMA_MISMATCH");
                    if !is_schema_mismatch || op.retry_count == 0 {
                        eprintln!("[sync execute_op ERROR] op={:?} entity={:?} err={}", op.op_type, op.entity_type, e);
                    }
                    let next = next_attempt_at(&op, &e);
                    let _ = db::mark_op_error(op.op_id, &e.short(), e.is_retryable(), next);
                    // Don't let a reminder schema error block ticket syncing in the same cycle
                    if op.entity_type != "reminder" {
                        cycle_error.get_or_insert_with(|| e.short());
                    }
                }
            }
        }
        if cycle_error.is_some() {
            break;
        }
    }

    // ---- Pull: refresh SQLite from the API (source of truth) ----
    let mut pull_error: Option<String> = None;
    let last_pull: i64 = db::sync_meta_get("last_pull_at")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if now.saturating_sub(last_pull) >= MIN_PULL_GAP_SECS {
        match pull(client).await {
            Ok(changed) => {
                let _ = db::sync_meta_set("last_pull_at", &now.to_string());
                if changed > 0 {
                    with_status(|s| s.revision = s.revision.wrapping_add(1));
                }
                with_status(|s| s.last_success_at = Some(crate::state::format_utc(now)));
            }
            Err(e) => pull_error = Some(e.short()),
        }
    }

    // ---- Employee directory: real-time sync with Google Sheets ----
    const EMPLOYEE_PULL_GAP_SECS: i64 = 10;
    let last_emp_pull: i64 = db::sync_meta_get("last_employee_pull_at")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if now.saturating_sub(last_emp_pull) >= EMPLOYEE_PULL_GAP_SECS {
        match pull_employees(client).await {
            Ok(count) => {
                let _ = db::sync_meta_set("last_employee_pull_at", &now.to_string());
                if count > 0 {
                    // Bump the revision so the Employees page and selectors reload.
                    with_status(|s| s.revision = s.revision.wrapping_add(1));
                }
            }
            // Directory failure never fails the cycle — tickets synced.
            Err(e) => {
                let backoff = if e.is_retryable() { 10 } else { 60 };
                let next_attempt = now - EMPLOYEE_PULL_GAP_SECS + backoff;
                let _ = db::sync_meta_set("last_employee_pull_at", &next_attempt.to_string());
                eprintln!("[sync] employee pull failed: {}", e.short());
            }
        }
    }

    // ---- Reminders: pulled every 60s ----
    const REMINDER_PULL_GAP_SECS: i64 = 60;
    let last_rem_pull: i64 = db::sync_meta_get("last_reminder_pull_at")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    if now.saturating_sub(last_rem_pull) >= REMINDER_PULL_GAP_SECS {
        match pull_reminders(client).await {
            Ok(count) => {
                let _ = db::sync_meta_set("last_reminder_pull_at", &now.to_string());
                if count > 0 {
                    with_status(|s| s.revision = s.revision.wrapping_add(1));
                }
            }
            Err(e) => {
                let backoff = if e.is_retryable() { 60 } else { 300 };
                let next_attempt = now - REMINDER_PULL_GAP_SECS + backoff;
                let _ = db::sync_meta_set("last_reminder_pull_at", &next_attempt.to_string());
                eprintln!("[sync] reminder pull failed: {}", e.short());
            }
        }
    }

    // ---- Final status ----
    let pending = db::pending_op_count();
    let conflicts = db::unresolved_conflict_count();
    let op_error = db::latest_active_error();
    with_status(|s| {
        s.pending_ops = pending;
        s.conflicts = conflicts;
        if let Some(err) = pull_error.or(cycle_error).or(op_error) {
            s.state = SyncState::Error;
            s.detail = if pending > 0 {
                format!("Sync issue · {pending} pending · {err}")
            } else {
                format!("Sync issue · {err}")
            };
        } else if pending > 0 {
            s.state = SyncState::Idle;
            s.detail = format!("{pending} change(s) waiting to sync");
        } else {
            s.state = SyncState::Idle;
            s.detail = format!(
                "Up to date · {}",
                s.last_success_at
                    .clone()
                    .unwrap_or_else(|| crate::state::format_utc(now))
            );
        }
    });
}

// ============================================================
// Push phase
// ============================================================

enum ExecOutcome {
    /// Server confirmed (or the operation is n/a server-side).
    Synced,
    /// Operation is no longer meaningful (row gone / folded elsewhere).
    Superseded,
}

/// Collapses multiple due operations for the same ticket into one so a
/// cycle never sends duplicate or conflicting requests for a single row.
fn merge_due_ops(ops: Vec<SyncOp>) -> Vec<SyncOp> {
    let mut merged: Vec<SyncOp> = Vec::new();
    for op in ops {
        let Some(pos) = merged.iter().position(|m| m.local_ticket_id == op.local_ticket_id) else {
            merged.push(op);
            continue;
        };
        let existing = &mut merged[pos];
        match (existing.op_type.as_str(), op.op_type.as_str()) {
            // A queued delete wins over anything else for the same row.
            (_, "DELETE") => {
                let _ = db::mark_op_superseded(existing.op_id);
                *existing = op;
            }
            ("DELETE", _) => {
                let _ = db::mark_op_superseded(op.op_id);
            }
            // CREATE absorbs UPDATE field patches; UPDATEs merge into one.
            ("CREATE", "UPDATE") | ("UPDATE", "UPDATE") | ("CREATE", "CREATE") => {
                let _ = db::merge_op_payload(existing.op_id, &op.payload);
                let _ = db::mark_op_superseded(op.op_id);
            }
            // Defensive: a CREATE after an UPDATE for one row can't happen
            // through the app — keep the first op.
            ("UPDATE", "CREATE") => {
                let _ = db::mark_op_superseded(op.op_id);
            }
            // Any other combination: keep the existing op, retain the new
            // one as superseded rather than dropping data silently.
            _ => {
                let _ = db::mark_op_superseded(op.op_id);
            }
        }
    }
    merged
}

/// True when the previous failure might have reached the server anyway
/// (timeout / transport / malformed response) — a CREATE is then verified
/// by subject+description before being retried, preventing duplicates.
fn ack_maybe_lost(last_error: &str) -> bool {
    last_error.starts_with("TIMEOUT")
        || last_error.starts_with("NETWORK")
        || last_error.starts_with("DECODE")
        || last_error.starts_with("HTTP 5")
        // The create reached the server but recording the mapping failed
        // locally — verify before retrying, or we would duplicate it.
        || last_error.starts_with("DB")
}

async fn execute_op(client: &ApiClient, op: &SyncOp) -> Result<ExecOutcome, ApiError> {
    if op.entity_type == "reminder" {
        return execute_reminder_op(client, op).await;
    }

    match op.op_type.as_str() {
        "CREATE" => {
            if op.retry_count > 0
                && ack_maybe_lost(op.last_error.as_deref().unwrap_or_default())
            {
                if let Some(remote) = find_matching_ticket(client, &op.payload).await? {
                    db::set_ticket_mapping(op.local_ticket_id, &remote.ticket_id, &remote.last_updated_on)?;
                    return Ok(ExecOutcome::Synced);
                }
            }
            let sanitized = sanitize_ticket_payload(client, &op.payload);
            let remote = client.tickets_create(&sanitized).await?;
            db::set_ticket_mapping(op.local_ticket_id, &remote.ticket_id, &remote.last_updated_on)?;
            Ok(ExecOutcome::Synced)
        }

        "UPDATE" => {
            let server_id = op
                .server_ticket_id
                .clone()
                .or_else(|| db::server_ticket_id(op.local_ticket_id));

            let Some(server_id) = server_id else {
                // The ticket never reached the server — push the current
                // row as a create instead (edits to pre-sync tickets).
                let Some(row) = db::ticket_create_payload(op.local_ticket_id) else {
                    return Ok(ExecOutcome::Superseded); // row is gone
                };
                let sanitized = sanitize_ticket_payload(client, &row);
                let remote = client.tickets_create(&sanitized).await?;
                db::set_ticket_mapping(op.local_ticket_id, &remote.ticket_id, &remote.last_updated_on)?;
                return Ok(ExecOutcome::Synced);
            };

            let sanitized = sanitize_ticket_payload(client, &op.payload);
            match client.tickets_update(&server_id, &sanitized).await {
                Ok(remote) => {
                    db::set_server_last_updated(op.local_ticket_id, &remote.last_updated_on)?;
                    Ok(ExecOutcome::Synced)
                }
                Err(e) if e.is_not_found() => {
                    // The server lost the row while we had local changes:
                    // record the divergence, keep the op (retained as
                    // failed) — never silently recreate or drop it.
                    db::record_conflict(
                        op.local_ticket_id,
                        Some(&server_id),
                        "update_target_missing_on_server",
                        db::ticket_create_payload(op.local_ticket_id).as_ref(),
                        None,
                        &crate::state::format_utc(now_epoch()),
                    )?;
                    Err(e)
                }
                Err(e) => Err(e),
            }
        }

        "DELETE" => {
            let server_id = op
                .server_ticket_id
                .clone()
                .or_else(|| db::server_ticket_id(op.local_ticket_id));
            let Some(server_id) = server_id else {
                return Ok(ExecOutcome::Synced); // never reached the server
            };
            match client.tickets_delete(&server_id).await {
                Ok(()) => Ok(ExecOutcome::Synced),
                // Already gone — deleting twice is a success.
                Err(e) if e.is_not_found() => Ok(ExecOutcome::Synced),
                Err(e) => Err(e),
            }
        }

        other => {
            // Unknown op type — shouldn't exist; retain it visibly.
            Err(ApiError::Api {
                code: "UNKNOWN_OP".to_string(),
                message: format!("unsupported operation type {other}"),
                retryable: false,
                retry_after: None,
            })
        }
    }
}

/// Applies the configured employee override (the sheet requires
/// `employee_code` to reference an existing employee row).
fn with_employee_code(client: &ApiClient, payload: &Value) -> Value {
    let Some(code) = client.config().employee_code.as_deref() else {
        return payload.clone();
    };
    let mut out = payload.clone();
    if let Some(obj) = out.as_object_mut() {
        obj.insert("employee_code".to_string(), json!(code));
    }
    out
}

/// Maps internal Kanban status ("Pending", "InProgress", "Onhold", "Completed")
/// to Google Sheet's validated dropdown choices for Cell S ("Open", "In Progress", "On Hold", "Done").
pub fn to_sheet_kanban_state(val: &str) -> &'static str {
    match val {
        "Pending" | "Open" | "To Do" | "Todo" | "New" => "Open",
        "InProgress" | "In Progress" | "In-Progress" => "In Progress",
        "Onhold" | "On Hold" | "On-Hold" | "Hold" => "On Hold",
        "Completed" | "Done" | "Resolved" | "Closed" => "Done",
        other => match other {
            s if s.eq_ignore_ascii_case("pending") || s.eq_ignore_ascii_case("open") || s.eq_ignore_ascii_case("new") => "Open",
            s if s.eq_ignore_ascii_case("inprogress") || s.eq_ignore_ascii_case("in progress") => "In Progress",
            s if s.eq_ignore_ascii_case("onhold") || s.eq_ignore_ascii_case("on hold") => "On Hold",
            s if s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("done") || s.eq_ignore_ascii_case("resolved") || s.eq_ignore_ascii_case("closed") => "Done",
            _ => "Open",
        },
    }
}

/// Maps internal stage/status to Google Sheet's validated dropdown choices for Cell T ("New", "Assigned", "Investigation", "Resolution").
pub fn to_sheet_stage(val: &str) -> &'static str {
    match val {
        "Pending" | "Open" | "New" | "To Do" | "Todo" => "New",
        "Assigned" => "Assigned",
        "InProgress" | "In Progress" | "Investigation" => "Investigation",
        "Onhold" | "On Hold" | "Hold" => "Assigned",
        "Completed" | "Done" | "Resolution" | "Resolved" | "Closed" => "Resolution",
        other => match other {
            s if s.eq_ignore_ascii_case("pending") || s.eq_ignore_ascii_case("open") || s.eq_ignore_ascii_case("new") => "New",
            s if s.eq_ignore_ascii_case("assigned") => "Assigned",
            s if s.eq_ignore_ascii_case("inprogress") || s.eq_ignore_ascii_case("in progress") || s.eq_ignore_ascii_case("investigation") => "Investigation",
            s if s.eq_ignore_ascii_case("onhold") || s.eq_ignore_ascii_case("on hold") => "Assigned",
            s if s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("done") || s.eq_ignore_ascii_case("resolution") || s.eq_ignore_ascii_case("resolved") => "Resolution",
            _ => "New",
        },
    }
}

/// Maps internal status to Google Sheet's status column ("Open", "In Progress", "On Hold", "Closed").
pub fn to_sheet_status(val: &str) -> &'static str {
    match val {
        "Pending" | "Open" | "To Do" | "Todo" | "New" => "Open",
        "InProgress" | "In Progress" | "In-Progress" => "In Progress",
        "Onhold" | "On Hold" | "On-Hold" | "Hold" => "On Hold",
        "Completed" | "Done" | "Resolved" | "Closed" => "Closed",
        other => match other {
            s if s.eq_ignore_ascii_case("pending") || s.eq_ignore_ascii_case("open") || s.eq_ignore_ascii_case("new") => "Open",
            s if s.eq_ignore_ascii_case("inprogress") || s.eq_ignore_ascii_case("in progress") => "In Progress",
            s if s.eq_ignore_ascii_case("onhold") || s.eq_ignore_ascii_case("on hold") => "On Hold",
            s if s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("done") || s.eq_ignore_ascii_case("resolved") || s.eq_ignore_ascii_case("closed") => "Closed",
            _ => "Open",
        },
    }
}

/// Maps Google Sheet's validated dropdown choices ("Open", "In Progress", "On Hold", "Done", "New", "Assigned", "Investigation", "Resolution")
/// to internal Kanban column status ("Pending", "InProgress", "Onhold", "Completed").
pub fn from_sheet_status(val: &str) -> &'static str {
    match val {
        "Open" | "Pending" | "To Do" | "Todo" | "New" => "Pending",
        "In Progress" | "InProgress" | "In-Progress" | "Investigation" => "InProgress",
        "On Hold" | "Onhold" | "On-Hold" | "Hold" | "Assigned" => "Onhold",
        "Done" | "Completed" | "Resolved" | "Closed" | "Resolution" => "Completed",
        other => match other {
            s if s.eq_ignore_ascii_case("open") || s.eq_ignore_ascii_case("pending") || s.eq_ignore_ascii_case("new") => "Pending",
            s if s.eq_ignore_ascii_case("in progress") || s.eq_ignore_ascii_case("inprogress") || s.eq_ignore_ascii_case("investigation") => "InProgress",
            s if s.eq_ignore_ascii_case("on hold") || s.eq_ignore_ascii_case("onhold") || s.eq_ignore_ascii_case("assigned") => "Onhold",
            s if s.eq_ignore_ascii_case("done") || s.eq_ignore_ascii_case("completed") || s.eq_ignore_ascii_case("resolved") || s.eq_ignore_ascii_case("closed") || s.eq_ignore_ascii_case("resolution") => "Completed",
            _ => "Pending",
        },
    }
}

/// Sanitizes ticket payload before sending to Google Sheets:
/// - Transforms kanban_state, stage, and status to Google Sheet's allowed data validation values
/// - Sanitizes employee_code so names in parentheses (e.g. "11135 (It)") don't break employee reference lookups
/// - Remaps activities to activities_text for Google Sheet column naming
pub fn sanitize_ticket_payload(client: &ApiClient, payload: &Value) -> Value {
    let mut out = with_employee_code(client, payload);
    if let Some(obj) = out.as_object_mut() {
        if let Some(emp) = obj.get("employee_code").and_then(|v| v.as_str()) {
            let code = if let Some(idx) = emp.find('(') {
                emp[..idx].trim()
            } else {
                emp.trim()
            };
            obj.insert("employee_code".to_string(), json!(code));
        }
        if let Some(kanban) = obj.get("kanban_state").and_then(|v| v.as_str()) {
            obj.insert("kanban_state".to_string(), json!(to_sheet_kanban_state(kanban)));
        }
        if let Some(stage) = obj.get("stage").and_then(|v| v.as_str()) {
            obj.insert("stage".to_string(), json!(to_sheet_stage(stage)));
        }
        if let Some(status) = obj.get("status").and_then(|v| v.as_str()) {
            obj.insert("status".to_string(), json!(to_sheet_status(status)));
        }
        if let Some(act) = obj.remove("activities") {
            if !obj.contains_key("activities_text") {
                obj.insert("activities_text".to_string(), act);
            }
        }
    }
    out
}

/// Duplicate-CREATE guard: looks for an unclaimed server row with the
/// same subject + description (only used before retrying a CREATE whose
/// acknowledgment may have been lost).
async fn find_matching_ticket(
    client: &ApiClient,
    payload: &Value,
) -> Result<Option<ServerTicket>, ApiError> {
    let subject = payload.get("subject").and_then(|v| v.as_str()).unwrap_or("");
    let desc = payload
        .get("problem_description")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if subject.is_empty() {
        return Ok(None);
    }
    let mut offset = 0u64;
    for _ in 0..MAX_PAGES {
        let page = client.tickets_list(offset, PAGE_LIMIT).await?;
        let count = page.items.len() as u64;
        for t in page.items {
            if t.subject == subject && t.problem_description == desc {
                return Ok(Some(t));
            }
        }
        if !page.has_more || count == 0 {
            break;
        }
        offset += count;
    }
    Ok(None)
}

/// Exponential backoff with jitter for retryable failures; `Retry-After`
/// wins when present; non-retryable failures use a fixed slow re-check.
fn next_attempt_at(op: &SyncOp, err: &ApiError) -> i64 {
    let now = now_epoch();
    if let Some(ra) = err.retry_after() {
        return now + ra as i64;
    }
    if !err.is_retryable() {
        return now + NON_RETRYABLE_DELAY_SECS as i64;
    }
    let shift = op.retry_count.clamp(0, 8) as u32;
    let exp = BACKOFF_BASE_SECS
        .saturating_mul(1u64 << shift)
        .min(BACKOFF_CAP_SECS);
    // Full jitter over the upper half: [exp/2, exp].
    let half = exp / 2;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| u64::from(d.subsec_nanos()))
        .unwrap_or(0);
    let jitter = half + (nanos % (half + 1));
    now + jitter as i64
}

// ============================================================
// Pull phase
// ============================================================

/// Fetches every server ticket and reconciles it with SQLite:
/// * new server row → insert locally,
/// * changed server row (and no pending local ops) → apply,
/// * server row gone (and no pending local ops) → delete locally,
/// * divergence while local ops are pending → record a conflict and
///   preserve **both** sides (local row untouched, server copy stored in
///   `sync_conflicts`).
///
/// Returns how many local rows changed (drives the UI refresh).
async fn pull(client: &ApiClient) -> Result<usize, ApiError> {
    // Collect pages keyed by server id (dedupes across page boundaries).
    let mut server: HashMap<String, ServerTicket> = HashMap::new();
    let mut invalid = 0usize;
    let mut offset = 0u64;
    for _ in 0..MAX_PAGES {
        let page = client.tickets_list(offset, PAGE_LIMIT).await?;
        invalid += page.invalid_items;
        let count = page.items.len() as u64;
        for t in page.items {
            server.insert(t.ticket_id.clone(), t);
        }
        if !page.has_more || count == 0 {
            break;
        }
        offset += count;
    }
    if invalid > 0 {
        eprintln!("[sync] skipped {invalid} invalid ticket row(s) from the API");
    }

    let pending_creates = db::pending_create_subjects();
    let mapped = db::mapped_tickets();
    let local_by_server: HashMap<&str, i64> =
        mapped.iter().map(|m| (m.server_id.as_str(), m.local_id)).collect();
    let mut changed = 0usize;
    let now_str = crate::state::format_utc(now_epoch());

    // ---- server rows → local ----
    for (server_id, t) in &server {
        let mut t_norm = (*t).clone();
        t_norm.kanban_state = from_sheet_status(&t.kanban_state).to_string();
        t_norm.stage = from_sheet_status(&t.stage).to_string();
        t_norm.status = from_sheet_status(&t.status).to_string();
        let t = &t_norm;
        // Never re-insert a row that has a queued DELETE.
        if db::server_id_has_active_delete(server_id) {
            continue;
        }
        // Don't duplicate a local CREATE that hasn't been confirmed yet.
        if !local_by_server.contains_key(server_id.as_str())
            && pending_creates.iter().any(|s| s == &t.subject)
        {
            continue;
        }

        match local_by_server.get(server_id.as_str()) {
            Some(&local_id) => {
                let last = mapped
                    .iter()
                    .find(|m| m.local_id == local_id)
                    .map(|m| m.server_last_updated.clone())
                    .unwrap_or_default();
                if last == t.last_updated_on && !db::has_unresolved_conflict(local_id) {
                    continue; // unchanged since last sync
                }
                if db::ticket_has_pending_ops(local_id) {
                    // Conflict: local changes are still queued — preserve
                    // the local row and store the server copy.
                    let local = db::ticket_create_payload(local_id);
                    db::record_conflict(
                        local_id,
                        Some(server_id),
                        "server_changed_while_local_pending",
                        local.as_ref(),
                        Some(&serde_json::to_value(t).unwrap_or(Value::Null)),
                        &now_str,
                    )?;
                } else {
                    // No local changes in flight: the server (source of
                    // truth) wins and any recorded divergence settles.
                    db::apply_server_ticket(local_id, t)?;
                    db::resolve_conflicts_for_ticket(local_id)?;
                    changed += 1;
                }
            }
            None => {
                db::insert_server_ticket(t)?;
                changed += 1;
            }
        }
    }

    // ---- local rows whose server counterpart is gone ----
    for m in &mapped {
        if server.contains_key(&m.server_id) {
            continue;
        }
        if db::ticket_has_pending_ops(m.local_id) {
            // Server deleted while we still have local changes — record
            // the divergence, keep the local row and the queued ops.
            let local = db::ticket_create_payload(m.local_id);
            db::record_conflict(
                m.local_id,
                Some(&m.server_id),
                "server_deleted_while_local_pending",
                local.as_ref(),
                None,
                &now_str,
            )?;
        } else {
            db::delete_ticket(m.local_id)?;
            changed += 1;
        }
    }

    Ok(changed)
}

/// Refreshes the local employee cache from `employees.list`.
///
/// The whole directory is replaced in one transaction (the sheet is the
/// source of truth), and the returned count lets the caller decide
/// whether the UI needs to reload.
async fn pull_employees(client: &ApiClient) -> Result<usize, ApiError> {
    let employees = client.employees_list().await?;
    let count = employees.len();
    if count == 0 {
        // An empty directory is suspicious (a filter glitch, a transient
        // sheet problem) — never wipe a populated cache with nothing.
        eprintln!("[sync] employees.list returned 0 rows — keeping existing cache");
        return Ok(0);
    }
    db::replace_employees(&employees).map_err(ApiError::from)?;
    Ok(count)
}

/// Forces the next cycle to refresh the employee directory immediately
/// (used by the Employees page's "Refresh" action).
pub fn invalidate_employee_cache() {
    let _ = db::sync_meta_set("last_employee_pull_at", "0");
}

/// Fetches all employees from Google Sheets immediately in real-time.
pub async fn pull_employees_now() -> Result<usize, ApiError> {
    let client = ensure_client().map_err(ApiError::Network)?;
    let count = pull_employees(client).await?;
    let now = now_epoch();
    let _ = db::sync_meta_set("last_employee_pull_at", &now.to_string());
    if count > 0 {
        with_status(|s| s.revision = s.revision.wrapping_add(1));
    }
    Ok(count)
}

// ============================================================
// Enqueue helpers (called by the UI right after a local write)
// ============================================================

fn now_str() -> String {
    crate::state::format_utc(now_epoch())
}

/// Queue a CREATE for a freshly inserted local row (payload is read from
/// SQLite so every create path shares one field mapping).
pub fn enqueue_create(local_ticket_id: i64) {
    let Some(payload) = db::ticket_create_payload(local_ticket_id) else {
        eprintln!("[sync] enqueue_create: row {local_ticket_id} not readable");
        return;
    };
    if let Err(e) = db::enqueue_create(local_ticket_id, &payload, &now_str()) {
        eprintln!("[sync] enqueue_create failed: {e}");
        return;
    }
    kick();
}

/// Queue a partial UPDATE (`fields` uses API field names).
pub fn enqueue_update(local_ticket_id: i64, fields: Value) {
    let server_id = db::server_ticket_id(local_ticket_id);
    if let Err(e) = db::enqueue_update(local_ticket_id, server_id, &fields, &now_str()) {
        eprintln!("[sync] enqueue_update failed: {e}");
        return;
    }
    kick();
}

/// Queue a DELETE. Must be called **before** the local row is removed
/// (the server mapping is read from the row).
pub fn enqueue_delete(local_ticket_id: i64) {
    let server_id = db::server_ticket_id(local_ticket_id);
    match db::enqueue_delete(local_ticket_id, server_id.as_deref(), &now_str()) {
        Ok(_) => {}
        Err(e) => eprintln!("[sync] enqueue_delete failed: {e}"),
    }
    kick();
}

// ============================================================
// Reminder sync & execution
// ============================================================
// Reminder sync helpers
// ============================================================

/// Google Apps Script reminders.create only accepts these four fields:
/// employee_code, service_category, reminder_message, send_at.
/// All other fields (employee_name, email_address, etc.) are auto-resolved
/// by the Apps Script from the Employees sheet and trigger UNKNOWN_FIELD if passed.
pub fn parse_to_iso8601(raw: &str) -> String {
    let trimmed = raw.trim();
    // Valid ISO 8601 starts with a 4-digit year e.g. 2026-
    if trimmed.len() >= 10
        && trimmed.as_bytes()[0..4].iter().all(|b| b.is_ascii_digit())
        && trimmed.as_bytes()[4] == b'-'
    {
        if trimmed.contains('T') {
            return trimmed.to_string();
        }
        if trimmed.len() >= 19 && trimmed.as_bytes()[10] == b' ' {
            return format!("{}T{}Z", &trimmed[..10], &trimmed[11..19]);
        }
        if trimmed.len() == 10 {
            return format!("{}T09:00:00Z", trimmed);
        }
    }

    let now = now_epoch();
    let lower = trimmed.to_lowercase();
    let target_epoch = if lower.contains("tomorrow") {
        now + 86_400
    } else if lower.contains("next week") {
        now + 7 * 86_400
    } else {
        now
    };

    let utc = crate::state::format_utc(target_epoch);
    let time_part = if lower.contains("2:00 pm") || lower.contains("14:00") {
        "14:00:00"
    } else if lower.contains("5:00 pm") || lower.contains("5:21 pm") || lower.contains("17:00") {
        "17:00:00"
    } else if lower.contains("4:30 pm") || lower.contains("16:30") {
        "16:30:00"
    } else if lower.contains("9:00 am") || lower.contains("09:00") {
        "09:00:00"
    } else if lower.contains("10:00 am") || lower.contains("10:00") {
        "10:00:00"
    } else {
        &utc[11..19]
    };

    format!("{}T{time_part}Z", &utc[..10])
}

fn ensure_iso8601(val: Option<&Value>) -> Value {
    let raw = val.and_then(|v| v.as_str()).unwrap_or("");
    json!(parse_to_iso8601(raw))
}

pub fn sanitize_reminder_create_payload(payload: &Value) -> Value {
    let mut out = serde_json::Map::new();
    if let Some(obj) = payload.as_object() {
        if let Some(v) = obj.get("employee_code") { out.insert("employee_code".to_string(), v.clone()); }
        if let Some(v) = obj.get("service_category") { out.insert("service_category".to_string(), v.clone()); }
        if let Some(v) = obj.get("reminder_message") { out.insert("reminder_message".to_string(), v.clone()); }
        out.insert("send_at".to_string(), ensure_iso8601(obj.get("send_at")));
    }
    Value::Object(out)
}

/// Google Apps Script reminders.update only accepts:
/// employee_code, service_category, reminder_message, send_at, status.
pub fn sanitize_reminder_update_payload(payload: &Value) -> Value {
    let mut out = serde_json::Map::new();
    if let Some(obj) = payload.as_object() {
        if let Some(v) = obj.get("employee_code") { out.insert("employee_code".to_string(), v.clone()); }
        if let Some(v) = obj.get("service_category") { out.insert("service_category".to_string(), v.clone()); }
        if let Some(v) = obj.get("reminder_message") { out.insert("reminder_message".to_string(), v.clone()); }
        if let Some(v) = obj.get("send_at") { out.insert("send_at".to_string(), ensure_iso8601(Some(v))); }
        if let Some(v) = obj.get("status") { out.insert("status".to_string(), v.clone()); }
    }
    Value::Object(out)
}

async fn execute_reminder_op(client: &ApiClient, op: &SyncOp) -> Result<ExecOutcome, ApiError> {
    match op.op_type.as_str() {
        "CREATE" => {
            let sanitized = sanitize_reminder_create_payload(&op.payload);
            let remote = client.reminders_create(&sanitized).await?;
            db::set_reminder_mapping(op.local_ticket_id, &remote.reminder_id, &remote.last_updated_at)?;
            Ok(ExecOutcome::Synced)
        }
        "UPDATE" => {
            let server_id = op
                .server_ticket_id
                .clone()
                .or_else(|| db::server_reminder_id(op.local_ticket_id));

            let Some(server_id) = server_id else {
                let Some(row) = db::reminder_create_payload(op.local_ticket_id) else {
                    return Ok(ExecOutcome::Superseded);
                };
                let sanitized = sanitize_reminder_create_payload(&row);
                let remote = client.reminders_create(&sanitized).await?;
                db::set_reminder_mapping(op.local_ticket_id, &remote.reminder_id, &remote.last_updated_at)?;
                return Ok(ExecOutcome::Synced);
            };

            let sanitized = sanitize_reminder_update_payload(&op.payload);
            let remote = client.reminders_update(&server_id, &sanitized).await?;
            db::set_reminder_mapping(op.local_ticket_id, &remote.reminder_id, &remote.last_updated_at)?;
            Ok(ExecOutcome::Synced)
        }
        "SEND_NOW" => {
            let server_id = op
                .server_ticket_id
                .clone()
                .or_else(|| db::server_reminder_id(op.local_ticket_id));

            if let Some(server_id) = server_id {
                let force = op.payload.get("force").and_then(|v| v.as_bool()).unwrap_or(false);
                client.reminders_send_now(&server_id, force).await?;
                let now_str = crate::state::format_utc(now_epoch());
                let _ = db::set_reminder_sent(op.local_ticket_id, &now_str);
            }
            Ok(ExecOutcome::Synced)
        }
        other => Err(ApiError::Api {
            code: "UNKNOWN_OP".to_string(),
            message: format!("unsupported reminder op {other}"),
            retryable: false,
            retry_after: None,
        }),
    }
}

async fn pull_reminders(client: &ApiClient) -> Result<usize, ApiError> {
    let mut server: HashMap<String, ServerReminder> = HashMap::new();
    let mut offset = 0u64;
    for _ in 0..MAX_PAGES {
        let page = client.reminders_list(offset, PAGE_LIMIT, None, None).await?;
        let count = page.items.len() as u64;
        for r in page.items {
            server.insert(r.reminder_id.clone(), r);
        }
        if !page.has_more || count == 0 {
            break;
        }
        offset += count;
    }

    let mapped = db::mapped_reminders();
    let local_by_server: HashMap<&str, i64> =
        mapped.iter().map(|m| (m.server_id.as_str(), m.local_id)).collect();
    let mut changed = 0usize;

    for (server_id, r) in &server {
        match local_by_server.get(server_id.as_str()) {
            Some(&local_id) => {
                let last = mapped
                    .iter()
                    .find(|m| m.local_id == local_id)
                    .map(|m| m.server_last_updated.clone())
                    .unwrap_or_default();
                if last != r.last_updated_at {
                    db::apply_server_reminder(local_id, r)?;
                    changed += 1;
                }
            }
            None => {
                db::insert_server_reminder(r)?;
                changed += 1;
            }
        }
    }
    Ok(changed)
}

/// Queue a CREATE for a freshly inserted local reminder.
pub fn enqueue_reminder_create(local_reminder_id: i64) {
    let Some(payload) = db::reminder_create_payload(local_reminder_id) else {
        eprintln!("[sync] enqueue_reminder_create: row {local_reminder_id} not readable");
        return;
    };
    if let Err(e) = db::enqueue_reminder_create(local_reminder_id, &payload, &now_str()) {
        eprintln!("[sync] enqueue_reminder_create failed: {e}");
        return;
    }
    kick();
}

/// Queue an UPDATE for a reminder.
pub fn enqueue_reminder_update(local_reminder_id: i64, fields: Value) {
    let server_id = db::server_reminder_id(local_reminder_id);
    if let Err(e) = db::enqueue_reminder_update(local_reminder_id, server_id, &fields, &now_str()) {
        eprintln!("[sync] enqueue_reminder_update failed: {e}");
        return;
    }
    kick();
}

/// Queue immediate sending of a reminder.
pub fn enqueue_reminder_send_now(local_reminder_id: i64, force: bool) {
    let Some(server_id) = db::server_reminder_id(local_reminder_id) else {
        eprintln!("[sync] enqueue_reminder_send_now: row {local_reminder_id} has no server_id");
        return;
    };
    if let Err(e) = db::enqueue_reminder_send_now(local_reminder_id, &server_id, force, &now_str()) {
        eprintln!("[sync] enqueue_reminder_send_now failed: {e}");
        return;
    }
    kick();
}

/// Immediate execution of reminders.sendNow (secure form email)
pub async fn send_reminder_now_immediate(reminder_id: &str, force: bool) -> Result<Value, ApiError> {
    let client = ensure_client().map_err(ApiError::Network)?;
    let result = client.reminders_send_now(reminder_id, force).await?;
    let now_str = crate::state::format_utc(now_epoch());
    let _ = db::set_server_reminder_sent(reminder_id, &now_str);
    Ok(result)
}

/// Immediate execution of employees.create
pub async fn create_employee_immediate(emp: &crate::api::Employee) -> Result<crate::api::Employee, ApiError> {
    let client = ensure_client().map_err(ApiError::Network)?;
    let payload = serde_json::to_value(emp).map_err(|e| ApiError::Decode(e.to_string()))?;
    let created = client.employees_create(&payload).await?;
    let _ = db::create_employee_db(&created);
    Ok(created)
}

/// Immediate execution of employees.update
pub async fn update_employee_immediate(
    emp_code: &str,
    fields: &Value,
) -> Result<crate::api::Employee, ApiError> {
    let client = ensure_client().map_err(ApiError::Network)?;
    let updated = client.employees_update(emp_code, fields).await?;
    let _ = db::update_employee_db(
        &updated.employee_code,
        &updated.employee_name,
        &updated.email,
        &updated.department,
        &updated.division,
    );
    Ok(updated)
}

/// Immediate execution of employees.delete
pub async fn delete_employee_immediate(emp_code: &str) -> Result<(), ApiError> {
    let client = ensure_client().map_err(ApiError::Network)?;
    client.employees_delete(emp_code).await?;
    let _ = db::delete_employee_db(emp_code);
    Ok(())
}
