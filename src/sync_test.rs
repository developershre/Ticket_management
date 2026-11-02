//! End-to-end synchronization test harness (integration Step 7).
//!
//! ```text
//! management --sync-test offline   # unreachable API: op retained, survives restart
//! management --sync-test resume    # same DB as `offline`: op syncs once "online"
//! management --sync-test online    # full roundtrip: create/update/delete, pull, conflict
//! ```
//!
//! `offline` and `resume` share one temporary database and run as separate
//! processes, which exercises restart survival of the persistent queue.
//! `online` uses a fresh database and talks to the real API — it requires
//! credentials (`TMS_API_KEY` or the Postman collection file) and
//! `TMS_EMPLOYEE_CODE` (an existing sheet employee) for creates. Test
//! tickets use the `SYNC-TEST` subject prefix and are cleaned up at the
//! start and end of the run.

use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::api::{ApiConfig, ApiClient, ApiError};
use crate::db::{self, SyncOp};
use crate::sync;

const OFFLINE_SUBJECT: &str = "SYNC-TEST-OFFLINE pending op retention";

pub fn run(scenario: &str) -> i32 {
    let dir = std::env::temp_dir().join("tms-sync-test");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("FAIL: cannot create test dir: {e}");
        return 1;
    }
    let db_file = dir.join("management.db");

    match scenario {
        "offline" => {
            // Fresh database; point the client at a dead endpoint.
            let _ = std::fs::remove_file(&db_file);
            std::env::set_var("TMS_DB_PATH", &db_file);
            std::env::set_var("TMS_API_BASE_URL", "http://127.0.0.1:1");
            if std::env::var("TMS_API_KEY").is_err() {
                std::env::set_var("TMS_API_KEY", "offline-test-key");
            }
        }
        "resume" => {
            std::env::set_var("TMS_DB_PATH", &db_file);
        }
        "online" => {
            let _ = std::fs::remove_file(&db_file);
            std::env::set_var("TMS_DB_PATH", &db_file);
        }
        other => {
            eprintln!("unknown scenario: {other} (expected offline|resume|online)");
            return 2;
        }
    }

    if let Err(e) = db::init_database() {
        eprintln!("FAIL: database init: {e}");
        return 1;
    }

    match scenario {
        "offline" => test_offline(),
        "resume" => test_resume(),
        _ => test_online(),
    }
}

// ============================================================
// Helpers
// ============================================================

macro_rules! check {
    ($cond:expr, $msg:expr) => {
        if !$cond {
            eprintln!("FAIL: {}", $msg);
            return 1;
        }
        println!("  ok: {}", $msg);
    };
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    rt.block_on(f)
}

fn cycle() {
    sync::run_cycle_blocking();
}

/// Makes the next cycle's pull phase run regardless of the min gap.
fn force_pull() {
    let _ = db::sync_meta_set("last_pull_at", "0");
}

fn active_ops(local_id: i64) -> Vec<SyncOp> {
    db::active_ops_for_ticket(local_id).unwrap_or_default()
}

fn find_local_by_subject(subject: &str) -> Option<i64> {
    let conn = db::get_connection().ok()?;
    conn.query_row(
        "SELECT ticket_id FROM tickets WHERE subject = ?1 ORDER BY ticket_id DESC LIMIT 1",
        params![subject],
        |r| r.get(0),
    )
    .optional()
    .ok()
    .flatten()
}

fn local_field(local_id: i64, column: &str) -> Option<String> {
    // Column names come from this file's own literals only.
    let conn = db::get_connection().ok()?;
    let sql = format!("SELECT {column} FROM tickets WHERE ticket_id = ?1");
    conn.query_row(&sql, params![local_id], |r| r.get(0))
        .optional()
        .ok()
        .flatten()
}

fn load_online_client() -> Option<ApiClient> {
    match ApiConfig::load() {
        Ok(cfg) => match ApiClient::new(cfg) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("FAIL: HTTP client: {e}");
                None
            }
        },
        Err(e) => {
            eprintln!("FAIL: {e}");
            eprintln!("       (set TMS_API_KEY / TMS_API_BASE_URL or run next to TMS_API.postman_collection.json)");
            None
        }
    }
}

/// Deletes every leftover `SYNC-TEST*` row on the server so runs are
/// repeatable even after an aborted previous run.
fn cleanup_server(client: &ApiClient) {
    let mut offset = 0u64;
    for _ in 0..20 {
        let Ok(page) = block_on(client.tickets_list(offset, 200)) else {
            return;
        };
        for t in &page.items {
            if t.subject.starts_with("SYNC-TEST") {
                let _ = block_on(client.tickets_delete(&t.ticket_id));
            }
        }
        if !page.has_more || page.items.is_empty() {
            return;
        }
        offset += page.items.len() as u64;
    }
}

