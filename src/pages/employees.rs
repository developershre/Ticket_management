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
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::icons::{PencilIcon, PlusIcon, TrashIcon, UsersIcon};
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

#[derive(Clone, Copy, PartialEq)]
struct EmployeeActionState {
    edit_open: Signal<bool>,
    delete_open: Signal<bool>,
    target_code: Signal<String>,
    target_name: Signal<String>,
    target_email: Signal<String>,
    target_dept: Signal<String>,
    target_div: Signal<String>,
}

#[component]
fn EmployeeActionCell(emp: EmployeeItem) -> Element {
    let mut state = use_context::<EmployeeActionState>();
    let emp_edit = emp.clone();
    let emp_del = emp.clone();
    rsx! {
        div { class: "flex items-center justify-end gap-1",
            button {
                r#type: "button",
                title: "Edit Employee",
                class: "flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-muted hover:text-foreground cursor-pointer transition-colors",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    state.target_code.set(emp_edit.emp_code.clone());
                    state.target_name.set(emp_edit.emp_name.clone());
                    state.target_email.set(emp_edit.emp_email.clone());
                    state.target_dept.set(emp_edit.department.clone());
                    state.target_div.set(emp_edit.division.clone());
                    state.edit_open.set(true);
                },
                PencilIcon { class: "size-3.5" }
            }
            button {
                r#type: "button",
                title: "Delete Employee",
                class: "flex size-7 items-center justify-center rounded text-muted-foreground hover:bg-destructive/15 hover:text-destructive cursor-pointer transition-colors",
                onclick: move |e: MouseEvent| {
                    e.stop_propagation();
                    state.target_code.set(emp_del.emp_code.clone());
                    state.target_name.set(emp_del.emp_name.clone());
                    state.delete_open.set(true);
                },
                TrashIcon { class: "size-3.5" }
            }
        }
    }
}

fn render_action_cell(e: &EmployeeItem) -> Element {
    rsx! {
        EmployeeActionCell { emp: e.clone() }
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
        DataTableColumn::new(
            "actions",
            "",
            render_action_cell,
            |_: &EmployeeItem| SortValue::Text(String::new()),
        ),
    ]
}

// ============================================================
// Page
// ============================================================

