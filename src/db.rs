//! SQLite database management layer for TaskFlow IT Management.
//! Reads and writes directly to `management.db` adhering to `DATABASE.SQL`.

use rusqlite::{params, Connection, Result};
use std::path::{Path, PathBuf};

/// Get the path to `management.db`
pub fn db_path() -> PathBuf {
    let local = Path::new("management.db");
    if local.exists() {
        return local.to_path_buf();
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let candidate = parent.join("management.db");
            if candidate.exists() {
                return candidate;
            }
            if let Some(up) = parent.parent() {
                let candidate2 = up.join("management.db");
                if candidate2.exists() {
                    return candidate2;
                }
            }
        }
    }
    PathBuf::from("management.db")
}

/// Open a connection with WAL mode enabled for smooth concurrency.
pub fn get_connection() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")?;
    Ok(conn)
}

// ============================================================
// Data Models matching DATABASE.SQL
// ============================================================

#[derive(Clone, Debug, PartialEq)]
pub struct DbUser {
    pub emp_id: i64,
    pub emp_code: i64,
    pub emp_name: String,
    pub emp_email: String,
    pub department: String,
    pub active: bool,
    pub is_deleted: bool,
}

#[derive(Clone, Debug, PartialEq)]
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
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbNote {
    pub id: i64,
    pub emp_code: String,
    pub content: String,
}

// ============================================================
// Database Initialization & Seeding
// ============================================================

