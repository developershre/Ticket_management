//! Global CRUD Implementation for Google Sheets Backend.
//!
//! Provides a standardized, type-safe interface for Create, Read, Update, and Delete
//! operations across any entity mapped to a Google Sheet (Tickets, Employees, Reminders, Notes).
//!
//! Supports automatic schema verification and initialization so that any Google Sheet
//! connected by a user can be validated and provisioned immediately.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::api::{ApiClient, ApiError, ApiProtocol, Employee, ServerReminder, ServerTicket};
use crate::db::DbNote;

/// Represents a model entity mapped to a specific tab in a connected Google Sheet.
pub trait SheetEntity: Serialize + DeserializeOwned + Send + Sync + 'static {
    const SHEET_NAME: &'static str;
    const ID_KEY: &'static str;
    const REQUIRED_COLUMNS: &'static [&'static str];

    fn entity_id(&self) -> String;
}

impl SheetEntity for ServerTicket {
    const SHEET_NAME: &'static str = "Tickets";
    const ID_KEY: &'static str = "Ticket ID";
    const REQUIRED_COLUMNS: &'static [&'static str] = &[
        "Ticket ID",
        "Employee Code",
        "Email Address",
        "Category",
        "Subject",
        "Problem Description",
        "Priority",
        "Assigned to",
        "Customer",
        "Kanban State",
        "Stage",
        "Status",
        "Created on",
        "Last Updated on",
    ];

    fn entity_id(&self) -> String {
        self.ticket_id.clone()
    }
}

impl SheetEntity for Employee {
    const SHEET_NAME: &'static str = "Employees";
    const ID_KEY: &'static str = "Employee Code";
    const REQUIRED_COLUMNS: &'static [&'static str] = &[
        "Employee Code",
        "Name",
        "Dept",
        "Division",
        "Mail",
    ];

    fn entity_id(&self) -> String {
        self.employee_code.clone()
    }
}

impl SheetEntity for ServerReminder {
    const SHEET_NAME: &'static str = "Reminders";
    const ID_KEY: &'static str = "Reminder ID";
    const REQUIRED_COLUMNS: &'static [&'static str] = &[
        "Reminder ID",
        "Employee Code",
        "Employee Name",
        "Email Address",
        "Reminder Message",
        "Send At",
        "Status",
        "Created At",
    ];

    fn entity_id(&self) -> String {
        self.reminder_id.clone()
    }
}

impl SheetEntity for DbNote {
    const SHEET_NAME: &'static str = "Notes";
    const ID_KEY: &'static str = "Note ID";
    const REQUIRED_COLUMNS: &'static [&'static str] = &[
        "Note ID",
        "Employee Code",
        "Content",
        "Created At",
        "Last Updated At",
    ];

    fn entity_id(&self) -> String {
        self.id.to_string()
    }
}

/// Generic page structure for paginated list queries.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CrudListPage<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub offset: u64,
    pub limit: u64,
    pub has_more: bool,
}

/// Report returned by Google Sheet schema verification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaVerificationReport {
    pub valid: bool,
    #[serde(default)]
    pub spreadsheet_title: Option<String>,
    #[serde(default)]
    pub sheets: HashMap<String, SheetValidation>,
    #[serde(default)]
    pub missing_sheets: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SheetValidation {
    #[serde(default)]
    pub exists: bool,
    #[serde(default)]
    pub total_rows: usize,
    #[serde(default)]
    pub columns_found: Vec<String>,
    #[serde(default)]
    pub missing_columns: Vec<String>,
    #[serde(default)]
    pub is_valid: bool,
}

