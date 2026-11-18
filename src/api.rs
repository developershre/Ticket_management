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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ApiProtocol {
    /// Legacy authenticated JSON-RPC envelope (`POST {{base_url}}` with `version: "v1"`, `action`, `auth.api_key`).
    LegacyV1,
    /// Unauthenticated BPSL Ticket API (`GET`/`POST {{base_url}}?api=...`).
    #[default]
    BpslRest,
}

impl ApiProtocol {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "rest" | "bpsl" | "bpsl_rest" | "query" => Some(Self::BpslRest),
            "v1" | "legacy" | "legacy_v1" | "envelope" => Some(Self::LegacyV1),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BpslRest => "rest",
            Self::LegacyV1 => "v1",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ApiConfig {
    pub base_url: String,
    pub api_key: String,
    /// Optional employee code used for `tickets.create`: the sheet requires
    /// `employee_code` to reference an existing employee row. Set via the
    /// `TMS_EMPLOYEE_CODE` environment variable.
    pub employee_code: Option<String>,
    /// API wire protocol (`BpslRest` `?api=...` vs `LegacyV1` JSON envelope).
    pub protocol: ApiProtocol,
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
    /// 2. `TMS_API_BASE_URL`, `TMS_API_KEY`, `TMS_EMPLOYEE_CODE`, `TMS_API_PROTOCOL` env vars or `.env` file
    /// 3. `base_url`/`baseUrl` & `api_key` variables of `TMS_API.postman_collection.json`
    ///    (searched in working dir, next to executable and one level above)
    pub fn load() -> Result<ApiConfig, String> {
        let (sys_url, sys_key, sys_emp) = crate::db::get_system_api_config();
        let sys_proto = crate::db::get_system_api_protocol();
        let (dot_url, dot_key, dot_emp, dot_proto) = Self::from_dotenv_file();
        let has_sys_key = sys_key.is_some();
        let env_url = sys_url.or_else(|| non_empty_env("TMS_API_BASE_URL")).or(dot_url);
        let env_key = sys_key.or_else(|| non_empty_env("TMS_API_KEY")).or(dot_key);
        let employee_code = sys_emp.or_else(|| non_empty_env("TMS_EMPLOYEE_CODE")).or(dot_emp);
        let raw_proto = sys_proto
            .or_else(|| non_empty_env("TMS_API_PROTOCOL"))
            .or(dot_proto);

        let (file_url, file_key) = Self::from_collection_file();

        let base_url = env_url
            .or(file_url)
            .ok_or_else(|| {
                "Missing TMS_API_BASE_URL. Please define TMS_API_BASE_URL in your .env file or environment.".to_string()
            })?;
        let api_key = if raw_proto.as_deref() == Some("rest") && !has_sys_key {
            String::new()
        } else {
            env_key.or(file_key).unwrap_or_default().trim().to_string()
        };

        let protocol = raw_proto
            .as_deref()
            .and_then(ApiProtocol::from_str_opt)
            .unwrap_or(if api_key.is_empty() {
                ApiProtocol::BpslRest
            } else {
                ApiProtocol::LegacyV1
            });

        Ok(ApiConfig {
            base_url: base_url.trim().to_string(),
            api_key,
            employee_code,
            protocol,
        })
    }

    /// Reads key-value pairs from `.env` in candidate paths.
    fn from_dotenv_file() -> (Option<String>, Option<String>, Option<String>, Option<String>) {
        for path in Self::candidate_paths(".env") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            let mut url = None;
            let mut key = None;
            let mut emp = None;
            let mut proto = None;
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
                            "TMS_API_PROTOCOL" => proto = Some(v.to_string()),
                            _ => {}
                        }
                    }
                }
            }
            if url.is_some() || key.is_some() || emp.is_some() || proto.is_some() {
                return (url, key, emp, proto);
            }
        }
        (None, None, None, None)
    }

    /// Reads the `base_url`/`baseUrl` and `api_key` collection variables from the
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
                    "base_url" | "baseUrl" => url = Some(var.value),
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

