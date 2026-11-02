//! Centralized Google Sheets TMS API client.
//!
//! Single place that knows how to talk to the backend described by
//! `TMS_API.postman_collection.json`:
//!
//! * one shared [`reqwest`] HTTP client with a request timeout,
//! * the JSON request envelope (`version` / `action` / `auth.api_key` /
//!   `request_id` / `data`),
//! * the response envelope (`success` / `data` / `error.retryable` …),
//! * typed helpers for the `tickets.*` actions.
//!
//! Configuration (base URL + API key) is resolved from environment
//! variables first and falls back to the Postman collection file — no
//! credentials are hardcoded in source.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Overall request timeout. Google Apps Script can be slow to cold-start,
/// so this is deliberately generous but still bounded.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

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
    /// 1. `TMS_API_BASE_URL`, `TMS_API_KEY`, `TMS_EMPLOYEE_CODE` env vars
    /// 2. `base_url` / `api_key` variables of `TMS_API.postman_collection.json`
    ///    (searched in the working directory, next to the executable and one
    ///    level above it — same lookup strategy as `management.db`).
    pub fn load() -> Result<ApiConfig, String> {
        let env_url = non_empty_env("TMS_API_BASE_URL");
        let env_key = non_empty_env("TMS_API_KEY");
        let employee_code = non_empty_env("TMS_EMPLOYEE_CODE");

        let (file_url, file_key) = Self::from_collection_file();

        let base_url = env_url.or(file_url).ok_or_else(|| {
            "API base URL not configured — set TMS_API_BASE_URL or keep \
             TMS_API.postman_collection.json next to the application"
                .to_string()
        })?;
        let api_key = env_key.or(file_key).ok_or_else(|| {
            "API key not configured — set TMS_API_KEY or keep \
             TMS_API.postman_collection.json next to the application"
                .to_string()
        })?;

        Ok(ApiConfig {
            base_url: base_url.trim().to_string(),
            api_key: api_key.trim().to_string(),
            employee_code,
        })
    }

    /// Reads the `base_url` and `api_key` collection variables from the
    /// Postman collection file, if any candidate location contains it.
    fn from_collection_file() -> (Option<String>, Option<String>) {
        for path in Self::collection_candidates() {
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

    fn collection_candidates() -> Vec<PathBuf> {
        const FILE: &str = "TMS_API.postman_collection.json";
        let mut out = vec![PathBuf::from(FILE)];
        if let Ok(cwd) = std::env::current_dir() {
            out.push(cwd.join(FILE));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                out.push(dir.join(FILE));
                if let Some(parent) = dir.parent() {
                    out.push(parent.join(FILE));
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
    /// Transport-level failure (DNS, connection refused, TLS …).
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
    /// database) — transient, retried next cycle.
    Db(String),
}

impl ApiError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ApiError::Network(_) | ApiError::Timeout | ApiError::Decode(_) | ApiError::Db(_) => true,
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

    /// True when the API reported "ticket not found" — an idempotent
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
                write!(f, "TIMEOUT: no response within {}s", REQUEST_TIMEOUT.as_secs())
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
/// ignored — the fallback backoff applies instead).
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
/// the sheet cell was written — both are normalized here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerTicket {
    pub ticket_id: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub employee_code: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub email_address: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub category: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub subject: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub problem_description: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub attachment_name: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub attachment_url: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub priority: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub assigned_to: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub customer: String,
    #[serde(default, deserialize_with = "flex_f64")]
    pub time_spent: f64,
    #[serde(default, deserialize_with = "flex_str", rename = "activities_text")]
    pub activities: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub my_activity: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub deadline: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub sla_status: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub sla_deadline: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub rating_avg_text: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub kanban_state: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub stage: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub status: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub created_on: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub last_updated_on: String,
}

/// One page of `tickets.list`.
pub struct TicketListPage {
    pub items: Vec<ServerTicket>,
    /// Rows that failed validation (missing `ticket_id` etc.) — skipped,
    /// never fatal for the whole page.
    pub invalid_items: usize,
    pub has_more: bool,
}

// ============================================================
// Employee row (shape verified against the live `employees.list`)
// ============================================================

/// One employee as returned by `employees.list`.
///
/// The sheet's real data is messy on purpose: `employee_code` is missing
/// on one row and shared by 27 rows, and 293 of 324 rows have no email
/// (4 emails are shared by several people) — so every field is optional
/// on the wire and normalized to a `String` here.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Employee {
    #[serde(default, deserialize_with = "flex_str")]
    pub employee_code: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub employee_name: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub department: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub email: String,
    #[serde(default, deserialize_with = "flex_str")]
    pub division: String,
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
            .user_agent("TaskFlow-TMS/1.0")
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
    /// answers with HTTP 200 even for failures — `success`/`error` live in
    /// the body.
    async fn post(&self, action: &str, data: Value) -> Result<Value, ApiError> {
        let envelope = RequestEnvelope {
            version: "v1",
            action,
            auth: Auth {
                api_key: &self.cfg.api_key,
            },
            request_id: self.request_id(),
            data,
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

        let text = resp.text().await.map_err(|e| {
            ApiError::Network(format!("failed to read response: {e}"))
        })?;

        if status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error() {
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

        // Destructure `error` once (it is moved by the first access).
        let (code, body_retryable) = match envelope.error {
            Some(err) => (
                err.code.unwrap_or_else(|| "UNKNOWN".to_string()),
                err.retryable.unwrap_or(false),
            ),
            None => ("UNKNOWN".to_string(), false),
        };

        Err(ApiError::Api {
            code,
            message: envelope
                .message
                .unwrap_or_else(|| "request failed".to_string()),
            retryable: body_retryable,
            retry_after,
        })
    }

    // ---- actions (TMS_API.postman_collection.json) ----

    /// `health.check` (no auth required).
    pub async fn health(&self) -> Result<Value, ApiError> {
        self.post("health.check", json!({})).await
    }

    /// `tickets.list` — optional filters are omitted to fetch every row;
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
        let data = self.post("tickets.get", json!({ "ticket_id": ticket_id })).await?;
        parse_ticket(data)
    }

    /// `tickets.create` — `fields` must contain only real row fields (the
    /// API rejects unknown fields with `UNKNOWN_FIELD`).
    pub async fn tickets_create(&self, fields: &Value) -> Result<ServerTicket, ApiError> {
        let data = self.post("tickets.create", fields.clone()).await?;
        parse_ticket(data)
    }

    /// `tickets.update` — "send ticket_id and only the fields to update".
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

    /// `tickets.delete` — hard delete; `NOT_FOUND` is treated as success
    /// by callers (already gone).
    pub async fn tickets_delete(&self, ticket_id: &str) -> Result<(), ApiError> {
        self.post("tickets.delete", json!({ "ticket_id": ticket_id })).await?;
        Ok(())
    }

    /// `employees.list` — the full employee directory. The endpoint
    /// accepts optional `department` / `division` / `query` filters and
    /// returns `{ items, total }` (not paginated: one call yields all
    /// 324 rows).
    pub async fn employees_list(&self) -> Result<Vec<Employee>, ApiError> {
        let data = self.post("employees.list", json!({})).await?;

        let mut items = Vec::new();
        if let Some(arr) = data.get("items").and_then(|v| v.as_array()) {
            for value in arr {
                // Rows without a usable name are skipped, never fatal.
                if let Ok(emp) = serde_json::from_value::<Employee>(value.clone()) {
                    if !emp.employee_name.trim().is_empty() {
                        items.push(emp);
                    }
                }
            }
        }
        Ok(items)
    }
}

fn parse_ticket(data: Value) -> Result<ServerTicket, ApiError> {
    serde_json::from_value(data).map_err(|e| ApiError::Decode(format!("invalid ticket payload: {e}")))
}