impl ApiClient {
    /// Global CRUD: List records with pagination and optional query filtering.
    pub async fn crud_list<T: DeserializeOwned>(
        &self,
        sheet_name: &str,
        offset: u64,
        limit: u64,
    ) -> Result<CrudListPage<T>, ApiError> {
        let offset_str = offset.to_string();
        let limit_str = limit.to_string();

        let raw_data = match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [
                    ("action", "list"),
                    ("sheet", sheet_name),
                    ("offset", &offset_str),
                    ("limit", &limit_str),
                ];
                self.send_rest(reqwest::Method::GET, "crud", &query, None).await?
            }
            ApiProtocol::LegacyV1 => {
                self.post(
                    "crud.list",
                    json!({
                        "sheet": sheet_name,
                        "offset": offset,
                        "limit": limit
                    }),
                )
                .await?
            }
        };

        let items_val = raw_data
            .get("items")
            .or_else(|| raw_data.get("data"))
            .cloned()
            .unwrap_or(raw_data.clone());

        let items: Vec<T> = if let Some(arr) = items_val.as_array() {
            arr.iter()
                .filter_map(|v| serde_json::from_value::<T>(v.clone()).ok())
                .collect()
        } else {
            Vec::new()
        };

        let total = raw_data
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(items.len() as u64);
        let has_more = raw_data
            .get("has_more")
            .and_then(|v| v.as_bool())
            .unwrap_or(offset + (items.len() as u64) < total);

        Ok(CrudListPage {
            items,
            total,
            offset,
            limit,
            has_more,
        })
    }

    /// Global CRUD: Get record by ID.
    pub async fn crud_get<T: DeserializeOwned>(
        &self,
        sheet_name: &str,
        id: &str,
    ) -> Result<T, ApiError> {
        let raw_data = match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [("action", "get"), ("sheet", sheet_name), ("id", id)];
                self.send_rest(reqwest::Method::GET, "crud", &query, None).await?
            }
            ApiProtocol::LegacyV1 => {
                self.post(
                    "crud.get",
                    json!({
                        "sheet": sheet_name,
                        "id": id
                    }),
                )
                .await?
            }
        };

        let unwrapped = raw_data
            .get("data")
            .or_else(|| raw_data.get("record"))
            .cloned()
            .unwrap_or(raw_data);

        serde_json::from_value::<T>(unwrapped)
            .map_err(|e| ApiError::Decode(format!("failed to deserialize record: {e}")))
    }

    /// Global CRUD: Create record directly in the Google Sheet.
    pub async fn crud_create<T: DeserializeOwned>(
        &self,
        sheet_name: &str,
        fields: &Value,
    ) -> Result<T, ApiError> {
        let raw_data = match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [("action", "create"), ("sheet", sheet_name)];
                self.send_rest(
                    reqwest::Method::POST,
                    "crud",
                    &query,
                    Some(fields.clone()),
                )
                .await?
            }
            ApiProtocol::LegacyV1 => {
                self.post(
                    "crud.create",
                    json!({
                        "sheet": sheet_name,
                        "data": fields
                    }),
                )
                .await?
            }
        };

        let unwrapped = raw_data
            .get("data")
            .or_else(|| raw_data.get("ticket"))
            .or_else(|| raw_data.get("record"))
            .cloned()
            .unwrap_or(raw_data);

        serde_json::from_value::<T>(unwrapped)
            .map_err(|e| ApiError::Decode(format!("failed to deserialize created record: {e}")))
    }

    /// Global CRUD: Update record by ID in the Google Sheet.
    pub async fn crud_update<T: DeserializeOwned>(
        &self,
        sheet_name: &str,
        id: &str,
        patch: &Value,
    ) -> Result<T, ApiError> {
        let raw_data = match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [
                    ("action", "update"),
                    ("sheet", sheet_name),
                    ("id", id),
                ];
                self.send_rest(
                    reqwest::Method::POST,
                    "crud",
                    &query,
                    Some(patch.clone()),
                )
                .await?
            }
            ApiProtocol::LegacyV1 => {
                self.post(
                    "crud.update",
                    json!({
                        "sheet": sheet_name,
                        "id": id,
                        "data": patch
                    }),
                )
                .await?
            }
        };

        let unwrapped = raw_data
            .get("data")
            .or_else(|| raw_data.get("ticket"))
            .or_else(|| raw_data.get("record"))
            .cloned()
            .unwrap_or(raw_data);

        serde_json::from_value::<T>(unwrapped)
            .map_err(|e| ApiError::Decode(format!("failed to deserialize updated record: {e}")))
    }

    /// Global CRUD: Delete record by ID from the Google Sheet.
    pub async fn crud_delete(&self, sheet_name: &str, id: &str) -> Result<(), ApiError> {
        match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [
                    ("action", "delete"),
                    ("sheet", sheet_name),
                    ("id", id),
                ];
                self.send_rest(reqwest::Method::POST, "crud", &query, Some(json!({}))).await?;
            }
            ApiProtocol::LegacyV1 => {
                self.post(
                    "crud.delete",
                    json!({
                        "sheet": sheet_name,
                        "id": id
                    }),
                )
                .await?;
            }
        }
        Ok(())
    }

    /// Verifies that the connected Google Sheet has the required tabs and columns.
    pub async fn crud_verify_schema(&self) -> Result<SchemaVerificationReport, ApiError> {
        let res = match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [("action", "verify")];
                self.send_rest(reqwest::Method::GET, "schema", &query, None).await
            }
            ApiProtocol::LegacyV1 => {
                self.post("schema.verify", json!({})).await
            }
        };

        match res {
            Ok(val) => {
                let unwrapped = val.get("data").cloned().unwrap_or(val);
                serde_json::from_value::<SchemaVerificationReport>(unwrapped)
                    .map_err(|e| ApiError::Decode(format!("failed to parse schema report: {e}")))
            }
            Err(e) => {
                // If the remote endpoint doesn't support schema.verify yet, test basic connectivity
                if e.is_not_found() || format!("{e}").contains("UNKNOWN_ROUTE") {
                    let health_test = self.health().await;
                    let is_healthy = health_test.is_ok();
                    let mut sheets = HashMap::new();
                    sheets.insert(
                        "Tickets".to_string(),
                        SheetValidation {
                            exists: is_healthy,
                            total_rows: 0,
                            columns_found: vec![],
                            missing_columns: vec![],
                            is_valid: is_healthy,
                        },
                    );
                    Ok(SchemaVerificationReport {
                        valid: is_healthy,
                        spreadsheet_title: Some("Connected Sheet (Legacy / REST)".to_string()),
                        sheets,
                        missing_sheets: vec![],
                        errors: if is_healthy {
                            vec![]
                        } else {
                            vec!["Could not connect to Google Sheet".to_string()]
                        },
                    })
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Auto-initializes or repairs all missing tabs and standard headers on the Google Sheet.
    pub async fn crud_init_schema(&self) -> Result<Value, ApiError> {
        match self.protocol() {
            ApiProtocol::BpslRest => {
                let query = [("action", "init")];
                self.send_rest(reqwest::Method::POST, "schema", &query, Some(json!({}))).await
            }
            ApiProtocol::LegacyV1 => {
                self.post("schema.init", json!({})).await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sheet_entity_contracts() {
        assert_eq!(ServerTicket::SHEET_NAME, "Tickets");
        assert_eq!(ServerTicket::ID_KEY, "Ticket ID");
        assert!(ServerTicket::REQUIRED_COLUMNS.contains(&"Ticket ID"));
        assert!(ServerTicket::REQUIRED_COLUMNS.contains(&"Problem Description"));

        assert_eq!(Employee::SHEET_NAME, "Employees");
        assert_eq!(Employee::ID_KEY, "Employee Code");

        assert_eq!(ServerReminder::SHEET_NAME, "Reminders");
        assert_eq!(ServerReminder::ID_KEY, "Reminder ID");

        assert_eq!(DbNote::SHEET_NAME, "Notes");
        assert_eq!(DbNote::ID_KEY, "Note ID");
    }

    #[test]
    fn test_schema_verification_report_deserialization() {
        let json_data = json!({
            "valid": true,
            "spreadsheet_title": "BPSL Support Master Sheet",
            "sheets": {
                "Tickets": {
                    "exists": true,
                    "total_rows": 55,
                    "columns_found": ["Ticket ID", "Subject", "Status"],
                    "missing_columns": [],
                    "is_valid": true
                },
                "Employees": {
                    "exists": true,
                    "total_rows": 120,
                    "columns_found": ["Employee Code", "Name"],
                    "missing_columns": [],
                    "is_valid": true
                }
            },
            "missing_sheets": [],
            "errors": []
        });

        let report: SchemaVerificationReport = serde_json::from_value(json_data).unwrap();
        assert!(report.valid);
        assert_eq!(report.spreadsheet_title.as_deref(), Some("BPSL Support Master Sheet"));
        assert_eq!(report.sheets.len(), 2);
        assert!(report.sheets["Tickets"].is_valid);
        assert_eq!(report.sheets["Tickets"].total_rows, 55);
    }

    #[test]
    fn test_server_ticket_to_ticket_item_conversion() {
        let server_ticket: ServerTicket = serde_json::from_value(json!({
            "ticket_id": "IT-0999",
            "employee_code": "11061",
            "email_address": "chaitram@bpsl.com",
            "category": "Hardware",
            "subject": "Workstation PSU broken",
            "problem_description": "Power supply failed",
            "priority": "High",
            "assigned_to": "Devendra Patel",
            "customer": "Internal",
            "time_spent": 2.5,
            "activities": "PSU replaced",
            "my_activity": "Bench testing",
            "deadline": "Tomorrow",
            "sla_status": "Within SLA",
            "sla_deadline": "Within 24 Hours",
            "rating_avg_text": "Unrated",
            "kanban_state": "InProgress",
            "stage": "In Progress",
            "status": "Open",
            "created_on": "2026-11-21T10:00:00Z",
            "last_updated_on": "2026-11-21T11:00:00Z"
        })).unwrap();

        let item = crate::pages::tickets::types::TicketItem::from_server_ticket(&server_ticket, 42);
        assert_eq!(item.id, "IT-0999");
        assert_eq!(item.ticket_id, 42);
        assert_eq!(item.title, "Workstation PSU broken");
        assert_eq!(item.kanban_state, "InProgress");
        assert_eq!(item.stage, "In Progress");
        assert_eq!(item.status, "Open");
    }
}