#[derive(Deserialize, Debug)]
struct ResponseEnvelope {
    #[serde(default)]
    success: Option<bool>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    data: Option<Value>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    error: Option<Value>,
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
        alias = "ticketId",
        alias = "id"
    )]
    pub ticket_id: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Employee Code",
        alias = "employee_code",
        alias = "employeeCode",
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
        alias = "attachment_name",
        alias = "attachmentName",
        alias = "fileName"
    )]
    pub attachment_name: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Attachment URL",
        alias = "attachment_url",
        alias = "attachmentUrl",
        alias = "fileUrl"
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
        alias = "assignee",
        alias = "agent"
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
        alias = "timeSpent",
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
        alias = "my_activity",
        alias = "myActivity"
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
        alias = "sla_status",
        alias = "slaStatus"
    )]
    pub sla_status: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "SLA Deadline",
        alias = "sla_deadline",
        alias = "slaDeadline"
    )]
    pub sla_deadline: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Rating Avg Text",
        alias = "rating_avg_text",
        alias = "ratingAvgText",
        alias = "ratingText",
        alias = "Rating",
        alias = "rating"
    )]
    pub rating_avg_text: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Kanban State",
        alias = "kanban_state",
        alias = "kanbanState"
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
        alias = "createdOn",
        alias = "Created At",
        alias = "created_at",
        alias = "createdAt"
    )]
    pub created_on: String,
    #[serde(
        default,
        deserialize_with = "flex_str",
        alias = "Last Updated on",
        alias = "last_updated_on",
        alias = "updatedOn",
        alias = "Last Updated At",
        alias = "last_updated_at",
        alias = "updatedAt"
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
        alias = "employeeCode",
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
        alias = "employeeName",
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
    pub fn protocol(&self) -> ApiProtocol {
        self.cfg.protocol
    }
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

            let is_success = envelope.success.unwrap_or_else(|| {
                envelope
                    .status
                    .as_deref()
                    .map(|s| s.eq_ignore_ascii_case("success") || s.eq_ignore_ascii_case("ok"))
                    .unwrap_or(false)
            });

            if is_success && envelope.error.is_none() {
                return Ok(envelope.data.unwrap_or(Value::Null));
            }

            let (code, message, mut body_retryable) = match envelope.error {
                Some(Value::String(err_str)) => {
                    let code = if err_str.contains("Unknown API route") || err_str.contains("UNKNOWN_ROUTE") {
                        "UNKNOWN_ROUTE".to_string()
                    } else {
                        "API_ERROR".to_string()
                    };
                    let msg = envelope.message.unwrap_or(err_str);
                    (code, msg, false)
                }
                Some(Value::Object(map)) => {
                    let code = map
                        .get("code")
                        .and_then(|v| v.as_str())
                        .unwrap_or("API_ERROR")
                        .to_string();
                    let msg = map
                        .get("message")
                        .and_then(|v| v.as_str())
                        .map(ToString::to_string)
                        .or(envelope.message)
                        .unwrap_or_else(|| "request failed".to_string());
                    let retry = map
                        .get("retryable")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    (code, msg, retry)
                }
                _ => (
                    "UNKNOWN".to_string(),
                    envelope.message.unwrap_or_else(|| "request failed".to_string()),
                    false,
                ),
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

    /// Builds a URL for the unauthenticated BPSL Ticket API (`?api=...`).
    fn rest_url(&self, api_route: &str, extra_query: &[(&str, &str)]) -> String {
        let base = self.cfg.base_url.trim_end_matches('&');
        let sep = if base.contains('?') { '&' } else { '?' };
        let mut url = format!("{base}{sep}api={api_route}");
        for (k, v) in extra_query {
            if !v.is_empty() {
                url.push('&');
                url.push_str(k);
                url.push('=');
                url.push_str(&v.replace(' ', "%20").replace('&', "%26"));
            }
        }
        url
    }

    /// Executes a `GET` or `POST` request against the unauthenticated BPSL Ticket API (`?api=...`)
    /// with the same transient retry logic (empty response, HTML cold start, 429/5xx, lock/busy).
    async fn send_rest(
        &self,
        method: reqwest::Method,
        api_route: &str,
        extra_query: &[(&str, &str)],
        body: Option<Value>,
    ) -> Result<Value, ApiError> {
        let url = self.rest_url(api_route, extra_query);
        let mut attempts = 0usize;
        const MAX_LOCK_ATTEMPTS: usize = 3;

        loop {
            attempts += 1;
            let mut req = self.http.request(method.clone(), &url);
            if let Some(ref b) = body {
                req = req.json(b);
            }

            let resp = req.send().await.map_err(|e| {
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
                return Err(ApiError::Decode(
                    "empty response from Google Apps Script (transient script cold start or reset)"
                        .to_string(),
                ));
            }

            if text_trimmed.starts_with('<') {
                if attempts < MAX_LOCK_ATTEMPTS {
                    let delay = retry_after.unwrap_or(2 * attempts as u64);
                    tokio::time::sleep(Duration::from_secs(delay)).await;
                    continue;
                }
                let preview: String = text_trimmed.chars().take(200).collect();
                return Err(ApiError::Decode(format!(
                    "server returned HTML error instead of JSON: {preview}"
                )));
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

            let val: Value = serde_json::from_str(&text)
                .map_err(|e| ApiError::Decode(format!("invalid JSON response: {e}")))?;

            // Handle error payloads in REST responses:
            // { "success": false, ... }, { "ok": false, ... }, or { "error": "..." }
            let explicit_fail = val.get("success").and_then(|v| v.as_bool()) == Some(false)
                || val.get("ok").and_then(|v| v.as_bool()) == Some(false);
            let error_msg = val.get("error").and_then(|err| {
                if err.is_null() {
                    None
                } else if let Some(s) = err.as_str() {
                    Some(s.to_string())
                } else {
                    err.get("message")
                        .and_then(|m| m.as_str())
                        .map(ToString::to_string)
                }
            });

            if explicit_fail || error_msg.is_some() {
                let message = error_msg
                    .or_else(|| {
                        val.get("message")
                            .and_then(|m| m.as_str())
                            .map(ToString::to_string)
                    })
                    .unwrap_or_else(|| "request failed".to_string());
                let code = val
                    .pointer("/error/code")
                    .or_else(|| val.get("code"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("API_ERROR")
                    .to_string();

                let lower = message.to_lowercase();
                let is_lock_or_busy = code == "LOCK_TIMEOUT"
                    || code == "RATE_LIMIT"
                    || code == "BUSY"
                    || lower.contains("busy")
                    || lower.contains("retry")
                    || lower.contains("lock");
                if is_lock_or_busy && attempts < MAX_LOCK_ATTEMPTS {
                    let delay = match attempts {
                        1 => Duration::from_millis(1500),
                        _ => Duration::from_millis(3000),
                    };
                    tokio::time::sleep(delay).await;
                    continue;
                }

                return Err(ApiError::Api {
                    code,
                    message,
                    retryable: is_lock_or_busy,
                    retry_after,
                });
            }

            if val.get("success").and_then(|v| v.as_bool()) == Some(true) {
                if let Some(inner) = val.get("data") {
                    return Ok(inner.clone());
                }
            }

            return Ok(val);
        }
    }

    // ---- actions (TMS_API.postman_collection.json & BPSL Ticket API) ----

    /// Health / reachability probe.
    pub async fn health(&self) -> Result<Value, ApiError> {
        match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                let _ = self
                    .send_rest(reqwest::Method::GET, "tickets", &[], None)
                    .await?;
                Ok(json!({ "status": "ok", "protocol": "bpsl_rest" }))
            }
            ApiProtocol::LegacyV1 => match self.post("health.check", json!({})).await {
                Ok(val) => Ok(val),
                Err(ApiError::Api { message, .. })
                    if message.contains("Unknown API route") || message.contains("UNKNOWN_ACTION") =>
                {
                    // Fallback probe: if endpoint is deployed as BPSL REST (?api=tickets)
                    let _ = self
                        .send_rest(reqwest::Method::GET, "tickets", &[], None)
                        .await?;
                    Ok(json!({ "status": "ok", "protocol": "bpsl_rest", "fallback": true }))
                }
                Err(e) => Err(e),
            },
        }
    }

    /// `tickets.list` / `GET ?api=tickets`
    pub async fn tickets_list(&self, offset: u64, limit: u64) -> Result<TicketListPage, ApiError> {
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                self.send_rest(reqwest::Method::GET, "tickets", &[], None)
                    .await?
            }
            ApiProtocol::LegacyV1 => {
                self.post("tickets.list", json!({ "offset": offset, "limit": limit }))
                    .await?
            }
        };

        let mut items = Vec::new();
        let mut invalid_items = 0usize;
        let arr = data
            .get("items")
            .and_then(|v| v.as_array())
            .or_else(|| data.get("tickets").and_then(|v| v.as_array()))
            .or_else(|| data.get("data").and_then(|v| v.as_array()))
            .or_else(|| data.as_array());

        if let Some(arr) = arr {
            for value in arr {
                match serde_json::from_value::<ServerTicket>(value.clone()) {
                    Ok(t) => items.push(t),
                    Err(_) => invalid_items += 1,
                }
            }
        }
        let has_more = match self.cfg.protocol {
            ApiProtocol::BpslRest => false,
            ApiProtocol::LegacyV1 => data
                .pointer("/pagination/has_more")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        };
        Ok(TicketListPage {
            items,
            invalid_items,
            has_more,
        })
    }

    /// `tickets.get` / `GET ?api=tickets&id={{ticketId}}`
    pub async fn tickets_get(&self, ticket_id: &str) -> Result<ServerTicket, ApiError> {
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                self.send_rest(reqwest::Method::GET, "tickets", &[("id", ticket_id)], None)
                    .await?
            }
            ApiProtocol::LegacyV1 => {
                self.post("tickets.get", json!({ "ticket_id": ticket_id }))
                    .await?
            }
        };
        parse_ticket(data)
    }

    /// `tickets.create` / `POST ?api=tickets`
    pub async fn tickets_create(&self, fields: &Value) -> Result<ServerTicket, ApiError> {
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                let body = to_bpsl_ticket_body(fields);
                self.send_rest(reqwest::Method::POST, "tickets", &[], Some(body))
                    .await?
            }
            ApiProtocol::LegacyV1 => self.post("tickets.create", fields.clone()).await?,
        };
        parse_ticket(data)
    }

    /// `tickets.update` / `POST ?api=tickets&id={{ticketId}}` (or `tickets/{{ticketId}}/stage` for stage-only changes)
    pub async fn tickets_update(
        &self,
        ticket_id: &str,
        fields: &Value,
    ) -> Result<ServerTicket, ApiError> {
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                let obj = fields.as_object();
                let is_stage_only = obj
                    .map(|m| m.len() == 1 && m.contains_key("stage"))
                    .unwrap_or(false);
                if is_stage_only {
                    let stage_val = obj
                        .and_then(|m| m.get("stage"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Pending");
                    let route = format!("tickets/{ticket_id}/stage");
                    self.send_rest(
                        reqwest::Method::POST,
                        &route,
                        &[],
                        Some(json!({ "stage": stage_val })),
                    )
                    .await?
                } else {
                    let body = to_bpsl_ticket_body(fields);
                    self.send_rest(
                        reqwest::Method::POST,
                        "tickets",
                        &[("id", ticket_id)],
                        Some(body),
                    )
                    .await?
                }
            }
            ApiProtocol::LegacyV1 => {
                let mut data = fields.as_object().cloned().unwrap_or_default();
                data.insert("ticket_id".to_string(), json!(ticket_id));
                self.post("tickets.update", Value::Object(data)).await?
            }
        };
        parse_ticket(data)
    }

    /// `POST ?api=tickets/{{ticketId}}/stage` (or `tickets.update` on LegacyV1).
    pub async fn tickets_update_stage(
        &self,
        ticket_id: &str,
        stage: &str,
    ) -> Result<ServerTicket, ApiError> {
        match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                let route = format!("tickets/{ticket_id}/stage");
                let data = self
                    .send_rest(
                        reqwest::Method::POST,
                        &route,
                        &[],
                        Some(json!({ "stage": stage })),
                    )
                    .await?;
                parse_ticket(data)
            }
            ApiProtocol::LegacyV1 => {
                self.tickets_update(ticket_id, &json!({ "stage": stage }))
                    .await
            }
        }
    }

    /// `POST ?api=tickets/{{ticketId}}/close` (or `tickets.update` on LegacyV1).
    pub async fn tickets_close(
        &self,
        ticket_id: &str,
        rating: &str,
    ) -> Result<ServerTicket, ApiError> {
        match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                let route = format!("tickets/{ticket_id}/close");
                let data = self
                    .send_rest(
                        reqwest::Method::POST,
                        &route,
                        &[],
                        Some(json!({ "rating": rating })),
                    )
                    .await?;
                parse_ticket(data)
            }
            ApiProtocol::LegacyV1 => {
                self.tickets_update(
                    ticket_id,
                    &json!({ "status": "Closed", "stage": "Closed", "rating_avg_text": rating }),
                )
                .await
            }
        }
    }

    /// `tickets.delete` â€” hard delete; `NOT_FOUND` is treated as success
    /// by callers (already gone).
    pub async fn tickets_delete(&self, ticket_id: &str) -> Result<(), ApiError> {
        match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                let route = format!("tickets/{ticket_id}/delete");
                self.send_rest(reqwest::Method::POST, &route, &[], Some(json!({})))
                    .await?;
            }
            ApiProtocol::LegacyV1 => {
                self.post("tickets.delete", json!({ "ticket_id": ticket_id }))
                    .await?;
            }
        }
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
        let body = json!({
            "file_name": file_name,
            "fileName": file_name,
            "mime_type": mime_type,
            "mimeType": mime_type,
            "data_base64": data_base64,
            "dataBase64": data_base64,
        });
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                self.send_rest(reqwest::Method::POST, "attachments", &[], Some(body))
                    .await?
            }
            ApiProtocol::LegacyV1 => self.post("attachments.upload", body).await?,
        };
        let url = data
            .get("attachment_url")
            .or_else(|| data.get("attachmentUrl"))
            .or_else(|| data.get("url"))
            .or_else(|| data.get("file_url"))
            .or_else(|| data.get("fileUrl"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToString::to_string);
        url.ok_or_else(|| ApiError::Decode("attachments.upload returned no URL".to_string()))
    }

    /// `employees.list` / `GET ?api=employees`
    pub async fn employees_list(&self) -> Result<Vec<Employee>, ApiError> {
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                self.send_rest(reqwest::Method::GET, "employees", &[], None)
                    .await?
            }
            ApiProtocol::LegacyV1 => match self.post("employees.list", json!({})).await {
                Ok(d) => d,
                Err(ApiError::Api { ref code, .. }) if code == "UNKNOWN_ACTION" => {
                    self.post("employees.listAll", json!({})).await?
                }
                Err(e) => return Err(e),
            },
        };

        let mut items = Vec::new();
        let arr = data
            .get("items")
            .and_then(|v| v.as_array())
            .or_else(|| data.as_array())
            .or_else(|| data.get("employees").and_then(|v| v.as_array()))
            .or_else(|| data.get("data").and_then(|v| v.as_array()));

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

    /// `employees.get` / `GET ?api=employees&code={{employeeCode}}`
    pub async fn employees_get(&self, employee_code: &str) -> Result<Employee, ApiError> {
        let data = match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                self.send_rest(
                    reqwest::Method::GET,
                    "employees",
                    &[("code", employee_code)],
                    None,
                )
                .await?
            }
            ApiProtocol::LegacyV1 => {
                self.post("employees.get", json!({ "employee_code": employee_code }))
                    .await?
            }
        };
        parse_employee(data)
    }

    /// `GET ?api=analytics&startDate=...&endDate=...&shiftHours=...`
    pub async fn analytics(
        &self,
        start_date: &str,
        end_date: &str,
        shift_hours: u32,
    ) -> Result<Value, ApiError> {
        let shift_str = shift_hours.to_string();
        match self.cfg.protocol {
            ApiProtocol::BpslRest => {
                self.send_rest(
                    reqwest::Method::GET,
                    "analytics",
                    &[
                        ("startDate", start_date),
                        ("endDate", end_date),
                        ("shiftHours", &shift_str),
                    ],
                    None,
                )
                .await
            }
            ApiProtocol::LegacyV1 => {
                self.post(
                    "analytics.closed",
                    json!({
                        "startDate": start_date,
                        "endDate": end_date,
                        "shiftHours": shift_hours,
                    }),
                )
                .await
            }
        }
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
        if self.cfg.protocol == ApiProtocol::BpslRest {
            return Ok(ReminderListPage {
                items: Vec::new(),
                invalid_items: 0,
                has_more: false,
            });
        }
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

fn to_bpsl_ticket_body(fields: &Value) -> Value {
    let Some(obj) = fields.as_object() else {
        return fields.clone();
    };
    let mut out = serde_json::Map::new();
    for (k, v) in obj {
        let mapped = match k.as_str() {
            "employee_code" => "employeeCode",
            "email_address" => "email",
            "problem_description" => "description",
            "assigned_to" => "agent",
            "attachment_name" => "attachmentName",
            "attachment_url" => "attachmentUrl",
            other => other,
        };
        out.insert(mapped.to_string(), v.clone());
    }
    Value::Object(out)
}

fn parse_ticket(data: Value) -> Result<ServerTicket, ApiError> {
    let unwrapped = data
        .get("ticket")
        .or_else(|| data.get("data"))
        .cloned()
        .unwrap_or(data);
    serde_json::from_value(unwrapped)
        .map_err(|e| ApiError::Decode(format!("invalid ticket payload: {e}")))
}

fn parse_employee(data: Value) -> Result<Employee, ApiError> {
    let unwrapped = data
        .get("employee")
        .or_else(|| data.get("data"))
        .cloned()
        .unwrap_or(data);
    let mut emp: Employee = serde_json::from_value(unwrapped)
        .map_err(|e| ApiError::Decode(format!("invalid employee payload: {e}")))?;
    if !emp.email.contains('@') && emp.division.contains('@') {
        std::mem::swap(&mut emp.email, &mut emp.division);
    }
    Ok(emp)
}

fn parse_reminder(data: Value) -> Result<ServerReminder, ApiError> {
    serde_json::from_value(data).map_err(|e| ApiError::Decode(format!("invalid reminder payload: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_envelope_unknown_route_string() {
        let json_str = r#"{"status":"error","error":"Unknown API route."}"#;
        let env: Result<ResponseEnvelope, _> = serde_json::from_str(json_str);
        assert!(env.is_ok(), "ResponseEnvelope must parse string error: {:?}", env.err());
        let env = env.unwrap();
        assert_eq!(env.error, Some(Value::String("Unknown API route.".to_string())));
        assert_eq!(env.status, Some("error".to_string()));
    }

    #[test]
    fn test_response_envelope_structured_error() {
        let json_str = r#"{"success":false,"error":{"code":"RATE_LIMIT","message":"lock busy","retryable":true}}"#;
        let env: Result<ResponseEnvelope, _> = serde_json::from_str(json_str);
        assert!(env.is_ok(), "ResponseEnvelope must parse structured error: {:?}", env.err());
        let env = env.unwrap();
        assert_eq!(env.success, Some(false));
        if let Some(Value::Object(map)) = env.error {
            assert_eq!(map.get("code").and_then(|v| v.as_str()), Some("RATE_LIMIT"));
        } else {
            panic!("Expected structured error object");
        }
    }

    #[test]
    fn test_response_envelope_success() {
        let json_str = r#"{"success":true,"data":{"ok":true}}"#;
        let env: Result<ResponseEnvelope, _> = serde_json::from_str(json_str);
        assert!(env.is_ok());
        let env = env.unwrap();
        assert_eq!(env.success, Some(true));
        assert!(env.data.is_some());
    }
}

