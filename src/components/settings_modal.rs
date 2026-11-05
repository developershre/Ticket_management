//! In-app Settings Dialog for configuring API Base URL and API Key.
//!
//! Stores the credentials locally in the system only:
//! - SQLite `sync_meta` table (under `system_api_base_url`, `system_api_key`, `system_employee_code`)
//! - `%APPDATA%\BPSL IT Support\.env` (for CLI / background scripts)
//!
//! Features:
//! - Real-time connection testing (`health.check` and `tickets.list` probe)
//! - Show/Hide password toggle for API key
//! - Immediate in-memory sync client reload without needing app restart
//! - Reset / Clear system settings option

use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::icons::{AlertCircleIcon, CheckIcon, EyeIcon, KeyIcon, SettingsIcon, XIcon};

#[derive(Clone, PartialEq, Debug)]
enum TestStatus {
    Idle,
    Testing,
    Success(String),
    Error(String),
}

#[derive(Props, Clone, PartialEq)]
pub struct SettingsModalProps {
    pub on_close: EventHandler<()>,
}

#[component]
pub fn SettingsModal(props: SettingsModalProps) -> Element {
    let toast = use_context::<ToastState>();

    // Initial load from system SQLite config, falling back to active loaded config
    let (initial_url, initial_key, initial_emp) = {
        let (sys_url, sys_key, sys_emp) = crate::db::get_system_api_config();
        let loaded = crate::api::ApiConfig::load().ok();
        (
            sys_url.or_else(|| loaded.as_ref().map(|c| c.base_url.clone())).unwrap_or_default(),
            sys_key.or_else(|| loaded.as_ref().map(|c| c.api_key.clone())).unwrap_or_default(),
            sys_emp.or_else(|| loaded.as_ref().and_then(|c| c.employee_code.clone())).unwrap_or_default(),
        )
    };

    let mut base_url = use_signal(|| initial_url);
    let mut api_key = use_signal(|| initial_key);
    let mut employee_code = use_signal(|| initial_emp);
    let mut show_key = use_signal(|| false);
    let mut test_status = use_signal(|| TestStatus::Idle);
    let mut is_saving = use_signal(|| false);

    let on_close = props.on_close;

    let handle_close = move |_| {
        on_close.call(());
    };

    let handle_test = move |_| {
        let url = base_url.read().trim().to_string();
        let key = api_key.read().trim().to_string();
        test_status.set(TestStatus::Testing);

        spawn(async move {
            match crate::sync::test_connection(&url, &key).await {
                Ok(msg) => test_status.set(TestStatus::Success(msg)),
                Err(err) => test_status.set(TestStatus::Error(err)),
            }
        });
    };

    let handle_save = move |_| {
        let url = base_url.read().trim().to_string();
        let key = api_key.read().trim().to_string();
        let emp = employee_code.read().trim().to_string();

        if url.is_empty() {
            toast.error("Validation Error", "Google Apps Script Web App URL cannot be empty.");
            return;
        }

        is_saving.set(true);

        if let Err(e) = crate::db::save_system_api_config(&url, &key, &emp) {
            is_saving.set(false);
            toast.error("Failed to Save Settings", format!("Database error: {e}"));
            return;
        }

        // Trigger immediate reload of the API client and background sync
        match crate::sync::reload_client() {
            Ok(_) => {
                is_saving.set(false);
                toast.success("Settings Saved", "API credentials stored in system. Sync refreshed!");
                on_close.call(());
            }
            Err(e) => {
                is_saving.set(false);
                toast.warning("Settings Saved", format!("Credentials stored, but client reload failed: {e}"));
                on_close.call(());
            }
        }
    };

    let handle_clear = move |_| {
        let _ = crate::db::clear_system_api_config();
        base_url.set(String::new());
        api_key.set(String::new());
        employee_code.set(String::new());
        test_status.set(TestStatus::Idle);
        toast.info("Cleared", "System API credentials have been reset.");
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-in fade-in-0 duration-150",
            onclick: handle_close,

            div {
                class: "relative w-full max-w-2xl rounded-2xl border border-border/80 bg-card p-6 text-card-foreground shadow-2xl animate-in zoom-in-95 duration-150 flex flex-col gap-5 max-h-[92vh] overflow-y-auto",
                onclick: move |e: MouseEvent| e.stop_propagation(),

                // MODAL HEADER
                div { class: "flex items-start justify-between border-b border-border/60 pb-4",
                    div { class: "flex items-center gap-3",
                        div { class: "flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary shadow-xs",
                            SettingsIcon { class: "size-5" }
                        }
                        div { class: "flex flex-col",
                            h2 { class: "text-base font-bold text-foreground leading-none",
                                "API & System Settings"
                            }
                            span { class: "text-xs text-muted-foreground mt-1",
                                "Configure your Google Sheets API endpoint and access credentials"
                            }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "rounded-lg p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer",
                        onclick: handle_close,
                        XIcon { class: "size-5" }
                    }
                }

                // SECURITY & PERSISTENCE NOTICE
                div { class: "flex items-start gap-2.5 rounded-lg border border-primary/20 bg-primary/5 p-3 text-xs text-muted-foreground",
                    div { class: "shrink-0 text-primary mt-0.5",
                        AlertCircleIcon { class: "size-4" }
                    }
                    p { class: "leading-relaxed",
                        span { class: "font-semibold text-foreground", "Local System Storage: " }
                        "These credentials are saved directly to your local system's SQLite database and user AppData folder. They are never committed to git or exposed remotely."
                    }
                }

                // FORM FIELDS
                div { class: "flex flex-col gap-4",

                    // 1. TMS_API_BASE_URL
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                            span { "Google Apps Script Web App URL" }
                            span { class: "text-[10px] text-muted-foreground font-mono", "TMS_API_BASE_URL" }
                        }
                        Input {
                            r#type: "text",
                            placeholder: "https://script.google.com/macros/s/AKfycb.../exec",
                            value: "{base_url}",
                            oninput: move |e: FormEvent| base_url.set(e.value()),
                            class: "font-mono text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary",
                        }
                        span { class: "text-[11px] text-muted-foreground",
                            "Paste the Web App URL deployed from your Google Apps Script editor (must end in /exec)."
                        }
                    }

                    // 2. TMS_API_KEY
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                            span { "API Secret Key" }
                            span { class: "text-[10px] text-muted-foreground font-mono", "TMS_API_KEY" }
                        }
                        div { class: "relative flex items-center",
                            input {
                                r#type: if *show_key.read() { "text" } else { "password" },
                                placeholder: "Enter API authentication key",
                                value: "{api_key}",
                                oninput: move |e: FormEvent| api_key.set(e.value()),
                                class: "w-full font-mono text-xs h-9 rounded-md border border-input bg-background pl-3 pr-10 focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/20 transition-colors",
                            }
                            button {
                                r#type: "button",
                                title: if *show_key.read() { "Hide API Key" } else { "Show API Key" },
                                class: "absolute right-2 p-1 text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                onclick: move |_| {
                                    let cur = *show_key.read();
                                    show_key.set(!cur);
                                },
                                EyeIcon { class: "size-4" }
                            }
                        }
                        span { class: "text-[11px] text-muted-foreground",
                            "Must match the API_KEY script property defined in your Google Apps Script."
                        }
                    }

                    // 3. TMS_EMPLOYEE_CODE
                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                            span { "Default Employee Code (Optional)" }
                            span { class: "text-[10px] text-muted-foreground font-mono", "TMS_EMPLOYEE_CODE" }
                        }
                        div { class: "relative flex items-center",
                            Input {
                                r#type: "text",
                                placeholder: "e.g. EMP-1001",
                                value: "{employee_code}",
                                oninput: move |e: FormEvent| employee_code.set(e.value()),
                                class: "font-mono text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary",
                            }
                        }
                        span { class: "text-[11px] text-muted-foreground",
                            "Optional employee code used when creating tickets or reminders if not selected."
                        }
                    }
                }

                // TEST CONNECTION SECTION
                div { class: "flex flex-col gap-2 rounded-xl border border-border/70 bg-muted/30 p-3.5",
                    div { class: "flex items-center justify-between",
                        div { class: "flex items-center gap-2",
                            KeyIcon { class: "size-4 text-primary" }
                            span { class: "text-xs font-semibold text-foreground", "Live Connection Test" }
                        }
                        Button {
                            variant: "outline",
                            class: "h-7 text-xs px-3 font-medium cursor-pointer shadow-2xs hover:bg-background",
                            disabled: *test_status.read() == TestStatus::Testing,
                            onclick: handle_test,
                            if *test_status.read() == TestStatus::Testing {
                                span { class: "flex items-center gap-1.5",
                                    span { class: "size-3 border-2 border-primary border-t-transparent rounded-full animate-spin" }
                                    "Testing…"
                                }
                            } else {
                                "Test Connection"
                            }
                        }
                    }

                    // Test Results Banner
                    match test_status.read().clone() {
                        TestStatus::Idle => rsx! {
                            span { class: "text-[11px] text-muted-foreground",
                                "Click 'Test Connection' to verify server reachability and API key authentication before saving."
                            }
                        },
                        TestStatus::Testing => rsx! {
                            div { class: "flex items-center gap-2 text-xs text-primary animate-pulse py-1",
                                span { "Connecting to Google Apps Script Web App…" }
                            }
                        },
                        TestStatus::Success(msg) => rsx! {
                            div { class: "flex items-start gap-2 rounded-md bg-emerald-500/10 border border-emerald-500/30 p-2 text-xs text-emerald-600 dark:text-emerald-400 animate-in fade-in-0",
                                CheckIcon { class: "size-4 shrink-0 mt-0.5" }
                                span { class: "font-medium leading-tight", "{msg}" }
                            }
                        },
                        TestStatus::Error(err) => rsx! {
                            div { class: "flex items-start gap-2 rounded-md bg-destructive/10 border border-destructive/30 p-2 text-xs text-destructive animate-in fade-in-0",
                                AlertCircleIcon { class: "size-4 shrink-0 mt-0.5" }
                                span { class: "font-medium leading-tight break-all", "{err}" }
                            }
                        },
                    }
                }

                // FOOTER ACTIONS
                div { class: "flex items-center justify-between border-t border-border/60 pt-4 mt-1",
                    button {
                        r#type: "button",
                        class: "text-xs text-muted-foreground hover:text-destructive transition-colors cursor-pointer underline-offset-4 hover:underline",
                        onclick: handle_clear,
                        "Reset to Default"
                    }

                    div { class: "flex items-center gap-2",
                        Button {
                            variant: "ghost",
                            class: "h-9 text-xs px-3 cursor-pointer",
                            onclick: handle_close,
                            "Cancel"
                        }
                        Button {
                            variant: "default",
                            class: "h-9 text-xs px-4 font-semibold cursor-pointer shadow-xs gap-1.5",
                            disabled: *is_saving.read(),
                            onclick: handle_save,
                            if *is_saving.read() {
                                span { class: "size-3 border-2 border-white border-t-transparent rounded-full animate-spin" }
                            } else {
                                CheckIcon { class: "size-3.5" }
                            }
                            "Save & Connect"
                        }
                    }
                }
            }
        }
    }
}
