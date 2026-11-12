use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Overall request timeout. Google Apps Script can take 60-90+ seconds to
/// cold-start its V8 engine and initialize spreadsheet connections, so this
/// is bounded to 120 seconds.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);

/// Maximum honored `Retry-After` value (seconds).
const MAX_RETRY_AFTER: u64 = 86_400;

// ============================================================
// Configuration
// ============================================================

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub base_url: String,
    pub api_key: String,
    /// Optional employee code used for `tickets.create`: the sheet requires
    /// `employee_code` to reference an existing employee row. Set via the
    /// `TMS_EMPLOYEE_CODE` environment variable.
    pub employee_code: Option<String>,
}

#[derive(Deserialize)]
struct CollectionFile {
    #[serde(default)]
    variable: Vec<CollectionVariable>,
}

#[derive(Deserialize)]
struct CollectionVariable {
    key: String,
    value: String,
}

impl ApiConfig {
    /// Resolution order:
    /// 1. User-configured custom settings saved in SQLite `sync_meta`
    /// 2. `TMS_API_BASE_URL`, `TMS_API_KEY`, `TMS_EMPLOYEE_CODE` env vars or `.env` file
    /// 3. `base_url` / `api_key` variables of `TMS_API.postman_collection.json`
    ///    (searched in working dir, next to executable and one level above)
    pub fn load() -> Result<ApiConfig, String> {
        let (sys_url, sys_key, sys_emp) = crate::db::get_system_api_config();
        let (dot_url, dot_key, dot_emp) = Self::from_dotenv_file();
        let env_url = sys_url.or_else(|| non_empty_env("TMS_API_BASE_URL")).or(dot_url);
        let env_key = sys_key.or_else(|| non_empty_env("TMS_API_KEY")).or(dot_key);
        let employee_code = sys_emp.or_else(|| non_empty_env("TMS_EMPLOYEE_CODE")).or(dot_emp);

        let (file_url, file_key) = Self::from_collection_file();

        let base_url = env_url
            .or(file_url)
            .ok_or_else(|| {
                "Missing TMS_API_BASE_URL. Please define TMS_API_BASE_URL in your .env file or environment.".to_string()
            })?;
        let api_key = env_key
            .or(file_key)
            .ok_or_else(|| {
                "Missing TMS_API_KEY. Please define TMS_API_KEY in your .env file or environment.".to_string()
            })?;

        Ok(ApiConfig {
            base_url: base_url.trim().to_string(),
            api_key: api_key.trim().to_string(),
            employee_code,
        })
    }

    /// Reads key-value pairs from `.env` in candidate paths.
    fn from_dotenv_file() -> (Option<String>, Option<String>, Option<String>) {
        for path in Self::candidate_paths(".env") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let mut url = None;
            let mut key = None;
            let mut emp = None;
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = trimmed.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'').trim();
                    if !v.is_empty() {
                        match k {
                            "TMS_API_BASE_URL" => url = Some(v.to_string()),
                            "TMS_API_KEY" => key = Some(v.to_string()),
                            "TMS_EMPLOYEE_CODE" => emp = Some(v.to_string()),
                            _ => {}
                        }
                    }
                }
            }
            if url.is_some() || key.is_some() || emp.is_some() {
                return (url, key, emp);
            }
        }
        (None, None, None)
    }

    /// Reads the `base_url` and `api_key` collection variables from the
    /// Postman collection file, if any candidate location contains it.
    fn from_collection_file() -> (Option<String>, Option<String>) {
        for path in Self::candidate_paths("TMS_API.postman_collection.json") {
            let Ok(bytes) = std::fs::read(&path) else {
                continue;
            };
            let Ok(file) = serde_json::from_slice::<CollectionFile>(&bytes) else {
                continue;
            };
            let mut url = None;
            let mut key = None;
            for var in file.variable {
                match var.key.as_str() {
                    "base_url" => url = Some(var.value),
                    "api_key" => key = Some(var.value),
                    _ => {}
                }
            }
            if url.is_some() || key.is_some() {
                return (url, key);
            }
        }
        (None, None)
    }

    fn candidate_paths(file_name: &str) -> Vec<PathBuf> {
        let mut out = vec![PathBuf::from(file_name)];
        if let Ok(cwd) = std::env::current_dir() {
            out.push(cwd.join(file_name));
        }
        if let Some(app_data) = std::env::var_os("APPDATA").or_else(|| std::env::var_os("LOCALAPPDATA")) {
            out.push(PathBuf::from(app_data).join("BPSL IT Support").join(file_name));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                out.push(dir.join(file_name));
                if let Some(parent) = dir.parent() {
                    out.push(parent.join(file_name));
                }
            }
        }
        out
    }
}