/// Ensures tables from `DATABASE.SQL` exist and seeds initial data if tables are empty.
pub fn init_database() -> Result<()> {
    let conn = get_connection()?;

    // 1. Create tables if not exists (from DATABASE.SQL)
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS USERS(
            emp_id INTEGER PRIMARY KEY AUTOINCREMENT,
            emp_code INTEGER NOT NULL,
            emp_name TEXT NOT NULL,
            emp_email TEXT UNIQUE NOT NULL,
            department TEXT,
            active BOOLEAN,
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
            last_updated_on DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS NOTES(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            emp_code TEXT,
            content TEXT
        );
        "#,
    )?;

    // 2. Seed initial users if empty
    let user_count: i64 = conn.query_row("SELECT count(*) FROM USERS", [], |r| r.get(0))?;
    if user_count == 0 {
        let initial_users = [
            (4019, "David Chen", "david.chen@corp.internal", "Engineering", true),
            (5082, "Sarah Miller", "sarah.miller@corp.internal", "Hardware", true),
            (1024, "Robert Taylor", "robert.taylor@corp.internal", "Finance", true),
            (3011, "Elena Rostova", "elena.rostova@corp.internal", "DevOps", true),
            (1001, "Gamer Rock", "admin@taskflow.internal", "IT Operations", true),
            (1190, "James Smith", "james.smith@corp.internal", "General Ops", true),
        ];

        for (code, name, email, dept, active) in initial_users {
            conn.execute(
                "INSERT INTO USERS (emp_code, emp_name, emp_email, department, active, is_deleted) VALUES (?1, ?2, ?3, ?4, ?5, 0)",
                params![code, name, email, dept, active],
            )?;
        }
    }

    // 3. Seed initial IT tickets if empty
    let ticket_count: i64 = conn.query_row("SELECT count(*) FROM tickets", [], |r| r.get(0))?;
    if ticket_count == 0 {
        let seed_tickets = [
            (
                "EMP-4019 (David Chen)",
                "david.chen@corp.internal",
                "Network",
                "VPN connection timeout on Cisco AnyConnect",
                "User reports tunnel disconnection every 15 minutes while accessing internal database clusters.",
                "High",
                "GR",
                "InProgress",
                "Today · 09:15 AM",
            ),
            (
                "EMP-5082 (Sarah Miller)",
                "sarah.miller@corp.internal",
                "Hardware",
                "MacBook Pro M3 workstation setup & provisioning",
                "New engineer onboarding workstation setup with standard dev toolchain and MDM enroll.",
                "Medium",
                "AL",
                "Pending",
                "Today · 10:30 AM",
            ),
            (
                "EMP-1024 (Robert Taylor)",
                "robert.taylor@corp.internal",
                "Access",
                "Active Directory account lock after password expiry",
                "Password expired yesterday; password reset self-service triggered max retry lockout.",
                "High",
                "JD",
                "Pending",
                "Today · 11:00 AM",
            ),
            (
                "EMP-3011 (Elena Rostova)",
                "elena.rostova@corp.internal",
                "Security",
                "SSL certificate expiring on internal staging gateway",
                "Wildcard certificate *.staging.corp.internal expires in 3 days; renewal needed via Certbot.",
                "High",
                "SL",
                "InProgress",
                "Yesterday · 02:20 PM",
            ),
            (
                "EMP-2201 (Mark Vance)",
                "mark.vance@corp.internal",
                "Hardware",
                "Replacement battery requested for Dell Latitude",
                "Battery health at 42%; swelling reported on lower trackpad housing.",
                "Low",
                "GR",
                "Onhold",
                "2 days ago",
            ),
            (
                "EMP-4109 (Lisa Wong)",
                "lisa.wong@corp.internal",
                "Access",
                "Awaiting manager approval for Production AWS IAM role",
                "Role requires secondary approval from engineering VP before granting assume-role credentials.",
                "Medium",
                "JD",
                "Onhold",
                "3 days ago",
            ),
            (
                "EMP-1190 (James Smith)",
                "james.smith@corp.internal",
                "Hardware",
                "Floor 3 Kyocera network printer offline",
                "Paper jam cleared, network interface reset, self-test page printing normally.",
                "Medium",
                "AL",
                "Completed",
                "Oct 28 · 04:40 PM",
            ),
            (
                "EMP-3402 (Alex Kumar)",
                "alex.kumar@corp.internal",
                "Software",
                "Docker Engine fails to start on Ubuntu 24.04 dev VM",
                "systemd service reports iptables backend mismatch with nftables; updated daemon config.",
                "Medium",
                "SL",
                "InProgress",
                "Oct 29 · 11:15 AM",
            ),
            (
                "EMP-8021 (Finance Team)",
                "finance-desk@corp.internal",
                "Network",
                "Wi-Fi access point degraded in Conference Room C",
                "Signal dropouts observed during large video calls; AP scheduled for firmware reboot.",
                "High",
                "GR",
                "Pending",
                "Today · 08:45 AM",
            ),
            (
                "EMP-9012 (DevOps Team)",
                "devops@corp.internal",
                "Infrastructure",
                "Upgrade PostgreSQL replica memory allocation",
                "Replica instance resized to 64GB RAM; replication lag resolved to <5ms.",
                "High",
                "SL",
                "Completed",
                "Oct 27 · 06:00 PM",
            ),
            (
                "EMP-1000 (SecOps)",
                "secops@corp.internal",
                "Security",
                "Security patch deployment for Windows 11 fleet",
                "Mandatory KB security rollups deployed across all 180 corporate endpoints.",
                "Medium",
                "JD",
                "Completed",
                "Oct 26 · 02:00 PM",
            ),
            (
                "EMP-6120 (Emily Davis)",
                "emily.davis@corp.internal",
                "Hardware",
                "External monitor flicker over DisplayPort adapter",
                "Awaiting warranty replacement cable shipment from vendor.",
                "Low",
                "AL",
                "Onhold",
                "Yesterday · 04:00 PM",
            ),
        ];

        for (emp, email, cat, subj, desc, prio, assignee, state, created) in seed_tickets {
            conn.execute(
                r#"
                INSERT INTO tickets (
                    employee_code, email_address, category, subject, problem_description,
                    priority, assigned_to, kanban_state, status, created_on, last_updated_on
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'Open', ?9, CURRENT_TIMESTAMP)
                "#,
                params![emp, email, cat, subj, desc, prio, assignee, state, created],
            )?;
        }
    }

    // 4. Seed initial notes if empty
    let note_count: i64 = conn.query_row("SELECT count(*) FROM NOTES", [], |r| r.get(0))?;
    if note_count == 0 {
        let seed_notes = [
            ("EMP-1001", "Review pull request #482 — Cisco VPN tunnel fixes"),
            ("EMP-1001", "Team standup notes & updates for sprint 24"),
            ("EMP-1001", "Renew corporate domain + SSL certificates"),
            ("EMP-1001", "Send weekly IT incident progress summary to leadership"),
            ("EMP-1001", "Sprint backlog grooming & capacity planning"),
            ("EMP-1001", "Clean up unused Docker images and staging volumes"),
        ];
        for (code, content) in seed_notes {
            conn.execute(
                "INSERT INTO NOTES (emp_code, content) VALUES (?1, ?2)",
                params![code, content],
            )?;
        }
    }

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
            COALESCE(last_updated_on, CURRENT_TIMESTAMP)
        FROM tickets
        ORDER BY ticket_id ASC
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
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Updates the kanban_state, stage, status, and last_updated_on of a ticket in `tickets` table.
pub fn update_ticket_status(ticket_id: i64, new_kanban_state: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        UPDATE tickets
        SET kanban_state = ?1, stage = ?1, status = ?1, last_updated_on = CURRENT_TIMESTAMP
        WHERE ticket_id = ?2
        "#,
        params![new_kanban_state, ticket_id],
    )?;
    Ok(())
}

