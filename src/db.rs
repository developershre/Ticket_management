//! SQLite database management layer for BPSL IT Support.
//! Reads and writes directly to `management.db` adhering to `DATABASE.SQL`.

use rusqlite::{params, Connection, OptionalExtension, Result};
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

fn resolve_db_path() -> PathBuf {
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
/// A busy timeout is set because the background sync worker and the UI
/// thread both write to the database.
pub fn get_connection() -> Result<Connection> {
    let path = db_path();
    let conn = Connection::open(&path)?;
    conn.busy_timeout(std::time::Duration::from_secs(5))?;
    conn.execute_batch(
        "PRAGMA journal_mode = WAL; \
         PRAGMA synchronous = NORMAL; \
         PRAGMA foreign_keys = ON; \
         PRAGMA cache_size = -2000;",
    )?;
    Ok(conn)
}

// ============================================================
// Data Models matching DATABASE.SQL
// ============================================================

#[derive(Clone, Debug, PartialEq)]
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
    pub server_ticket_id: Option<String>,
    pub server_last_updated: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DbNote {
    pub id: i64,
    pub emp_code: String,
    pub content: String,
}

#[derive(Clone, Debug, PartialEq)]
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
    pub server_reminder_id: String,
    pub server_last_updated: String,
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
}

impl Default for NewReminderInput {
    fn default() -> Self {
        Self {
            employee_code: "11135".to_string(),
            employee_name: "IT Support".to_string(),
            email_address: "it.support@bpsl.net".to_string(),
            service_category: "General".to_string(),
            reminder_message: String::new(),
            send_at: "Today · 2:00 PM".to_string(),
            status: "Pending".to_string(),
            email_sent_at: "Not sent".to_string(),
            ticket_id: String::new(),
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
            last_updated_on DATETIME DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS NOTES(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            emp_code TEXT,
            content TEXT
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
            last_updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )?;

    // 1b. Google Sheets ↔ SQLite sync layer. The mapping columns are added
    //     once (SQLite ALTER TABLE has no IF NOT EXISTS, so they are checked
    //     first); the queue/meta/conflict tables are idempotent.
    ensure_ticket_sync_columns(&conn)?;
    ensure_reminder_sync_columns(&conn)?;
    ensure_sync_queue_columns(&conn)?;
    // 1c. Employee cache schema (nullable/duplicate emails + division —
    //     the shape the sheet's employees.list actually returns).
    ensure_user_schema(&conn)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS sync_queue (
            op_id INTEGER PRIMARY KEY AUTOINCREMENT,
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

    seed_employees_if_empty(&conn)?;

    Ok(())
}

fn seed_employees_if_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM USERS", [], |r| r.get(0)).unwrap_or(0);
    if count == 0 {
        let mut stmt = conn.prepare(
            "INSERT INTO USERS (emp_code, emp_name, emp_email, department, division, active, is_deleted) \
             VALUES (?1, ?2, ?3, ?4, ?5, 1, 0)",
        )?;
        for line in INITIAL_SHEET_EMPLOYEES_CSV.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("Employee Code") || line.starts_with("categories") {
                continue;
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 4 {
                let code = parts[0].trim();
                let name = parts[1].trim();
                let dept = parts[2].trim();
                let div = parts[3].trim();
                let email = if parts.len() >= 5 && !parts[4].trim().is_empty() {
                    Some(parts[4].trim())
                } else {
                    None
                };
                let _ = stmt.execute(params![code, name, email, dept, div]);
            }
        }
    }
    Ok(())
}

const INITIAL_SHEET_EMPLOYEES_CSV: &str = r#"10001,AMIT SHARMA,FINANCE AND ACCOUNTS,SPONGE IRON DIVISION,aks@bhagwatipower.com
10037,JANT SEN,FINANCE AND ACCOUNTS,SPONGE IRON DIVISION,
10054,GANESH DEWANGAN,MECHANICAL,SPONGE IRON DIVISION,
10062,DHARMENDRA SACHAN,MECHANICAL,SPONGE IRON DIVISION,
10072,RUPESH SINGH,MECH - PPD,POWER PLANT DIVISION,
10075,KHEM NAURANGE,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10081,ABHISHEK SHARMA,FINANCE AND ACCOUNTS,SPONGE IRON DIVISION,abhishek.sharma@bhagwatipower.com
10089,OMPRAKASH SAHU,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
11092,HR HEAD,HR,CORPORATE,hr@bhagwatipower.com
10100,RATAN SEN,QUALITY CONTROL,SPONGE IRON DIVISION,
10122,TRILOCHAN SEN,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10123,SEWAK SAHU,MECHANICAL,SPONGE IRON DIVISION,
10142,RAJ NISAD,AUTOMOBILE,SPONGE IRON DIVISION,
10163,SHANKAR SAHU,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
10218,SHIMPI AGARWAL,FINANCE AND ACCOUNTS,SPONGE IRON DIVISION,shimpi.agarwal@bhagwatipower.com
10219,YOGENDRA VERMA,MECHANICAL,SPONGE IRON DIVISION,
10226,SUNIL RAY,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10234,KUSHAL SHENDE,STORES,SPONGE IRON DIVISION,
10255,KUMAR SAHU,MECHANICAL,SPONGE IRON DIVISION,
10257,RAMESH VISHWAKARMA,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
10264,BINOD AGRAWAL,FINANCE AND ACCOUNTS,SPONGE IRON DIVISION,
10267,BEDRAM NIRMALKAR,PERSONNEL & ADMINISTRATION,SPONGE IRON DIVISION,
10330,SATISH KSHATRI,AUTOMOBILE,SPONGE IRON DIVISION,
10350,RAJKUMAR VERMA,AUTOMOBILE,SPONGE IRON DIVISION,
10352,ASHARAM BANJARE,MECHANICAL,SPONGE IRON DIVISION,
10385,SATANAND BAGHMAR,QUALITY CONTROL,SPONGE IRON DIVISION,
10407,KHEMRAJ VERMA,STORES,SPONGE IRON DIVISION,kverma@bhagwatipower.com
10425,THAKUR RAM,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
10476,ISHWARI PRASAD,OPERATION,POWER PLANT DIVISION,
10484,RAJKUMAR KEJRIWAL KEJRIWAL,FINANCE AND ACCOUNTS,CORPORATE DIVISION,
10497,SANJIV KUMAR,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
10513,ROSHAN LAL,STORES,SPONGE IRON DIVISION,
10528,SANTOSH KUMAR DHIWAR KUMAR DHIWAR,MECHANICAL,SPONGE IRON DIVISION,
10559,RAJNISH ROY,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
10581,MANIK RAM,QUALITY CONTROL,SPONGE IRON DIVISION,
10583,JITENDRA KUMAR,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10586,BENU RAM,QUALITY CONTROL,SPONGE IRON DIVISION,
10595,VED RAM,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10599,KESHAV PRASAD,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10616,BAISHAKHU RAM,MECHANICAL,SPONGE IRON DIVISION,
10628,SHASHIKANT BAWANKAR,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,s.bawankar@bhagwatipower.com
10633,DORI LAL,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10640,VIRENDRA KHARWAR,PERSONNEL & ADMINISTRATION,SPONGE IRON DIVISION,
10645,MANGESHWAR KUMAR,OPERATION,POWER PLANT DIVISION,
10648,VIKASH .,MECHANICAL,POWER PLANT DIVISION,
10667,NEMCHAND JAISWAL,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
10669,RAMESH KUMAR,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10672,RAMGOPAL NISHAD,MECHANICAL,SPONGE IRON DIVISION,
10682,MANORANJAN BERA,MECHANICAL,SPONGE IRON DIVISION,
10685,ASHOK KUMAR,MECHANICAL,SPONGE IRON DIVISION,
10692,ISHWAR RAM,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
10694,NARAYAN DAS,MECHANICAL,SPONGE IRON DIVISION,
10702,D.RAVI TEJA,FINANCE AND ACCOUNTS,CORPORATE DIVISION,
10711,RAKESH KUMAR,AUTOMOBILE,SPONGE IRON DIVISION,
10723,SHUBHAM AGRAWAL,PERSONNEL & ADMINISTRATION,SPONGE IRON DIVISION,
10734,BHUNESHWAR VERMA,MECHANICAL,SPONGE IRON DIVISION,
10743,DINESH MARKAM,MECHANICAL,SPONGE IRON DIVISION,
10744,SATISH KR,MECHANICAL,SPONGE IRON DIVISION,
10751,TANESHWAR SAHU,MECHANICAL,SPONGE IRON DIVISION,
10754,MOTI CHAKRADHARI,QUALITY CONTROL,SPONGE IRON DIVISION,
10756,YUVRAJ .,RAW MATERIAL PLANT,CORPORATE DIVISION,
10758,RAJESH YADAV,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10768,GHANSHYAM SAHU,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
10787,UPENDRA KUMAR,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
10791,YOGESHWER KUMAR,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10800,TORAN LAL,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
10812,PULESHWAR PATEL,MECHANICAL,SPONGE IRON DIVISION,
10829,KISHOR KUMAR,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10841,NARENDRA SAHU,SALES AND DISPATCH,CORPORATE DIVISION,
10842,JEEVAN THAKUR,STORES,CORPORATE DIVISION,
10855,KHIROD PRADHAN,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10856,JITENDRA YADAV,SALES AND DISPATCH,CORPORATE DIVISION,
10867,RAJ RAM,AUTOMOBILE,SPONGE IRON DIVISION,
10874,AMIT LAHARI,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10877,KAMLESH .,MECHANICAL,SPONGE IRON DIVISION,
10882,AJAY KUMAR,STORES,CORPORATE DIVISION,
10885,SHRAVAN KUMAR,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10887,KAVERI SAHU,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,pc1@bhagwatipower.com
10896,MAHENDRA KUMAR,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
10898,GOVIND SAHU,OPERATION,POWER PLANT DIVISION,
10904,PANKAJ .,MECHANICAL,SPONGE IRON DIVISION,
10909,RAHUL SAHU,FINANCE AND ACCOUNTS,CORPORATE DIVISION,
10911,MAHENDRA NAYAK,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10921,TARAN CHAKRDHARI,MECHANICAL,SPONGE IRON DIVISION,
10926,KAMALKISHOR DHIWAR,MECHANICAL,SPONGE IRON DIVISION,
10929,UMASHANKAR .,MECHANICAL,SPONGE IRON DIVISION,
10944,SENJEET VERMA,MECHANICAL,POWER PLANT DIVISION,
10948,MANOJ .,QUALITY CONTROL(D.M. PLANT),POWER PLANT DIVISION,
10949,MANISH BAGHEL,PRODUCTION,SPONGE IRON DIVISION,
10953,MANISH .,MECHANICAL,SPONGE IRON DIVISION,
10956,DEVID CHAKRADHARI,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10960,YUGAL YADAV,OPERATION,POWER PLANT DIVISION,
10964,LEKHRAM YADU,AUTOMOBILE,SPONGE IRON DIVISION,
10969,HIRENDRA LODHI,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
10970,GYANCHAND KASHYAP,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
10971,Arvind Singh Rathod,OPERATION,POWER PLANT DIVISION,
10974,RAMESH KEWAT,AUTOMOBILE,SPONGE IRON DIVISION,
10975,HARIGOPAL JAISWAL,AUTOMOBILE,SPONGE IRON DIVISION,
10978,PURSHOTTAM VERMA,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
10983,Digeshwar Nishad,QUALITY CONTROL(D.M. PLANT),POWER PLANT DIVISION,
10988,Komal sahu,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10989,JAYRAM SAHU,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
10990,TULENDRA KUMAR,MECHANICAL,SPONGE IRON DIVISION,
10993,SANDEEP KUMAR,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11005,SHIVSHANKAR YADAV,AUTOMOBILE,SPONGE IRON DIVISION,
11007,YAMAN SONAWANI,MECHANICAL,SPONGE IRON DIVISION,
11009,AKASH DAS,STORES,CORPORATE DIVISION,
11010,RAJESH .,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
11018,RAJAT .,MECHANICAL,SPONGE IRON DIVISION,
11020,MANOJ VERMA,MECHANICAL,SPONGE IRON DIVISION,
11021,SURESH VERMA,MECHANICAL,SPONGE IRON DIVISION,
11022,KEDARNATH VERMA,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
11023,PREETAM BANJARE,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
11024,RAKESH .,OPERATION,POWER PLANT DIVISION,
11025,DIPESH VERMA,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
11026,BALRAM SAHU,QUALITY CONTROL(D.M. PLANT),POWER PLANT DIVISION,
11027,SHRIRAM BARETH,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11034,RAMESH KUMAR,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
11036,LALIT SAHU,OPERATION,POWER PLANT DIVISION,
11039,RAVI SHYAM,OPERATION,POWER PLANT DIVISION,
11040,KAMLESH NETAM,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
11044,SAHIL BARETH,OPERATION,POWER PLANT DIVISION,
11047,SHREECHANDRA YADAV,AUTOMOBILE,SPONGE IRON DIVISION,
11048,NAGENDRA KUMAR,OPERATION,POWER PLANT DIVISION,
11049,HEMLAL SARANG,MECHANICAL,POWER PLANT DIVISION,
11050,DUSHYANT KUMAR,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11053,RAHUL .,ELECTTRICAL & INSTRUMENT (E&I),POWER PLANT DIVISION,
11054,SHAILENDRA .,OPERATION,POWER PLANT DIVISION,
11055,MITHLESH SINGH,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
11057,CHANDRAKANT SAHU,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
11058,BHUPENDRA NISHAD,MECHANICAL,SPONGE IRON DIVISION,
11060,ASHUTOSH Singh,PURCHASE,CORPORATE DIVISION,purchase@bhagwatipower.com
11061,CHAITRAM .,INFORMATION TECHNOLOGY,CORPORATE DIVISION,dme@bhagwatipower.com
11064,DEV .,OPERATION,POWER PLANT DIVISION,
11066,RITESH SAHU,MECHANICAL,POWER PLANT DIVISION,
11070,RAJU DHIWAR,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
11071,SHAILESH KUMAR,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11072,AMAN CHANDRAKAR,OPERATION,POWER PLANT DIVISION,
11076,SHUBHAM VERMA,OPERATION,POWER PLANT DIVISION,
11077,NEMICHANDRA SAHU,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
11081,BHUPENDRA DHRUW,MECHANICAL,POWER PLANT DIVISION,
11082,KISHOR KUMAR,OPERATION,POWER PLANT DIVISION,
11083,RAJAT KUMAR,MECHANICAL,POWER PLANT DIVISION,
11084,KRISHNA KUMAR,MECHANICAL,POWER PLANT DIVISION,
11085,SUNIL KUMAR,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
11086,SURYA PRAKASH,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
11087,SANJIV KUMAR,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
11088,KRISHNA KUMAR,PURCHASE,CORPORATE DIVISION,
11089,NARENDRA SAHU,PURCHASE,CORPORATE DIVISION,purchase@bhagwatipower.com
11090,RAHUL PATEL,OPERATION,POWER PLANT DIVISION,
11091,SURYA PRAKASH,OPERATION,POWER PLANT DIVISION,
11092,SHEEVENDRA .,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
11093,ABHISHEK VISHWAKARMA,OPERATION,POWER PLANT DIVISION,
11096,Uttam Kumar,SALES & DISPATCH,CORPORATE DIVISION,
11097,Ayush Upadhyay,Admin,CORPORATE DIVISION,mis@bhagwatipower.com
11098,RITESH KUMAR,Admin,CORPORATE DIVISION,
11099,KHEMCHAND RATHORE,OPERATION,POWER PLANT DIVISION,
11100,MANISH KUMAR,MECHANICAL,POWER PLANT DIVISION,
11101,TIKENDRA CHANDRABANSHI,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
11102,KHUBI RAM,MECHANICAL,POWER PLANT DIVISION,
11103,DULAR SAHU,AUTOMOBILE,SPONGE IRON DIVISION,
11104,SHUBHAM NISHAD,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
11105,PUNIT .,OPERATION,POWER PLANT DIVISION,
11106,GAJENDRA YADAV,OPERATION,POWER PLANT DIVISION,
11107,SOHAN SAHU,MECHANICAL,POWER PLANT DIVISION,
11108,R.VIGNESH .,MECHANICAL,POWER PLANT DIVISION,
11109,KHILAWAN SAHU,MECHANICAL,POWER PLANT DIVISION,
11110,SAGAR NISHAD,OPERATION,POWER PLANT DIVISION,
11111,LAKHENDRA .,ELECTRICAL DEPARTMENT(E&I),SPONGE IRON DIVISION,
11112,MANSISH CHANDRA .,OPERATION,POWER PLANT DIVISION,
11114,Raman Kumar Nirmalkar,MECHANICAL,POWER PLANT DIVISION,
11116,Bheem Singh Sahu,OPERATION,POWER PLANT DIVISION,
11117,RAVI BARMAN,OPERATION,POWER PLANT DIVISION,
11118,ROOP .,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11119,KEWALRAM .,MECHANICAL,POWER PLANT DIVISION,
11120,Ravendra Kumar .,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11121,Heera Lal .,OPERATION,POWER PLANT DIVISION,
11122,Manish Chaudhari,MECHANICAL,SPONGE IRON DIVISION,
11123,Topesh Kumar Chakradhari,MECHANICAL,SPONGE IRON DIVISION,
11126,Kunal Prasad,MECHANICAL,POWER PLANT DIVISION,
11129,Tilak Ram Verma,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
11130,DUSHYANT ,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
11131,Parasmani Deshmukh,PROCESS/PRODUCTION,POWER PLANT DIVISION,
11132,Satish Verma,ELECTRICAL DEPARTMENT(E&I),POWER PLANT DIVISION,
11133,PRAVEEN KUMAR,OPERATION,POWER PLANT DIVISION,
11135,Shreyansh Awadhiya,INFORMATION TECHNOLOGY,CORPORATE DIVISION,it@bhagwatipower.com
11136,Narendra Kumar Banjare,AUTOMOBILE,SPONGE IRON DIVISION,
11137,Niraj Kumar .,MECHANICAL,SPONGE IRON DIVISION,
11138,Narendra Jangde,MECHANICAL,SPONGE IRON DIVISION,
20003,RAM .,QUALITY CONTROL,SMS DIVISION,
20006,ANAND LAMA,MECHANICAL,SMS DIVISION,
20017,ARUN NAIR,ELECTRICAL DEPARTMENT(E&I),ROLLING MILL DIVISION,
20026,RAJKISHORE KUSHWAHA,MECHANICAL,ROLLING MILL DIVISION,
20047,RAMU SHARMA,MILL,ROLLING MILL DIVISION,
20056,SHUDAMA VISHWAKARMA,MECHANICAL,ROLLING MILL DIVISION,
20083,DINESH .,MECHANICAL,ROLLING MILL DIVISION,
20098,MANJIT .,MILL,ROLLING MILL DIVISION,
20163,CHANDRA .,MILL,ROLLING MILL DIVISION,
20210,PITAMBER .,ELECTRICAL DEPARTMENT(E&I),ROLLING MILL DIVISION,
20211,RAMSWRUP PARGNIHA,MILL,ROLLING MILL DIVISION,
20281,BHUPENDRA SAHU,MECHANICAL,ROLLING MILL DIVISION,
20286,VIKRAM TRIPATHI,SALES AND DISPATCH,CORPORATE DIVISION,dispatch@bhagwatipower.com
20288,GOVINDA RAY,MILL,ROLLING MILL DIVISION,
20306,DAYA .,MECHANICAL,ROLLING MILL DIVISION,
20315,RAVIKUMAR KARSH,MECHANICAL,ROLLING MILL DIVISION,
20325,ASHWANI SAHU,MECHANICAL,ROLLING MILL DIVISION,
20330,DEEPAK MAHTO,MECHANICAL,ROLLING MILL DIVISION,
20337,VISHNU PANIK,MILL,ROLLING MILL DIVISION,
20344,MAHENDRA MESHRAM,STORES,SMS DIVISION,
20347,MITHILESH JHA,PRODUCTION,SMS DIVISION,
20367,AKASH CHANDEL,MILL,ROLLING MILL DIVISION,
20400,SUNIL SAHU,MILL,ROLLING MILL DIVISION,
20437,SHARAD DIWAN,PRODUCTION,SMS DIVISION,sms.operations@bhagwatipower.com
20438,MANOJ KUMAR,PRODUCTION,SMS DIVISION,
20439,LAXMAN VERMA,MILL,ROLLING MILL DIVISION,
20445,SANTOSHKUMAR MANHARE,MECHANICAL,SMS DIVISION,
20459,SANJU LAL,MECHANICAL,SMS DIVISION,
20467,CHITRANJAN PATEL,QUALITY CONTROL,SMS DIVISION,
20488,GAJANAND SAHU,PRODUCTION,SMS DIVISION,
20489,AJAY KUMAR,QUALITY CONTROL,SMS DIVISION,
20495,HARISHANKAR YADAV,MECHANICAL,ROLLING MILL DIVISION,
20509,PREM .,MILL,ROLLING MILL DIVISION,
20511,PAVAN DHIWAR,QUALITY CONTROL,SMS DIVISION,
20513,RAJKUMAR PATRE,MILL,ROLLING MILL DIVISION,
20518,TARUN .,MILL,ROLLING MILL DIVISION,
20520,MANOJ VERMA,PRODUCTION,SMS DIVISION,
20521,BAJAN .,MILL,ROLLING MILL DIVISION,
20527,SATEESH .,MECHANICAL,ROLLING MILL DIVISION,
20534,VIKAS YADAV,MECHANICAL,ROLLING MILL DIVISION,
20547,RAMKUMAR KASHYAP,ELECTRICAL DEPARTMENT(E&I),SMS DIVISION,
20549,Dhirendra tiwari,MILL,ROLLING MILL DIVISION,
20551,TARUN NIRMALKAR,ELECTRICAL DEPARTMENT(E&I),ROLLING MILL DIVISION,
20552,DHANESH .,ELECTRICAL DEPARTMENT(E&I),ROLLING MILL DIVISION,
20558,RAKESH KARSH,MILL,ROLLING MILL DIVISION,
20559,RAHUL KUAMAR,MILL,ROLLING MILL DIVISION,
20560,LUCKY RAJPUT,MILL,ROLLING MILL DIVISION,
20564,PRAKASH SAHU,MECHANICAL,SMS DIVISION,
20567,PRAKASH CHANDRA,MILL,ROLLING MILL DIVISION,
20570,MEVARAM DHRUW,MECHANICAL,SMS DIVISION,
20571,PRADIP KUMAR,ELECTRICAL&INSTRUMENT (E&I) FURNACE,SMS DIVISION,
20575,KHEMHU SAHU,ELECTTRICAL & INSTRUMENT (E&I),ROLLING MILL DIVISION,
20579,VIKAS .,QUALITY CONTROL,ROLLING MILL DIVISION,
20580,MUKESH MIRI,MECHANICAL,SMS DIVISION,
20581,AMIT KUMAR,MECHANICAL,SMS DIVISION,
20583,RAMAWTAR BANJARE,MECHANICAL,SMS DIVISION,
20586,ACHCHELAL KHARWAR,QUALITY CONTROL,ROLLING MILL DIVISION,
20588,SHANKAR VERMA,MECHANICAL,SMS DIVISION,
20589,ASHIF ALI,MILL,ROLLING MILL DIVISION,
20590,SUNIL BARETH,MILL,ROLLING MILL DIVISION,
20591,TULA VERMA,MILL,ROLLING MILL DIVISION,
20592,DIWAKAR PATEL,ELECTRICAL&INSTRUMENT (E&I) FURNACE,SMS DIVISION,
20594,SOUMYA DUBEY,PRODUCTION,SMS DIVISION,
20596,GAGANDEEP .,MECHANICAL,ROLLING MILL DIVISION,
20601,KHAGESH SAHU,MECHANICAL,ROLLING MILL DIVISION,
20603,PAWAN SAHU,QUALITY CONTROL,ROLLING MILL DIVISION,
20608,SUNIL KUMAR,ELECTRICAL DEPARTMENT(E&I),SMS DIVISION,
20613,PAWAN KUMAR,ELECTRICAL&INSTRUMENT (E&I) FURNACE,SMS DIVISION,
20614,YOGESH .,MILL,ROLLING MILL DIVISION,
20617,NITIN .,MECHANICAL,ROLLING MILL DIVISION,
20619,MANISH NIRMALKAR,MILL,ROLLING MILL DIVISION,
20620,RAMESHWAR SAHU,MILL,ROLLING MILL DIVISION,
20622,GOVINDA KUMAR,MECHANICAL,ROLLING MILL DIVISION,
20624,DHANSAY KOSHLE,MECHANICAL,SMS DIVISION,
20625,LAKESHRU .,ELECTRICAL&INSTRUMENT (E&I) FURNACE,SMS DIVISION,
20626,YUGAL .,MECHANICAL,ROLLING MILL DIVISION,
20627,KRISHNA CHANDRA,MECHANICAL,ROLLING MILL DIVISION,
20628,MANTU .,MECHANICAL,ROLLING MILL DIVISION,
20629,DOMENDRA SAHU,MECHANICAL,ROLLING MILL DIVISION,
20631,OMPRAKASH SAHU,MECHANICAL,ROLLING MILL DIVISION,
20632,ANAND KUSHWAHA,MECHANICAL,ROLLING MILL DIVISION,
20633,KISHAN KUMAR KHANDE KHANDE,MECHANICAL,SMS DIVISION,
20634,YASH KUMAR CHAUHAN,MECHANICAL,SMS DIVISION,
20635,Gajendra Kumar Kaushik,MECHANICAL,SMS DIVISION,
20636,Mani Prasad .,MECHANICAL,SMS DIVISION,
20637,KUNAL DINKAR,MECHANICAL,ROLLING MILL DIVISION,
20638,Rajesh Kumar Yadav,MECHANICAL,ROLLING MILL DIVISION,
20639,Ravibhan kurmi,MILL,ROLLING MILL DIVISION,
20640,Amit .,QUALITY CONTROL,SMS DIVISION,
20642,Jitendra Kumar .,MECHANICAL,ROLLING MILL DIVISION,
20643,Nageshwar Kumar Baghel,ELECTRICAL DEPARTMENT(E&I),ROLLING MILL DIVISION,
20644,Chandrakesh Ravat,MECHANICAL,ROLLING MILL DIVISION,
20645,Yogesh Ray,MECHANICAL,ROLLING MILL DIVISION,
20646,VIJAY KUMAR UPADHYAY,MECHANICAL,SMS DIVISION,
20647,DEO KUMAR JAHIREY,MECHNICAL - FURNACE,SMS DIVISION,
20648,VIMLESH KUMAR KUSHWAHA,MILL,ROLLING MILL DIVISION,
20649,SUNIL JANGDE,MILL,ROLLING MILL DIVISION,
20650,Sunil Kumar Kaiwart,MILL,ROLLING MILL DIVISION,
20651,Laxmikant Singh,MILL,ROLLING MILL DIVISION,
20652,Tajinder Singh,MILL,ROLLING MILL DIVISION,
20653,Dhanraj .,MILL,ROLLING MILL DIVISION,
23001,N. Rath .,Admin,CORPORATE DIVISION,
23004,Tam Lal Sahu,OPERATION,POWER PLANT DIVISION,
30001,KAILASH KUMAR,STORES,CORPORATE DIVISION,
30003,MAN ANUP,MECHANICAL,SMS DIVISION,
30007,DANI RAM,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
30008,KULESHVAR PRASAD,AUTOMOBILE,SPONGE IRON DIVISION,
30009,RUPENDRA SAHU,AUTOMOBILE,SPONGE IRON DIVISION,
30010,VIJAY KUMAR,AUTOMOBILE,SPONGE IRON DIVISION,
30012,LIKHESHWAR PAIKRA,ELECTRICAL DEPARTMENT(E&I),SMS DIVISION,
30014,ESHWAR PRASAD,QUALITY CONTROL,SMS DIVISION,
30015,BHUSHAN NISHAD,QUALITY CONTROL(D.M. PLANT),POWER PLANT DIVISION,
30016,GANESH DHRUVE,MECHANICAL,SPONGE IRON DIVISION,
30018,BODHAN CHAKRADHARI,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
30021,KUMBHKARAN NIRMALKAR,MECHANICAL,SPONGE IRON DIVISION,
30022,DEVENDRA NISHAD,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
30023,BHIKHAM SAHU,MECHANICAL,SPONGE IRON DIVISION,
30024,AJAY .,MECHNICAL - FURNACE,SMS DIVISION,
30025,RAMLAL .,AUTOMOBILE,SPONGE IRON DIVISION,
30026,DAYA SHANKAR,AUTOMOBILE,SPONGE IRON DIVISION,
30029,CHANDRA PRAKASH,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
30034,DHARMENDRA KUMAR,QUALITY CONTROL(D.M. PLANT),POWER PLANT DIVISION,
30036,DINESH SONI,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
30037,MOFAT LAL,PROCESS/PRODUCTION,SPONGE IRON DIVISION,
30038,DINDAYAL SAHU,QUALITY CONTROL(D.M. PLANT),POWER PLANT DIVISION,
30041,KANHAIYALAL SAHU,OPERATION,POWER PLANT DIVISION,
30043,T. JAGANNATH,AUTOMOBILE,SPONGE IRON DIVISION,
30046,DEEPAK .,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
30049,KRISHNA SAHU,QUALITY CONTROL,SMS DIVISION,
30051,CHOLARAM .,RAW MATERIAL PLANT,SPONGE IRON DIVISION,
30052,BURAS BAG,PERSONNEL & ADMINISTRATION,CORPORATE DIVISION,
30055,RAJU SAHU,MECHANICAL,SMS DIVISION,
30059,BIRSINGH SINHA,STORES,SMS DIVISION,
30065,GOVIND PAL,OPERATION,POWER PLANT DIVISION,
30067,VIVEK VERMA,QUALITY CONTROL,SPONGE IRON DIVISION,
30080,RITESH KASHYAP,STORES,SMS DIVISION,"#;

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
            server_ticket_id: row.get(23)?,
            server_last_updated: row.get(24)?,
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
            employee_code: "11135".to_string(),
            email_address: "it.support@bpsl.net".to_string(),
            category: "General".to_string(),
            subject: String::new(),
            problem_description: String::new(),
            attachment_name: String::new(),
            attachment_url: String::new(),
            priority: "Medium".to_string(),
            assigned_to: "IT Support".to_string(),
            customer: "Internal".to_string(),
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
    conn.execute(
        r#"
        INSERT INTO tickets (
            employee_code, email_address, category, subject, problem_description,
            attachment_name, attachment_url, priority, assigned_to, customer,
            time_spent, activities, my_activity, deadline, sla_status, sla_deadline,
            rating_avg_text, kanban_state, stage, status, created_on, last_updated_on
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10,
            ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
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
        ],
    )?;
    Ok(conn.last_insert_rowid())
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
    let input = NewTicketInput {
        employee_code: employee_code.to_string(),
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
        Ok(DbEmployee {
            emp_id: row.get(0)?,
            emp_code: row.get(1)?,
            emp_name: row.get(2)?,
            emp_email: row.get(3)?,
            department: row.get(4)?,
            division: row.get(5)?,
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Lookup a single employee by their employee code.
pub fn find_employee_by_code(code: &str) -> Option<DbEmployee> {
    let conn = get_connection().ok()?;
    conn.query_row(
        "SELECT emp_id, emp_code, emp_name, emp_email, department, division FROM USERS WHERE emp_code = ?1 LIMIT 1",
        params![code],
        |row| {
            Ok(DbEmployee {
                emp_id: row.get(0)?,
                emp_code: row.get(1)?,
                emp_name: row.get(2)?,
                emp_email: row.get(3)?,
                department: row.get(4)?,
                division: row.get(5)?,
            })
        },
    )
    .optional()
    .ok()
    .flatten()
}

/// Number of cached employees (sidebar badge).
pub fn employee_count() -> usize {
    let conn = match get_connection() {
        Ok(c) => c,
        Err(_) => return 0,
    };
    conn.query_row(
        "SELECT COUNT(*) FROM USERS WHERE is_deleted = 0",
        [],
        |r| r.get::<_, i64>(0),
    )
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
            let email = if e.email.trim().is_empty() {
                None
            } else {
                Some(e.email.trim())
            };
            stmt.execute(params![
                e.employee_code.trim(),
                e.employee_name.trim(),
                email,
                e.department.trim(),
                e.division.trim(),
            ])?;
        }
    }
    tx.commit()?;
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
            COALESCE(server_last_updated, '')
        FROM reminders
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
        })
    });

    match rows {
        Ok(iter) => iter.filter_map(Result::ok).collect(),
        Err(_) => Vec::new(),
    }
}