fn non_empty_env(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

// ============================================================
// Errors
// ============================================================

/// Everything that can go wrong with one API call. `is_retryable` drives
/// the sync worker's exponential backoff decisions.
#[derive(Clone, Debug)]
pub enum ApiError {
    /// Transport-level failure (DNS, connection refused, TLS â€¦).
    Network(String),
    Timeout,
    /// Non-2xx HTTP status.
    Http {
        status: u16,
        retryable: bool,
        retry_after: Option<u64>,
        body: String,
    },
    /// `success: false` envelope from the API (`error.retryable` honored).
    Api {
        code: String,
        message: String,
        retryable: bool,
        retry_after: Option<u64>,
    },
    /// Response body was not the expected JSON envelope.
    Decode(String),
    /// Local SQLite failure while recording sync bookkeeping (busy/locked
    /// database) â€” transient, retried next cycle.
    Db(String),
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Network(_) | ApiError::Timeout | ApiError::Decode(_) | ApiError::Db(_) => {
                true
            }
            ApiError::Http { retryable, .. } => *retryable,
            ApiError::Api { retryable, .. } => *retryable,
        }
    }

    /// `Retry-After` hint (seconds) when the server provided one.
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            ApiError::Http { retry_after, .. } | ApiError::Api { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// True when the API reported "ticket not found" â€” an idempotent
    /// success for DELETE, a recorded conflict for UPDATE.
    pub fn is_not_found(&self) -> bool {
        matches!(self, ApiError::Api { code, .. } if code == "NOT_FOUND")
    }

    /// Compact single-line representation stored in `sync_queue.last_error`
    /// and shown in the sync status pill.
    pub fn short(&self) -> String {
        format!("{self}").chars().take(300).collect()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(m) => write!(f, "NETWORK: {m}"),
            ApiError::Timeout => {
                write!(
                    f,
                    "TIMEOUT: no response within {}s",
                    REQUEST_TIMEOUT.as_secs()
                )
            }
            ApiError::Http { status, .. } => write!(f, "HTTP {status}"),
            ApiError::Api { code, message, .. } => write!(f, "{code}: {message}"),
            ApiError::Decode(m) => write!(f, "DECODE: {m}"),
            ApiError::Db(m) => write!(f, "DB: {m}"),
        }
    }
}

impl std::error::Error for ApiError {}

/// Local SQLite failures surface as a retryable `ApiError` so the sync
/// worker's `?` conversions stay uniform (a busy/locked database is
/// transient and must be retried next cycle, never discarded).
impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        ApiError::Db(e.to_string())
    }
}

/// Parses the integer-seconds form of `Retry-After` (HTTP-date form is
/// ignored â€” the fallback backoff applies instead).
fn parse_retry_after(v: &str) -> Option<u64> {
    v.trim().parse::<u64>().ok().map(|s| s.min(MAX_RETRY_AFTER))
}

// ============================================================
// Request / response envelopes
// ============================================================

#[derive(Serialize)]
struct Auth<'a> {
    api_key: &'a str,
}

#[derive(Serialize)]
struct RequestEnvelope<'a> {
    version: &'static str,
    action: &'a str,
    auth: Auth<'a>,
    request_id: String,
    data: Value,
}

#[derive(Deserialize)]
struct ResponseEnvelope {
    success: bool,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    error: Option<ErrorBody>,
}

