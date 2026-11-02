//! Employee directory — read-only view of the Google Sheet's
//! `employees.list` (cached in the local `USERS` table).
//!
//! The sheet is the source of truth: rows are refreshed by the background
//! sync worker (every 30 minutes and on launch) and can be refreshed on
//! demand with the toolbar button. Offline, the cached copy is shown.

#![allow(clippy::redundant_closure)]

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
use crate::components::ui::button::Button;
use crate::components::ui::data_table::{DataTable, DataTableColumn, DataTableFilter, SortValue};
use crate::icons::UsersIcon;
use crate::state::AppDbRevision;

/// One employee row rendered in the directory table.
#[derive(Clone, PartialEq, Debug)]
pub struct EmployeeItem {
    pub emp_id: i64,
    pub emp_code: String,
    pub emp_name: String,
    pub emp_email: String,
    pub department: String,
    pub division: String,
}

impl EmployeeItem {
    fn from_db(e: crate::db::DbEmployee) -> Self {
        EmployeeItem {
            emp_id: e.emp_id,
            emp_code: e.emp_code,
            emp_name: e.emp_name,
            emp_email: e.emp_email,
            department: e.department,
            division: e.division,
        }
    }

    /// Initials for the avatar chip ("ABHISHEK KUMAR SHARMA" → "AK").
    fn initials(&self) -> String {
        self.emp_name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

// ============================================================
// DataTable plumbing (plain fn pointers, as the table requires)
// ============================================================

fn employee_row_id(e: &EmployeeItem) -> String {
    // Codes are not unique in the sheet (27 rows share 10081), so fall
    // back to the local primary key.
    format!("emp-{}", e.emp_id)
}

fn employee_search_text(e: &EmployeeItem) -> String {
    format!(
        "{} {} {} {} {}",
        e.emp_name, e.emp_code, e.emp_email, e.department, e.division
    )
}

fn employee_department_value(e: &EmployeeItem) -> String {
    if e.department.is_empty() {
        "—".to_string()
    } else {
        e.department.clone()
    }
}

fn employee_division_value(e: &EmployeeItem) -> String {
    if e.division.is_empty() {
        "—".to_string()
    } else {
        e.division.clone()
    }
}

fn render_code_cell(e: &EmployeeItem) -> Element {
    let code = if e.emp_code.is_empty() {
        "—".to_string()
    } else {
        e.emp_code.clone()
    };
    rsx! {
        span { class: "font-mono text-xs text-muted-foreground", "{code}" }
    }
}

fn render_name_cell(e: &EmployeeItem) -> Element {
    rsx! {
        div { class: "flex items-center gap-2.5 min-w-0",
            span {
                class: "flex size-7 shrink-0 items-center justify-center rounded-full \
                        bg-primary/10 text-primary border border-primary/20 text-[10px] font-bold",
                "{e.initials()}"
            }
            span { class: "truncate font-medium text-foreground", "{e.emp_name}" }
        }
    }
}

fn render_email_cell(e: &EmployeeItem) -> Element {
    if e.emp_email.is_empty() {
        // 293 of 324 sheet rows genuinely have no email.
        rsx! {
            span { class: "text-xs italic text-muted-foreground/60", "no email on file" }
        }
    } else {
        rsx! {
            span {
                class: "text-xs text-muted-foreground truncate",
                title: "{e.emp_email}",
                "{e.emp_email}"
            }
        }
    }
}

fn render_department_cell(e: &EmployeeItem) -> Element {
    let dept = if e.department.is_empty() {
        "—".to_string()
    } else {
        e.department.clone()
    };
    rsx! {
        Badge { variant: "outline".to_string(), "{dept}" }
    }
}

fn render_division_cell(e: &EmployeeItem) -> Element {
    let div = if e.division.is_empty() {
        "—".to_string()
    } else {
        e.division.clone()
    };
    rsx! {
        span { class: "text-xs text-muted-foreground", "{div}" }
    }
}

fn employee_columns() -> Vec<DataTableColumn<EmployeeItem>> {
    vec![
        DataTableColumn::new(
            "name",
            "Employee",
            render_name_cell,
            |e: &EmployeeItem| SortValue::Text(e.emp_name.clone()),
        ),
        DataTableColumn::new(
            "code",
            "Code",
            render_code_cell,
            |e: &EmployeeItem| SortValue::Text(e.emp_code.clone()),
        ),
        DataTableColumn::new(
            "email",
            "Email",
            render_email_cell,
            |e: &EmployeeItem| SortValue::Text(e.emp_email.clone()),
        ),
        DataTableColumn::new(
            "department",
            "Department",
            render_department_cell,
            |e: &EmployeeItem| SortValue::Text(e.department.clone()),
        ),
        DataTableColumn::new(
            "division",
            "Division",
            render_division_cell,
            |e: &EmployeeItem| SortValue::Text(e.division.clone()),
        ),
    ]
}

// ============================================================
// Page
// ============================================================

#[component]
pub fn Employees() -> Element {
    let db_sync = try_use_context::<Signal<AppDbRevision>>();

    // Reload whenever the sync worker bumps the revision (employee pull)
    // or the user hits Refresh.
    let local_revision = use_signal(|| 0u64);

    let employees: Vec<EmployeeItem> = {
        let _ = local_revision();
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        crate::db::get_all_employees()
            .into_iter()
            .map(EmployeeItem::from_db)
            .collect()
    };

    let total_count = employees.len();
    let department_options = crate::db::employee_departments();
    let division_options = crate::db::employee_divisions();

    // Facet filters are built from the cached data so they always match
    // what is actually present.
    let department_filter: Vec<DataTableFilter> = department_options
        .iter()
        .map(|d| DataTableFilter::new(d.clone(), d.clone()))
        .collect();
    let division_filter: Vec<DataTableFilter> = division_options
        .iter()
        .map(|d| DataTableFilter::new(d.clone(), d.clone()))
        .collect();

    rsx! {
        div { class: "flex flex-1 flex-col h-full overflow-hidden bg-background",

            // ============================================================
            // Page Header
            // ============================================================
            header { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-border/70 bg-card/60 backdrop-blur-md px-6 py-4.5",
                div { class: "flex items-center gap-3",
                    div { class: "flex size-10 items-center justify-center rounded-xl bg-muted text-foreground border border-border/80 shadow-xs",
                        UsersIcon { class: "size-5" }
                    }
                    div { class: "flex flex-col",
                        div { class: "flex items-center gap-2.5",
                            h1 { class: "text-lg font-bold tracking-tight text-foreground", "Employees" }
                            span { class: "rounded-full bg-muted border border-border/80 px-2.5 py-0.5 text-xs font-semibold text-muted-foreground",
                                "{total_count} in directory"
                            }
                        }
                        p { class: "mt-0.5 text-xs text-muted-foreground",
                            "Synced from Google Sheets · cached locally for offline use"
                        }
                    }
                }

                div { class: "flex items-center gap-2",
                    Button {
                        variant: "outline".to_string(),
                        size: "sm".to_string(),
                        title: "Pull the latest employee list from Google Sheets now".to_string(),
                        onclick: move |_| {
                            // Reset the pull timer and wake the worker —
                            // the refresh itself happens in the background.
                            crate::sync::invalidate_employee_cache();
                            crate::sync::kick();
                            if let Some(mut sync) = db_sync {
                                sync.with_mut(|r| r.0 += 1);
                            }
                        },
                        RefreshIcon { class: "size-3.5 mr-1.5" }
                        "Refresh from Sheet"
                    }
                }
            }

            // ============================================================
            // Directory table (search + department/division facets)
            // ============================================================
            div { class: "flex flex-1 min-h-0 flex-col gap-3 p-6 overflow-hidden",
                if total_count == 0 {
                    div { class: "flex flex-1 flex-col items-center justify-center gap-3 rounded-lg border border-dashed border-border/70 bg-card/40 text-center",
                        UsersIcon { class: "size-8 text-muted-foreground/50" }
                        div { class: "flex flex-col gap-1",
                            p { class: "text-sm font-semibold text-foreground", "No employees cached yet" }
                            p { class: "text-xs text-muted-foreground max-w-sm",
                                "The directory loads from Google Sheets during the next sync — \
                                 hit \"Refresh from Sheet\" to pull it now."
                            }
                        }
                    }
                } else {
                    DataTable {
                        data: employees,
                        columns: employee_columns(),
                        row_id: employee_row_id,
                        search_text: employee_search_text,
                        search_placeholder: Some("Search name, code, email, department...".to_string()),
                        filter_label: Some("Department".to_string()),
                        filter: department_filter,
                        filter_value: Some(employee_department_value as fn(&EmployeeItem) -> String),
                        filter2_label: Some("Division".to_string()),
                        filter2: division_filter,
                        filter2_value: Some(employee_division_value as fn(&EmployeeItem) -> String),
                        page_size: 25,
                        initial_sort: Some(("name".to_string(), true)),
                    }
                }
            }
        }
    }
}

/// Small inline refresh icon (lucide "refresh-cw") — not part of the
/// shared icon set.
#[component]
fn RefreshIcon(props: crate::icons::IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" }
            path { d: "M21 3v5h-5" }
            path { d: "M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" }
            path { d: "M8 16H3v5" }
        }
    }
}
