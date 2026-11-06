//! Advanced Settings Panel for BPSL IT Support.
//!
//! Multi-tab settings dialog with:
//! - **API & Connection**: Base URL, API Key, connection testing
//! - **Appearance**: Theme mode (Light/Dark/System), UI density, sidebar behavior
//! - **Sync & Data**: Sync interval tuning, data management, export/import, cache purge
//! - **About**: App version, database location, system diagnostics
//!
//! All preferences are persisted in SQLite `sync_meta` and survive app restarts.

use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::icons::{AlertCircleIcon, CheckIcon, EyeIcon, KeyIcon, SettingsIcon, XIcon};

// ============================================================
// Settings Tab Enum
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SettingsTab {
    Api,
    Appearance,
    SyncData,
    About,
}

impl SettingsTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Api => "API & Connection",
            Self::Appearance => "Appearance",
            Self::SyncData => "Sync & Data",
            Self::About => "About",
        }
    }

    fn icon_path(&self) -> &'static str {
        match self {
            Self::Api => "M15 7h3a5 5 0 0 1 0 10h-3m-6 0H6a5 5 0 0 1 0-10h3 M8 12h8",
            Self::Appearance => "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707 M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z",
            Self::SyncData => "M21 2v6h-6 M3 12a9 9 0 0 1 15-6.7L21 8 M3 22v-6h6 M21 12a9 9 0 0 1-15 6.7L3 16",
            Self::About => "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z M12 16v-4 M12 8h.01",
        }
    }
}

const TABS: [SettingsTab; 4] = [
    SettingsTab::Api,
    SettingsTab::Appearance,
    SettingsTab::SyncData,
    SettingsTab::About,
];

// ============================================================
// Connection Test Status
// ============================================================

#[derive(Clone, PartialEq, Debug)]
enum TestStatus {
    Idle,
    Testing,
    Success(String),
    Error(String),
}

// ============================================================
// Props
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SettingsModalProps {
    pub on_close: EventHandler<()>,
}

// ============================================================
// Main Component
// ============================================================

