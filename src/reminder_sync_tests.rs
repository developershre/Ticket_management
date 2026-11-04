#[cfg(test)]
mod tests {
    use rusqlite::params;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    use crate::api::ApiError;
    use crate::db::{self, NewReminderInput};
    use crate::sync;

    static DB_COUNTER: AtomicU64 = AtomicU64::new(1);
    static TEST_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// Sets up a clean, isolated SQLite database in temp dir for each test.
    fn setup_test_db() -> (std::path::PathBuf, std::sync::MutexGuard<'static, ()>) {
        let lock = TEST_MUTEX.lock().unwrap();
        let count = DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!("tms_test_dir_{}_{}", std::process::id(), count));
        let _ = std::fs::create_dir_all(&dir);
        let db_path = dir.join("management.db");
        let _ = std::fs::remove_file(&db_path);
        std::env::set_var("TMS_DB_PATH", db_path.to_str().unwrap());

        db::init_database().expect("Failed to init database");
        (db_path, lock)
    }

    /// Test 1: Updating a pending reminder
    #[test]
    fn test_updating_pending_reminder() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "IT Support".to_string(),
            reminder_message: "Pending Reminder 1".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: "IT-101".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");
        assert_eq!(db::is_reminder_completed(local_id), false);

        // Map it to a server reminder ID
        db::set_reminder_mapping(local_id, "REM-20261104-0001", "2026-11-04 10:00:00")
            .expect("mapping");

        // Enqueue an update
        sync::enqueue_reminder_update(
            local_id,
            json!({
                "service_category": "Hardware",
                "reminder_message": "Updated Message"
            }),
        );

        let due = db::due_ops(1_000_000_000_000, 10).expect("due ops");
        assert_eq!(due.len(), 1);
        let op = &due[0];
        assert_eq!(op.entity_type, "reminder");
        assert_eq!(op.op_type, "UPDATE");
        assert_eq!(op.local_ticket_id, local_id);
        assert_eq!(op.server_ticket_id.as_deref(), Some("REM-20261104-0001"));
        assert_eq!(op.status, "pending");
    }

    /// Test 2: Updating a completed reminder
    #[test]
    fn test_updating_completed_reminder() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "IT Support".to_string(),
            reminder_message: "Completed Reminder".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Completed".to_string(),
            email_sent_at: "2026-11-04 10:00:00".to_string(),
            ticket_id: "IT-200".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");
        assert_eq!(db::is_reminder_completed(local_id), true);

        // Attempting to enqueue update on completed reminder must be rejected
        sync::enqueue_reminder_update(local_id, json!({ "status": "Cancelled" }));

        let due = db::due_ops(1_000_000_000_000, 10).expect("due ops");
        assert_eq!(due.len(), 0, "No operations should be queued for completed reminder");
    }

    /// Test 3: Updating a missing reminder (NOT_FOUND handling)
    #[test]
    fn test_updating_missing_reminder_reconciliation() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "Hardware".to_string(),
            reminder_message: "Missing Remote Reminder".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: "IT-300".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");
        db::set_reminder_mapping(local_id, "REM-DELETED-9999", "2026-11-04 10:00:00")
            .expect("mapping");

        sync::enqueue_reminder_update(local_id, json!({ "service_category": "Software" }));

        // Simulate terminal NOT_FOUND reconciliation
        db::mark_reminder_server_not_found(local_id).expect("mark not found");

        // Verify local reminder still exists and ticket history is preserved
        let conn = db::get_connection().expect("conn");
        let (srv_id, ticket_id, msg): (String, String, String) = conn
            .query_row(
                "SELECT COALESCE(server_reminder_id, ''), COALESCE(ticket_id, ''), reminder_message FROM reminders WHERE reminder_id = ?1",
                params![local_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .expect("query row");

        assert_eq!(srv_id, "", "server_reminder_id should be cleared");
        assert_eq!(ticket_id, "IT-300", "associated ticket_id must be preserved");
        assert_eq!(msg, "Missing Remote Reminder", "reminder record preserved");

        // Verify queued ops were superseded and not retryable
        let pending = db::due_ops(1_000_000_000_000, 10).expect("due ops");
        assert_eq!(pending.len(), 0);
    }

    /// Test 4: Reconciliation after a reminder is completed remotely (CONFLICT & pull)
    #[test]
    fn test_reconciliation_when_completed_remotely() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "Network".to_string(),
            reminder_message: "Reconcile Remote Completion".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: "IT-400".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");
        db::set_reminder_mapping(local_id, "REM-COMPLETED-400", "2026-11-04 10:00:00")
            .expect("mapping");

        // Queue a pending update that conflicts with server completion
        sync::enqueue_reminder_update(local_id, json!({ "service_category": "Security" }));

        // Simulate server returning CONFLICT: Completed reminders cannot be updated
        db::update_reminder_status(local_id, "Completed").expect("update status");
        let cancelled = db::cancel_queued_updates_for_reminder(local_id).expect("cancel");
        assert!(cancelled >= 1, "Cancelled obsolete queued updates");

        assert_eq!(db::is_reminder_completed(local_id), true);

        // Ensure ticket_id is preserved
        let conn = db::get_connection().expect("conn");
        let ticket_id: String = conn
            .query_row(
                "SELECT ticket_id FROM reminders WHERE reminder_id = ?1",
                params![local_id],
                |r| r.get(0),
            )
            .expect("query row");
        assert_eq!(ticket_id, "IT-400");
    }

    /// Test 5: Removing obsolete queued updates
    #[test]
    fn test_removing_obsolete_queued_updates() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "General".to_string(),
            reminder_message: "Obsolete Updates Test".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: "IT-500".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");
        db::set_reminder_mapping(local_id, "REM-OBSOLETE-500", "2026-11-04 10:00:00")
            .expect("mapping");

        sync::enqueue_reminder_update(local_id, json!({ "reminder_message": "Edit 1" }));
        sync::enqueue_reminder_update(local_id, json!({ "reminder_message": "Edit 2" }));

        let count = db::cancel_queued_updates_for_reminder(local_id).expect("cancel updates");
        assert!(count >= 1);

        let due = db::due_ops(1_000_000_000_000, 10).expect("due ops");
        assert_eq!(due.len(), 0, "All obsolete updates should be superseded");
    }

    /// Test 6: Restarting the app with an obsolete reminder update in SQLite
    #[test]
    fn test_restart_app_with_obsolete_reminder_update() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "General".to_string(),
            reminder_message: "Terminal Failure Restart Test".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Completed".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: "IT-600".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");

        // Manually insert a failed op with terminal CONFLICT error
        let conn = db::get_connection().expect("conn");
        conn.execute(
            "INSERT INTO sync_queue (entity_type, local_ticket_id, server_ticket_id, op_type, payload, status, retry_count, last_error, created_at) \
             VALUES ('reminder', ?1, 'REM-600', 'UPDATE', '{}', 'failed', 3, 'CONFLICT: Completed reminders cannot be updated', '2026-11-04 10:00:00')",
            params![local_id],
        )
        .expect("insert op");

        // Simulate app restart: reset_failed_sync_ops is called
        db::reset_failed_sync_ops().expect("reset failed ops");

        // The terminal conflict op MUST remain failed and NOT be reset to pending
        let status: String = conn
            .query_row(
                "SELECT status FROM sync_queue WHERE local_ticket_id = ?1",
                params![local_id],
                |r| r.get(0),
            )
            .expect("query status");
        assert_eq!(status, "failed", "Terminal conflict op must not be reset to pending on restart");
    }

    /// Test 7: Retrying transient network failures
    #[test]
    fn test_retrying_transient_failures() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11061".to_string(),
            employee_name: "Chaitram".to_string(),
            email_address: "chaitram@example.com".to_string(),
            service_category: "General".to_string(),
            reminder_message: "Transient Retry Test".to_string(),
            send_at: "2026-11-05 10:00:00".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: "IT-700".to_string(),
            cc_emails: "".to_string(),
            repeat_frequency: "daily".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create reminder");

        // Check ApiError retryable determination
        let lock_timeout_err = ApiError::Api {
            code: "LOCK_TIMEOUT".to_string(),
            message: "The server is busy. Please retry.".to_string(),
            retryable: true,
            retry_after: Some(2),
        };
        assert_eq!(lock_timeout_err.is_retryable(), true);

        let not_found_err = ApiError::Api {
            code: "NOT_FOUND".to_string(),
            message: "Reminder was not found.".to_string(),
            retryable: false,
            retry_after: None,
        };
        assert_eq!(not_found_err.is_retryable(), false);

        // Insert a failed transient op (e.g. LOCK_TIMEOUT)
        let conn = db::get_connection().expect("conn");
        conn.execute(
            "INSERT INTO sync_queue (entity_type, local_ticket_id, server_ticket_id, op_type, payload, status, retry_count, last_error, created_at) \
             VALUES ('reminder', ?1, 'REM-700', 'UPDATE', '{}', 'failed', 1, 'LOCK_TIMEOUT: The server is busy.', '2026-11-04 10:00:00')",
            params![local_id],
        )
        .expect("insert op");

        // Transient op should be reset to pending on cycle/restart
        db::reset_failed_sync_ops().expect("reset ops");

        let status: String = conn
            .query_row(
                "SELECT status FROM sync_queue WHERE local_ticket_id = ?1",
                params![local_id],
                |r| r.get(0),
            )
            .expect("query status");
        assert_eq!(status, "pending", "Transient failure should be reset to pending for retry");
    }

    /// Test 8: Fresh database has ZERO default/seed data and conforms to TMS_SHEET_HEADERS_README.md
    #[test]
    fn test_fresh_database_zero_default_data_and_exact_schema() {
        let (_path, _lock) = setup_test_db();
        let conn = db::get_connection().expect("conn");

        // Verify NO default or demo data is seeded into any table
        let count_tickets: i64 = conn.query_row("SELECT count(*) FROM tickets", [], |r| r.get(0)).unwrap();
        let count_reminders: i64 = conn.query_row("SELECT count(*) FROM reminders", [], |r| r.get(0)).unwrap();
        let count_users: i64 = conn.query_row("SELECT count(*) FROM USERS", [], |r| r.get(0)).unwrap();
        let count_notes: i64 = conn.query_row("SELECT count(*) FROM NOTES", [], |r| r.get(0)).unwrap();
        let count_queue: i64 = conn.query_row("SELECT count(*) FROM sync_queue", [], |r| r.get(0)).unwrap();

        assert_eq!(count_tickets, 0, "tickets must have 0 rows initially");
        assert_eq!(count_reminders, 0, "reminders must have 0 rows initially");
        assert_eq!(count_users, 0, "USERS (employees) must have 0 rows initially");
        assert_eq!(count_notes, 0, "NOTES must have 0 rows initially");
        assert_eq!(count_queue, 0, "sync_queue must have 0 rows initially");

        // Helper to retrieve column names for a table
        let get_cols = |table: &str| -> Vec<String> {
            let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})")).unwrap();
            stmt.query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .filter_map(Result::ok)
                .collect()
        };

        // 1. Tickets sheet headers (23 columns + local sync mapping columns)
        let ticket_cols = get_cols("tickets");
        for expected in &[
            "employee_code", "email_address", "category", "subject", "problem_description",
            "attachment_name", "attachment_url", "priority", "assigned_to", "customer",
            "time_spent", "activities", "my_activity", "deadline", "sla_status", "sla_deadline",
            "rating_avg_text", "kanban_state", "stage", "status", "created_on", "last_updated_on",
            "server_ticket_id", "server_last_updated"
        ] {
            assert!(
                ticket_cols.iter().any(|c| c == expected),
                "Missing expected ticket column: {expected}"
            );
        }

        // 2. Reminders sheet headers (14 columns + local sync mapping columns)
        let reminder_cols = get_cols("reminders");
        for expected in &[
            "employee_code", "employee_name", "email_address", "service_category",
            "reminder_message", "send_at", "status", "email_sent_at", "ticket_id",
            "created_at", "last_updated_at", "cc_emails", "repeat_frequency",
            "server_reminder_id", "server_last_updated"
        ] {
            assert!(
                reminder_cols.iter().any(|c| c == expected),
                "Missing expected reminder column: {expected}"
            );
        }

        // 3. Employee sheet headers (5 columns: Employee Code, Name, Dept, Mail, Division)
        let user_cols = get_cols("USERS");
        for expected in &["emp_code", "emp_name", "emp_email", "department", "division"] {
            assert!(
                user_cols.iter().any(|c| c == expected),
                "Missing expected employee column: {expected}"
            );
        }
    }

    /// Test 9: Employee directory filtering by employee code and name
    #[test]
    fn test_employee_directory_filter_by_code_and_name() {
        let (_path, _lock) = setup_test_db();

        // Populate test employees
        let emps = vec![
            crate::api::Employee {
                employee_code: "11061".to_string(),
                employee_name: "Chaitram Sahu".to_string(),
                email: "chaitram.sahu@bpsl.net".to_string(),
                department: "IT".to_string(),
                division: "Power".to_string(),
            },
            crate::api::Employee {
                employee_code: "11082".to_string(),
                employee_name: "Rahul Verma".to_string(),
                email: "rahul.v@bpsl.net".to_string(),
                department: "Electrical".to_string(),
                division: "Steel".to_string(),
            },
        ];
        db::replace_employees(&emps).expect("insert employees");

        // Lookup by exact code
        let found_by_code = db::find_employee_by_code("11061").expect("found by code");
        assert_eq!(found_by_code.emp_name, "Chaitram Sahu");
        assert_eq!(found_by_code.emp_email, "chaitram.sahu@bpsl.net");

        // Filter all employees by name substring
        let all = db::get_all_employees();
        assert_eq!(all.len(), 2);
        let filter_name: Vec<_> = all
            .iter()
            .filter(|e| e.emp_name.to_lowercase().contains("rahul"))
            .collect();
        assert_eq!(filter_name.len(), 1);
        assert_eq!(filter_name[0].emp_code, "11082");

        // Filter all employees by code substring
        let filter_code: Vec<_> = all
            .iter()
            .filter(|e| e.emp_code.contains("1106"))
            .collect();
        assert_eq!(filter_code.len(), 1);
        assert_eq!(filter_code[0].emp_name, "Chaitram Sahu");
    }
}