#[derive(Deserialize)]
struct ErrorBody {
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    retryable: Option<bool>,
}

// ============================================================
// Server-side ticket row (shape verified against the live API)
// ============================================================

fn flex_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string or number")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_unit<E: de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }
        fn visit_none<E: de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }
        fn visit_some<D2: de::Deserializer<'de>>(self, d: D2) -> Result<String, D2::Error> {
            d.deserialize_string(V)
        }
    }
    deserializer.deserialize_any(V)
}

/// Deserializes a field that the sheet/API may return as a plain string
/// (`"a@b.c, d@e.f"`) or as an array (`["a@b.c", "d@e.f"]`) into a single
/// comma-separated string. Used for `cc_emails` (sheet column `CC Emails`).
fn flex_list_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = String;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a string, number or array of strings")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<String, E> {
            Ok(v.to_string())
        }
        fn visit_none<E: de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }
        fn visit_some<D2: de::Deserializer<'de>>(self, d: D2) -> Result<String, D2::Error> {
            d.deserialize_string(V)
        }
        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<String, A::Error> {
            let mut parts: Vec<String> = Vec::new();
            while let Some(item) = seq.next_element::<String>()? {
                let item = item.trim().to_string();
                if !item.is_empty() {
                    parts.push(item);
                }
            }
            Ok(parts.join(", "))
        }
        fn visit_unit<E: de::Error>(self) -> Result<String, E> {
            Ok(String::new())
        }
    }
    deserializer.deserialize_any(V)
}

fn default_daily_frequency() -> String {
    "daily".to_string()
}

fn flex_frequency_str<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = flex_str(deserializer)?;
    let clean = s.trim().to_ascii_lowercase();
    if clean == "once" {
        Ok("once".to_string())
    } else {
        Ok("daily".to_string())
    }
}

fn flex_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct V;
    impl<'de> Visitor<'de> for V {
        type Value = f64;
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a number or numeric string")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<f64, E> {
            Ok(v.trim().parse().unwrap_or(0.0))
        }
        fn visit_i64<E: de::Error>(self, v: i64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_u64<E: de::Error>(self, v: u64) -> Result<f64, E> {
            Ok(v as f64)
        }
        fn visit_f64<E: de::Error>(self, v: f64) -> Result<f64, E> {
            Ok(v)
        }
        fn visit_unit<E: de::Error>(self) -> Result<f64, E> {
            Ok(0.0)
        }
        fn visit_none<E: de::Error>(self) -> Result<f64, E> {
            Ok(0.0)
        }
        fn visit_some<D2: de::Deserializer<'de>>(self, d: D2) -> Result<f64, D2::Error> {
            d.deserialize_f64(V)
        }
    }
    deserializer.deserialize_any(V)
}