/// Creates a ticket directly through the API — simulates another user
/// working in the Google Sheet.
fn remote_create(client: &ApiClient, subject: &str, priority: &str) -> Result<String, ApiError> {
    // Test-only default matching a known sheet employee; production code
    // never hardcodes employee codes (uses TMS_EMPLOYEE_CODE or the row's
    // own value).
    let employee = client
        .config()
        .employee_code
        .clone()
        .unwrap_or_else(|| "10081".to_string());
    let payload = json!({
        "employee_code": employee,
        "email_address": "sync-test@example.com",
        "category": "IT Support",
        "subject": subject,
        "problem_description": format!("Created by the sync test harness ({subject})."),
        "priority": priority,
        "assigned_to": "IT Helpdesk",
        "customer": "Head Office",
        "status": "Open",
        "stage": "Triage",
        "kanban_state": "Pending",
    });
    block_on(client.tickets_create(&payload)).map(|t| t.ticket_id)
}

// ============================================================
// Scenario: offline
// ============================================================

fn test_offline() -> i32 {
    println!("[offline] local write against an unreachable API…");

    let input = db::NewTicketInput {
        subject: OFFLINE_SUBJECT.to_string(),
        priority: "High".to_string(),
        ..Default::default()
    };
    let local_id = match db::create_ticket_full(&input) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("FAIL: local create: {e}");
            return 1;
        }
    };
    sync::enqueue_create(local_id);
    cycle();

    // 1. The operation is retained with its payload and a retry count.
    let ops = active_ops(local_id);
    check!(ops.len() == 1, format!("exactly one queued op (got {})", ops.len()));
    let op = &ops[0];
    check!(op.op_type == "CREATE", format!("op type is CREATE ({})", op.op_type));
    check!(
        op.status == "pending" || op.status == "failed",
        format!("op retained, not lost (status {})", op.status)
    );
    check!(op.retry_count >= 1, format!("retry_count >= 1 ({})", op.retry_count));
    check!(
        op.payload.get("subject").and_then(Value::as_str) == Some(OFFLINE_SUBJECT),
        format!("payload intact: {}", op.payload)
    );
    check!(
        op.next_attempt_at.unwrap_or(0) > now(),
        "exponential backoff scheduled in the future"
    );

    // 2. Status surfaced to the UI.
    let status = sync::snapshot();
    check!(status.state == sync::SyncState::Error, format!("state is Error ({:?})", status.state));
    check!(status.pending_ops >= 1, format!("pending_ops >= 1 ({})", status.pending_ops));

    // 3. Simulated crash mid-operation: an `in_flight` op must return to
    //    `pending` on restart (init_database runs at every app start).
    {
        let Ok(conn) = db::get_connection() else {
            eprintln!("FAIL: connection");
            return 1;
        };
        if conn
            .execute(
                "UPDATE sync_queue SET status = 'in_flight' WHERE op_id = ?1",
                params![op.op_id],
            )
            .is_err()
        {
            eprintln!("FAIL: could not mark op in_flight");
            return 1;
        }
    }
    if let Err(e) = db::init_database() {
        eprintln!("FAIL: re-init: {e}");
        return 1;
    }
    let ops = active_ops(local_id);
    check!(ops.len() == 1, "op still present after simulated restart");
    check!(ops[0].status == "pending", format!("in_flight recovered to pending ({})", ops[0].status));

    println!("PASS offline: queued op retained with payload, backoff, and restart recovery");
    0
}

// ============================================================
// Scenario: resume (connectivity returns, separate process)
// ============================================================

