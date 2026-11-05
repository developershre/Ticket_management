const sqlite3 = require('node:sqlite');
const fs = require('fs');
const path = require('path');

const dbPath = path.resolve(process.argv[2] || 'management.db');
if (fs.existsSync(dbPath)) {
    try { fs.unlinkSync(dbPath); } catch (_) {}
}

const db = new sqlite3.DatabaseSync(dbPath);

// Execute exact full schema
db.exec(`
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
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    server_note_id TEXT,
    server_last_updated TEXT,
    is_deleted BOOLEAN DEFAULT 0
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
CREATE INDEX IF NOT EXISTS idx_sync_queue_due ON sync_queue(status, next_attempt_at);

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
CREATE INDEX IF NOT EXISTS idx_sync_conflicts ON sync_conflicts(status, local_ticket_id);
`);

db.exec("VACUUM;");

const tables = db.prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name").all();
console.log(`[generate_empty_db] Tables initialized in ${path.basename(dbPath)}:`);
for (const t of tables) {
    if (t.name.startsWith('sqlite_')) continue;
    const count = db.prepare(`SELECT COUNT(*) as c FROM ${t.name}`).get().c;
    console.log(`  - ${t.name}: ${count} rows`);
    if (count !== 0) {
        console.error(`ERROR: Table ${t.name} is not empty! Count: ${count}`);
        process.exit(1);
    }
}

db.close();
console.log(`[generate_empty_db] Success: ${path.basename(dbPath)} contains all schema tables with 0 rows.`);