/// Creates a new reminder with all 12 fields and returns the autogenerated `reminder_id`.
pub fn create_reminder_full(input: &NewReminderInput) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        INSERT INTO reminders (
            employee_code, employee_name, email_address, service_category,
            reminder_message, send_at, status, email_sent_at, ticket_id,
            created_at, last_updated_at
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
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
            last_updated_at = CURRENT_TIMESTAMP
        WHERE reminder_id = ?10
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
            reminder_id,
        ],
    )?;
    Ok(())
}

/// Deletes a reminder by reminder_id.
pub fn delete_reminder_db(reminder_id: i64) -> Result<()> {
    let conn = get_connection()?;
    conn.execute("DELETE FROM reminders WHERE reminder_id = ?1", params![reminder_id])?;
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
        conn.execute("ALTER TABLE tickets ADD COLUMN server_last_updated TEXT", [])?;
    }
    Ok(())
}

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
        conn.execute("ALTER TABLE reminders ADD COLUMN server_reminder_id TEXT", [])?;
    }
    if !has_column("server_last_updated") {
        conn.execute("ALTER TABLE reminders ADD COLUMN server_last_updated TEXT", [])?;
    }
    Ok(())
}

fn ensure_sync_queue_columns(conn: &Connection) -> Result<()> {
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
        conn.execute("ALTER TABLE sync_queue ADD COLUMN entity_type TEXT NOT NULL DEFAULT 'ticket'", [])?;
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
    Ok(rows.collect::<Result<Vec<_>>>()?)
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
    insert_entity_op(conn, "ticket", local_ticket_id, server_ticket_id, op_type, payload, created_at)
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
        params![local_id, server_id, op_type, payload.to_string(), created_at, entity_type],
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
    insert_op(&conn, local_ticket_id, server_ticket_id.as_deref(), "UPDATE", fields, created_at)
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
    Ok(rows.collect::<Result<Vec<_>>>()?)
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
pub fn mark_op_error(
    op_id: i64,
    error: &str,
    retryable: bool,
    next_attempt_at: i64,
) -> Result<()> {
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

/// Resets any previously failed operations in sync_queue to pending so they can retry.
pub fn reset_failed_sync_ops() -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE sync_queue SET status = 'pending', next_attempt_at = NULL, last_error = NULL WHERE status = 'failed'",
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
        .filter_map(|v| v.get("subject").and_then(|s| s.as_str()).map(str::to_string))
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

// ---- ticket ↔ server mapping ----

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
            if !email.is_empty() { map.insert("email_address".to_string(), json!(email)); }
            if !cat.is_empty() { map.insert("category".to_string(), json!(cat)); }
            if !desc.is_empty() { map.insert("problem_description".to_string(), json!(desc)); }
            if !prio.is_empty() { map.insert("priority".to_string(), json!(prio)); }
            if !assigned.is_empty() { map.insert("assigned_to".to_string(), json!(assigned)); }
            if !cust.is_empty() { map.insert("customer".to_string(), json!(cust)); }
            if !status.is_empty() { map.insert("status".to_string(), json!(status)); }
            if !stage.is_empty() { map.insert("stage".to_string(), json!(stage)); }
            if !kanban.is_empty() { map.insert("kanban_state".to_string(), json!(kanban)); }
            if !att_name.is_empty() { map.insert("attachment_name".to_string(), json!(att_name)); }
            if !att_url.is_empty() { map.insert("attachment_url".to_string(), json!(att_url)); }
            if time_spent > 0.0 { map.insert("time_spent".to_string(), json!(time_spent)); }
            if !activities.is_empty() {
                map.insert("activities".to_string(), json!(activities));
                map.insert("activities_text".to_string(), json!(activities));
            }
            if !my_act.is_empty() { map.insert("my_activity".to_string(), json!(my_act)); }
            if !deadline.is_empty() { map.insert("deadline".to_string(), json!(deadline)); }
            if !sla_status.is_empty() { map.insert("sla_status".to_string(), json!(sla_status)); }
            if !sla_deadline.is_empty() { map.insert("sla_deadline".to_string(), json!(sla_deadline)); }
            if !rating.is_empty() { map.insert("rating_avg_text".to_string(), json!(rating)); }

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
    conn.execute(
        r#"
        INSERT INTO reminders (
            employee_code, employee_name, email_address, service_category,
            reminder_message, send_at, status, email_sent_at, ticket_id,
            created_at, last_updated_at, server_reminder_id, server_last_updated
        ) VALUES (
            ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13
        )
        "#,
        params![
            r.employee_code,
            r.employee_name,
            r.email_address,
            r.service_category,
            r.reminder_message,
            r.send_at,
            r.status,
            r.email_sent_at,
            r.ticket_id,
            r.created_at,
            r.last_updated_at,
            r.reminder_id,
            r.last_updated_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn apply_server_reminder(local_reminder_id: i64, r: &crate::api::ServerReminder) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        r#"
        UPDATE reminders SET
            employee_code = ?1, employee_name = ?2, email_address = ?3,
            service_category = ?4, reminder_message = ?5, send_at = ?6,
            status = ?7, email_sent_at = ?8, ticket_id = ?9,
            created_at = ?10, last_updated_at = ?11,
            server_reminder_id = ?12, server_last_updated = ?13
        WHERE reminder_id = ?14
        "#,
        params![
            r.employee_code,
            r.employee_name,
            r.email_address,
            r.service_category,
            r.reminder_message,
            r.send_at,
            r.status,
            r.email_sent_at,
            r.ticket_id,
            r.created_at,
            r.last_updated_at,
            r.reminder_id,
            r.last_updated_at,
            local_reminder_id,
        ],
    )?;
    Ok(())
}

pub fn reminder_create_payload(local_reminder_id: i64) -> Option<Value> {
    let conn = get_connection().ok()?;
    let mut stmt = conn
        .prepare(
            "SELECT employee_code, service_category, reminder_message, send_at \
             FROM reminders WHERE reminder_id = ?1",
        )
        .ok()?;

    let value = stmt
        .query_row(params![local_reminder_id], |r| {
            let emp_code: String = r.get(0)?;
            let cat: String = r.get(1)?;
            let msg: String = r.get(2)?;
            let send_at: String = r.get(3)?;

            let mut map = serde_json::Map::new();
            map.insert("employee_code".to_string(), json!(emp_code));
            map.insert("service_category".to_string(), json!(cat));
            map.insert("reminder_message".to_string(), json!(msg));
            map.insert("send_at".to_string(), json!(send_at));

            Ok(Value::Object(map))
        })
        .optional()
        .ok()
        .flatten()?;

    Some(value)
}

pub fn enqueue_reminder_create(local_reminder_id: i64, payload: &Value, created_at: &str) -> Result<i64> {
    let conn = get_connection()?;
    insert_entity_op(&conn, "reminder", local_reminder_id, None, "CREATE", payload, created_at)
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

// ============================================================
// Employee DB helpers (for local cache modifications)
// ============================================================

pub fn create_employee_db(emp: &crate::api::Employee) -> Result<i64> {
    let conn = get_connection()?;
    conn.execute(
        "INSERT INTO USERS (emp_code, emp_name, emp_email, department, division, active, is_deleted) \
         VALUES (?1, ?2, ?3, ?4, ?5, 1, 0)",
        params![
            emp.employee_code.trim(),
            emp.employee_name.trim(),
            if emp.email.trim().is_empty() { None } else { Some(emp.email.trim()) },
            emp.department.trim(),
            emp.division.trim(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_employee_db(
    emp_code: &str,
    emp_name: &str,
    emp_email: &str,
    department: &str,
    division: &str,
) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE USERS SET emp_name = ?1, emp_email = ?2, department = ?3, division = ?4 \
         WHERE emp_code = ?5 AND is_deleted = 0",
        params![
            emp_name.trim(),
            if emp_email.trim().is_empty() { None } else { Some(emp_email.trim()) },
            department.trim(),
            division.trim(),
            emp_code.trim(),
        ],
    )?;
    Ok(())
}

pub fn delete_employee_db(emp_code: &str) -> Result<()> {
    let conn = get_connection()?;
    conn.execute(
        "UPDATE USERS SET is_deleted = 1 WHERE emp_code = ?1",
        params![emp_code.trim()],
    )?;
    Ok(())
}