/// One ticket row as returned by `tickets.get` / `tickets.create` /
/// `tickets.update` / `tickets.list`.
///
/// `activities_text` is the server-side name of the local `activities`
/// field; `time_spent` arrives as a number or a string depending on how
/// the sheet cell was written â€” both are normalized here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerTicket {
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Ticket ID",
        alias = "ticket_id",
        alias = "id"
    )]
    pub ticket_id: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Employee Code",
        alias = "employee_code",
        alias = "code"
    )]
    pub employee_code: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Email Address",
        alias = "email_address",
        alias = "email",
        alias = "Mail"
    )]
    pub email_address: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Category",
        alias = "category"
    )]
    pub category: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Subject",
        alias = "subject",
        alias = "title"
    )]
    pub subject: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Problem Description",
        alias = "problem_description",
        alias = "description"
    )]
    pub problem_description: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Attachment Name",
        alias = "attachment_name"
    )]
    pub attachment_name: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Attachment URL",
        alias = "attachment_url"
    )]
    pub attachment_url: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Priority",
        alias = "priority"
    )]
    pub priority: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Assigned to",
        alias = "Assigned To",
        alias = "assigned_to",
        alias = "assignee"
    )]
    pub assigned_to: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Customer",
        alias = "customer"
    )]
    pub customer: String,
    #[serde(
        default,
        deserialize_with = "flex_f64",
        alias = "Time Spent",
        alias = "time_spent",
        alias = "Time Spent (hours)",
        alias = "time_spent_hours"
    )]
    pub time_spent: f64,
    #[serde(
        default,
        deserialize_with = "flex_str",
        rename = "activities_text",
        alias = "activities",
        alias = "Activities",
        alias = "Activities Summary"
    )]
    pub activities: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "My Activity",
        alias = "my_activity"
    )]
    pub my_activity: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Deadline",
        alias = "deadline"
    )]
    pub deadline: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "SLA Status",
        alias = "sla_status"
    )]
    pub sla_status: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "SLA Deadline",
        alias = "sla_deadline"
    )]
    pub sla_deadline: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Rating Avg Text",
        alias = "rating_avg_text",
        alias = "Rating"
    )]
    pub rating_avg_text: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Kanban State",
        alias = "kanban_state"
    )]
    pub kanban_state: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Stage",
        alias = "stage"
    )]
    pub stage: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Status",
        alias = "status"
    )]
    pub status: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Created on",
        alias = "created_on",
        alias = "Created At",
        alias = "created_at"
    )]
    pub created_on: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Last Updated on",
        alias = "last_updated_on",
        alias = "Last Updated At",
        alias = "last_updated_at"
    )]
    pub last_updated_on: String,
}

/// One page of `tickets.list`.
pub struct TicketListPage {
    pub items: Vec<ServerTicket>,
    /// Rows that failed validation (missing `ticket_id` etc.) â€” skipped,
    /// never fatal for the whole page.
    pub invalid_items: usize,
    pub has_more: bool,
}

// ============================================================
// Employee row (shape verified against the live `employees.list` & sheet)
// ============================================================

/// One employee as returned by `employees.list`.
/// Headers in sheet: Employee Code, Name, Dept, Division, Mail
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Employee {
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Employee Code",
        alias = "employee_code",
        alias = "code",
        alias = "emp_code",
        alias = "id"
    )]
    pub employee_code: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Name",
        alias = "name",
        alias = "Employee Name",
        alias = "employee_name",
        alias = "emp_name"
    )]
    pub employee_name: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Dept",
        alias = "dept",
        alias = "Department",
        alias = "department"
    )]
    pub department: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Division",
        alias = "division",
        alias = "div"
    )]
    pub division: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Mail",
        alias = "mail",
        alias = "Email Address",
        alias = "email_address",
        alias = "email",
        alias = "emp_email"
    )]
    pub email: String,
}

// ============================================================
// Reminder row (shape verified against the live `reminders.*` & sheet)
// ============================================================

/// One reminder as returned by `reminders.get` / `reminders.create` /
/// `reminders.update` / `reminders.list`.
/// Sheet headers: Reminder ID, Employee Code, Employee Name, Email Address,
/// Service / Category, Reminder Message, Send At, Status, Email Sent At,
/// Ticket ID, Created At, Last Updated At, CC Emails
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerReminder {
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Reminder ID",
        alias = "reminder_id",
        alias = "id"
    )]
    pub reminder_id: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Employee Code",
        alias = "employee_code",
        alias = "code"
    )]
    pub employee_code: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Employee Name",
        alias = "employee_name",
        alias = "Name",
        alias = "name"
    )]
    pub employee_name: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Email Address",
        alias = "email_address",
        alias = "email",
        alias = "Mail",
        alias = "mail"
    )]
    pub email_address: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Service / Category",
        alias = "Service/Category",
        alias = "Service Category",
        alias = "service_category",
        alias = "Category",
        alias = "category"
    )]
    pub service_category: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Reminder Message",
        alias = "reminder_message",
        alias = "message"
    )]
    pub reminder_message: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Send At",
        alias = "send_at",
        alias = "Send on",
        alias = "send_on"
    )]
    pub send_at: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Status",
        alias = "status"
    )]
    pub status: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Email Sent At",
        alias = "email_sent_at",
        alias = "Email Sent on",
        alias = "email_sent_on"
    )]
    pub email_sent_at: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Ticket ID",
        alias = "ticket_id"
    )]
    pub ticket_id: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Created At",
        alias = "created_at",
        alias = "Created on",
        alias = "created_on"
    )]
    pub created_at: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Last Updated At",
        alias = "last_updated_at",
        alias = "Last Updated on",
        alias = "last_updated_on"
    )]
    pub last_updated_at: String,
    /// Sheet column M (`CC Emails`); empty when the row has no recipients.
    #[serde(
        default,
        deserialize_with = "flex_list_str",
        alias = "CC Emails",
        alias = "cc_emails",
        alias = "cc"
    )]
    pub cc_emails: String,
    /// Sheet column N (`Repeat Frequency`): "daily" or "once". Treated as "daily" when blank.
    #[serde(
        default = "default_daily_frequency",
        deserialize_with = "flex_frequency_str",
        alias = "Repeat Frequency",
        alias = "repeat_frequency",
        alias = "Frequency",
        alias = "frequency"
    )]
    pub repeat_frequency: String,
}