/// Inserts a new ticket into `tickets` table and returns the generated `ticket_id`.
pub fn create_ticket(
    subject: &str,
    category: &str,
    priority: &str,
    employee_code: &str,
    assigned_to: &str,
    kanban_state: &str,
    description: &str,
) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        INSERT INTO tickets (
            employee_code, category, subject, problem_description,
            priority, assigned_to, kanban_state, status, created_on, last_updated_on
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'Open', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        "#,
        params![
            employee_code,
            category,
            subject,
            description,
            priority,
            assigned_to,
            kanban_state
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Deletes a ticket by `ticket_id`.
pub fn delete_ticket(ticket_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM tickets WHERE ticket_id = ?1", params![ticket_id])?;
    Ok(())
}

/// Updates the priority of a ticket.
pub fn update_ticket_priority(ticket_id: i64, new_priority: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET priority = ?1, last_updated_on = CURRENT_TIMESTAMP WHERE ticket_id = ?2",
        params![new_priority, ticket_id],
    )?;
    Ok(())
}

/// Updates the category of a ticket.
pub fn update_ticket_category(ticket_id: i64, new_category: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET category = ?1, last_updated_on = CURRENT_TIMESTAMP WHERE ticket_id = ?2",
        params![new_category, ticket_id],
    )?;
    Ok(())
}

/// Updates the assignee of a ticket.
pub fn update_ticket_assignee(ticket_id: i64, new_assignee: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE tickets SET assigned_to = ?1, last_updated_on = CURRENT_TIMESTAMP WHERE ticket_id = ?2",
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
            last_updated_on = CURRENT_TIMESTAMP
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

    let mut stmt = match conn.prepare("SELECT id, COALESCE(emp_code, 'EMP-1001'), COALESCE(content, '') FROM NOTES ORDER BY id ASC") {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok(DbNote {
            id: row.get(0)?,
            emp_code: row.get(1)?,
            content: row.get(2)?,
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
        "INSERT INTO NOTES (emp_code, content) VALUES (?1, ?2)",
        params![emp_code, content],
    )?;
    Ok(conn.last_insert_rowid())
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

// ============================================================
// Users Queries & Operations
// ============================================================

/// Fetches all active users from `USERS` table.
pub fn get_all_users() -> Vec<DbUser> {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut stmt = match conn.prepare(
        r#"
        SELECT
            emp_id, emp_code, emp_name, emp_email,
            COALESCE(department, 'Operations'),
            COALESCE(active, 1),
            COALESCE(is_deleted, 0)
        FROM USERS
        WHERE is_deleted = 0
        ORDER BY emp_id ASC
        "#,
    ) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let rows = stmt.query_map([], |row| {
        Ok(DbUser {
            emp_id: row.get(0)?,
            emp_code: row.get(1)?,
            emp_name: row.get(2)?,
            emp_email: row.get(3)?,
            department: row.get(4)?,
            active: row.get(5)?,
            is_deleted: row.get(6)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}
