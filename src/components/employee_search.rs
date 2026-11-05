use dioxus::prelude::*;

use crate::db::DbEmployee;
use crate::icons::{CheckIcon, SearchIcon};

#[derive(Props, Clone, PartialEq)]
pub struct EmployeeSearchInputProps {
    /// Signal holding the selected employee code
    pub employee_code: Signal<String>,
    /// Signal holding the employee name to auto-fill
    pub employee_name: Signal<String>,
    /// Signal holding the email address to auto-fill
    pub email_address: Signal<String>,
    /// Optional placeholder text
    #[props(default = "Search by Employee Code or Name (e.g. 11061 or Chaitram)...".to_string())]
    pub placeholder: String,
    /// Additional CSS classes for outer container
    #[props(default)]
    pub class: String,
}

#[component]
pub fn EmployeeSearchInput(props: EmployeeSearchInputProps) -> Element {
    let mut query = use_signal(String::new);
    let mut is_open = use_signal(|| false);

    // Reactively refresh when database syncs (e.g. when Google Sheets employees are pulled)
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();
    if let Some(sync) = db_sync {
        let _ = sync.read();
    }
    let all = crate::db::get_all_employees();
    let total_count = all.len();

    let mut emp_code_signal = props.employee_code;
    let mut emp_name_signal = props.employee_name;
    let mut email_signal = props.email_address;
    let mut last_synced_code = use_signal(String::new);

    // Sync query with external code changes (e.g. form reset, external edit, or initial load)
    use_effect(move || {
        let code = emp_code_signal.read().clone();
        if code != *last_synced_code.read() {
            last_synced_code.set(code.clone());
            if !code.is_empty() {
                // Always query database directly to get the latest info
                if let Some(emp) = crate::db::find_employee_by_code(&code) {
                    query.set(format!("{code} - {}", emp.emp_name));
                    emp_name_signal.set(emp.emp_name);
                    email_signal.set(emp.emp_email);
                } else {
                    let name = emp_name_signal.read().clone();
                    if !name.is_empty() {
                        query.set(format!("{code} - {name}"));
                    } else {
                        query.set(code);
                    }
                }
            } else {
                query.set(String::new());
                emp_name_signal.set(String::new());
                email_signal.set(String::new());
            }
        }
    });

    let q = query.read().trim().to_lowercase();
    let selected_code = emp_code_signal.read().trim().to_string();
    let selected_name = emp_name_signal.read().trim().to_string();

    // Check if the current query text represents an already selected employee
    let is_current_selection = !selected_code.is_empty()
        && (q == selected_code.to_lowercase()
            || q == selected_name.to_lowercase()
            || q == format!("{selected_code} - {selected_name}").to_lowercase());

    // Filter employees by either Employee Code OR Name (or Department / Email)
    let filtered: Vec<DbEmployee> = if q.is_empty() || is_current_selection {
        all.iter().take(60).cloned().collect()
    } else {
        all.iter()
            .filter(|e| {
                e.emp_code.to_lowercase().contains(&q)
                    || e.emp_name.to_lowercase().contains(&q)
                    || e.department.to_lowercase().contains(&q)
                    || e.emp_email.to_lowercase().contains(&q)
            })
            .take(60)
            .cloned()
            .collect()
    };

    let first_match = filtered.first().cloned();

    rsx! {
        div {
            class: "relative w-full {props.class}",

            // Search input field
            div {
                class: "relative z-50 flex items-center",
                div {
                    class: "pointer-events-none absolute left-3 flex items-center text-muted-foreground",
                    SearchIcon { class: "size-4" }
                }
                input {
                    r#type: "text",
                    class: "h-10 w-full rounded-md border border-input bg-card pl-9 pr-16 text-sm text-foreground placeholder:text-muted-foreground focus:border-primary focus:ring-1 focus:ring-primary/20 focus:outline-none transition-colors",
                    placeholder: "{props.placeholder}",
                    value: "{query}",
                    onfocus: move |_| is_open.set(true),
                    onclick: move |_| is_open.set(true),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Escape {
                            is_open.set(false);
                        } else if e.key() == Key::Enter {
                            if let Some(emp_m) = first_match.as_ref() {
                                let code = emp_m.emp_code.clone();
                                last_synced_code.set(code.clone());
                                emp_code_signal.set(code.clone());
                                // Always query database directly to get latest info
                                if let Some(emp) = crate::db::find_employee_by_code(&code) {
                                    emp_name_signal.set(emp.emp_name.clone());
                                    email_signal.set(emp.emp_email.clone());
                                    query.set(format!("{} - {}", emp.emp_code, emp.emp_name));
                                } else {
                                    emp_name_signal.set(emp_m.emp_name.clone());
                                    email_signal.set(emp_m.emp_email.clone());
                                    query.set(format!("{} - {}", emp_m.emp_code, emp_m.emp_name));
                                }
                                is_open.set(false);
                            }
                        }
                    },
                    oninput: move |e: FormEvent| {
                        let val = e.value();
                        query.set(val.clone());
                        is_open.set(true);

                        let trimmed = val.trim();
                        if trimmed.is_empty() {
                            last_synced_code.set(String::new());
                            emp_code_signal.set(String::new());
                            emp_name_signal.set(String::new());
                            email_signal.set(String::new());
                        } else if let Some(emp) = crate::db::find_employee_by_code(trimmed) {
                            // If user typed an exact employee code, auto-fill details immediately from DB
                            last_synced_code.set(emp.emp_code.clone());
                            emp_code_signal.set(emp.emp_code.clone());
                            emp_name_signal.set(emp.emp_name);
                            email_signal.set(emp.emp_email);
                        } else if let Some(emp_match) = all.iter().find(|e| e.emp_name.eq_ignore_ascii_case(trimmed)) {
                            // If user typed an exact employee name, lookup by code in DB for latest info
                            let code = emp_match.emp_code.clone();
                            last_synced_code.set(code.clone());
                            emp_code_signal.set(code.clone());
                            if let Some(emp) = crate::db::find_employee_by_code(&code) {
                                emp_name_signal.set(emp.emp_name);
                                email_signal.set(emp.emp_email);
                            } else {
                                emp_name_signal.set(emp_match.emp_name.clone());
                                email_signal.set(emp_match.emp_email.clone());
                            }
                        }
                    },
                }

                // Controls on the right: Clear (x) button and Dropdown toggle chevron
                div {
                    class: "absolute right-2.5 flex items-center gap-1",
                    if !query.read().is_empty() {
                        button {
                            r#type: "button",
                            class: "flex size-5 items-center justify-center rounded-full text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer",
                            title: "Clear employee selection",
                            onclick: move |_| {
                                query.set(String::new());
                                last_synced_code.set(String::new());
                                emp_code_signal.set(String::new());
                                emp_name_signal.set(String::new());
                                email_signal.set(String::new());
                                is_open.set(true);
                            },
                            svg {
                                class: "size-3",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2.5",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                line { x1: "18", y1: "6", x2: "6", y2: "18" }
                                line { x1: "6", y1: "6", x2: "18", y2: "18" }
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "flex size-5 items-center justify-center text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                        title: "Toggle employee list",
                        onclick: move |_| {
                            let curr = *is_open.read();
                            is_open.set(!curr);
                        },
                        svg {
                            class: if *is_open.read() { "size-3.5 rotate-180 transition-transform duration-200" } else { "size-3.5 transition-transform duration-200" },
                            view_box: "0 0 24 24",
                            fill: "none",
                            stroke: "currentColor",
                            stroke_width: "2",
                            stroke_linecap: "round",
                            stroke_linejoin: "round",
                            polyline { points: "6 9 12 15 18 9" }
                        }
                    }
                }
            }

            // Filtered Directory Dropdown
            if *is_open.read() {
                // Invisible backdrop to close on outside click
                div {
                    class: "fixed inset-0 z-40",
                    onclick: move |_| is_open.set(false),
                }

                div {
                    class: "absolute left-0 top-full mt-1.5 w-full z-50 rounded-lg border border-border/80 bg-popover/98 backdrop-blur-md shadow-2xl overflow-hidden max-h-64 overflow-y-auto p-1 animate-in fade-in-0 zoom-in-95",
                    if filtered.is_empty() {
                        div {
                            class: "p-4 text-center text-xs text-muted-foreground",
                            "No employee found matching \""
                            span { class: "font-semibold text-foreground", "{q}" }
                            "\". Try another code or name."
                        }
                    } else {
                        div {
                            class: "px-2 py-1 text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex justify-between items-center border-b border-border/40 mb-1",
                            span { "Directory Matches (Filtered by code or name)" }
                            span { "{filtered.len()} of {total_count}" }
                        }
                        for emp in filtered {
                            {
                                let e_c = emp.clone();
                                let is_selected = *emp_code_signal.read() == emp.emp_code;
                                let item_cls = if is_selected {
                                    "bg-primary/10 text-primary font-medium border-l-2 border-primary"
                                } else {
                                    "text-popover-foreground hover:bg-accent/80 hover:text-accent-foreground"
                                };
                                rsx! {
                                    div {
                                        key: "{emp.emp_id}",
                                        class: "relative flex flex-col gap-0.5 rounded-md px-3 py-2 cursor-pointer transition-colors {item_cls}",
                                        onclick: move |_| {
                                            let code = e_c.emp_code.clone();
                                            last_synced_code.set(code.clone());
                                            emp_code_signal.set(code.clone());
                                            // Always query database directly to get latest info
                                            if let Some(emp) = crate::db::find_employee_by_code(&code) {
                                                emp_name_signal.set(emp.emp_name.clone());
                                                email_signal.set(emp.emp_email.clone());
                                                query.set(format!("{} - {}", emp.emp_code, emp.emp_name));
                                            } else {
                                                emp_name_signal.set(e_c.emp_name.clone());
                                                email_signal.set(e_c.emp_email.clone());
                                                query.set(format!("{} - {}", e_c.emp_code, e_c.emp_name));
                                            }
                                            is_open.set(false);
                                        },
                                        div {
                                            class: "flex items-center justify-between gap-2",
                                            div { class: "flex items-center gap-1.5 min-w-0",
                                                span { class: "font-semibold text-xs text-foreground truncate", "{emp.emp_name}" }
                                                if is_selected {
                                                    CheckIcon { class: "size-3 text-primary shrink-0" }
                                                }
                                            }
                                            span {
                                                class: "font-mono text-[11px] font-bold px-1.5 py-0.5 rounded bg-muted/70 text-foreground shrink-0 border border-border/40",
                                                "#{emp.emp_code}"
                                            }
                                        }
                                        div {
                                            class: "flex items-center justify-between gap-2 text-[11px] text-muted-foreground mt-0.5",
                                            span { class: "truncate text-xs", "{emp.department}" }
                                            if !emp.emp_email.is_empty() {
                                                span { class: "truncate font-mono text-[10px] text-primary/80", "{emp.emp_email}" }
                                            } else {
                                                span { class: "italic text-[10px] text-muted-foreground/60", "No email" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