/// One page of `reminders.list`.
pub struct ReminderListPage {
    pub items: Vec<ServerReminder>,
    pub invalid_items: usize,
    pub has_more: bool,
}

// ============================================================
// Client
// ============================================================

/// The application's single HTTP client for the TMS API.
pub struct ApiClient {
    http: reqwest::Client,
    cfg: ApiConfig,
    counter: AtomicU64,
}

impl ApiClient {
    pub fn new(cfg: ApiConfig) -> Result<Self, String> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent("BPSL-IT-Support/1.0")
            .redirect(reqwest::redirect::Policy::limited(10))
            .tcp_nodelay(true)
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| format!("HTTP client init failed: {e}"))?;
        Ok(Self {
            http,
            cfg,
            counter: AtomicU64::new(0),
        })
    }

    pub fn config(&self) -> &ApiConfig {
        &self.cfg
    }

    fn request_id(&self) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let n = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("tms-{nanos}-{n}")
    }

    /// Posts one envelope and unwraps the response envelope. Note the API
    /// answers with HTTP 200 even for failures â€” `success`/`error` live in
    /// Posts one envelope and unwraps the response envelope. Note the API
    /// answers with HTTP 200 even for failures â€” success/error live in
    /// the body. Automatically retries transient LOCK_TIMEOUT and busy errors.
    async fn post(&self, action: &str, data: Value) -> Result<Value, ApiError> {
        let mut attempts = 0usize;
        const MAX_LOCK_ATTEMPTS: usize = 3;

        loop {
            attempts += 1;
            let envelope = RequestEnvelope {
                version: "v1",
                action,
                auth: Auth {
                    api_key: &self.cfg.api_key,
                },
                request_id: self.request_id(),
                data: data.clone(),
            };

            let resp = self
                .http
                .post(&self.cfg.base_url)
                .json(&envelope)
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        ApiError::Timeout
                    } else {
                        ApiError::Network(e.to_string())
                    }
                })?;

            let status = resp.status();
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(parse_retry_after);

            let text = resp
                .text()
                .await
                .map_err(|e| ApiError::Network(format!("failed to read response: {e}")))?;

            let text_trimmed = text.trim();
            if text_trimmed.is_empty() {
                if attempts < MAX_LOCK_ATTEMPTS {
                    let delay = retry_after.unwrap_or(2 * attempts as u64);
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    continue;
                }
                return Err(ApiError::Decode("empty response from Google Apps Script (transient script cold start or reset)".to_string()));
            }

            if text_trimmed.starts_with('<') {
                if attempts < MAX_LOCK_ATTEMPTS {
                    let delay = retry_after.unwrap_or(2 * attempts as u64);
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    continue;
                }
                let preview: String = text_trimmed.chars().take(200).collect();
                return Err(ApiError::Decode(format!("server returned HTML error instead of JSON: {preview}")));
            }

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
                if attempts < MAX_LOCK_ATTEMPTS {
                    let delay = retry_after.unwrap_or(2 * attempts as u64);
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    continue;
                }
                return Err(ApiError::Http {
                    status: status.as_u16(),
                    retryable: true,
                    retry_after,
                    body: text.chars().take(300).collect(),
                });
            }
            if status.is_client_error() {
                return Err(ApiError::Http {
                    status: status.as_u16(),
                    retryable: false,
                    retry_after,
                    body: text.chars().take(300).collect(),
                });
            }

            let envelope: ResponseEnvelope = serde_json::from_str(&text)
                .map_err(|e| ApiError::Decode(format!("invalid JSON response: {e}")))?;

            if envelope.success {
                return Ok(envelope.data.unwrap_or(Value::Null));
            }

            let message = envelope
                .message
                .unwrap_or_else(|| "request failed".to_string());

            let (code, mut body_retryable) = match envelope.error {
                Some(err) => (
                    err.code.unwrap_or_else(|| "UNKNOWN".to_string()),
                    err.retryable.unwrap_or(false),
                ),
                None => ("UNKNOWN".to_string(), false),
            };

            // Detect transient Google Apps Script LockService/busy conditions
            let is_lock_or_busy = code == "LOCK_TIMEOUT"
                || code == "RATE_LIMIT"
                || code == "RATE_LIMITED"
                || code == "BUSY"
                || code == "CONCURRENT_ACCESS"
                || code == "SERVICE_UNAVAILABLE"
                || message.to_lowercase().contains("busy")
                || message.to_lowercase().contains("retry")
                || message.to_lowercase().contains("lock");

            if is_lock_or_busy {
                body_retryable = true;
                if attempts < MAX_LOCK_ATTEMPTS {
                    // Script locks in GAS are typically released within 1-2 seconds.
                    // Pause briefly before retrying so the call succeeds transparently.
                    let delay = match attempts {
                        1 => Duration::from_millis(1500),
                        _ => Duration::from_millis(3000),
                    };
                    tokio::time::sleep(delay).await;
                    continue;
                }
            }

            return Err(ApiError::Api {
                code,
                message,
                retryable: body_retryable,
                retry_after,
            });
        }
    }

    // ---- actions (TMS_API.postman_collection.json) ----

    /// `health.check` (no auth required).
    pub async fn health(&self) -> Result<Value, ApiError> {
        self.post("health.check", json!({})).await
    }

    /// `tickets.list` â€” optional filters are omitted to fetch every row;
    /// `limit` maxes out at 200 per the API documentation.
    pub async fn tickets_list(&self, offset: u64, limit: u64) -> Result<TicketListPage, ApiError> {
        let data = self
            .post("tickets.list", json!({ "offset": offset, "limit": limit }))
            .await?;

        let mut items = Vec::new();
        let mut invalid_items = 0usize;
        if let Some(arr) = data.get("items").and_then(|v| v.as_array()) {
            for value in arr {
                match serde_json::from_value::<ServerTicket>(value.clone()) {
                    Ok(t) => items.push(t),
                    Err(_) => invalid_items += 1,
                }
            }
        }
        let has_more = data
            .pointer("/pagination/has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(TicketListPage {
            items,
            invalid_items,
            has_more,
        })
    }

    /// `tickets.get`
    pub async fn tickets_get(&self, ticket_id: &str) -> Result<ServerTicket, ApiError> {
        let data = self
            .post("tickets.get", json!({ "ticket_id": ticket_id }))
            .await?;
        parse_ticket(data)
    }

    /// `tickets.create` â€” `fields` must contain only real row fields (the
    /// API rejects unknown fields with `UNKNOWN_FIELD`).
    pub async fn tickets_create(&self, fields: &Value) -> Result<ServerTicket, ApiError> {
        let data = self.post("tickets.create", fields.clone()).await?;
        parse_ticket(data)
    }

    /// `tickets.update` â€” "send ticket_id and only the fields to update".
    pub async fn tickets_update(
        &self,
        ticket_id: &str,
        fields: &Value,
    ) -> Result<ServerTicket, ApiError> {
        let mut data = fields.as_object().cloned().unwrap_or_default();
        data.insert("ticket_id".to_string(), json!(ticket_id));
        let data = self.post("tickets.update", Value::Object(data)).await?;
        parse_ticket(data)
    }

    /// `tickets.delete` â€” hard delete; `NOT_FOUND` is treated as success
    /// by callers (already gone).
    pub async fn tickets_delete(&self, ticket_id: &str) -> Result<(), ApiError> {
        self.post("tickets.delete", json!({ "ticket_id": ticket_id }))
            .await?;
        Ok(())
    }

    /// `attachments.upload` — uploads a local temporary attachment file (base64)
    /// to Apps Script / Google Drive and returns the resulting shareable URL.
    pub async fn attachments_upload(
        &self,
        file_name: &str,
        mime_type: &str,
        data_base64: &str,
    ) -> Result<String, ApiError> {
        let data = self
            .post(
                "attachments.upload",
                json!({
                    "file_name": file_name,
                    "mime_type": mime_type,
                    "data_base64": data_base64,
                }),
            )
            .await?;
        let url = data
            .get("attachment_url")
            .or_else(|| data.get("url"))
            .or_else(|| data.get("file_url"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string);
        url.ok_or_else(|| ApiError::Decode("attachments.upload returned no URL".to_string()))
    }

    /// `employees.list` — fetches directory data (per TMS_API.postman_collection.json).
    /// Falls back to `employees.listAll` if the server returns UNKNOWN_ACTION.
    pub async fn employees_list(&self) -> Result<Vec<Employee>, ApiError> {
        let data = match self.post("employees.list", json!({})).await {
            Ok(d) => d,
            Err(ApiError::Api { ref code, .. }) if code == "UNKNOWN_ACTION" => {
                self.post("employees.listAll", json!({})).await?
            }
            Err(e) => return Err(e),
        };

        let mut items = Vec::new();
        let arr = data
            .get("items")
            .and_then(|v| v.as_array())
            .or_else(|| data.as_array())
            .or_else(|| data.get("employees").and_then(|v| v.as_array()));

        if let Some(arr) = arr {
            for value in arr {
                // Rows without a usable name/code are skipped, never fatal.
                if let Ok(mut emp) = serde_json::from_value::<Employee>(value.clone()) {
                    if !emp.employee_name.trim().is_empty() || !emp.employee_code.trim().is_empty()
                    {
                        if !emp.email.contains('@') && emp.division.contains('@') {
                            std::mem::swap(&mut emp.email, &mut emp.division);
                        }
                        items.push(emp);
                    }
                }
            }
        }
        Ok(items)
    }

    /// `employees.get`
    pub async fn employees_get(&self, employee_code: &str) -> Result<Employee, ApiError> {
        let data = self
            .post("employees.get", json!({ "employee_code": employee_code }))
            .await?;
        parse_employee(data)
    }

    /// `employees.create`
    pub async fn employees_create(&self, fields: &Value) -> Result<Employee, ApiError> {
        let data = self.post("employees.create", fields.clone()).await?;
        parse_employee(data)
    }

    /// `employees.update`
    pub async fn employees_update(
        &self,
        employee_code: &str,
        fields: &Value,
    ) -> Result<Employee, ApiError> {
        let mut data = fields.as_object().cloned().unwrap_or_default();
        data.insert("employee_code".to_string(), json!(employee_code));
        let data = self.post("employees.update", Value::Object(data)).await?;
        parse_employee(data)
    }

    /// `employees.delete`
    pub async fn employees_delete(&self, employee_code: &str) -> Result<(), ApiError> {
        self.post(
            "employees.delete",
            json!({ "employee_code": employee_code }),
        )
        .await?;
        Ok(())
    }

    // ---- reminders (TMS_API.postman_collection.json) ----

    /// `reminders.list`
    pub async fn reminders_list(
        &self,
        offset: u64,
        limit: u64,
        status: Option<&str>,
        employee_code: Option<&str>,
    ) -> Result<ReminderListPage, ApiError> {
        let mut query = json!({ "offset": offset, "limit": limit });
        if let Some(s) = status {
            query
                .as_object_mut()
                .unwrap()
                .insert("status".to_string(), json!(s));
        }
        if let Some(ec) = employee_code {
            query
                .as_object_mut()
                .unwrap()
                .insert("employee_code".to_string(), json!(ec));
        }

        // `reminders.list` is the only documented list action for reminders
        // (`reminders.listAll` does not exist and returns UNKNOWN_ACTION).
        let data = self.post("reminders.list", query).await?;

        let mut items = Vec::new();
        let mut invalid_items = 0usize;
        let arr = data
            .get("items")
            .and_then(|v| v.as_array())
            .or_else(|| data.as_array())
            .or_else(|| data.get("reminders").and_then(|v| v.as_array()));

        if let Some(arr) = arr {
            for value in arr {
                match serde_json::from_value::<ServerReminder>(value.clone()) {
                    Ok(r) => items.push(r),
                    Err(_) => invalid_items += 1,
                }
            }
        }
        let has_more = data
            .pointer("/pagination/has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(ReminderListPage {
            items,
            invalid_items,
            has_more,
        })
    }

    /// `reminders.get`
    pub async fn reminders_get(&self, reminder_id: &str) -> Result<ServerReminder, ApiError> {
        let data = self
            .post("reminders.get", json!({ "reminder_id": reminder_id }))
            .await?;
        parse_reminder(data)
    }

    /// `reminders.create`
    pub async fn reminders_create(&self, fields: &Value) -> Result<ServerReminder, ApiError> {
        let data = self.post("reminders.create", fields.clone()).await?;
        parse_reminder(data)
    }

    /// `reminders.update`
    pub async fn reminders_update(
        &self,
        reminder_id: &str,
        fields: &Value,
    ) -> Result<ServerReminder, ApiError> {
        let mut data = fields.as_object().cloned().unwrap_or_default();
        data.insert("reminder_id".to_string(), json!(reminder_id));
        let data = self.post("reminders.update", Value::Object(data)).await?;
        parse_reminder(data)
    }

    /// `reminders.sendNow`
    pub async fn reminders_send_now(
        &self,
        reminder_id: &str,
        force: bool,
    ) -> Result<Value, ApiError> {
        let mut payload = json!({ "reminder_id": reminder_id });
        if force {
            payload
                .as_object_mut()
                .unwrap()
                .insert("force".to_string(), json!(true));
        }
        self.post("reminders.sendNow", payload).await
    }

    /// reminders.delete - permanently deletes the reminder row and revokes all tokens.
    /// NOT_FOUND is treated as success so repeated calls are idempotent.
    pub async fn reminders_delete(&self, reminder_id: &str) -> Result<Value, ApiError> {
        self.post("reminders.delete", json!({ "reminder_id": reminder_id }))
            .await
            .or_else(|e| match e {
                ApiError::Api { ref code, .. } if code == "NOT_FOUND" => {
                    Ok(json!({ "reminder_id": reminder_id, "deleted": true }))
                }
                other => Err(other),
            })
    }
}

fn parse_ticket(data: Value) -> Result<ServerTicket, ApiError> {
    serde_json::from_value(data)
        .map_err(|e| ApiError::Decode(format!("invalid ticket payload: {e}")))
}

fn parse_employee(data: Value) -> Result<Employee, ApiError> {
    let mut emp: Employee = serde_json::from_value(data)
        .map_err(|e| ApiError::Decode(format!("invalid employee payload: {e}")))?;
    if !emp.email.contains('@') && emp.division.contains('@') {
        std::mem::swap(&mut emp.email, &mut emp.division);
    }
    Ok(emp)
}

fn parse_reminder(data: Value) -> Result<ServerReminder, ApiError> {
    serde_json::from_value(data).map_err(|e| ApiError::Decode(format!("invalid reminder payload: {e}")))
}