fn test_resume() -> i32 {
    println!("[resume] pending op from the offline run, connectivity restored…");

    let Some(client) = load_online_client() else {
        return 1;
    };

    // 1. The queued op survived the process restart.
    let local_id = match find_local_by_subject(OFFLINE_SUBJECT) {
        Some(id) => id,
        None => {
            eprintln!("FAIL: offline ticket missing — run `--sync-test offline` first");
            return 1;
        }
    };
    let ops = active_ops(local_id);
    check!(!ops.is_empty(), "queued op survived the restart");

    // 2. Let the backoff window lapse, then sync with connectivity back.
    {
        let Ok(conn) = db::get_connection() else {
            eprintln!("FAIL: connection");
            return 1;
        };
        let _ = conn.execute("UPDATE sync_queue SET next_attempt_at = NULL", []);
    }
    cycle();

    let ops = active_ops(local_id);
    if !ops.is_empty() {
        eprintln!(
            "FAIL: op still pending after sync: {} (hint: set TMS_EMPLOYEE_CODE to an existing sheet employee)",
            ops[0].last_error.clone().unwrap_or_default()
        );
        return 1;
    }
    println!("  ok: pending op confirmed by the server after connectivity returned");

    // 3. Mapping recorded and the row exists on the server.
    let server_id = match db::server_ticket_id(local_id) {
        Some(id) => id,
        None => {
            eprintln!("FAIL: server mapping missing");
            return 1;
        }
    };
    let remote = match block_on(client.tickets_get(&server_id)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: tickets.get: {e}");
            return 1;
        }
    };
    check!(remote.subject == OFFLINE_SUBJECT, format!("server row matches ({})", remote.subject));

    // 4. Cleanup: local delete is pushed, server row disappears.
    sync::enqueue_delete(local_id);
    let _ = db::delete_ticket(local_id);
    cycle();
    match block_on(client.tickets_get(&server_id)) {
        Err(ApiError::Api { code, .. }) if code == "NOT_FOUND" => {
            println!("  ok: delete confirmed server-side");
        }
        other => {
            eprintln!("FAIL: expected NOT_FOUND after delete, got {other:?}");
            return 1;
        }
    }

    println!("PASS resume: queue survived restart and synced after connectivity returned");
    0
}

// ============================================================
// Scenario: online (full roundtrip + pull + conflict)
// ============================================================

