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

    #[test]
    fn test_ticket_real_timestamps_preserved() {
        use crate::state::{arrange_date_time, arrange_time_only};
        // Verify real ticket timestamps from production sheets retain their exact time
        assert_eq!(arrange_date_time("2026-04-15 03:56 PM"), "Apr 15, 2026 · 03:56 PM");
        assert_eq!(arrange_date_time("2026-04-17 02:19 PM"), "Apr 17, 2026 · 02:19 PM");
        assert_eq!(arrange_date_time("2026-07-02 12:14 PM"), "Jul 02, 2026 · 12:14 PM");
        assert_eq!(arrange_date_time("2026-06-06 04:53 PM"), "Jun 06, 2026 · 04:53 PM");
        assert_eq!(arrange_date_time("2026-09-09 04:40 PM"), "Sep 09, 2026 · 04:40 PM");
        assert_eq!(arrange_date_time("2026-08-04 11:07 AM"), "Aug 04, 2026 · 11:07 AM");
        assert_eq!(arrange_date_time("2026-10-29 10:46 AM"), "Oct 29, 2026 · 10:46 AM");
        assert_eq!(arrange_date_time("2026-11-12 10:21 AM"), "Nov 12, 2026 · 10:21 AM");

        // Verify arrange_time_only extracts clean times
        assert_eq!(arrange_time_only("2026-11-12 10:21 AM"), "10:21 AM");
        assert_eq!(arrange_time_only("2026-04-15 03:56 PM"), "03:56 PM");
    }

    #[test]
    fn test_employee_email_normalization_and_database_sanitization() {
        let (_path, _lock) = setup_test_db();

        // 1. Test normalize_employee in memory
        let mut emp_with_division_in_email = crate::api::Employee {
            employee_code: "11093".to_string(),
            employee_name: "ABHISHEK VISHWAKARMA".to_string(),
            department: "OPERATION".to_string(),
            division: "".to_string(),
            email: "POWER PLANT DIVISION".to_string(),
        };
        crate::api::normalize_employee(&mut emp_with_division_in_email);
        assert_eq!(emp_with_division_in_email.email, "", "Email without '@' must be cleared");
        assert_eq!(emp_with_division_in_email.division, "POWER PLANT DIVISION", "Division must be preserved");

        let mut emp_swapped = crate::api::Employee {
            employee_code: "10001".to_string(),
            employee_name: "AMIT SHARMA".to_string(),
            department: "FINANCE AND ACCOUNTS".to_string(),
            division: "aks@bhagwatipower.com".to_string(),
            email: "SPONGE IRON DIVISION".to_string(),
        };
        crate::api::normalize_employee(&mut emp_swapped);
        assert_eq!(emp_swapped.email, "aks@bhagwatipower.com");
        assert_eq!(emp_swapped.division, "SPONGE IRON DIVISION");

        let mut emp_placeholder = crate::api::Employee {
            employee_code: "99999".to_string(),
            employee_name: "TEST PLACEHOLDER".to_string(),
            department: "IT".to_string(),
            division: "null".to_string(),
            email: "undefined".to_string(),
        };
        crate::api::normalize_employee(&mut emp_placeholder);
        assert_eq!(emp_placeholder.email, "");
        assert_eq!(emp_placeholder.division, "");

        // 2. Test replace_employees and DB persistence
        let emps = vec![
            emp_with_division_in_email,
            emp_swapped,
            emp_placeholder,
        ];
        db::replace_employees(&emps).expect("replace employees");

        let found = db::find_employee_by_code("11093").expect("found employee");
        assert_eq!(found.emp_email, "", "Database email must be empty");
        assert_eq!(found.division, "POWER PLANT DIVISION", "Database division must be preserved");

        let found_swapped = db::find_employee_by_code("10001").expect("found swapped");
        assert_eq!(found_swapped.emp_email, "aks@bhagwatipower.com");
        assert_eq!(found_swapped.division, "SPONGE IRON DIVISION");
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
        assert!(!db::is_reminder_completed(local_id));

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
        assert!(db::is_reminder_completed(local_id));

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

        assert!(db::is_reminder_completed(local_id));

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
        assert!(lock_timeout_err.is_retryable());

        let not_found_err = ApiError::Api {
            code: "NOT_FOUND".to_string(),
            message: "Reminder was not found.".to_string(),
            retryable: false,
            retry_after: None,
        };
        assert!(!not_found_err.is_retryable());

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

    /// Test 10: Reminder deduplication - matches unmapped row with pulled server row and merges duplicates
    #[test]
    fn test_reminder_deduplication_and_unmapped_matching() {
        let (_path, _lock) = setup_test_db();

        let input = NewReminderInput {
            employee_code: "11135".to_string(),
            employee_name: "Shreyansh Awadhiya".to_string(),
            email_address: "it@bhagwatipower.com".to_string(),
            service_category: "Software".to_string(),
            reminder_message: "testing: testing mail".to_string(),
            send_at: "2026-11-04 23:55:00".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent yet".to_string(),
            ticket_id: String::new(),
            cc_emails: String::new(),
            repeat_frequency: "once".to_string(),
        };

        let local_id = db::create_reminder_full(&input).expect("create local unmapped reminder");

        let server_r = crate::api::ServerReminder {
            reminder_id: "REM-20261104235243-86307EC2".to_string(),
            employee_code: "11135".to_string(),
            employee_name: "Shreyansh Awadhiya".to_string(),
            email_address: "it@bhagwatipower.com".to_string(),
            service_category: "Software".to_string(),
            reminder_message: "testing: testing mail".to_string(),
            send_at: "2026-11-04T18:25:00.000Z".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "2026-11-04T18:24:26.846Z".to_string(),
            ticket_id: String::new(),
            created_at: "2026-11-04T18:22:43.542Z".to_string(),
            last_updated_at: "2026-11-04T18:24:26.846Z".to_string(),
            cc_emails: String::new(),
            repeat_frequency: "once".to_string(),
        };

        // 1. Matching should find the local unmapped row
        let matched = db::find_unmapped_or_duplicate_reminder(&server_r);
        assert_eq!(matched, Some(local_id));

        // 2. If a duplicate was inserted, deduplicate_reminders_db merges them into one
        let dup_id = db::insert_server_reminder(&server_r).expect("insert duplicate");
        assert_ne!(dup_id, local_id);
        assert_eq!(db::get_all_reminders().len(), 2);

        let merged_count = db::deduplicate_reminders_db().expect("deduplicate");
        assert_eq!(merged_count, 1);

        let all = db::get_all_reminders();
        assert_eq!(all.len(), 1, "Only one reminder must remain after deduplication");
        assert_eq!(all[0].reminder_id, local_id);
        assert_eq!(all[0].server_reminder_id, "REM-20261104235243-86307EC2");
    }

    /// Test 11: Flexible employee code matching across different prefix and formatting styles
    #[test]
    fn test_emp_code_search_matching() {
        let (_path, _lock) = setup_test_db();

        // Direct matching
        assert!(db::matches_emp_query("11082", "Rajesh Kumar", "rajesh@bpsl.com", "11082"));
        assert!(db::matches_emp_query("11082", "Rajesh Kumar", "rajesh@bpsl.com", "emp-11082"));
        assert!(db::matches_emp_query("11082", "Rajesh Kumar", "rajesh@bpsl.com", "EMP 11082"));
        assert!(db::matches_emp_query("EMP-11082", "Rajesh Kumar", "rajesh@bpsl.com", "11082"));
        assert!(db::matches_emp_query("11082 (Rajesh)", "Rajesh Kumar", "rajesh@bpsl.com", "11082"));
        assert!(db::matches_emp_query("11082 (Rajesh)", "Rajesh Kumar", "rajesh@bpsl.com", "EMP-11082"));
        assert!(db::matches_emp_query("11082", "Rajesh Kumar", "rajesh@bpsl.com", "Rajesh"));
        assert!(db::matches_emp_query("11082", "Rajesh Kumar", "rajesh@bpsl.com", "rajesh@bpsl.com"));
    }

    /// Test 12: Unified arrange_date_time helper display in UI while preserving clean data
    #[test]
    fn test_12hr_format_display() {
        use crate::state::arrange_date_time;

        // ISO 8601 with explicit IST timezone
        let out1 = arrange_date_time("2026-11-06T14:30:00+05:30");
        assert_eq!(out1, "Nov 06, 2026 · 02:30 PM");

        // UTC timestamp ending in 'Z' converts to local IST (+05:30)
        // 07:00 UTC + 5h30m = 12:30 IST (the exact user scenario reported)
        let out_user = arrange_date_time("2026-11-05T07:00:00.000Z");
        assert_eq!(out_user, "Nov 05, 2026 · 12:30 PM");

        let out2 = arrange_date_time("2026-11-06T09:15:00Z");
        assert_eq!(out2, "Nov 06, 2026 · 02:45 PM");

        // Timestamps without explicit timezone are already local civil time (IST) from Indian Google Sheets / users
        let out_sheet = arrange_date_time("2026-11-05 07:00:00");
        assert_eq!(out_sheet, "Nov 05, 2026 · 07:00 AM");

        let out3 = arrange_date_time("2026-11-04 18:24:26");
        assert_eq!(out3, "Nov 04, 2026 · 06:24 PM");

        // Indian 12-hour AM/PM timestamps from sheet
        let out_ampm1 = arrange_date_time("2026-04-15 03:56 PM");
        assert_eq!(out_ampm1, "Apr 15, 2026 · 03:56 PM");

        let out_ampm2 = arrange_date_time("2026-10-29 10:46 AM");
        assert_eq!(out_ampm2, "Oct 29, 2026 · 10:46 AM");

        let out_ampm3 = arrange_date_time("2026-07-02 12:14 PM");
        assert_eq!(out_ampm3, "Jul 02, 2026 · 12:14 PM");

        // Indian DD/MM/YYYY slash format
        let out_slash = arrange_date_time("31/10/2026 16:32:00");
        assert_eq!(out_slash, "Oct 31, 2026 · 04:32 PM");

        // Date only
        let out4 = arrange_date_time("2026-11-06");
        assert_eq!(out4, "Nov 06, 2026");

        // Pre-formatted human text is preserved
        let out5 = arrange_date_time("Today · 2:00 PM");
        assert_eq!(out5, "Today · 2:00 PM");

        // parse_to_iso8601 ensures explicit +05:30 offset is sent to Google Sheets
        assert_eq!(crate::sync::parse_to_iso8601("2026-11-05 07:00:00"), "2026-11-05T07:00:00+05:30");
        assert_eq!(crate::sync::parse_to_iso8601("2026-11-05T07:00:00.000Z"), "2026-11-05T12:30:00+05:30");
        assert_eq!(crate::sync::parse_to_iso8601("2026-11-05T12:30:00+05:30"), "2026-11-05T12:30:00+05:30");

        // 24-Hour format toggle in Settings dynamically reflects in UI
        crate::state::set_time_format_24hr(true);
        let out_24hr = arrange_date_time("2026-11-05T14:30:00+05:30");
        assert_eq!(out_24hr, "Nov 05, 2026 · 14:30");
        crate::state::set_time_format_24hr(false); // Reset back to default 12-hour
    }

    /// Test 13: Application runs standalone from database without .env file
    #[test]
    fn test_standalone_database_without_env_file() {
        let (_path, _lock) = setup_test_db();

        // 1. ApiConfig loads cleanly even without .env or TMS_API_BASE_URL
        let cfg = crate::api::ApiConfig::load().expect("ApiConfig must load from database defaults");
        assert_eq!(cfg.base_url, db::DEFAULT_REST_SHEET_URL);
        assert_eq!(cfg.protocol, crate::api::ApiProtocol::BpslRest);

        // 2. Modifying sheet URL in database reflects immediately in ApiConfig
        let custom_url = "https://script.google.com/macros/s/CUSTOM_SHEET_ID/exec";
        db::save_system_api_config(custom_url, "", "10081").expect("save config to sync_meta");

        let updated_cfg = crate::api::ApiConfig::load().expect("ApiConfig must reload updated config");
        assert_eq!(updated_cfg.base_url, custom_url);
        assert_eq!(updated_cfg.employee_code.as_deref(), Some("10081"));
    }

    /// Test 14: Tracking and displaying changes fetched from Google Sheet
    #[test]
    fn test_sheet_changes_recording_and_diff_tracking() {
        let (_path, _lock) = setup_test_db();

        // 1. Clear any prior changes
        db::clear_sheet_changes().expect("clear sheet changes");
        let changes = db::get_recent_sheet_changes(10);
        assert!(changes.is_empty(), "Changes list must be empty initially");

        // 2. Record simulated changes fetched from Google Sheet
        db::record_sheet_change(
            "ticket",
            "TC-0014",
            "Fix Waybill - Invoice Lighthouse",
            "Status",
            "Open",
            "Solved(closed)",
            "2026-11-21 09:45:00",
        ).expect("record sheet change");

        db::record_sheet_change(
            "ticket",
            "TC-0014",
            "Fix Waybill - Invoice Lighthouse",
            "Priority",
            "Low",
            "High",
            "2026-11-21 09:45:00",
        ).expect("record second sheet change");

        db::record_sheet_change(
            "reminder",
            "REM-101",
            "Server backup check",
            "Status",
            "Pending",
            "Completed",
            "2026-11-21 09:46:00",
        ).expect("record reminder sheet change");

        // 3. Verify retrieved records and values
        let fetched = db::get_recent_sheet_changes(10);
        assert_eq!(fetched.len(), 3);

        // Most recent first (ordered by change_id DESC)
        assert_eq!(fetched[0].entity_id, "REM-101");
        assert_eq!(fetched[0].field_name, "Status");
        assert_eq!(fetched[0].old_value, "Pending");
        assert_eq!(fetched[0].new_value, "Completed");

        assert_eq!(fetched[1].entity_id, "TC-0014");
        assert_eq!(fetched[1].field_name, "Priority");
        assert_eq!(fetched[1].old_value, "Low");
        assert_eq!(fetched[1].new_value, "High");

        assert_eq!(fetched[2].entity_id, "TC-0014");
        assert_eq!(fetched[2].field_name, "Status");
        assert_eq!(fetched[2].old_value, "Open");
        assert_eq!(fetched[2].new_value, "Solved(closed)");

        // 4. Test clearing changes log
        db::clear_sheet_changes().expect("clear sheet changes");
        assert!(db::get_recent_sheet_changes(10).is_empty());
    }

    /// Test 16: Activity resolution for solved/closed tickets across my_activity, stage, and activities
    #[test]
    fn test_resolve_ticket_activity_and_badge_extraction() {
        use crate::pages::tickets::types::{
            extract_processed_activities, normalize_activity_badges, resolve_ticket_activity,
            strip_actioned_prefix, TicketItem,
        };

        // 1. strip_actioned_prefix
        assert_eq!(strip_actioned_prefix("Actioned: WB,SALE ORDER,AMD"), "WB,SALE ORDER,AMD");
        assert_eq!(strip_actioned_prefix("Actioned : PO,AMD"), "PO,AMD");
        assert_eq!(strip_actioned_prefix("actioned: SO,TOL"), "SO,TOL");
        assert_eq!(strip_actioned_prefix("CHANGE,CMOS Battery"), "CHANGE,CMOS Battery");

        // 2. extract_processed_activities from macro logged string
        assert_eq!(
            extract_processed_activities("Processed WB,SALE ORDER,AMD in 1h 34m"),
            Some("WB,SALE ORDER,AMD".to_string())
        );
        assert_eq!(
            extract_processed_activities("Processed SO,TOL in 22h 44m"),
            Some("SO,TOL".to_string())
        );
        assert_eq!(
            extract_processed_activities("Processed Party Bill Change,Backand in 25m"),
            Some("Party Bill Change,Backand".to_string())
        );
        assert_eq!(extract_processed_activities("Custom note without format"), None);

        // 3. normalize_activity_badges deduplicates and filters generic states
        assert_eq!(
            normalize_activity_badges("WB, SALE ORDER, WB, AMD"),
            "WB, SALE ORDER, AMD"
        );
        assert_eq!(
            normalize_activity_badges("Actioned: PO, AMD, Triage, RATE, CHANGE"),
            "PO, AMD, RATE, CHANGE"
        );

        // 4. resolve_ticket_activity with Actioned: in my_activity
        let act1 = resolve_ticket_activity(
            "Actioned: WB,SALE ORDER,AMD",
            "WB,SALE ORDER,AMD",
            "Processed WB,SALE ORDER,AMD in 1h 34m",
        );
        assert_eq!(act1, "WB, SALE ORDER, AMD");

        // 5. resolve_ticket_activity fallback to stage when my_activity is empty
        let act2 = resolve_ticket_activity(
            "",
            "PO,AMD,RATE,CHANGE",
            "Updated rate",
        );
        assert_eq!(act2, "PO, AMD, RATE, CHANGE");

        // 6. resolve_ticket_activity fallback to stage when my_activity is generic "Pending review"
        let act3 = resolve_ticket_activity(
            "Pending review",
            "Increment, Update",
            "Ticket open for 4d 1h",
        );
        assert_eq!(act3, "INCREMENT, UPDATE");

        // 7. resolve_ticket_activity fallback to activities when stage is generic Solved(closed)
        let act4 = resolve_ticket_activity(
            "",
            "Solved(closed)",
            "Processed SO,TOL in 22h 44m",
        );
        assert_eq!(act4, "SO, TOL");

        // 8. TicketItem::from_db_ticket resolves activity into item.my_activity
        let db_t = crate::db::DbTicket {
            ticket_id: 1004,
            subject: "Fly Ash WHRB".to_string(),
            stage: "WB,SALE ORDER,AMD".to_string(),
            my_activity: "Actioned: WB,SALE ORDER,AMD".to_string(),
            activities: "Processed WB,SALE ORDER,AMD in 1h 34m".to_string(),
            kanban_state: "Solved(closed)".to_string(),
            ..Default::default()
        };

        let item = TicketItem::from_db_ticket(db_t);
        assert_eq!(item.my_activity, "WB, SALE ORDER, AMD");

        // 9. backfill_resolved_ticket_activities stores resolved activities into SQLite
        let (_path, _lock) = setup_test_db();
        let conn = db::get_connection().expect("test db conn");
        conn.execute(
            "INSERT INTO tickets (ticket_id, subject, stage, my_activity, activities, kanban_state) \
             VALUES (9001, 'Test Backfill Ticket', 'WB,SALE ORDER,AMD', 'Actioned: WB,SALE ORDER,AMD', '', 'Solved(closed)')",
            [],
        ).expect("insert test row");

        db::backfill_resolved_ticket_activities(&conn).expect("backfill runs");
        let backfilled: String = conn.query_row(
            "SELECT my_activity FROM tickets WHERE ticket_id = 9001",
            [],
            |r| r.get(0),
        ).expect("query backfilled");
        assert_eq!(backfilled, "WB, SALE ORDER, AMD");

        // 10. resolve_ticket_activity prioritizes stage over generic placeholder my_activity
        let act5 = resolve_ticket_activity(
            "Actioned: ",
            "WB,OFFLINE,ENTRY",
            "",
        );
        assert_eq!(act5, "WB, OFFLINE, ENTRY");

        // 11. normalize_activity_badges retains compound words like CANCEL in PO, Cancel
        let act6 = resolve_ticket_activity(
            "",
            "PO,Cancel",
            "",
        );
        assert_eq!(act6, "PO, CANCEL");

        let act7 = resolve_ticket_activity(
            "",
            "Patch Cord Cable,Damage/Break",
            "",
        );
        assert_eq!(act7, "PATCH CORD CABLE, DAMAGE/BREAK");
    }

    #[test]
    fn test_stage_activity_preservation_in_sync_payload() {
        use crate::api::{ApiClient, ApiConfig, ApiProtocol};
        use crate::sync::sanitize_ticket_payload;

        let cfg = ApiConfig {
            base_url: "https://example.com/api".to_string(),
            api_key: "test_key".to_string(),
            protocol: ApiProtocol::LegacyV1,
            employee_code: None,
        };
        let client = ApiClient::new(cfg).unwrap();

        let payload = json!({
            "stage": "WB,OFFLINE,ENTRY",
            "my_activity": "WB,OFFLINE,ENTRY",
            "kanban_state": "Solved",
            "status": "Closed"
        });

        let sanitized = sanitize_ticket_payload(&client, &payload, false);
        // Stage must NOT be converted to "Resolution" or "New" - it must preserve the user's activity string!
        assert_eq!(sanitized["stage"], "WB,OFFLINE,ENTRY");
        assert_eq!(sanitized["my_activity"], "WB,OFFLINE,ENTRY");
        assert_eq!(sanitized["kanban_state"], "Solved");
        assert_eq!(sanitized["status"], "Closed");
    }
}