#[component]
pub fn Employees() -> Element {
    let db_sync = try_use_context::<Signal<AppDbRevision>>();
    let toast = use_context::<ToastState>();

    // Reload whenever the sync worker bumps the revision (employee pull)
    // or the user hits Refresh.
    let local_revision = use_signal(|| 0u64);

    let mut edit_open = use_signal(|| false);
    let mut delete_open = use_signal(|| false);
    let target_code = use_signal(String::new);
    let mut target_name = use_signal(String::new);
    let mut target_email = use_signal(String::new);
    let mut target_dept = use_signal(String::new);
    let mut target_div = use_signal(String::new);

    let mut add_open = use_signal(|| false);
    let mut add_code = use_signal(String::new);
    let mut add_name = use_signal(String::new);
    let mut add_email = use_signal(String::new);
    let mut add_dept = use_signal(String::new);
    let mut add_div = use_signal(String::new);

    use_context_provider(|| EmployeeActionState {
        edit_open,
        delete_open,
        target_code,
        target_name,
        target_email,
        target_dept,
        target_div,
    });

    let (employees, department_options, division_options) = {
        let memo = use_memo(move || {
            let _ = local_revision();
            if let Some(sync) = db_sync {
                let _ = sync();
            }
            let emps: Vec<EmployeeItem> = crate::db::get_all_employees()
                .into_iter()
                .map(EmployeeItem::from_db)
                .collect();
            let depts = crate::db::employee_departments();
            let divs = crate::db::employee_divisions();
            (emps, depts, divs)
        });
        memo()
    };
    let total_count = employees.len();

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
                            "Synced with Google Sheets API · cached locally for offline use"
                        }
                    }
                }

                div { class: "flex items-center gap-2",
                    Button {
                        variant: "outline".to_string(),
                        size: "sm".to_string(),
                        title: "Pull the latest employee list from Google Sheets now".to_string(),
                        onclick: move |_| {
                            crate::sync::invalidate_employee_cache();
                            crate::sync::kick();
                            if let Some(mut sync) = db_sync {
                                sync.with_mut(|r| r.0 += 1);
                            }
                            toast.info("Syncing", "Pulling employees from Google Sheets…");
                        },
                        RefreshIcon { class: "size-3.5 mr-1.5" }
                        "Refresh from Sheet"
                    }
                    Button {
                        variant: "default".to_string(),
                        size: "sm".to_string(),
                        onclick: move |_| {
                            add_code.write().clear();
                            add_name.write().clear();
                            add_email.write().clear();
                            add_dept.write().clear();
                            add_div.write().clear();
                            add_open.set(true);
                        },
                        PlusIcon { class: "size-3.5 mr-1.5" }
                        "Add Employee"
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

            // ============================================================
            // Add Employee Dialog
            // ============================================================
            if add_open() {
                Dialog {
                    open: Some(add_open),
                    DialogContent {
                        class: "max-w-md",
                        DialogHeader {
                            DialogTitle { "Add Employee" }
                            DialogDescription { "Create a new employee profile in Google Sheets." }
                        }
                        div { class: "flex flex-col gap-3 py-2 text-xs",
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Employee Code *" }
                                Input {
                                    placeholder: "e.g. EMP001".to_string(),
                                    value: add_code.read().clone(),
                                    oninput: move |e: FormEvent| add_code.set(e.value()),
                                }
                            }
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Employee Name *" }
                                Input {
                                    placeholder: "e.g. Aarav Sharma".to_string(),
                                    value: add_name.read().clone(),
                                    oninput: move |e: FormEvent| add_name.set(e.value()),
                                }
                            }
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Email Address" }
                                Input {
                                    placeholder: "e.g. employee@example.com".to_string(),
                                    value: add_email.read().clone(),
                                    oninput: move |e: FormEvent| add_email.set(e.value()),
                                }
                            }
                            div { class: "grid grid-cols-2 gap-2",
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Department" }
                                    Input {
                                        placeholder: "e.g. IT".to_string(),
                                        value: add_dept.read().clone(),
                                        oninput: move |e: FormEvent| add_dept.set(e.value()),
                                    }
                                }
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Division" }
                                    Input {
                                        placeholder: "e.g. Corporate".to_string(),
                                        value: add_div.read().clone(),
                                        oninput: move |e: FormEvent| add_div.set(e.value()),
                                    }
                                }
                            }
                        }
                        DialogFooter {
                            Button {
                                variant: "outline",
                                onclick: move |_| add_open.set(false),
                                "Cancel"
                            }
                            Button {
                                variant: "default",
                                onclick: move |_| {
                                    let code = add_code.read().trim().to_string();
                                    let name = add_name.read().trim().to_string();
                                    if code.is_empty() || name.is_empty() {
                                        toast.error("Validation Error", "Employee code and name are required.");
                                        return;
                                    }
                                    let emp = crate::api::Employee {
                                        employee_code: code.clone(),
                                        employee_name: name.clone(),
                                        department: add_dept.read().trim().to_string(),
                                        email: add_email.read().trim().to_string(),
                                        division: add_div.read().trim().to_string(),
                                    };
                                    spawn(async move {
                                        match crate::sync::create_employee_immediate(&emp).await {
                                            Ok(_) => {
                                                toast.success("Employee Created", format!("{name} ({code}) added to sheet"));
                                                add_open.set(false);
                                                if let Some(mut sync) = db_sync {
                                                    sync.with_mut(|r| r.0 += 1);
                                                }
                                            }
                                            Err(e) => {
                                                toast.error("Creation Failed", format!("Failed to create employee: {e}"));
                                            }
                                        }
                                    });
                                },
                                "Create"
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Edit Employee Dialog
            // ============================================================
            if edit_open() {
                Dialog {
                    open: Some(edit_open),
                    DialogContent {
                        class: "max-w-md",
                        DialogHeader {
                            DialogTitle { "Edit Employee: {target_code}" }
                            DialogDescription { "Update employee information in Google Sheets." }
                        }
                        div { class: "flex flex-col gap-3 py-2 text-xs",
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Employee Name *" }
                                Input {
                                    value: target_name.read().clone(),
                                    oninput: move |e: FormEvent| target_name.set(e.value()),
                                }
                            }
                            div { class: "flex flex-col gap-1",
                                label { class: "text-xs font-semibold text-foreground", "Email Address" }
                                Input {
                                    value: target_email.read().clone(),
                                    oninput: move |e: FormEvent| target_email.set(e.value()),
                                }
                            }
                            div { class: "grid grid-cols-2 gap-2",
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Department" }
                                    Input {
                                        value: target_dept.read().clone(),
                                        oninput: move |e: FormEvent| target_dept.set(e.value()),
                                    }
                                }
                                div { class: "flex flex-col gap-1",
                                    label { class: "text-xs font-semibold text-foreground", "Division" }
                                    Input {
                                        value: target_div.read().clone(),
                                        oninput: move |e: FormEvent| target_div.set(e.value()),
                                    }
                                }
                            }
                        }
                        DialogFooter {
                            Button {
                                variant: "outline",
                                onclick: move |_| edit_open.set(false),
                                "Cancel"
                            }
                            Button {
                                variant: "default",
                                onclick: move |_| {
                                    let code = target_code.read().clone();
                                    let name = target_name.read().trim().to_string();
                                    let email = target_email.read().trim().to_string();
                                    let dept = target_dept.read().trim().to_string();
                                    let div = target_div.read().trim().to_string();

                                    let fields = serde_json::json!({
                                        "employee_name": name,
                                        "email": email,
                                        "department": dept,
                                        "division": div,
                                    });

                                    spawn(async move {
                                        match crate::sync::update_employee_immediate(&code, &fields).await {
                                            Ok(_) => {
                                                toast.success("Employee Updated", format!("Updated {code} in Google Sheets"));
                                                edit_open.set(false);
                                                if let Some(mut sync) = db_sync {
                                                    sync.with_mut(|r| r.0 += 1);
                                                }
                                            }
                                            Err(e) => {
                                                toast.error("Update Failed", format!("Failed to update: {e}"));
                                            }
                                        }
                                    });
                                },
                                "Save Changes"
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Delete Confirmation Dialog
            // ============================================================
            if delete_open() {
                Dialog {
                    open: Some(delete_open),
                    DialogContent {
                        class: "max-w-sm",
                        DialogHeader {
                            DialogTitle { "Delete Employee" }
                            DialogDescription { "Are you sure you want to delete {target_name} ({target_code})? Note: Deletion is blocked if tickets reference this employee." }
                        }
                        DialogFooter {
                            Button {
                                variant: "outline",
                                onclick: move |_| delete_open.set(false),
                                "Cancel"
                            }
                            Button {
                                variant: "destructive",
                                onclick: move |_| {
                                    let code = target_code.read().clone();
                                    spawn(async move {
                                        match crate::sync::delete_employee_immediate(&code).await {
                                            Ok(_) => {
                                                toast.success("Deleted", format!("Removed {code} from Google Sheets"));
                                                delete_open.set(false);
                                                if let Some(mut sync) = db_sync {
                                                    sync.with_mut(|r| r.0 += 1);
                                                }
                                            }
                                            Err(e) => {
                                                toast.error("Deletion Blocked", format!("{e}"));
                                                delete_open.set(false);
                                            }
                                        }
                                    });
                                },
                                "Delete"
                            }
                        }
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