fn test_online() -> i32 {
    println!("[online] full roundtrip against the real API…");

    let Some(client) = load_online_client() else {
        return 1;
    };
    cleanup_server(&client);

    // ---- 1. local create → pushed to the sheet ----
    let input = db::NewTicketInput {
        subject: "SYNC-TEST-LOCAL roundtrip".to_string(),
        priority: "Medium".to_string(),
        ..Default::default()
    };
    let local_a = match db::create_ticket_full(&input) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("FAIL: local create: {e}");
            return 1;
        }
    };
    sync::enqueue_create(local_a);
    cycle();

    let server_a = match db::server_ticket_id(local_a) {
        Some(id) => id,
        None => {
            eprintln!("FAIL: CREATE not confirmed (mapping missing)");
            return 1;
        }
    };
    let remote_a = match block_on(client.tickets_get(&server_a)) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("FAIL: tickets.get after create: {e}");
            return 1;
        }
    };
    check!(
        remote_a.subject == "SYNC-TEST-LOCAL roundtrip",
        format!("create confirmed on server ({})", remote_a.subject)
    );

    // ---- duplicate prevention: rapid edits coalesce into ONE op ----
    db::update_ticket_priority(local_a, "High").ok();
    sync::enqueue_update(local_a, json!({ "priority": "High" }));
    db::update_ticket_priority(local_a, "Critical").ok();
    sync::enqueue_update(local_a, json!({ "priority": "Critical" }));
    let ops = active_ops(local_a);
    check!(ops.len() == 1, format!("two edits → one queued op (got {})", ops.len()));
    check!(
        ops[0].payload.get("priority").and_then(Value::as_str) == Some("Critical"),
        "coalesced payload holds the newest value"
    );
    cycle();
    let remote_a = block_on(client.tickets_get(&server_a)).expect("tickets.get");
    check!(remote_a.priority == "Critical", format!("update confirmed ({})", remote_a.priority));

    // ---- 2. server → local pull (remote create, then remote update) ----
    let remote_b = match remote_create(&client, "SYNC-TEST-REMOTE", "Medium") {
        Ok(id) => id,
        Err(e) => {
            eprintln!("FAIL: remote create: {e}");
            return 1;
        }
    };
    force_pull();
    cycle();
    let local_b = match find_local_by_subject("SYNC-TEST-REMOTE") {
        Some(id) => id,
        None => {
            eprintln!("FAIL: remote ticket not pulled into SQLite");
            return 1;
        }
    };
    check!(
        db::server_ticket_id(local_b).as_deref() == Some(remote_b.as_str()),
        "pulled row carries the server mapping"
    );

    if let Err(e) = block_on(client.tickets_update(&remote_b, &json!({ "priority": "High" }))) {
        eprintln!("FAIL: remote update: {e}");
        return 1;
    }
    force_pull();
    cycle();
    check!(
        local_field(local_b, "priority").as_deref() == Some("High"),
        format!("remote change applied locally ({:?})", local_field(local_b, "priority"))
    );

    // ---- 3. local delete → pushed ----
    sync::enqueue_delete(local_a);
    let _ = db::delete_ticket(local_a);
    cycle();
    match block_on(client.tickets_get(&server_a)) {
        Err(ApiError::Api { code, .. }) if code == "NOT_FOUND" => {
            println!("  ok: local delete pushed to the server");
        }
        other => {
            eprintln!("FAIL: expected NOT_FOUND after local delete, got {other:?}");
            return 1;
        }
    }

    // ---- 4. remote delete → pulled (local row removed) ----
    if let Err(e) = block_on(client.tickets_delete(&remote_b)) {
        eprintln!("FAIL: remote delete: {e}");
        return 1;
    }
    force_pull();
    cycle();
    check!(
        find_local_by_subject("SYNC-TEST-REMOTE").is_none(),
        "server-side delete removed the local row"
    );

    // ---- 5. conflict: pending local edit vs remote change ----
    let remote_c = match remote_create(&client, "SYNC-TEST-CONFLICT", "Medium") {
        Ok(id) => id,
        Err(e) => {
            eprintln!("FAIL: remote create: {e}");
            return 1;
        }
    };
    force_pull();
    cycle();
    let local_c = match find_local_by_subject("SYNC-TEST-CONFLICT") {
        Some(id) => id,
        None => {
            eprintln!("FAIL: conflict ticket not pulled");
            return 1;
        }
    };

    // Local change is written + queued, but deferred (as if still in
    // backoff), while the remote row changes underneath it.
    db::update_ticket_priority(local_c, "Critical").ok();
    sync::enqueue_update(local_c, json!({ "priority": "Critical" }));
    {
        let Ok(conn) = db::get_connection() else {
            eprintln!("FAIL: connection");
            return 1;
        };
        let _ = conn.execute(
            "UPDATE sync_queue SET next_attempt_at = ?1 WHERE local_ticket_id = ?2",
            params![now() + 3600, local_c],
        );
    }
    if let Err(e) = block_on(client.tickets_update(
        &remote_c,
        &json!({ "priority": "Low", "subject": "SYNC-TEST-CONFLICT-REMOTE" }),
    )) {
        eprintln!("FAIL: remote conflict update: {e}");
        return 1;
    }
    force_pull();
    cycle();

    check!(
        db::unresolved_conflict_count() >= 1,
        "conflict recorded in sync_conflicts"
    );
    check!(
        local_field(local_c, "priority").as_deref() == Some("Critical"),
        "local pending change preserved (not overwritten by server)"
    );
    let server_c = block_on(client.tickets_get(&remote_c)).expect("tickets.get");
    check!(
        server_c.subject == "SYNC-TEST-CONFLICT-REMOTE" && server_c.priority == "Low",
        "server data preserved in the conflict record cycle"
    );

    // Local op becomes due → pushes; next pull applies the remote subject.
    {
        let Ok(conn) = db::get_connection() else {
            eprintln!("FAIL: connection");
            return 1;
        };
        let _ = conn.execute(
            "UPDATE sync_queue SET next_attempt_at = NULL WHERE local_ticket_id = ?1",
            params![local_c],
        );
    }
    cycle(); // push: priority=Critical lands on the server

    // Pull until the conflict settles. Bounded retries tolerate a single
    // transient API error (Apps Script cold-starts); a real defect would
    // fail every attempt.
    let mut resolved = false;
    for attempt in 1..=3 {
        force_pull();
        cycle();
        if db::unresolved_conflict_count() == 0
            && local_field(local_c, "subject").as_deref() == Some("SYNC-TEST-CONFLICT-REMOTE")
        {
            resolved = true;
            break;
        }
        println!("  ... resolution attempt {attempt} incomplete, retrying");
    }
    check!(
        resolved,
        format!("conflict auto-resolved ({} left)", db::unresolved_conflict_count())
    );
    check!(
        local_field(local_c, "subject").as_deref() == Some("SYNC-TEST-CONFLICT-REMOTE"),
        format!("remote change applied ({:?})", local_field(local_c, "subject"))
    );
    let server_c = block_on(client.tickets_get(&remote_c)).expect("tickets.get");
    check!(
        server_c.priority == "Critical",
        format!("local change retained on server ({})", server_c.priority)
    );

    // ---- cleanup ----
    sync::enqueue_delete(local_c);
    let _ = db::delete_ticket(local_c);
    cycle();
    cleanup_server(&client);

    println!("PASS online: create/update/delete roundtrip, pull, dedupe, conflict handling");
    0
}