#[component]
pub fn SettingsModal(props: SettingsModalProps) -> Element {
    let toast = use_context::<ToastState>();
    let on_close = props.on_close;

    // Active tab
    let mut active_tab = use_signal(|| SettingsTab::Api);

    // ---- API Tab State ----
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

    // ---- Appearance Tab State ----
    // Sidebar position preference (stored in sync_meta)
    let initial_sidebar_pos = crate::db::sync_meta_get("ui_sidebar_position").unwrap_or_else(|| "left".to_string());
    let mut sidebar_position = use_signal(|| initial_sidebar_pos);

    // Card density preference
    let initial_density = crate::db::sync_meta_get("ui_card_density").unwrap_or_else(|| "default".to_string());
    let mut card_density = use_signal(|| initial_density);

    // Animations enabled
    let initial_animations = crate::db::sync_meta_get("ui_animations_enabled").unwrap_or_else(|| "true".to_string());
    let mut animations_enabled = use_signal(|| initial_animations == "true");

    // Date format preference
    let initial_date_fmt = crate::db::sync_meta_get("ui_date_format").unwrap_or_else(|| "12hr".to_string());
    let mut date_format = use_signal(|| initial_date_fmt);

    // ---- Sync Tab State ----
    let initial_sync_interval = crate::db::sync_meta_get("sync_cycle_secs").unwrap_or_else(|| "25".to_string());
    let mut sync_interval_str = use_signal(|| initial_sync_interval);

    let initial_auto_sync = crate::db::sync_meta_get("sync_auto_enabled").unwrap_or_else(|| "true".to_string());
    let mut auto_sync_enabled = use_signal(|| initial_auto_sync == "true");

    let initial_notify_sync = crate::db::sync_meta_get("sync_notify_on_pull").unwrap_or_else(|| "false".to_string());
    let mut notify_on_pull = use_signal(|| initial_notify_sync == "true");

    // ---- Handlers ----

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

    let handle_save_api = move |_| {
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

        match crate::sync::reload_client() {
            Ok(_) => {
                is_saving.set(false);
                toast.success("Settings Saved", "API credentials stored in system. Sync refreshed!");
            }
            Err(e) => {
                is_saving.set(false);
                toast.warning("Settings Saved", format!("Credentials stored, but client reload failed: {e}"));
            }
        }
    };

    let handle_clear_api = move |_| {
        let _ = crate::db::clear_system_api_config();
        base_url.set(String::new());
        api_key.set(String::new());
        employee_code.set(String::new());
        test_status.set(TestStatus::Idle);
        toast.info("Cleared", "System API credentials have been reset.");
    };

    let save_appearance = move |_| {
        let _ = crate::db::sync_meta_set("ui_sidebar_position", &sidebar_position.read());
        let _ = crate::db::sync_meta_set("ui_card_density", &card_density.read());
        let _ = crate::db::sync_meta_set("ui_animations_enabled", if *animations_enabled.read() { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_date_format", &date_format.read());
        toast.success("Appearance Saved", "UI preferences updated.");
    };

    let save_sync_settings = move |_| {
        let interval = sync_interval_str.read().trim().to_string();
        let interval_val: u64 = interval.parse().unwrap_or(25);
        let clamped = interval_val.clamp(5, 300);
        sync_interval_str.set(clamped.to_string());
        let _ = crate::db::sync_meta_set("sync_cycle_secs", &clamped.to_string());
        let _ = crate::db::sync_meta_set("sync_auto_enabled", if *auto_sync_enabled.read() { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("sync_notify_on_pull", if *notify_on_pull.read() { "true" } else { "false" });
        toast.success("Sync Settings Saved", format!("Sync interval set to {clamped}s."));
    };

    let handle_reset_sync_queue = move |_| {
        let _ = crate::db::reset_failed_sync_ops();
        toast.info("Sync Queue Reset", "All failed operations have been retried.");
    };

    let handle_force_sync = move |_| {
        crate::sync::kick();
        toast.info("Sync Triggered", "A background sync cycle has been initiated.");
    };

    // ---- Diagnostic Data ----
    let db_path_display = crate::db::db_path().display().to_string();
    let pending_ops = crate::db::pending_op_count();
    let ticket_count = crate::db::get_all_tickets().len();
    let reminder_count = crate::db::get_all_reminders().len();
    let employee_count = crate::db::employee_count();
    let note_count = crate::db::get_all_notes().len();
    let conflict_count = crate::db::unresolved_conflict_count();
    let latest_error = crate::db::latest_active_error().unwrap_or_else(|| "None".to_string());

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 animate-dialog-overlay",
            onclick: handle_close,

            div {
                class: "relative w-full max-w-4xl rounded-2xl border border-border/80 bg-card text-card-foreground shadow-2xl animate-dialog-content flex flex-col max-h-[92vh] overflow-hidden",
                onclick: move |e: MouseEvent| e.stop_propagation(),

                // HEADER
                div { class: "flex items-center justify-between border-b border-border/60 px-6 py-4 shrink-0",
                    div { class: "flex items-center gap-3",
                        div { class: "flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary shadow-xs",
                            SettingsIcon { class: "size-5" }
                        }
                        div { class: "flex flex-col",
                            h2 { class: "text-lg font-bold text-foreground leading-none", "Settings" }
                            span { class: "text-xs text-muted-foreground mt-0.5", "Configure your app, API, appearance, and sync preferences" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "rounded-lg p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground active:scale-95 transition-all duration-150 cursor-pointer",
                        onclick: handle_close,
                        XIcon { class: "size-5" }
                    }
                }

                // BODY: TAB NAV + CONTENT
                div { class: "flex flex-1 overflow-hidden min-h-0",

                    // Left Tab Navigation Rail
                    nav { class: "w-48 shrink-0 border-r border-border/50 bg-muted/20 p-2 flex flex-col gap-0.5 overflow-y-auto",
                        for tab in TABS.iter() {
                            {
                                let t = *tab;
                                let is_active = *active_tab.read() == t;
                                let cls = if is_active {
                                    "bg-primary/10 text-primary font-semibold border-primary/30 shadow-2xs"
                                } else {
                                    "text-muted-foreground hover:bg-muted hover:text-foreground border-transparent"
                                };
                                rsx! {
                                    button {
                                        key: "{t.label()}",
                                        r#type: "button",
                                        class: "flex items-center gap-2.5 w-full text-left rounded-lg px-3 py-2.5 text-xs border transition-all duration-150 cursor-pointer active:scale-[0.98] {cls}",
                                        onclick: move |_| active_tab.set(t),
                                        svg {
                                            class: "size-4 shrink-0",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            path { d: "{t.icon_path()}" }
                                        }
                                        span { "{t.label()}" }
                                    }
                                }
                            }
                        }

                        // Version badge at bottom of nav
                        div { class: "mt-auto pt-4 px-2",
                            span { class: "text-[10px] text-muted-foreground/60 font-mono", "v0.1.0" }
                        }
                    }

                    // Right Content Area
                    div { class: "flex-1 overflow-y-auto p-6",

                        // ========================================
                        // TAB 1: API & Connection
                        // ========================================
                        if *active_tab.read() == SettingsTab::Api {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Security notice
                                div { class: "flex items-start gap-2.5 rounded-lg border border-primary/20 bg-primary/5 p-3 text-xs text-muted-foreground",
                                    div { class: "shrink-0 text-primary mt-0.5",
                                        AlertCircleIcon { class: "size-4" }
                                    }
                                    p { class: "leading-relaxed",
                                        span { class: "font-semibold text-foreground", "Local System Storage: " }
                                        "Credentials are saved to your local SQLite database and AppData. Never committed to git or exposed remotely."
                                    }
                                }

                                // Base URL
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

                                // API Key
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
                                        "Must match the API_KEY script property in your Google Apps Script."
                                    }
                                }

                                // Employee Code
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Default Employee Code (Optional)" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "TMS_EMPLOYEE_CODE" }
                                    }
                                    Input {
                                        r#type: "text",
                                        placeholder: "e.g. EMP-1001",
                                        value: "{employee_code}",
                                        oninput: move |e: FormEvent| employee_code.set(e.value()),
                                        class: "font-mono text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary",
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Optional employee code used when creating tickets or reminders if not selected."
                                    }
                                }

                                // Test Connection
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

                                    match test_status.read().clone() {
                                        TestStatus::Idle => rsx! {
                                            span { class: "text-[11px] text-muted-foreground",
                                                "Click 'Test Connection' to verify server reachability and API key authentication."
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

                                // Footer
                                div { class: "flex items-center justify-between border-t border-border/60 pt-4 mt-1",
                                    button {
                                        r#type: "button",
                                        class: "text-xs text-muted-foreground hover:text-destructive transition-colors cursor-pointer underline-offset-4 hover:underline",
                                        onclick: handle_clear_api,
                                        "Reset to Default"
                                    }
                                    div { class: "flex items-center gap-2",
                                        Button {
                                            variant: "default",
                                            class: "h-9 text-xs px-4 font-semibold cursor-pointer shadow-xs gap-1.5",
                                            disabled: *is_saving.read(),
                                            onclick: handle_save_api,
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

                        // ========================================
                        // TAB 2: Appearance
                        // ========================================
                        if *active_tab.read() == SettingsTab::Appearance {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Theme Mode Section
                                div { class: "flex flex-col gap-3",
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-sm font-semibold text-foreground", "Theme Mode" }
                                            span { class: "text-[11px] text-muted-foreground", "Choose between light, dark, or auto-detect from your system settings." }
                                        }
                                    }
                                    crate::components::ui::theme::ThemeSegmented {}
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Date & Time Format
                                SettingRow {
                                    title: "Time Display Format",
                                    description: "How timestamps appear across tickets, reminders, and cards.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("12hr", "12-Hour (AM/PM)"), ("24hr", "24-Hour")] {
                                            {
                                                let is_active = *date_format.read() == val;
                                                rsx! {
                                                    button {
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: move |_| date_format.set(val.to_string()),
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Card Density
                                SettingRow {
                                    title: "Card Density",
                                    description: "Control how compact or spacious the ticket and reminder cards appear.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("compact", "Compact"), ("default", "Default"), ("comfortable", "Comfortable")] {
                                            {
                                                let is_active = *card_density.read() == val;
                                                rsx! {
                                                    button {
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: move |_| card_density.set(val.to_string()),
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Animations Toggle
                                SettingToggle {
                                    title: "Animations & Transitions",
                                    description: "Enable smooth micro-interactions, card hover effects, and dialog transitions.",
                                    checked: *animations_enabled.read(),
                                    on_toggle: move |_| {
                                        let next = !*animations_enabled.read();
                                        animations_enabled.set(next);
                                    },
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Sidebar Position
                                SettingRow {
                                    title: "Sidebar Position",
                                    description: "Where the navigation sidebar appears in the app window.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("left", "Left"), ("right", "Right")] {
                                            {
                                                let is_active = *sidebar_position.read() == val;
                                                rsx! {
                                                    button {
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: move |_| sidebar_position.set(val.to_string()),
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Save
                                div { class: "flex justify-end pt-3 border-t border-border/50",
                                    Button {
                                        variant: "default",
                                        class: "h-9 text-xs px-4 font-semibold cursor-pointer shadow-xs gap-1.5",
                                        onclick: save_appearance,
                                        CheckIcon { class: "size-3.5" }
                                        "Save Appearance"
                                    }
                                }
                            }
                        }

                        // ========================================
                        // TAB 3: Sync & Data
                        // ========================================
                        if *active_tab.read() == SettingsTab::SyncData {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Auto Sync Toggle
                                SettingToggle {
                                    title: "Automatic Background Sync",
                                    description: "Continuously sync tickets and reminders with Google Sheets in the background.",
                                    checked: *auto_sync_enabled.read(),
                                    on_toggle: move |_| {
                                        let next = !*auto_sync_enabled.read();
                                        auto_sync_enabled.set(next);
                                    },
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Sync Interval
                                SettingRow {
                                    title: "Sync Interval (seconds)",
                                    description: "How often the app polls Google Sheets for changes. Range: 5–300 seconds.",
                                    div { class: "flex items-center gap-2",
                                        input {
                                            r#type: "number",
                                            min: "5",
                                            max: "300",
                                            value: "{sync_interval_str}",
                                            oninput: move |e: FormEvent| sync_interval_str.set(e.value()),
                                            class: "w-20 h-8 rounded-md border border-input bg-background px-2 text-xs font-mono text-center focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/20",
                                        }
                                        span { class: "text-xs text-muted-foreground", "seconds" }
                                    }
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Notify on Pull
                                SettingToggle {
                                    title: "Notify on Remote Changes",
                                    description: "Show a toast notification when new data arrives from Google Sheets.",
                                    checked: *notify_on_pull.read(),
                                    on_toggle: move |_| {
                                        let next = !*notify_on_pull.read();
                                        notify_on_pull.set(next);
                                    },
                                }

                                // Divider
                                div { class: "border-t border-border/50" }

                                // Sync Queue Status
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Sync Queue Status" }
                                    div { class: "grid grid-cols-2 gap-3",
                                        StatCard { label: "Pending Operations", value: "{pending_ops}", accent: if pending_ops > 0 { "text-amber-600" } else { "text-emerald-600" } }
                                        StatCard { label: "Unresolved Conflicts", value: "{conflict_count}", accent: if conflict_count > 0 { "text-destructive" } else { "text-emerald-600" } }
                                    }
                                    if !latest_error.is_empty() && latest_error != "None" {
                                        div { class: "rounded-md bg-destructive/5 border border-destructive/20 p-2.5",
                                            span { class: "text-[10px] font-bold text-destructive uppercase tracking-wider", "Latest Sync Error" }
                                            p { class: "text-xs text-destructive/90 mt-1 font-mono break-all leading-relaxed", "{latest_error}" }
                                        }
                                    }
                                }

                                // Actions row
                                div { class: "flex items-center gap-2 flex-wrap",
                                    Button {
                                        variant: "outline",
                                        class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5",
                                        onclick: handle_force_sync,
                                        svg {
                                            class: "size-3.5",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            path { d: "M21 2v6h-6" }
                                            path { d: "M3 12a9 9 0 0 1 15-6.7L21 8" }
                                        }
                                        "Force Sync Now"
                                    }
                                    Button {
                                        variant: "outline",
                                        class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5",
                                        onclick: handle_reset_sync_queue,
                                        AlertCircleIcon { class: "size-3.5" }
                                        "Retry Failed Ops"
                                    }
                                }

                                // Save
                                div { class: "flex justify-end pt-3 border-t border-border/50",
                                    Button {
                                        variant: "default",
                                        class: "h-9 text-xs px-4 font-semibold cursor-pointer shadow-xs gap-1.5",
                                        onclick: save_sync_settings,
                                        CheckIcon { class: "size-3.5" }
                                        "Save Sync Settings"
                                    }
                                }
                            }
                        }

                        // ========================================
                        // TAB 4: About
                        // ========================================
                        if *active_tab.read() == SettingsTab::About {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // App identity
                                div { class: "flex items-center gap-4 pb-4 border-b border-border/50",
                                    div { class: "flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary shadow-xs",
                                        crate::icons::AppLogo { class: "size-8" }
                                    }
                                    div { class: "flex flex-col gap-1",
                                        h3 { class: "text-lg font-bold text-foreground", "BPSL IT Support" }
                                        span { class: "text-xs text-muted-foreground", "Ticket & Reminder Management System" }
                                        div { class: "flex items-center gap-2 mt-1",
                                            span { class: "text-[10px] font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/60", "v0.1.0" }
                                            span { class: "text-[10px] font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/60", "Dioxus 0.7.1" }
                                            span { class: "text-[10px] font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/60", "Rust" }
                                        }
                                    }
                                }

                                // Database Statistics
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Database Statistics" }
                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3",
                                        StatCard { label: "Tickets", value: "{ticket_count}", accent: "text-blue-600" }
                                        StatCard { label: "Reminders", value: "{reminder_count}", accent: "text-amber-600" }
                                        StatCard { label: "Employees", value: "{employee_count}", accent: "text-emerald-600" }
                                        StatCard { label: "Notes", value: "{note_count}", accent: "text-purple-600" }
                                    }
                                }

                                // System Diagnostics
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "System Diagnostics" }

                                    DiagRow {
                                        label: "Database Location",
                                        value: db_path_display.clone(),
                                        mono: true,
                                    }
                                    DiagRow {
                                        label: "Sync Cycle Interval",
                                        value: format!("{} seconds", crate::sync::CYCLE_SECS),
                                        mono: false,
                                    }
                                    DiagRow {
                                        label: "Pending Sync Operations",
                                        value: pending_ops.to_string(),
                                        mono: false,
                                    }
                                    DiagRow {
                                        label: "Unresolved Conflicts",
                                        value: conflict_count.to_string(),
                                        mono: false,
                                    }
                                    DiagRow {
                                        label: "Operating System",
                                        value: std::env::consts::OS.to_string(),
                                        mono: false,
                                    }
                                    DiagRow {
                                        label: "Architecture",
                                        value: std::env::consts::ARCH.to_string(),
                                        mono: false,
                                    }
                                }

                                // Credits
                                div { class: "text-center py-3",
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Built with ❤ by BPSL IT Team • Powered by Dioxus & Rust"
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


// ============================================================
// Reusable Setting Row Components
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct SettingRowProps {
    title: &'static str,
    description: &'static str,
    children: Element,
}

#[component]
fn SettingRow(props: SettingRowProps) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-4",
            div { class: "flex flex-col gap-0.5 min-w-0",
                span { class: "text-sm font-semibold text-foreground", "{props.title}" }
                span { class: "text-[11px] text-muted-foreground leading-relaxed", "{props.description}" }
            }
            div { class: "shrink-0",
                {props.children}
            }
        }
    }
}

// ============================================================
// Toggle Switch Component
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct SettingToggleProps {
    title: &'static str,
    description: &'static str,
    checked: bool,
    on_toggle: EventHandler<MouseEvent>,
}

#[component]
fn SettingToggle(props: SettingToggleProps) -> Element {
    let bg = if props.checked { "bg-primary" } else { "bg-muted-foreground/30" };
    let knob_pos = if props.checked { "translate-x-4" } else { "translate-x-0.5" };

    rsx! {
        div { class: "flex items-center justify-between gap-4",
            div { class: "flex flex-col gap-0.5 min-w-0",
                span { class: "text-sm font-semibold text-foreground", "{props.title}" }
                span { class: "text-[11px] text-muted-foreground leading-relaxed", "{props.description}" }
            }
            button {
                r#type: "button",
                class: "relative inline-flex h-6 w-10 shrink-0 cursor-pointer rounded-full border border-transparent transition-colors duration-200 ease-in-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary {bg}",
                onclick: move |e| props.on_toggle.call(e),
                span {
                    class: "pointer-events-none inline-block size-5 rounded-full bg-white shadow-lg ring-0 transition-transform duration-200 ease-in-out {knob_pos}",
                }
            }
        }
    }
}


// ============================================================
// Stat Card
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct StatCardProps {
    label: &'static str,
    value: String,
    #[props(default = "text-foreground".to_string())]
    accent: String,
}

#[component]
fn StatCard(props: StatCardProps) -> Element {
    rsx! {
        div { class: "flex flex-col gap-1 rounded-lg border border-border/60 bg-card p-3 shadow-2xs",
            span { class: "text-[10px] font-bold text-muted-foreground uppercase tracking-wider", "{props.label}" }
            span { class: "text-xl font-extrabold {props.accent}", "{props.value}" }
        }
    }
}


// ============================================================
// Diagnostic Row
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct DiagRowProps {
    label: &'static str,
    value: String,
    #[props(default = false)]
    mono: bool,
}

#[component]
fn DiagRow(props: DiagRowProps) -> Element {
    let val_cls = if props.mono {
        "text-xs font-mono text-foreground truncate max-w-[360px]"
    } else {
        "text-xs text-foreground"
    };
    rsx! {
        div { class: "flex items-center justify-between py-1.5 border-b border-border/30 last:border-b-0",
            span { class: "text-xs text-muted-foreground font-medium", "{props.label}" }
            span { class: "{val_cls}", title: "{props.value}", "{props.value}" }
        }
    }
}
