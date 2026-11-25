use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::card::{
    Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::label::Label;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    AlertCircleIcon, AlertTriangleIcon, BuildingIcon, CalendarIcon, CheckCircle2Icon, CheckIcon,
    ClockIcon, CopyIcon, DatabaseIcon, EyeIcon, EyeOffIcon, GlobeIcon, KeyIcon, LaptopIcon,
    LockIcon, LogOutIcon, MailIcon, PencilIcon, PhoneIcon, PlusIcon, RefreshIcon, SearchIcon,
    ShieldCheckIcon, TrashIcon, UploadCloudIcon, UserIcon, UsersIcon, XIcon,
};
use crate::Route;

/// Curated enterprise color themes for avatar badges
const AVATAR_PALETTE: &[(&str, &str, &str)] = &[
    ("bg-blue-600", "Blue", "#2563eb"),
    ("bg-indigo-600", "Indigo", "#4f46e5"),
    ("bg-violet-600", "Violet", "#7c3aed"),
    ("bg-emerald-600", "Emerald", "#059669"),
    ("bg-amber-600", "Amber", "#d97706"),
    ("bg-rose-600", "Rose", "#e11d48"),
];

#[component]
pub fn Profile() -> Element {
    let nav = use_navigator();
    let toast = try_use_context::<ToastState>();
    let auth_signal = try_use_context::<Signal<crate::auth::AuthState>>();

    let current_user_opt = auth_signal
        .as_ref()
        .and_then(|s| s.read().user.clone())
        .or_else(crate::auth::current_active_user);

    // ------------------------------------------------------------
    // All hooks MUST be called unconditionally before early returns
    // ------------------------------------------------------------
    let initial_user = current_user_opt.clone();

    // Active tab in enterprise layout: profile | security | permissions | team | domains
    let mut active_tab = use_signal(|| "profile".to_string());

    // Profile form state
    let mut edit_name = use_signal(|| {
        initial_user
            .as_ref()
            .map(|u| u.display_name.clone())
            .unwrap_or_default()
    });
    let mut edit_email = use_signal(|| {
        initial_user
            .as_ref()
            .map(|u| u.email.clone())
            .unwrap_or_default()
    });
    let mut edit_dept = use_signal(|| {
        initial_user
            .as_ref()
            .map(|u| u.department.clone())
            .unwrap_or_default()
    });
    let mut edit_phone = use_signal(|| {
        initial_user
            .as_ref()
            .map(|u| u.phone.clone())
            .unwrap_or_default()
    });
    let mut profile_msg = use_signal(|| Option::<(bool, String)>::None);

    // Password form state
    let mut current_pass = use_signal(String::new);
    let mut new_pass = use_signal(String::new);
    let mut confirm_new_pass = use_signal(String::new);
    let mut show_passwords = use_signal(|| false);
    let mut pass_msg = use_signal(|| Option::<(bool, String)>::None);

    // Super Admin: Create User & Search state
    let mut user_search = use_signal(String::new);
    let mut show_create_user = use_signal(|| false);
    let mut new_u_name = use_signal(String::new);
    let mut new_u_username = use_signal(String::new);
    let mut new_u_email = use_signal(String::new);
    let mut new_u_dept = use_signal(|| "IT Support".to_string());
    let mut new_u_phone = use_signal(String::new);
    let mut new_u_role = use_signal(|| "agent".to_string());
    let mut new_u_pass = use_signal(String::new);
    let mut create_u_msg = use_signal(|| Option::<(bool, String)>::None);

    // Super Admin: Email Domain management state
    let mut domain_revision = use_signal(|| 0);
    let mut new_domain_input = use_signal(String::new);
    let mut new_domain_desc = use_signal(String::new);
    let mut domain_msg = use_signal(|| Option::<(bool, String)>::None);
    let mut editing_domain_id = use_signal(|| Option::<i64>::None);
    let mut edit_domain_val = use_signal(String::new);
    let mut edit_domain_desc = use_signal(String::new);

    // Super Admin: Google Sheets & System Administration State
    let active_sheet_src = crate::db::get_active_sheet_source();
    let mut sheet_name_input = use_signal(|| active_sheet_src.name.clone());
    let mut sheet_url_input = use_signal(|| active_sheet_src.base_url.clone());
    let mut sheet_key_input = use_signal(|| active_sheet_src.api_key.clone());
    let sheet_proto_input = use_signal(|| active_sheet_src.protocol.clone());
    let mut show_sheet_key = use_signal(|| false);

    // Testing and saving states
    let mut sheet_test_status = use_signal(|| Option::<(bool, String, u128)>::None);
    let mut is_testing_sheet = use_signal(|| false);
    let mut sheet_ctrl_msg = use_signal(|| Option::<(bool, String)>::None);

    // Schema health states
    let mut schema_report = use_signal(|| Option::<crate::sheet_crud::SchemaVerificationReport>::None);
    let mut is_verifying_schema = use_signal(|| false);
    let mut is_initializing_schema = use_signal(|| false);
    let mut schema_ctrl_msg = use_signal(|| Option::<(bool, String)>::None);

    // Sync Engine & Maintenance states
    let init_sync_enabled = crate::db::sync_meta_get("sync_auto_enabled").unwrap_or_else(|| "true".to_string()) == "true";
    let mut auto_sync_enabled = use_signal(|| init_sync_enabled);
    let init_sync_cadence = crate::db::sync_meta_get("sync_cycle_secs").unwrap_or_else(|| "25".to_string());
    let mut sync_interval_secs = use_signal(|| init_sync_cadence);
    let mut is_pulling_now = use_signal(|| false);
    let mut is_vacuuming = use_signal(|| false);
    let mut maintenance_msg = use_signal(|| Option::<(bool, String)>::None);
    let mut maintenance_revision = use_signal(|| 0);

    let db_stats = use_memo(move || {
        let _ = maintenance_revision();
        let tickets_cnt = crate::db::get_all_tickets().len();
        let employees_cnt = crate::db::get_all_employees().len();
        let reminders_cnt = crate::db::get_all_reminders().len();
        let notes_cnt = crate::db::get_all_notes().len();
        let users_cnt = crate::auth::list_all_users().len();
        let queue_cnt = crate::db::pending_op_count();
        (tickets_cnt, employees_cnt, reminders_cnt, notes_cnt, users_cnt, queue_cnt)
    });

    // Team management list state
    let mut team_revision = use_signal(|| 0);
    let is_admin_flag = current_user_opt.as_ref().map(|u| u.is_admin()).unwrap_or(false);

    let team_users = use_memo(move || {
        let _ = team_revision();
        if is_admin_flag {
            crate::auth::list_all_users()
        } else {
            Vec::new()
        }
    });

    let email_domains = use_memo(move || {
        let _ = domain_revision();
        crate::auth::list_email_domains()
    });

    // Keep inputs synced if user changes
    let mut synced_user_id = use_signal(|| initial_user.as_ref().map(|u| u.id).unwrap_or(-1));
    if let Some(ref u) = current_user_opt {
        if *synced_user_id.peek() != u.id {
            synced_user_id.set(u.id);
            edit_name.set(u.display_name.clone());
            edit_email.set(u.email.clone());
            edit_dept.set(u.department.clone());
            edit_phone.set(u.phone.clone());
        }
    }

    // ------------------------------------------------------------
    // Unauthenticated fallback
    // ------------------------------------------------------------
    let Some(user) = current_user_opt else {
        return rsx! {
            div { class: "flex flex-col items-center justify-center p-8 min-h-[60vh] text-center max-w-md mx-auto my-auto gap-4",
                div { class: "flex size-16 items-center justify-center rounded-2xl bg-muted text-muted-foreground shadow-xs border border-border/60",
                    UserIcon { class: "size-8" }
                }
                div { class: "space-y-1.5",
                    h2 { class: "text-xl font-bold tracking-tight text-foreground", "Sign In Required" }
                    p { class: "text-xs text-muted-foreground leading-relaxed",
                        "You must be signed in to view and manage your account profile and permissions."
                    }
                }
                Link {
                    to: Route::Login {},
                    class: "inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-xs font-semibold text-primary-foreground shadow-xs hover:bg-primary/90 transition-colors cursor-pointer",
                    UserIcon { class: "size-4" }
                    "Sign In"
                }
            }
        };
    };

    let user_id = user.id;
    let is_super_admin = user.is_super_admin();
    let is_admin = user.is_admin();

    // ------------------------------------------------------------
    // Action Handlers
    // ------------------------------------------------------------
    let mut handle_profile_save = move |_| {
        let name_val = edit_name.read().trim().to_string();
        let email_val = edit_email.read().trim().to_string();
        let dept_val = edit_dept.read().trim().to_string();
        let phone_val = edit_phone.read().trim().to_string();

        match crate::auth::update_profile(user_id, &name_val, &email_val, &dept_val, &phone_val) {
            Ok(updated) => {
                if let Some(mut state) = auth_signal {
                    state.with_mut(|s| s.user = Some(updated.clone()));
                }
                profile_msg.set(Some((true, "Profile details saved successfully.".to_string())));
                if let Some(t) = toast {
                    t.success("Profile Updated", "Your account information has been saved.");
                }
            }
            Err(e) => {
                profile_msg.set(Some((false, e)));
            }
        }
    };

    let handle_color_change = move |color_cls: &'static str| {
        match crate::auth::update_avatar_color(user_id, color_cls) {
            Ok(updated) => {
                if let Some(mut state) = auth_signal {
                    state.with_mut(|s| s.user = Some(updated.clone()));
                }
                if let Some(t) = toast {
                    t.success("Theme Updated", "Avatar accent color saved.");
                }
            }
            Err(e) => {
                if let Some(t) = toast {
                    t.error("Error", e.to_string());
                }
            }
        }
    };

    let mut handle_password_change = move |_| {
        let old_p = current_pass.read().to_string();
        let new_p = new_pass.read().to_string();
        let conf_p = confirm_new_pass.read().to_string();

        if old_p.is_empty() || new_p.is_empty() {
            pass_msg.set(Some((false, "Please fill in all password fields.".to_string())));
            return;
        }
        if new_p.len() < 6 {
            pass_msg.set(Some((false, "New password must be at least 6 characters.".to_string())));
            return;
        }
        if new_p != conf_p {
            pass_msg.set(Some((false, "New passwords do not match.".to_string())));
            return;
        }

        match crate::auth::change_password(user_id, &old_p, &new_p) {
            Ok(_) => {
                current_pass.set(String::new());
                new_pass.set(String::new());
                confirm_new_pass.set(String::new());
                pass_msg.set(Some((true, "Password updated successfully.".to_string())));
                if let Some(t) = toast {
                    t.success("Security Updated", "Your account password has been updated.");
                }
            }
            Err(e) => {
                pass_msg.set(Some((false, e)));
            }
        }
    };

    let mut handle_create_user = move |_| {
        let req = crate::auth::RegisterRequest {
            username: new_u_username.read().clone(),
            email: new_u_email.read().clone(),
            display_name: new_u_name.read().clone(),
            password: new_u_pass.read().clone(),
            department: new_u_dept.read().clone(),
            phone: new_u_phone.read().clone(),
            requested_role: Some(new_u_role.read().clone()),
        };

        match crate::auth::create_user_by_admin(user_id, req) {
            Ok(created) => {
                new_u_name.set(String::new());
                new_u_username.set(String::new());
                new_u_email.set(String::new());
                new_u_pass.set(String::new());
                new_u_phone.set(String::new());
                team_revision.with_mut(|r| *r += 1);
                create_u_msg.set(Some((
                    true,
                    format!("User account '@{}' created successfully.", created.username),
                )));
                if let Some(t) = toast {
                    t.success(
                        "User Created",
                        format!("{} (@{}) added to team directory.", created.display_name, created.username),
                    );
                }
            }
            Err(e) => {
                create_u_msg.set(Some((false, e)));
            }
        }
    };

    let mut handle_add_domain = move |_| {
        let dom = new_domain_input.read().clone();
        let desc = new_domain_desc.read().clone();
        match crate::auth::add_email_domain(&dom, &desc) {
            Ok(added) => {
                new_domain_input.set(String::new());
                new_domain_desc.set(String::new());
                domain_revision.with_mut(|r| *r += 1);
                domain_msg.set(Some((
                    true,
                    format!("Domain '@{}' configured and active.", added.domain),
                )));
                if let Some(t) = toast {
                    t.success("Domain Allowed", format!("Emails ending in @{} are now permitted.", added.domain));
                }
            }
            Err(e) => {
                domain_msg.set(Some((false, e)));
            }
        }
    };

    // Google Sheets & System Administration Handlers (Super Admin)
    let mut handle_test_sheet = move |_| {
        let url = sheet_url_input.read().trim().to_string();
        let key = sheet_key_input.read().trim().to_string();
        if url.is_empty() {
            sheet_ctrl_msg.set(Some((false, "Please provide a Google Apps Script Web App URL first.".to_string())));
            return;
        }
        is_testing_sheet.set(true);
        sheet_test_status.set(None);
        sheet_ctrl_msg.set(None);

        spawn(async move {
            let start = std::time::Instant::now();
            match crate::sync::test_connection(&url, &key).await {
                Ok(msg) => {
                    let lat = start.elapsed().as_millis();
                    sheet_test_status.set(Some((true, msg, lat)));
                    is_testing_sheet.set(false);
                    if let Some(t) = toast {
                        t.success("Connected", format!("Google Sheet reached successfully in {}ms.", lat));
                    }
                }
                Err(err) => {
                    let lat = start.elapsed().as_millis();
                    sheet_test_status.set(Some((false, err.clone(), lat)));
                    is_testing_sheet.set(false);
                    if let Some(t) = toast {
                        t.error("Connection Failed", err);
                    }
                }
            }
        });
    };

    let mut handle_save_sheet = move |_| {
        let url = sheet_url_input.read().trim().to_string();
        let key = sheet_key_input.read().trim().to_string();
        let name_val = sheet_name_input.read().trim().to_string();
        let proto = sheet_proto_input.read().trim().to_string();

        let name = if name_val.is_empty() { "Primary Sheet".to_string() } else { name_val };
        let active_id = crate::db::get_active_sheet_source_id();
        let source_id = if active_id.is_empty() { "primary".to_string() } else { active_id };

        let src = crate::db::SheetSource {
            id: source_id.clone(),
            name,
            base_url: url.clone(),
            api_key: key.clone(),
            protocol: proto,
        };

        let mut all_sources = crate::db::get_all_sheet_sources();
        if let Some(pos) = all_sources.iter().position(|s| s.id == source_id) {
            all_sources[pos] = src.clone();
        } else {
            all_sources.push(src.clone());
        }
        let _ = crate::db::save_all_sheet_sources(&all_sources);
        let _ = crate::sync::switch_sheet_profile_and_sync(&source_id);
        crate::sync::kick();
        sheet_ctrl_msg.set(Some((true, "Google Sheets connection configuration saved and activated.".to_string())));
        if let Some(t) = toast {
            t.success("Profile Saved", "Active Google Sheet profile updated.");
        }
    };

    let mut handle_disconnect_sheet = move |_| {
        let _ = crate::db::clear_system_api_config();
        sheet_url_input.set(String::new());
        sheet_key_input.set(String::new());
        sheet_test_status.set(None);
        schema_report.set(None);
        sheet_ctrl_msg.set(Some((true, "Google Sheets connection cleared. Local offline mode active.".to_string())));
        if let Some(t) = toast {
            t.info("Disconnected", "Google Sheets credentials have been cleared.");
        }
    };

    let mut handle_verify_schema = move |_| {
        let url = sheet_url_input.read().trim().to_string();
        let key = sheet_key_input.read().trim().to_string();
        let proto = sheet_proto_input.read().trim().to_string();

        if url.is_empty() {
            schema_ctrl_msg.set(Some((false, "Please configure a valid Web App URL before verifying schema.".to_string())));
            return;
        }

        is_verifying_schema.set(true);
        schema_ctrl_msg.set(None);

        spawn(async move {
            let cfg = crate::api::ApiConfig {
                base_url: url,
                api_key: key,
                employee_code: None,
                protocol: crate::api::ApiProtocol::from_str_opt(&proto).unwrap_or(crate::api::ApiProtocol::BpslRest),
            };
            match crate::api::ApiClient::new(cfg) {
                Ok(client) => match client.crud_verify_schema().await {
                    Ok(report) => {
                        let is_valid = report.valid;
                        schema_report.set(Some(report));
                        is_verifying_schema.set(false);
                        if is_valid {
                            schema_ctrl_msg.set(Some((true, "All 4 spreadsheet tabs and header columns verified successfully.".to_string())));
                            if let Some(t) = toast {
                                t.success("Schema Valid", "Connected sheet structure is 100% compliant.");
                            }
                        } else {
                            schema_ctrl_msg.set(Some((false, "Some tabs or columns are missing. Click 'Auto-Initialize Schema' below to fix.".to_string())));
                            if let Some(t) = toast {
                                t.warning("Schema Incomplete", "Required tabs or headers are missing.");
                            }
                        }
                    }
                    Err(e) => {
                        is_verifying_schema.set(false);
                        schema_ctrl_msg.set(Some((false, format!("Failed to verify schema: {e}"))));
                    }
                },
                Err(e) => {
                    is_verifying_schema.set(false);
                    schema_ctrl_msg.set(Some((false, format!("API Client initialization error: {e}"))));
                }
            }
        });
    };

    let mut handle_init_schema = move |_| {
        let url = sheet_url_input.read().trim().to_string();
        let key = sheet_key_input.read().trim().to_string();
        let proto = sheet_proto_input.read().trim().to_string();

        if url.is_empty() {
            schema_ctrl_msg.set(Some((false, "Please configure a valid Web App URL first.".to_string())));
            return;
        }

        is_initializing_schema.set(true);
        schema_ctrl_msg.set(None);

        spawn(async move {
            let cfg = crate::api::ApiConfig {
                base_url: url.clone(),
                api_key: key.clone(),
                employee_code: None,
                protocol: crate::api::ApiProtocol::from_str_opt(&proto).unwrap_or(crate::api::ApiProtocol::BpslRest),
            };
            match crate::api::ApiClient::new(cfg.clone()) {
                Ok(client) => match client.crud_init_schema().await {
                    Ok(_) => {
                        if let Ok(report) = client.crud_verify_schema().await {
                            schema_report.set(Some(report));
                        }
                        is_initializing_schema.set(false);
                        schema_ctrl_msg.set(Some((true, "Google Sheet tabs (Tickets, Employees, Reminders, Notes) initialized successfully.".to_string())));
                        if let Some(t) = toast {
                            t.success("Schema Initialized", "Spreadsheet tabs and standard headers created.");
                        }
                    }
                    Err(e) => {
                        is_initializing_schema.set(false);
                        schema_ctrl_msg.set(Some((false, format!("Auto-initialization failed: {e}"))));
                    }
                },
                Err(e) => {
                    is_initializing_schema.set(false);
                    schema_ctrl_msg.set(Some((false, format!("Client initialization error: {e}"))));
                }
            }
        });
    };

    let mut handle_toggle_auto_sync = move |_| {
        let cur = *auto_sync_enabled.read();
        let new_val = !cur;
        auto_sync_enabled.set(new_val);
        let _ = crate::db::sync_meta_set("sync_auto_enabled", if new_val { "true" } else { "false" });
        if new_val {
            crate::sync::kick();
        }
        if let Some(t) = toast {
            t.info("Sync Setting", if new_val { "Background auto-sync enabled." } else { "Background auto-sync paused." });
        }
    };

    let mut handle_change_sync_cadence = move |val: String| {
        sync_interval_secs.set(val.clone());
        let _ = crate::db::sync_meta_set("sync_cycle_secs", &val);
        if let Some(t) = toast {
            t.info("Cadence Set", format!("Sync interval set to {} seconds.", val));
        }
    };

    let mut handle_force_pull_now = move |_| {
        is_pulling_now.set(true);
        maintenance_msg.set(None);
        spawn(async move {
            let t_res = crate::sync::pull_tickets_now().await;
            let e_res = crate::sync::pull_employees_now().await;
            is_pulling_now.set(false);
            maintenance_revision.with_mut(|r| *r += 1);
            match (t_res, e_res) {
                (Ok(t_cnt), Ok(e_cnt)) => {
                    maintenance_msg.set(Some((true, format!("Pulled {} tickets and {} employees from Google Sheets.", t_cnt, e_cnt))));
                    if let Some(t) = toast {
                        t.success("Sync Complete", format!("Updated {} tickets, {} staff.", t_cnt, e_cnt));
                    }
                }
                (Ok(t_cnt), Err(e_err)) => {
                    maintenance_msg.set(Some((false, format!("Tickets synced ({t_cnt}), but employees failed: {e_err}"))));
                }
                (Err(t_err), Ok(e_cnt)) => {
                    maintenance_msg.set(Some((false, format!("Employees synced ({e_cnt}), but tickets failed: {t_err}"))));
                }
                (Err(t_err), Err(e_err)) => {
                    maintenance_msg.set(Some((false, format!("Sync failed: {t_err}; {e_err}"))));
                }
            }
        });
    };

    let mut handle_clear_sync_queue = move |_| {
        let _ = crate::db::reset_failed_sync_ops();
        let _ = crate::db::purge_resolved_conflicts();
        maintenance_revision.with_mut(|r| *r += 1);
        maintenance_msg.set(Some((true, "Outbox sync queue reset and failed operations queued for clean retry.".to_string())));
        if let Some(t) = toast {
            t.success("Queue Reset", "Pending sync operations reset for retry.");
        }
    };

    let mut handle_vacuum_db = move |_| {
        is_vacuuming.set(true);
        let _ = crate::db::purge_resolved_conflicts();
        match crate::db::optimize_database() {
            Ok(()) => {
                is_vacuuming.set(false);
                maintenance_revision.with_mut(|r| *r += 1);
                maintenance_msg.set(Some((true, "Database vacuumed, SQLite WAL flushed, tables defragmented.".to_string())));
                if let Some(t) = toast {
                    t.success("Optimized", "Database tables defragmented and vacuumed.");
                }
            }
            Err(e) => {
                is_vacuuming.set(false);
                maintenance_msg.set(Some((false, format!("Optimization error: {e}"))));
                if let Some(t) = toast {
                    t.error("Error", e.to_string());
                }
            }
        }
    };

    let handle_logout = move |_| {
        let _ = crate::auth::logout();
        if let Some(mut state) = auth_signal {
            state.set(crate::auth::AuthState::default());
        }
        if let Some(t) = toast {
            t.info("Signed Out", "You have been logged out of this device.");
        }
        let _ = nav.replace(Route::Login {});
    };

    let handle_copy_username = {
        let uname = user.username.clone();
        move |_| {
            let js = format!("navigator.clipboard.writeText('{}');", uname);
            let _ = document::eval(&js);
            if let Some(t) = toast {
                t.info("Copied", format!("Username @{} copied to clipboard", uname));
            }
        }
    };

    // Domain data & policy hint
    let active_domains_list: Vec<String> = email_domains()
        .into_iter()
        .filter(|d| d.is_active)
        .map(|d| format!("@{}", d.domain))
        .collect();

    let active_domain_hint = if active_domains_list.is_empty() {
        "Open policy — all valid email domains permitted".to_string()
    } else {
        format!("Restricted to: {}", active_domains_list.join(", "))
    };

    // Password strength check
    let new_p_str = new_pass.read();
    let p_len = new_p_str.len();
    let p_upper = new_p_str.chars().any(|c| c.is_uppercase());
    let p_digit = new_p_str.chars().any(|c| c.is_ascii_digit());
    let p_match = !new_p_str.is_empty() && *new_p_str == *confirm_new_pass.read();
    let pass_score = if p_len < 6 {
        0
    } else {
        1 + usize::from(p_len >= 8) + usize::from(p_upper) + usize::from(p_digit)
    };
    drop(new_p_str);

    // Team directory metrics
    let all_users = team_users();
    let total_users_count = all_users.len();
    let active_users_count = all_users.iter().filter(|u| u.is_active).count();
    let admin_users_count = all_users.iter().filter(|u| u.is_admin()).count();
    let agent_users_count = all_users.iter().filter(|u| u.role == "agent").count();

    let search_q = user_search.read().trim().to_lowercase();
    let filtered_users: Vec<crate::auth::AuthUser> = all_users
        .into_iter()
        .filter(|u| {
            if search_q.is_empty() {
                return true;
            }
            u.display_name.to_lowercase().contains(&search_q)
                || u.username.to_lowercase().contains(&search_q)
                || u.email.to_lowercase().contains(&search_q)
                || u.department.to_lowercase().contains(&search_q)
        })
        .collect();

    let domains_all = email_domains();
    let total_domains_count = domains_all.len();
    let active_domains_count = domains_all.iter().filter(|d| d.is_active).count();

    let cur_tab = active_tab.read().clone();

    rsx! {
        div { class: "flex flex-1 flex-col h-full overflow-y-auto bg-muted/15 p-4 sm:p-6 lg:p-8 animate-in fade-in-50 duration-200",
            div { class: "max-w-7xl mx-auto w-full space-y-6 pb-12",

                // ============================================================
                // Top Executive Header Banner
                // ============================================================
                div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 pb-2 border-b border-border/50",
                    div { class: "space-y-1",
                        div { class: "flex items-center gap-2",
                            span { class: "inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-md text-[11px] font-semibold bg-primary/10 text-primary border border-primary/20",
                                ShieldCheckIcon { class: "size-3" }
                                "Identity & Access Center"
                            }
                            span { class: "text-xs text-muted-foreground", "•" }
                            span { class: "text-xs font-mono text-muted-foreground", "BPSL Enterprise Support" }
                        }
                        h1 { class: "text-2xl font-bold tracking-tight text-foreground", "Account & Security Profile" }
                    }
                    div { class: "flex items-center gap-2",
                        Button {
                            variant: "outline".to_string(),
                            size: "sm".to_string(),
                            class: "h-9 text-xs font-medium cursor-pointer bg-card hover:bg-muted/60 border-border/80 text-foreground",
                            onclick: move |_| {
                                team_revision.with_mut(|r| *r += 1);
                                domain_revision.with_mut(|r| *r += 1);
                                if let Some(t) = toast {
                                    t.info("Refreshed", "Account directory and policies synced.");
                                }
                            },
                            RefreshIcon { class: "size-3.5 mr-1.5 text-muted-foreground" }
                            "Refresh"
                        }
                        Button {
                            variant: "outline".to_string(),
                            size: "sm".to_string(),
                            class: "h-9 text-xs font-medium cursor-pointer bg-card hover:bg-destructive/10 text-destructive border-destructive/30 shadow-2xs",
                            onclick: move |_| handle_logout(()),
                            LogOutIcon { class: "size-3.5 mr-1.5" }
                            "Sign Out"
                        }
                    }
                }

                // ============================================================
                // Two-Column Executive Layout
                // Left: Identity Hero & Navigation Menu
                // Right: Tab Content Panels
                // ============================================================
                div { class: "grid grid-cols-1 lg:grid-cols-12 gap-6 items-start",

                    // --------------------------------------------------------
                    // LEFT COLUMN: Sticky User Identity & Vertical Navigation (Cols 4/12)
                    // --------------------------------------------------------
                    div { class: "lg:col-span-4 space-y-4",

                        // Identity Card
                        Card { class: "overflow-hidden border-border/80 shadow-sm bg-card",
                            // Subtle pattern header
                            div { class: "h-24 w-full bg-gradient-to-br from-primary/20 via-primary/10 to-transparent border-b border-border/50 relative p-3 flex justify-between items-start",
                                span { class: "px-2 py-0.5 rounded text-[10px] font-mono font-bold uppercase tracking-wider bg-background/80 backdrop-blur-xs text-foreground/70 border border-border/40",
                                    "BPSL ID #{user.id}"
                                }
                                span { class: "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/15 text-emerald-700 dark:text-emerald-400 border border-emerald-500/25",
                                    span { class: "size-1.5 rounded-full bg-emerald-500 animate-pulse" }
                                    "Session Active"
                                }
                            }

                            CardContent { class: "px-5 pb-5 pt-0 space-y-4",
                                // Avatar with ring
                                div { class: "flex items-end justify-between -mt-10",
                                    div { class: "relative shrink-0",
                                        div { class: "flex size-20 items-center justify-center rounded-2xl text-2xl font-bold text-white shadow-md ring-4 ring-card {user.avatar_color} transition-all",
                                            "{user.initials()}"
                                        }
                                        span {
                                            class: "absolute -bottom-0.5 -right-0.5 size-4 rounded-full bg-emerald-500 ring-2 ring-card flex items-center justify-center",
                                            title: "Authenticated",
                                            CheckIcon { class: "size-2.5 text-white stroke-[3]" }
                                        }
                                    }

                                    // Role badge
                                    span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold border {user.role_badge_color()} shadow-2xs",
                                        if is_super_admin {
                                            ShieldCheckIcon { class: "size-3.5 text-amber-500" }
                                        } else if is_admin {
                                            ShieldCheckIcon { class: "size-3.5 text-indigo-500" }
                                        } else {
                                            UserIcon { class: "size-3.5" }
                                        }
                                        "{user.role_label()}"
                                    }
                                }

                                // User Details
                                div { class: "space-y-1.5 pt-1",
                                    h2 { class: "text-lg font-bold tracking-tight text-foreground", "{user.display_name}" }
                                    div { class: "flex items-center gap-2 flex-wrap text-xs",
                                        button {
                                            r#type: "button",
                                            class: "group inline-flex items-center gap-1 font-mono px-2 py-0.5 rounded bg-muted hover:bg-muted/80 text-foreground/90 border border-border/60 text-[11px] cursor-pointer transition-colors",
                                            onclick: handle_copy_username,
                                            title: "Click to copy username",
                                            span { "@{user.username}" }
                                            CopyIcon { class: "size-3 opacity-60 group-hover:opacity-100" }
                                        }
                                        if !user.department.is_empty() {
                                            span { class: "inline-flex items-center gap-1 px-2 py-0.5 rounded bg-muted/50 text-muted-foreground text-[11px] border border-border/40",
                                                BuildingIcon { class: "size-3" }
                                                "{user.department}"
                                            }
                                        }
                                    }
                                }

                                // Quick Info Grid
                                div { class: "rounded-xl border border-border/60 bg-muted/20 p-3 space-y-2 text-xs",
                                    div { class: "flex items-center justify-between text-[11px]",
                                        span { class: "text-muted-foreground inline-flex items-center gap-1.5",
                                            MailIcon { class: "size-3" }
                                            "Email Address"
                                        }
                                        span { class: "font-medium text-foreground truncate max-w-[160px]", title: "{user.email}", "{user.email}" }
                                    }
                                    if !user.phone.is_empty() {
                                        div { class: "flex items-center justify-between text-[11px]",
                                            span { class: "text-muted-foreground inline-flex items-center gap-1.5",
                                                PhoneIcon { class: "size-3" }
                                                "Phone"
                                            }
                                            span { class: "font-medium text-foreground", "{user.phone}" }
                                        }
                                    }
                                    div { class: "flex items-center justify-between text-[11px]",
                                        span { class: "text-muted-foreground inline-flex items-center gap-1.5",
                                            CalendarIcon { class: "size-3" }
                                            "Member Since"
                                        }
                                        span { class: "font-medium text-foreground", "{user.created_at}" }
                                    }
                                    if let Some(last) = user.last_login_at.as_ref() {
                                        div { class: "flex items-center justify-between text-[11px]",
                                            span { class: "text-muted-foreground inline-flex items-center gap-1.5",
                                                ClockIcon { class: "size-3" }
                                                "Last Sign In"
                                            }
                                            span { class: "font-medium text-foreground", "{last}" }
                                        }
                                    }
                                }
                            }
                        }

                        // Navigation Card (Vertical Pill Menu)
                        Card { class: "border-border/80 shadow-xs bg-card overflow-hidden",
                            CardHeader { class: "px-4 py-3 border-b border-border/50 bg-muted/15",
                                p { class: "text-[11px] font-bold uppercase tracking-wider text-muted-foreground", "Navigation Menu" }
                            }
                            div { class: "p-2 space-y-1",
                                // 1. Profile Info
                                button {
                                    r#type: "button",
                                    class: format!(
                                        "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer text-left {}",
                                        if cur_tab == "profile" {
                                            "bg-primary text-primary-foreground shadow-xs font-bold"
                                        } else {
                                            "hover:bg-muted/60 text-foreground"
                                        }
                                    ),
                                    onclick: move |_| active_tab.set("profile".to_string()),
                                    div { class: "flex items-center gap-2.5",
                                        UserIcon { class: "size-4" }
                                        div { class: "flex flex-col",
                                            span { "Profile Information" }
                                            span { class: format!("text-[10px] font-normal {}", if cur_tab == "profile" { "text-primary-foreground/80" } else { "text-muted-foreground" }),
                                                "Display name, email & phone"
                                            }
                                        }
                                    }
                                }

                                // 2. Security
                                button {
                                    r#type: "button",
                                    class: format!(
                                        "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer text-left {}",
                                        if cur_tab == "security" {
                                            "bg-primary text-primary-foreground shadow-xs font-bold"
                                        } else {
                                            "hover:bg-muted/60 text-foreground"
                                        }
                                    ),
                                    onclick: move |_| active_tab.set("security".to_string()),
                                    div { class: "flex items-center gap-2.5",
                                        LockIcon { class: "size-4" }
                                        div { class: "flex flex-col",
                                            span { "Security & Credentials" }
                                            span { class: format!("text-[10px] font-normal {}", if cur_tab == "security" { "text-primary-foreground/80" } else { "text-muted-foreground" }),
                                                "Password & encryption settings"
                                            }
                                        }
                                    }
                                }

                                // 3. Permissions
                                button {
                                    r#type: "button",
                                    class: format!(
                                        "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer text-left {}",
                                        if cur_tab == "permissions" {
                                            "bg-primary text-primary-foreground shadow-xs font-bold"
                                        } else {
                                            "hover:bg-muted/60 text-foreground"
                                        }
                                    ),
                                    onclick: move |_| active_tab.set("permissions".to_string()),
                                    div { class: "flex items-center gap-2.5",
                                        ShieldCheckIcon { class: "size-4" }
                                        div { class: "flex flex-col",
                                            span { "Roles & Capabilities" }
                                            span { class: format!("text-[10px] font-normal {}", if cur_tab == "permissions" { "text-primary-foreground/80" } else { "text-muted-foreground" }),
                                                "Access privileges matrix"
                                            }
                                        }
                                    }
                                }

                                // 4. Team Directory (Admin / Super Admin)
                                if is_admin {
                                    button {
                                        r#type: "button",
                                        class: format!(
                                            "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer text-left {}",
                                            if cur_tab == "team" {
                                                "bg-primary text-primary-foreground shadow-xs font-bold"
                                            } else {
                                                "hover:bg-muted/60 text-foreground"
                                            }
                                        ),
                                        onclick: move |_| active_tab.set("team".to_string()),
                                        div { class: "flex items-center gap-2.5",
                                            UsersIcon { class: "size-4" }
                                            div { class: "flex flex-col",
                                                span { "User Directory" }
                                                span { class: format!("text-[10px] font-normal {}", if cur_tab == "team" { "text-primary-foreground/80" } else { "text-muted-foreground" }),
                                                    if is_super_admin { "Provision & manage accounts" } else { "View team roster" }
                                                }
                                            }
                                        }
                                        span {
                                            class: format!(
                                                "px-1.5 py-0.5 rounded-full text-[10px] font-bold {}",
                                                if cur_tab == "team" { "bg-primary-foreground/20 text-primary-foreground" } else { "bg-muted text-muted-foreground" }
                                            ),
                                            "{total_users_count}"
                                        }
                                    }
                                }

                                // 5. Email Domains (Super Admin only)
                                if is_super_admin {
                                    button {
                                        r#type: "button",
                                        class: format!(
                                            "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer text-left {}",
                                            if cur_tab == "domains" {
                                                "bg-primary text-primary-foreground shadow-xs font-bold"
                                            } else {
                                                "hover:bg-muted/60 text-foreground"
                                            }
                                        ),
                                        onclick: move |_| active_tab.set("domains".to_string()),
                                        div { class: "flex items-center gap-2.5",
                                            GlobeIcon { class: "size-4" }
                                            div { class: "flex flex-col",
                                                span { "Email Domain Policy" }
                                                span { class: format!("text-[10px] font-normal {}", if cur_tab == "domains" { "text-primary-foreground/80" } else { "text-muted-foreground" }),
                                                    "Company domain whitelist"
                                                }
                                            }
                                        }
                                        span {
                                            class: format!(
                                                "px-1.5 py-0.5 rounded-full text-[10px] font-bold {}",
                                                if cur_tab == "domains" { "bg-primary-foreground/20 text-primary-foreground" } else { "bg-muted text-muted-foreground" }
                                            ),
                                            "{total_domains_count}"
                                        }
                                    }
                                }

                                // 6. Google Sheets & System Administration (Super Admin only)
                                if is_super_admin {
                                    button {
                                        r#type: "button",
                                        class: format!(
                                            "w-full flex items-center justify-between px-3 py-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer text-left {}",
                                            if cur_tab == "sheets" {
                                                "bg-primary text-primary-foreground shadow-xs font-bold"
                                            } else {
                                                "hover:bg-muted/60 text-foreground"
                                            }
                                        ),
                                        onclick: move |_| active_tab.set("sheets".to_string()),
                                        div { class: "flex items-center gap-2.5",
                                            UploadCloudIcon { class: "size-4" }
                                            div { class: "flex flex-col",
                                                span { "Google Sheets & Sync" }
                                                span { class: format!("text-[10px] font-normal {}", if cur_tab == "sheets" { "text-primary-foreground/80" } else { "text-muted-foreground" }),
                                                    "Cloud sync & database control"
                                                }
                                            }
                                        }
                                        span {
                                            class: format!(
                                                "px-1.5 py-0.5 rounded-full text-[10px] font-bold {}",
                                                if cur_tab == "sheets" {
                                                    "bg-primary-foreground/20 text-primary-foreground"
                                                } else if !sheet_url_input.read().trim().is_empty() {
                                                    "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400"
                                                } else {
                                                    "bg-muted text-muted-foreground"
                                                }
                                            ),
                                            if !sheet_url_input.read().trim().is_empty() { "Active" } else { "Setup" }
                                        }
                                    }
                                }
                            }
                        }

                        }

                    // --------------------------------------------------------
                    // RIGHT COLUMN: Selected Tab Content Panels (Cols 8/12)
                    // --------------------------------------------------------
                    div { class: "lg:col-span-8 space-y-6",

                        // ====================================================
                        // TAB 1: Profile Information
                        // ====================================================
                        if cur_tab == "profile" {
                            // Section 1: Contact & Personal Details Card
                            Card { class: "border-border/80 shadow-xs overflow-hidden",
                                CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4",
                                    div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-2",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold", "Personal Information" }
                                            CardDescription { class: "text-xs",
                                                "Update your contact details and organizational role"
                                            }
                                        }
                                        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-medium bg-muted/60 border border-border/60 text-muted-foreground",
                                            GlobeIcon { class: "size-3" }
                                            "{active_domain_hint}"
                                        }
                                    }
                                }

                                CardContent { class: "p-6 space-y-5",
                                    if let Some((success, msg)) = profile_msg.read().as_ref() {
                                        div {
                                            class: format!(
                                                "flex items-center gap-2.5 p-3 rounded-lg text-xs border animate-in slide-in-from-top-1 {}",
                                                if *success { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                            ),
                                            if *success {
                                                CheckCircle2Icon { class: "size-4 shrink-0" }
                                            } else {
                                                AlertCircleIcon { class: "size-4 shrink-0" }
                                            }
                                            span { class: "font-medium", "{msg}" }
                                        }
                                    }

                                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-5",
                                        // Display Name
                                        div { class: "space-y-1.5",
                                            Label { r#for: "prof_name", class: "text-xs font-semibold text-foreground flex items-center gap-1.5",
                                                UserIcon { class: "size-3.5 text-muted-foreground" }
                                                "Display Name *"
                                            }
                                            Input {
                                                id: "prof_name",
                                                value: edit_name.read().clone(),
                                                class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                placeholder: "Your full name".to_string(),
                                                oninput: move |e: FormEvent| edit_name.set(e.value()),
                                            }
                                            p { class: "text-[11px] text-muted-foreground", "Used across tickets, comments, and notifications." }
                                        }

                                        // Username (Read-Only)
                                        div { class: "space-y-1.5",
                                            Label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                                span { class: "flex items-center gap-1.5",
                                                    LockIcon { class: "size-3 text-muted-foreground" }
                                                    "Username"
                                                }
                                                span { class: "text-[10px] text-muted-foreground font-mono font-normal", "Permanent Handle" }
                                            }
                                            div { class: "relative",
                                                Input {
                                                    disabled: true,
                                                    value: format!("@{}", user.username),
                                                    class: "text-xs h-9 bg-muted/50 font-mono text-muted-foreground cursor-not-allowed",
                                                }
                                            }
                                            p { class: "text-[11px] text-muted-foreground", "System identity key assigned during registration." }
                                        }

                                        // Email Address
                                        div { class: "space-y-1.5",
                                            Label { r#for: "prof_email", class: "text-xs font-semibold text-foreground flex items-center gap-1.5",
                                                MailIcon { class: "size-3.5 text-muted-foreground" }
                                                "Email Address *"
                                            }
                                            Input {
                                                id: "prof_email",
                                                r#type: "email".to_string(),
                                                value: edit_email.read().clone(),
                                                class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                oninput: move |e: FormEvent| edit_email.set(e.value()),
                                            }
                                            p { class: "text-[11px] text-muted-foreground", "{active_domain_hint}" }
                                        }

                                        // Department
                                        div { class: "space-y-1.5",
                                            Label { r#for: "prof_dept", class: "text-xs font-semibold text-foreground flex items-center gap-1.5",
                                                BuildingIcon { class: "size-3.5 text-muted-foreground" }
                                                "Department"
                                            }
                                            Input {
                                                id: "prof_dept",
                                                value: edit_dept.read().clone(),
                                                class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                placeholder: "e.g. IT Support".to_string(),
                                                oninput: move |e: FormEvent| edit_dept.set(e.value()),
                                            }
                                            p { class: "text-[11px] text-muted-foreground", "Your organizational unit or technical division." }
                                        }

                                        // Phone
                                        div { class: "space-y-1.5 sm:col-span-2",
                                            Label { r#for: "prof_phone", class: "text-xs font-semibold text-foreground flex items-center gap-1.5",
                                                PhoneIcon { class: "size-3.5 text-muted-foreground" }
                                                "Phone / Extension"
                                            }
                                            Input {
                                                id: "prof_phone",
                                                value: edit_phone.read().clone(),
                                                class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                placeholder: "e.g. +91 98765 43210 or Ext. 402".to_string(),
                                                oninput: move |e: FormEvent| edit_phone.set(e.value()),
                                            }
                                            p { class: "text-[11px] text-muted-foreground", "Internal extension or direct contact for high-priority incidents." }
                                        }
                                    }
                                }

                                CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-4 flex items-center justify-between gap-4",
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Changes take effect immediately across all support views."
                                    }
                                    Button {
                                        size: "sm".to_string(),
                                        class: "text-xs font-semibold shadow-xs cursor-pointer px-5 h-9 bg-primary text-primary-foreground hover:bg-primary/90",
                                        onclick: move |_| handle_profile_save(()),
                                        CheckCircle2Icon { class: "size-3.5 mr-1.5" }
                                        "Save Profile Changes"
                                    }
                                }
                            }

                            // Section 2: Avatar Appearance Palette Card
                            Card { class: "border-border/80 shadow-xs overflow-hidden",
                                CardHeader { class: "border-b border-border/50 bg-muted/15 pb-3",
                                    CardTitle { class: "text-sm font-semibold", "Avatar Theme Accent" }
                                    CardDescription { class: "text-xs", "Customize your avatar accent badge color in team lists and ticket comments" }
                                }
                                CardContent { class: "p-6",
                                    div { class: "flex items-center gap-3 flex-wrap",
                                        for (cls, name, hex) in AVATAR_PALETTE.iter() {
                                            {
                                                let c_cls = *cls;
                                                let c_name = *name;
                                                let is_active = user.avatar_color == c_cls;
                                                rsx! {
                                                    button {
                                                        key: "{c_cls}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "inline-flex items-center gap-2 px-3 py-1.5 rounded-lg border text-xs font-medium transition-all cursor-pointer {}",
                                                            if is_active {
                                                                "border-primary ring-2 ring-primary/20 bg-muted/60 font-bold"
                                                            } else {
                                                                "border-border/60 hover:bg-muted/40"
                                                            }
                                                        ),
                                                        onclick: move |_| handle_color_change(c_cls),
                                                        span {
                                                            class: "size-4 rounded-full shadow-2xs shrink-0 ring-1 ring-black/10",
                                                            style: format!("background-color: {hex}"),
                                                        }
                                                        span { "{c_name}" }
                                                        if is_active {
                                                            CheckIcon { class: "size-3 text-primary stroke-[3]" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Section 3: Workstation Audit & Session Context
                            Card { class: "border-border/80 shadow-xs bg-muted/10",
                                CardHeader { class: "border-b border-border/50 bg-muted/20 pb-3",
                                    CardTitle { class: "text-sm font-semibold flex items-center gap-2",
                                        LaptopIcon { class: "size-4 text-muted-foreground" }
                                        "Workstation & Security Context"
                                    }
                                }
                                CardContent { class: "p-5",
                                    div { class: "grid grid-cols-1 sm:grid-cols-3 gap-4 text-xs",
                                        div { class: "space-y-1",
                                            p { class: "text-muted-foreground text-[11px]", "Platform Environment" }
                                            p { class: "font-semibold text-foreground", "Windows Desktop Native" }
                                        }
                                        div { class: "space-y-1",
                                            p { class: "text-muted-foreground text-[11px]", "Credential Encryption" }
                                            p { class: "font-semibold text-emerald-600 dark:text-emerald-400 font-mono", "SHA-256 + 128-bit Salt" }
                                        }
                                        div { class: "space-y-1",
                                            p { class: "text-muted-foreground text-[11px]", "Active Database" }
                                            p { class: "font-semibold text-foreground font-mono", "SQLite WAL Engine" }
                                        }
                                    }
                                }
                            }
                        }

                        // ====================================================
                        // TAB 2: Security & Password
                        // ====================================================
                        if cur_tab == "security" {
                            div { class: "space-y-6",
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-row items-center justify-between",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold", "Change Account Password" }
                                            CardDescription { class: "text-xs", "Update your password regularly to protect enterprise support records" }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "inline-flex items-center gap-1.5 text-[11px] font-medium text-muted-foreground hover:text-foreground px-2.5 py-1 rounded-md border border-border/60 bg-background cursor-pointer shadow-2xs",
                                            onclick: move |_| {
                                                let cur = show_passwords();
                                                show_passwords.set(!cur);
                                            },
                                            if *show_passwords.read() {
                                                EyeOffIcon { class: "size-3.5" }
                                                "Hide Passwords"
                                            } else {
                                                EyeIcon { class: "size-3.5" }
                                                "Reveal Passwords"
                                            }
                                        }
                                    }
                                    CardContent { class: "p-6 space-y-5",
                                        if let Some((success, msg)) = pass_msg.read().as_ref() {
                                            div {
                                                class: format!(
                                                    "flex items-center gap-2.5 p-3 rounded-lg text-xs border animate-in slide-in-from-top-1 {}",
                                                    if *success { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                                ),
                                                if *success { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                                span { class: "font-medium", "{msg}" }
                                            }
                                        }

                                        div { class: "space-y-1.5",
                                            Label { r#for: "curr_pass", class: "text-xs font-semibold", "Current Password *" }
                                            Input {
                                                id: "curr_pass",
                                                r#type: if *show_passwords.read() { "text".to_string() } else { "password".to_string() },
                                                value: current_pass.read().clone(),
                                                class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                placeholder: "Enter existing account password".to_string(),
                                                oninput: move |e: FormEvent| current_pass.set(e.value()),
                                            }
                                        }

                                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                                            div { class: "space-y-1.5",
                                                Label { r#for: "new_pass", class: "text-xs font-semibold", "New Password *" }
                                                Input {
                                                    id: "new_pass",
                                                    r#type: if *show_passwords.read() { "text".to_string() } else { "password".to_string() },
                                                    value: new_pass.read().clone(),
                                                    class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                    placeholder: "Minimum 6 characters".to_string(),
                                                    oninput: move |e: FormEvent| new_pass.set(e.value()),
                                                }
                                            }
                                            div { class: "space-y-1.5",
                                                Label { r#for: "conf_pass", class: "text-xs font-semibold", "Confirm New Password *" }
                                                Input {
                                                    id: "conf_pass",
                                                    r#type: if *show_passwords.read() { "text".to_string() } else { "password".to_string() },
                                                    value: confirm_new_pass.read().clone(),
                                                    class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                    placeholder: "Re-enter new password".to_string(),
                                                    oninput: move |e: FormEvent| confirm_new_pass.set(e.value()),
                                                }
                                            }
                                        }

                                        // Dynamic Strength Meter & Checklist
                                        if !new_pass.read().is_empty() {
                                            div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 space-y-3",
                                                div { class: "flex items-center justify-between text-xs",
                                                    span { class: "font-semibold text-muted-foreground", "Password Complexity Rating" }
                                                    span {
                                                        class: match pass_score {
                                                            0 | 1 => "text-destructive font-bold",
                                                            2 => "text-amber-500 font-bold",
                                                            3 => "text-sky-500 font-bold",
                                                            _ => "text-emerald-500 font-bold",
                                                        },
                                                        match pass_score {
                                                            0 | 1 => "Weak Security",
                                                            2 => "Fair Complexity",
                                                            3 => "Good Protection",
                                                            _ => "Strong Enterprise Grade",
                                                        }
                                                    }
                                                }
                                                div { class: "h-2 w-full bg-muted rounded-full overflow-hidden flex gap-1",
                                                    for i in 1..=4 {
                                                        div {
                                                            class: format!(
                                                                "h-full flex-1 rounded-full transition-colors {}",
                                                                if pass_score >= i {
                                                                    match pass_score {
                                                                        1 => "bg-destructive",
                                                                        2 => "bg-amber-500",
                                                                        3 => "bg-sky-500",
                                                                        _ => "bg-emerald-500",
                                                                    }
                                                                } else {
                                                                    "bg-border/60"
                                                                }
                                                            )
                                                        }
                                                    }
                                                }

                                                // Checklist
                                                div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2 pt-1 text-[11px]",
                                                    div { class: format!("flex items-center gap-1.5 {}", if p_len >= 6 { "text-emerald-600 dark:text-emerald-400 font-medium" } else { "text-muted-foreground" }),
                                                        if p_len >= 6 { CheckIcon { class: "size-3 stroke-[3]" } } else { span { class: "size-1.5 rounded-full bg-muted-foreground/60" } }
                                                        "Min 6 chars"
                                                    }
                                                    div { class: format!("flex items-center gap-1.5 {}", if p_upper { "text-emerald-600 dark:text-emerald-400 font-medium" } else { "text-muted-foreground" }),
                                                        if p_upper { CheckIcon { class: "size-3 stroke-[3]" } } else { span { class: "size-1.5 rounded-full bg-muted-foreground/60" } }
                                                        "Uppercase letter"
                                                    }
                                                    div { class: format!("flex items-center gap-1.5 {}", if p_digit { "text-emerald-600 dark:text-emerald-400 font-medium" } else { "text-muted-foreground" }),
                                                        if p_digit { CheckIcon { class: "size-3 stroke-[3]" } } else { span { class: "size-1.5 rounded-full bg-muted-foreground/60" } }
                                                        "Number (0-9)"
                                                    }
                                                    div { class: format!("flex items-center gap-1.5 {}", if p_match { "text-emerald-600 dark:text-emerald-400 font-medium" } else { "text-muted-foreground" }),
                                                        if p_match { CheckIcon { class: "size-3 stroke-[3]" } } else { span { class: "size-1.5 rounded-full bg-muted-foreground/60" } }
                                                        "Passwords match"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-4 flex items-center justify-between",
                                        span { class: "text-[11px] text-muted-foreground", "Salted hash verification prevents dictionary attacks" }
                                        Button {
                                            size: "sm".to_string(),
                                            class: "text-xs font-semibold shadow-xs cursor-pointer px-5 h-9 bg-primary text-primary-foreground",
                                            onclick: move |_| handle_password_change(()),
                                            KeyIcon { class: "size-3.5 mr-1.5" }
                                            "Update Password"
                                        }
                                    }
                                }

                                // Active Session Card
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4",
                                        CardTitle { class: "text-base font-semibold", "Current Device Session" }
                                        CardDescription { class: "text-xs", "Active authentication state on this Windows desktop installation" }
                                    }
                                    CardContent { class: "p-6 space-y-4",
                                        div { class: "flex items-center justify-between p-4 rounded-xl bg-card border border-border/80",
                                            div { class: "flex items-center gap-3.5",
                                                div { class: "flex size-10 items-center justify-center rounded-xl bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20",
                                                    LaptopIcon { class: "size-5" }
                                                }
                                                div { class: "space-y-0.5",
                                                    p { class: "text-xs font-bold text-foreground", "Windows Desktop Client (Current Device)" }
                                                    p { class: "text-[11px] text-muted-foreground",
                                                        "Connected via local database session token"
                                                    }
                                                }
                                            }
                                            span { class: "px-2.5 py-1 rounded-full text-[11px] font-semibold bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 border border-emerald-500/30",
                                                "Active Now"
                                            }
                                        }
                                    }
                                    CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-3.5 flex justify-end",
                                        Button {
                                            variant: "outline".to_string(),
                                            size: "sm".to_string(),
                                            class: "text-xs text-destructive hover:text-destructive hover:bg-destructive/10 border-destructive/30 cursor-pointer",
                                            onclick: move |_| handle_logout(()),
                                            LogOutIcon { class: "size-3.5 mr-1.5" }
                                            "Sign Out of This Device"
                                        }
                                    }
                                }
                            }
                        }

                        // ====================================================
                        // TAB 3: Roles & Permissions Matrix
                        // ====================================================
                        if cur_tab == "permissions" {
                            Card { class: "border-border/80 shadow-xs overflow-hidden",
                                CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-row items-center justify-between",
                                    div { class: "space-y-1",
                                        CardTitle { class: "text-base font-semibold", "Role Capabilities & Authorization" }
                                        CardDescription { class: "text-xs", "Permissions granted to your assigned security role" }
                                    }
                                    span { class: "px-3 py-1 rounded-full text-xs font-semibold border {user.role_badge_color()} shadow-2xs",
                                        "{user.role_label()}"
                                    }
                                }
                                CardContent { class: "p-6 space-y-4",
                                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3.5 text-xs",
                                        // Capability 1: View Support Tickets
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "Support Ticket Workspace" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Browse Kanban boards, ticket tables, search filters, and resolution analytics."
                                                }
                                            }
                                            span { class: "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 font-semibold text-[11px]",
                                                CheckIcon { class: "size-3 stroke-[3]" }
                                                "Granted"
                                            }
                                        }

                                        // Capability 2: Edit & Progress Tickets
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "Create, Edit & Advance Tickets" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Create support requests, progress Kanban columns, assign agents, and log technical notes."
                                                }
                                            }
                                            span {
                                                class: if user.is_agent() {
                                                    "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 font-semibold text-[11px]"
                                                } else {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium text-[11px]"
                                                },
                                                if user.is_agent() { "Granted" } else { "Read-Only" }
                                            }
                                        }

                                        // Capability 3: Google Sheets Sync
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "Google Sheets Sync Engine" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Push local tickets to cloud spreadsheets, pull remote rows, and handle conflict resolutions."
                                                }
                                            }
                                            span {
                                                class: if user.is_agent() {
                                                    "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 font-semibold text-[11px]"
                                                } else {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium text-[11px]"
                                                },
                                                if user.is_agent() { "Granted" } else { "Disabled" }
                                            }
                                        }

                                        // Capability 4: Reminders & Alerts
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "Reminders & Follow-up Scheduling" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Schedule ticket follow-ups, snooze pending items, and configure deadline notifications."
                                                }
                                            }
                                            span {
                                                class: if user.is_agent() {
                                                    "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 font-semibold text-[11px]"
                                                } else {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium text-[11px]"
                                                },
                                                if user.is_agent() { "Granted" } else { "Read-Only" }
                                            }
                                        }

                                        // Capability 5: User Account Administration
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "User Directory & Provisioning" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Create user accounts with custom brackets in usernames, assign roles, and activate/deactivate staff."
                                                }
                                            }
                                            span {
                                                class: if is_super_admin {
                                                    "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-amber-500/15 text-amber-700 dark:text-amber-400 font-bold text-[11px]"
                                                } else if is_admin {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-indigo-500/15 text-indigo-700 dark:text-indigo-400 font-semibold text-[11px]"
                                                } else {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium text-[11px]"
                                                },
                                                if is_super_admin { "Full Admin" } else if is_admin { "Directory View" } else { "Restricted" }
                                            }
                                        }

                                        // Capability 6: Domain-Based Email System
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "Domain-Based Email Governance" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Allow or restrict employee emails to specific corporate domains (.bhagwatipower.com, .gmail.com)."
                                                }
                                            }
                                            span {
                                                class: if is_super_admin {
                                                    "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-amber-500/15 text-amber-700 dark:text-amber-400 font-bold text-[11px]"
                                                } else {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium text-[11px]"
                                                },
                                                if is_super_admin { "Super Admin" } else { "Restricted" }
                                            }
                                        }

                                        // Capability 7: Google Sheets & Database Administration
                                        div { class: "flex items-start justify-between gap-3 p-4 rounded-xl border border-border/70 bg-card shadow-2xs",
                                            div { class: "space-y-1",
                                                p { class: "font-bold text-foreground", "Google Sheets & Database Maintenance" }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "Configure Google Apps Script deployments, verify/init spreadsheet schemas, control sync cadence, and optimize SQLite storage."
                                                }
                                            }
                                            span {
                                                class: if is_super_admin {
                                                    "shrink-0 inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full bg-amber-500/15 text-amber-700 dark:text-amber-400 font-bold text-[11px]"
                                                } else {
                                                    "shrink-0 inline-flex items-center px-2.5 py-0.5 rounded-full bg-muted text-muted-foreground font-medium text-[11px]"
                                                },
                                                if is_super_admin { "Super Admin" } else { "Restricted" }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // ====================================================
                        // TAB 4: User Directory (Admin & Super Admin)
                        // ====================================================
                        if cur_tab == "team" && is_admin {
                            div { class: "space-y-5",
                                // Top KPI Stats Bar
                                div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3",
                                    div { class: "rounded-xl border border-border/80 bg-card p-4 shadow-2xs space-y-1",
                                        p { class: "text-[11px] font-semibold text-muted-foreground uppercase tracking-wider", "Total Accounts" }
                                        p { class: "text-2xl font-bold text-foreground", "{total_users_count}" }
                                    }
                                    div { class: "rounded-xl border border-border/80 bg-card p-4 shadow-2xs space-y-1",
                                        p { class: "text-[11px] font-semibold text-muted-foreground uppercase tracking-wider", "Active Staff" }
                                        p { class: "text-2xl font-bold text-emerald-600 dark:text-emerald-400", "{active_users_count}" }
                                    }
                                    div { class: "rounded-xl border border-border/80 bg-card p-4 shadow-2xs space-y-1",
                                        p { class: "text-[11px] font-semibold text-muted-foreground uppercase tracking-wider", "Administrators" }
                                        p { class: "text-2xl font-bold text-indigo-600 dark:text-indigo-400", "{admin_users_count}" }
                                    }
                                    div { class: "rounded-xl border border-border/80 bg-card p-4 shadow-2xs space-y-1",
                                        p { class: "text-[11px] font-semibold text-muted-foreground uppercase tracking-wider", "Support Agents" }
                                        p { class: "text-2xl font-bold text-primary", "{agent_users_count}" }
                                    }
                                }

                                // Directory Card
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold", "Team Accounts Directory" }
                                            CardDescription { class: "text-xs",
                                                if is_super_admin {
                                                    "Provision user accounts, configure roles, and manage system access"
                                                } else {
                                                    "List of all registered system accounts"
                                                }
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            div { class: "relative w-48 sm:w-60",
                                                div { class: "pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground",
                                                    SearchIcon { class: "size-3.5" }
                                                }
                                                Input {
                                                    class: "pl-8 h-8 text-xs bg-background focus:ring-1 focus:ring-primary",
                                                    placeholder: "Search accounts...".to_string(),
                                                    value: user_search.read().clone(),
                                                    oninput: move |e: FormEvent| user_search.set(e.value()),
                                                }
                                            }
                                            if is_super_admin {
                                                Button {
                                                    variant: if *show_create_user.read() { "outline".to_string() } else { "default".to_string() },
                                                    size: "sm".to_string(),
                                                    class: "text-xs font-semibold shrink-0 cursor-pointer h-8",
                                                    onclick: move |_| {
                                                        let cur = show_create_user();
                                                        show_create_user.set(!cur);
                                                        create_u_msg.set(None);
                                                    },
                                                    if *show_create_user.read() {
                                                        XIcon { class: "size-3.5 mr-1.5" }
                                                        "Cancel"
                                                    } else {
                                                        PlusIcon { class: "size-3.5 mr-1.5" }
                                                        "New Account"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    CardContent { class: "p-6 space-y-4",
                                        // Super Admin Account Creation Drawer
                                        if is_super_admin && *show_create_user.read() {
                                            div { class: "rounded-xl border border-primary/30 bg-primary/5 p-5 space-y-4 animate-in slide-in-from-top-2",
                                                div { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-1 pb-2 border-b border-primary/15",
                                                    div { class: "space-y-0.5",
                                                        h4 { class: "text-xs font-bold text-foreground", "Provision New Employee Account" }
                                                        p { class: "text-[11px] text-muted-foreground", "Brackets () [] are supported in usernames throughout the app" }
                                                    }
                                                    span { class: "inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-semibold bg-primary/10 text-primary border border-primary/20",
                                                        "{active_domain_hint}"
                                                    }
                                                }

                                                if let Some((ok, msg)) = create_u_msg.read().as_ref() {
                                                    div {
                                                        class: format!(
                                                            "flex items-center gap-2 p-3 rounded-lg text-xs border {}",
                                                            if *ok { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                                        ),
                                                        if *ok { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                                        span { class: "font-medium", "{msg}" }
                                                    }
                                                }

                                                div { class: "grid grid-cols-1 sm:grid-cols-3 gap-3.5",
                                                    div { class: "space-y-1",
                                                        Label { class: "text-[11px] font-semibold", "Full Name *" }
                                                        Input {
                                                            class: "text-xs h-8 bg-background focus:ring-1 focus:ring-primary",
                                                            placeholder: "e.g. Rahul Sharma".to_string(),
                                                            value: new_u_name.read().clone(),
                                                            oninput: move |e: FormEvent| new_u_name.set(e.value()),
                                                        }
                                                    }
                                                    div { class: "space-y-1",
                                                        Label { class: "text-[11px] font-semibold", "Username (brackets allowed) *" }
                                                        Input {
                                                            class: "text-xs h-8 bg-background focus:ring-1 focus:ring-primary",
                                                            placeholder: "e.g. rahul(it) or [support]01".to_string(),
                                                            value: new_u_username.read().clone(),
                                                            oninput: move |e: FormEvent| new_u_username.set(e.value()),
                                                        }
                                                    }
                                                    div { class: "space-y-1",
                                                        Label { class: "text-[11px] font-semibold", "Email Address *" }
                                                        Input {
                                                            r#type: "email".to_string(),
                                                            class: "text-xs h-8 bg-background focus:ring-1 focus:ring-primary",
                                                            placeholder: "e.g. rahul@bhagwatipower.com".to_string(),
                                                            value: new_u_email.read().clone(),
                                                            oninput: move |e: FormEvent| new_u_email.set(e.value()),
                                                        }
                                                    }
                                                    div { class: "space-y-1",
                                                        Label { class: "text-[11px] font-semibold", "Department" }
                                                        Input {
                                                            class: "text-xs h-8 bg-background focus:ring-1 focus:ring-primary",
                                                            placeholder: "e.g. IT Support".to_string(),
                                                            value: new_u_dept.read().clone(),
                                                            oninput: move |e: FormEvent| new_u_dept.set(e.value()),
                                                        }
                                                    }
                                                    div { class: "space-y-1",
                                                        Label { class: "text-[11px] font-semibold", "Assigned Role" }
                                                        select {
                                                            class: "w-full h-8 px-2.5 text-xs rounded-md border border-input bg-background text-foreground focus:outline-none focus:ring-1 focus:ring-primary cursor-pointer",
                                                            value: new_u_role.read().as_str(),
                                                            onchange: move |e| new_u_role.set(e.value()),
                                                            option { value: "agent", "Support Agent (Full Ticket Access)" }
                                                            option { value: "viewer", "Read-Only Viewer" }
                                                            option { value: "admin", "Administrator (Management)" }
                                                        }
                                                    }
                                                    div { class: "space-y-1",
                                                        Label { class: "text-[11px] font-semibold", "Initial Password *" }
                                                        Input {
                                                            r#type: "password".to_string(),
                                                            class: "text-xs h-8 bg-background focus:ring-1 focus:ring-primary",
                                                            placeholder: "Min 6 characters".to_string(),
                                                            value: new_u_pass.read().clone(),
                                                            oninput: move |e: FormEvent| new_u_pass.set(e.value()),
                                                        }
                                                    }
                                                }

                                                div { class: "flex justify-end pt-1",
                                                    Button {
                                                        size: "sm".to_string(),
                                                        class: "text-xs font-semibold cursor-pointer h-8 px-4 bg-primary text-primary-foreground",
                                                        onclick: move |_| handle_create_user(()),
                                                        PlusIcon { class: "size-3.5 mr-1.5" }
                                                        "Create User Account"
                                                    }
                                                }
                                            }
                                        }

                                        // Users Table
                                        if filtered_users.is_empty() {
                                            div { class: "rounded-xl border border-dashed border-border/80 p-8 text-center text-xs text-muted-foreground space-y-1",
                                                p { class: "font-semibold text-foreground", "No matching user accounts found" }
                                                p { "Try adjusting your search criteria or clear the query filter." }
                                            }
                                        } else {
                                            div { class: "rounded-xl border border-border/70 overflow-hidden text-xs shadow-2xs",
                                                div { class: "grid grid-cols-12 p-3 bg-muted/40 font-semibold border-b border-border/70 text-[11px] text-muted-foreground uppercase tracking-wider",
                                                    span { class: "col-span-4", "User / Identity" }
                                                    span { class: "col-span-3", "Email Address" }
                                                    span { class: "col-span-2", "Department" }
                                                    span { class: "col-span-3 text-right", "Role & Status Control" }
                                                }

                                                for u in filtered_users.iter() {
                                                    {
                                                        let target_id = u.id;
                                                        let target_role = u.role.clone();
                                                        let target_is_super = u.is_super_admin();
                                                        let is_self = target_id == user_id;
                                                        let active_status = u.is_active;

                                                        rsx! {
                                                            div {
                                                                key: "{target_id}",
                                                                class: "grid grid-cols-12 p-3.5 border-b border-border/60 last:border-b-0 items-center hover:bg-muted/20 transition-colors",
                                                                // User column
                                                                div { class: "col-span-4 flex items-center gap-3 min-w-0",
                                                                    div { class: "relative shrink-0",
                                                                        div { class: "flex size-8 items-center justify-center rounded-lg text-xs font-bold text-white {u.avatar_color} shadow-2xs",
                                                                            "{u.initials()}"
                                                                        }
                                                                        span {
                                                                            class: format!(
                                                                                "absolute -bottom-0.5 -right-0.5 size-2.5 rounded-full ring-2 ring-card {}",
                                                                                if active_status { "bg-emerald-500" } else { "bg-zinc-400" }
                                                                            )
                                                                        }
                                                                    }
                                                                    div { class: "flex flex-col min-w-0 truncate",
                                                                        span { class: "font-semibold text-foreground truncate flex items-center gap-1",
                                                                            "{u.display_name}"
                                                                            if is_self {
                                                                                span { class: "px-1.5 py-0.2 rounded text-[10px] bg-primary/10 text-primary font-bold", "You" }
                                                                            }
                                                                        }
                                                                        span { class: "text-[11px] font-mono text-muted-foreground truncate", "@{u.username}" }
                                                                    }
                                                                }

                                                                // Email column
                                                                div { class: "col-span-3 truncate text-muted-foreground",
                                                                    "{u.email}"
                                                                }

                                                                // Department column
                                                                div { class: "col-span-2 truncate text-muted-foreground",
                                                                    if u.department.is_empty() { "—" } else { "{u.department}" }
                                                                }

                                                                // Actions column
                                                                div { class: "col-span-3 flex items-center justify-end gap-2",
                                                                    if target_is_super {
                                                                        span { class: "px-2.5 py-0.5 rounded-full text-[10px] font-bold border bg-amber-500/15 text-amber-700 dark:text-amber-400 border-amber-500/30",
                                                                            "Super Admin"
                                                                        }
                                                                    } else {
                                                                        select {
                                                                            class: "h-7 px-2 text-[11px] rounded-md border border-input bg-background font-medium focus:outline-none focus:ring-1 focus:ring-primary shadow-2xs cursor-pointer",
                                                                            disabled: is_self || !is_super_admin,
                                                                            value: target_role.as_str(),
                                                                            onchange: move |e| {
                                                                                let new_r = e.value();
                                                                                match crate::auth::update_user_role(target_id, &new_r) {
                                                                                    Ok(_) => {
                                                                                        team_revision.with_mut(|r| *r += 1);
                                                                                        if let Some(t) = toast {
                                                                                            t.success("Role Updated", format!("User role changed to {new_r}."));
                                                                                        }
                                                                                    }
                                                                                    Err(err) => {
                                                                                        if let Some(t) = toast {
                                                                                            t.error("Error", err);
                                                                                        }
                                                                                    }
                                                                                }
                                                                            },
                                                                            option { value: "agent", "Agent" }
                                                                            option { value: "viewer", "Viewer" }
                                                                            option { value: "admin", "Admin" }
                                                                        }

                                                                        if !is_self && is_super_admin {
                                                                            button {
                                                                                r#type: "button",
                                                                                class: format!(
                                                                                    "h-7 px-2.5 text-[11px] rounded-md font-semibold transition-colors cursor-pointer {}",
                                                                                    if active_status { "bg-muted text-muted-foreground hover:bg-destructive/15 hover:text-destructive" } else { "bg-emerald-500/15 text-emerald-600 hover:bg-emerald-500/25" }
                                                                                ),
                                                                                onclick: move |_| {
                                                                                    match crate::auth::toggle_user_active(target_id, user_id) {
                                                                                        Ok(active) => {
                                                                                        team_revision.with_mut(|r| *r += 1);
                                                                                        if let Some(t) = toast {
                                                                                            t.info("Status Changed", if active { "User account activated" } else { "User account deactivated" });
                                                                                        }
                                                                                    }
                                                                                    Err(err) => {
                                                                                        if let Some(t) = toast {
                                                                                            t.error("Error", err);
                                                                                        }
                                                                                    }
                                                                                }
                                                                            },
                                                                            if active_status { "Deactivate" } else { "Activate" }
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
                        }

                        }

                    // ====================================================
                    // TAB 5: Email Domains Policy (Super Admin Only)
                    // ====================================================
                    if cur_tab == "domains" && is_super_admin {
                            Card { class: "border-border/80 shadow-xs overflow-hidden",
                                CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                                    div { class: "space-y-1",
                                        CardTitle { class: "text-base font-semibold", "Domain-Based Email Governance" }
                                        CardDescription { class: "text-xs",
                                            "Control which company domains (.bhagwatipower.com, .gmail.com) are permitted for accounts"
                                        }
                                    }
                                    span {
                                        class: format!(
                                            "px-3 py-1 rounded-full text-xs font-semibold border shrink-0 w-fit {}",
                                            if active_domains_count == 0 {
                                                "bg-sky-500/10 text-sky-600 dark:text-sky-400 border-sky-500/30"
                                            } else {
                                                "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30"
                                            }
                                        ),
                                        if active_domains_count == 0 {
                                            "Open Policy (Any Email Permitted)"
                                        } else {
                                            "{active_domains_count} Approved Corporate Domain(s)"
                                        }
                                    }
                                }
                                CardContent { class: "p-6 space-y-5",
                                    if let Some((ok, msg)) = domain_msg.read().as_ref() {
                                        div {
                                            class: format!(
                                                "flex items-center gap-2.5 p-3 rounded-lg text-xs border {}",
                                                if *ok { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                            ),
                                            if *ok { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                            span { class: "font-medium", "{msg}" }
                                        }
                                    }

                                    // Add Domain Box
                                    div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 space-y-3",
                                        div { class: "space-y-0.5",
                                            h4 { class: "text-xs font-bold text-foreground", "Whitelist New Email Domain" }
                                            p { class: "text-[11px] text-muted-foreground",
                                                "Once configured, users can only register or update emails matching active allowed domains."
                                            }
                                        }
                                        div { class: "grid grid-cols-1 sm:grid-cols-12 gap-3 items-end",
                                            div { class: "sm:col-span-5 space-y-1",
                                                Label { class: "text-[11px] font-semibold", "Domain Name *" }
                                                Input {
                                                    class: "text-xs h-8 bg-background font-mono focus:ring-1 focus:ring-primary",
                                                    placeholder: "e.g. .bhagwatipower.com or .gmail.com".to_string(),
                                                    value: new_domain_input.read().clone(),
                                                    oninput: move |e: FormEvent| new_domain_input.set(e.value()),
                                                }
                                            }
                                            div { class: "sm:col-span-5 space-y-1",
                                                Label { class: "text-[11px] font-semibold", "Company Label (Optional)" }
                                                Input {
                                                    class: "text-xs h-8 bg-background focus:ring-1 focus:ring-primary",
                                                    placeholder: "e.g. Bhagwati Power Corporate".to_string(),
                                                    value: new_domain_desc.read().clone(),
                                                    oninput: move |e: FormEvent| new_domain_desc.set(e.value()),
                                                }
                                            }
                                            div { class: "sm:col-span-2",
                                                Button {
                                                    size: "sm".to_string(),
                                                    class: "w-full h-8 text-xs font-semibold cursor-pointer bg-primary text-primary-foreground",
                                                    onclick: move |_| handle_add_domain(()),
                                                    PlusIcon { class: "size-3.5 mr-1" }
                                                    "Add Domain"
                                                }
                                            }
                                        }
                                    }

                                    // Domain List Table
                                    if domains_all.is_empty() {
                                        div { class: "rounded-xl border border-dashed border-border/80 p-8 text-center text-xs text-muted-foreground space-y-1.5",
                                            p { class: "font-semibold text-foreground text-sm", "No Domain Whitelist Configured" }
                                            p { class: "max-w-md mx-auto leading-relaxed",
                                                "Currently, any valid email domain is allowed. Add your company domain (e.g. .bhagwatipower.com) above to enforce strict corporate email verification."
                                            }
                                        }
                                    } else {
                                        div { class: "rounded-xl border border-border/70 overflow-hidden text-xs shadow-2xs",
                                            div { class: "grid grid-cols-12 p-3 bg-muted/40 font-semibold border-b border-border/70 text-[11px] text-muted-foreground uppercase tracking-wider",
                                                span { class: "col-span-4", "Domain Suffix" }
                                                span { class: "col-span-4", "Organization / Label" }
                                                span { class: "col-span-1", "Status" }
                                                span { class: "col-span-3 text-right", "Actions" }
                                            }
                                            for dom in domains_all.iter() {
                                                {
                                                    let dom_id = dom.id;
                                                    let dom_str = dom.domain.clone();
                                                    let dom_desc = dom.description.clone();
                                                    let dom_active = dom.is_active;
                                                    let is_editing = *editing_domain_id.read() == Some(dom_id);

                                                    rsx! {
                                                        div {
                                                            key: "{dom_id}",
                                                            class: "grid grid-cols-12 p-3.5 border-b border-border/60 last:border-b-0 items-center gap-2 hover:bg-muted/20 transition-colors",
                                                            if is_editing {
                                                                div { class: "col-span-4",
                                                                    Input {
                                                                        class: "text-xs h-7 font-mono",
                                                                        value: edit_domain_val.read().clone(),
                                                                        oninput: move |e: FormEvent| edit_domain_val.set(e.value()),
                                                                    }
                                                                }
                                                                div { class: "col-span-4",
                                                                    Input {
                                                                        class: "text-xs h-7",
                                                                        value: edit_domain_desc.read().clone(),
                                                                        oninput: move |e: FormEvent| edit_domain_desc.set(e.value()),
                                                                    }
                                                                }
                                                                div { class: "col-span-4 flex items-center justify-end gap-1.5",
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "h-7 px-2.5 rounded-md bg-primary text-primary-foreground text-[11px] font-semibold cursor-pointer",
                                                                        onclick: move |_| {
                                                                            let val = edit_domain_val.read().clone();
                                                                            let dsc = edit_domain_desc.read().clone();
                                                                            match crate::auth::update_email_domain(dom_id, &val, &dsc) {
                                                                                Ok(_) => {
                                                                                    editing_domain_id.set(None);
                                                                                    domain_revision.with_mut(|r| *r += 1);
                                                                                    domain_msg.set(Some((true, "Domain updated successfully.".to_string())));
                                                                                }
                                                                                Err(err) => domain_msg.set(Some((false, err))),
                                                                            }
                                                                        },
                                                                        "Save"
                                                                    }
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "h-7 px-2 rounded-md bg-muted text-muted-foreground text-[11px] cursor-pointer",
                                                                        onclick: move |_| editing_domain_id.set(None),
                                                                        "Cancel"
                                                                    }
                                                                }
                                                            } else {
                                                                div { class: "col-span-4 font-mono font-semibold text-foreground truncate flex items-center gap-1.5",
                                                                    span { class: "size-2 rounded-full bg-primary" }
                                                                    "@{dom_str}"
                                                                }
                                                                div { class: "col-span-4 text-muted-foreground truncate",
                                                                    if dom_desc.is_empty() { "—" } else { "{dom_desc}" }
                                                                }
                                                                div { class: "col-span-1",
                                                                    span {
                                                                        class: format!(
                                                                            "px-2 py-0.5 rounded-full text-[10px] font-semibold {}",
                                                                            if dom_active { "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400" } else { "bg-zinc-500/15 text-zinc-500" }
                                                                        ),
                                                                        if dom_active { "Active" } else { "Paused" }
                                                                    }
                                                                }
                                                                div { class: "col-span-3 flex items-center justify-end gap-1.5",
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "h-7 px-2 rounded-md bg-muted hover:bg-muted/80 text-foreground text-[11px] font-medium cursor-pointer inline-flex items-center gap-1",
                                                                        title: "Edit domain",
                                                                        onclick: {
                                                                            let ds = dom_str.clone();
                                                                            let dd = dom_desc.clone();
                                                                            move |_| {
                                                                                edit_domain_val.set(ds.clone());
                                                                                edit_domain_desc.set(dd.clone());
                                                                                editing_domain_id.set(Some(dom_id));
                                                                            }
                                                                        },
                                                                        PencilIcon { class: "size-3" }
                                                                    }
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "h-7 px-2.5 rounded-md bg-muted hover:bg-muted/80 text-muted-foreground text-[11px] font-medium cursor-pointer",
                                                                        onclick: move |_| {
                                                                            match crate::auth::toggle_email_domain_active(dom_id) {
                                                                                Ok(_) => domain_revision.with_mut(|r| *r += 1),
                                                                                Err(err) => domain_msg.set(Some((false, err))),
                                                                            }
                                                                        },
                                                                        if dom_active { "Pause" } else { "Enable" }
                                                                    }
                                                                    button {
                                                                        r#type: "button",
                                                                        class: "h-7 px-2 rounded-md bg-muted hover:bg-destructive/15 text-muted-foreground hover:text-destructive text-[11px] cursor-pointer inline-flex items-center",
                                                                        title: "Delete domain",
                                                                        onclick: move |_| {
                                                                            match crate::auth::delete_email_domain(dom_id) {
                                                                                Ok(_) => {
                                                                                    domain_revision.with_mut(|r| *r += 1);
                                                                                    domain_msg.set(Some((true, "Domain removed.".to_string())));
                                                                                }
                                                                                Err(err) => domain_msg.set(Some((false, err))),
                                                                            }
                                                                        },
                                                                        TrashIcon { class: "size-3" }
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

                        // ====================================================
                        
                        // TAB 6: Google Sheets & System Administration (Super Admin)
                        // ====================================================
                        if cur_tab == "sheets" && is_super_admin {
                            div { class: "space-y-6",

                                // --------------------------------------------
                                // Card 1: Google Sheets Connection & Credentials
                                // --------------------------------------------
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold flex items-center gap-2",
                                                UploadCloudIcon { class: "size-4 text-primary" }
                                                "Google Sheets Web App Connection"
                                            }
                                            CardDescription { class: "text-xs",
                                                "Configure Google Apps Script endpoint deployment URL and API authentication token"
                                            }
                                        }
                                        span {
                                            class: format!(
                                                "inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-xs font-semibold border shrink-0 {}",
                                                if !sheet_url_input.read().trim().is_empty() {
                                                    "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30"
                                                } else {
                                                    "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30"
                                                }
                                            ),
                                            if !sheet_url_input.read().trim().is_empty() {
                                                CheckCircle2Icon { class: "size-3.5" }
                                            } else {
                                                AlertTriangleIcon { class: "size-3.5" }
                                            }
                                            if !sheet_url_input.read().trim().is_empty() {
                                                "Connected to Cloud Sheet"
                                            } else {
                                                "Not Configured (Local Offline)"
                                            }
                                        }
                                    }

                                    CardContent { class: "p-6 space-y-5",
                                        if let Some((ok, msg)) = sheet_ctrl_msg.read().as_ref() {
                                            div {
                                                class: format!(
                                                    "flex items-center gap-2.5 p-3 rounded-lg text-xs border animate-in slide-in-from-top-1 {}",
                                                    if *ok { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                                ),
                                                if *ok { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                                span { class: "font-medium", "{msg}" }
                                            }
                                        }

                                        // Connection Fields
                                        div { class: "space-y-4",
                                            // Web App URL
                                            div { class: "space-y-1.5",
                                                Label { r#for: "gs_url", class: "text-xs font-semibold flex items-center justify-between",
                                                    span { "Google Apps Script Web App URL *" }
                                                    span { class: "text-[10px] text-muted-foreground font-mono font-normal", "Must end with /exec" }
                                                }
                                                Input {
                                                    id: "gs_url",
                                                    class: "text-xs h-9 bg-background font-mono focus:ring-1 focus:ring-primary",
                                                    placeholder: "https://script.google.com/macros/s/.../exec".to_string(),
                                                    value: sheet_url_input.read().clone(),
                                                    oninput: move |e: FormEvent| sheet_url_input.set(e.value()),
                                                }
                                                p { class: "text-[11px] text-muted-foreground",
                                                    "Deploy your Google Sheet script as a Web App with access set to 'Anyone' or use your corporate gateway."
                                                }
                                            }

                                            // API Key & Profile Name Grid
                                            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                                                div { class: "space-y-1.5",
                                                    Label { r#for: "gs_key", class: "text-xs font-semibold flex items-center justify-between",
                                                        span { "API Secret Key (Optional)" }
                                                        button {
                                                            r#type: "button",
                                                            class: "text-[11px] text-muted-foreground hover:text-foreground cursor-pointer flex items-center gap-1",
                                                            onclick: move |_| {
                                                                let cur = show_sheet_key();
                                                                show_sheet_key.set(!cur);
                                                            },
                                                            if *show_sheet_key.read() {
                                                                EyeOffIcon { class: "size-3" }
                                                                "Hide"
                                                            } else {
                                                                EyeIcon { class: "size-3" }
                                                                "Show"
                                                            }
                                                        }
                                                    }
                                                    Input {
                                                        id: "gs_key",
                                                        r#type: if *show_sheet_key.read() { "text".to_string() } else { "password".to_string() },
                                                        class: "text-xs h-9 bg-background font-mono focus:ring-1 focus:ring-primary",
                                                        placeholder: "Enter authorization key if required".to_string(),
                                                        value: sheet_key_input.read().clone(),
                                                        oninput: move |e: FormEvent| sheet_key_input.set(e.value()),
                                                    }
                                                }

                                                div { class: "space-y-1.5",
                                                    Label { r#for: "gs_name", class: "text-xs font-semibold", "Profile Display Name" }
                                                    Input {
                                                        id: "gs_name",
                                                        class: "text-xs h-9 bg-background focus:ring-1 focus:ring-primary",
                                                        placeholder: "e.g. Primary Production Sheet".to_string(),
                                                        value: sheet_name_input.read().clone(),
                                                        oninput: move |e: FormEvent| sheet_name_input.set(e.value()),
                                                    }
                                                }
                                            }
                                        }

                                        // Test Status Banner
                                        if let Some((ok, msg, lat)) = sheet_test_status.read().as_ref() {
                                            div {
                                                class: format!(
                                                    "p-3 rounded-lg border text-xs flex items-center justify-between gap-3 {}",
                                                    if *ok { "bg-emerald-500/10 text-emerald-700 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                                ),
                                                div { class: "flex items-center gap-2",
                                                    if *ok { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                                    span { class: "font-medium", "{msg}" }
                                                }
                                                if *ok {
                                                    span { class: "font-mono font-bold px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-800 dark:text-emerald-300 text-[10px]",
                                                        "⚡ {lat}ms"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-3.5 flex flex-wrap items-center justify-between gap-3",
                                        div { class: "flex items-center gap-2",
                                            Button {
                                                variant: "outline".to_string(),
                                                size: "sm".to_string(),
                                                disabled: *is_testing_sheet.read(),
                                                class: "text-xs font-semibold cursor-pointer h-9 bg-background hover:bg-muted",
                                                onclick: move |_| handle_test_sheet(()),
                                                RefreshIcon { class: format!("size-3.5 mr-1.5 {}", if *is_testing_sheet.read() { "animate-spin" } else { "" }) }
                                                if *is_testing_sheet.read() { "Testing Connection..." } else { "Test Connection" }
                                            }
                                            if !sheet_url_input.read().trim().is_empty() {
                                                Button {
                                                    variant: "outline".to_string(),
                                                    size: "sm".to_string(),
                                                    class: "text-xs text-destructive hover:bg-destructive/10 border-destructive/30 cursor-pointer h-9",
                                                    onclick: move |_| handle_disconnect_sheet(()),
                                                    TrashIcon { class: "size-3.5 mr-1.5" }
                                                    "Disconnect Sheet"
                                                }
                                            }
                                        }
                                        Button {
                                            size: "sm".to_string(),
                                            class: "text-xs font-semibold shadow-xs cursor-pointer px-5 h-9 bg-primary text-primary-foreground hover:bg-primary/90",
                                            onclick: move |_| handle_save_sheet(()),
                                            CheckCircle2Icon { class: "size-3.5 mr-1.5" }
                                            "Save & Activate Sheet"
                                        }
                                    }
                                }

                                // --------------------------------------------
                                // Card 2: Spreadsheet Schema Health & Auto-Setup
                                // --------------------------------------------
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold flex items-center gap-2",
                                                DatabaseIcon { class: "size-4 text-primary" }
                                                "Spreadsheet Schema Health & Auto-Setup"
                                            }
                                            CardDescription { class: "text-xs",
                                                "Inspect required spreadsheet tabs (Tickets, Employees, Reminders, Notes) and column definitions"
                                            }
                                        }
                                        span { class: "text-xs font-mono text-muted-foreground", "4 Standard Tables" }
                                    }

                                    CardContent { class: "p-6 space-y-5",
                                        if let Some((ok, msg)) = schema_ctrl_msg.read().as_ref() {
                                            div {
                                                class: format!(
                                                    "flex items-center gap-2.5 p-3 rounded-lg text-xs border animate-in slide-in-from-top-1 {}",
                                                    if *ok { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                                ),
                                                if *ok { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                                span { class: "font-medium", "{msg}" }
                                            }
                                        }

                                        // 4 Tabs Overview Grid
                                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3.5 text-xs",
                                            // Tickets Tab Card
                                            div { class: "p-4 rounded-xl border border-border/70 bg-card space-y-2",
                                                div { class: "flex items-center justify-between",
                                                    span { class: "font-bold text-foreground flex items-center gap-1.5",
                                                        span { class: "size-2 rounded-full bg-blue-500" }
                                                        "Tickets Tab"
                                                    }
                                                    span { class: "text-[10px] font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground font-semibold",
                                                        "17 Columns"
                                                    }
                                                }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "id, title, description, priority, category, status, assignee, department, requester, resolution, created_at..."
                                                }
                                            }

                                            // Employees Tab Card
                                            div { class: "p-4 rounded-xl border border-border/70 bg-card space-y-2",
                                                div { class: "flex items-center justify-between",
                                                    span { class: "font-bold text-foreground flex items-center gap-1.5",
                                                        span { class: "size-2 rounded-full bg-emerald-500" }
                                                        "Employees Tab"
                                                    }
                                                    span { class: "text-[10px] font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground font-semibold",
                                                        "8 Columns"
                                                    }
                                                }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "id, name, department, email, phone, role, status, notes for technician and staff routing."
                                                }
                                            }

                                            // Reminders Tab Card
                                            div { class: "p-4 rounded-xl border border-border/70 bg-card space-y-2",
                                                div { class: "flex items-center justify-between",
                                                    span { class: "font-bold text-foreground flex items-center gap-1.5",
                                                        span { class: "size-2 rounded-full bg-amber-500" }
                                                        "Reminders Tab"
                                                    }
                                                    span { class: "text-[10px] font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground font-semibold",
                                                        "8 Columns"
                                                    }
                                                }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "id, ticket_id, title, remind_at, status, created_by, notes, priority for SLA & follow-up tracking."
                                                }
                                            }

                                            // Notes Tab Card
                                            div { class: "p-4 rounded-xl border border-border/70 bg-card space-y-2",
                                                div { class: "flex items-center justify-between",
                                                    span { class: "font-bold text-foreground flex items-center gap-1.5",
                                                        span { class: "size-2 rounded-full bg-violet-500" }
                                                        "Notes Tab"
                                                    }
                                                    span { class: "text-[10px] font-mono px-2 py-0.5 rounded bg-muted text-muted-foreground font-semibold",
                                                        "6 Columns"
                                                    }
                                                }
                                                p { class: "text-[11px] text-muted-foreground leading-relaxed",
                                                    "id, ticket_id, author, content, created_at, is_internal for quick technician scratchpads."
                                                }
                                            }
                                        }

                                        // Detailed report banner if checked
                                        if let Some(rep) = schema_report.read().as_ref() {
                                            div {
                                                class: format!(
                                                    "p-3.5 rounded-xl border text-xs space-y-2 {}",
                                                    if rep.valid { "bg-emerald-500/10 border-emerald-500/30 text-emerald-800 dark:text-emerald-300" } else { "bg-amber-500/10 border-amber-500/30 text-amber-800 dark:text-amber-300" }
                                                ),
                                                div { class: "flex items-center gap-2 font-semibold",
                                                    if rep.valid { CheckCircle2Icon { class: "size-4 text-emerald-600 dark:text-emerald-400" } } else { AlertTriangleIcon { class: "size-4 text-amber-600 dark:text-amber-400" } }
                                                    if rep.valid { "Remote Google Sheet Schema is Fully Valid" } else { "Remote Google Sheet Requires Structure Initialization" }
                                                }
                                                p { class: "text-[11px] opacity-90",
                                                    if rep.valid {
                                                        "All 4 spreadsheet tabs and corresponding standard column headers were found and verified."
                                                    } else {
                                                        "One or more tabs or header columns are missing. Click 'Auto-Initialize Schema' below to automatically build them in your spreadsheet."
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-3.5 flex items-center justify-between gap-3",
                                        Button {
                                            variant: "outline".to_string(),
                                            size: "sm".to_string(),
                                            disabled: *is_verifying_schema.read(),
                                            class: "text-xs font-semibold cursor-pointer h-9 bg-background hover:bg-muted",
                                            onclick: move |_| handle_verify_schema(()),
                                            RefreshIcon { class: format!("size-3.5 mr-1.5 {}", if *is_verifying_schema.read() { "animate-spin" } else { "" }) }
                                            if *is_verifying_schema.read() { "Verifying Remote Schema..." } else { "Verify Schema Health" }
                                        }
                                        Button {
                                            size: "sm".to_string(),
                                            disabled: *is_initializing_schema.read(),
                                            class: "text-xs font-semibold cursor-pointer h-9 bg-primary text-primary-foreground hover:bg-primary/90",
                                            onclick: move |_| handle_init_schema(()),
                                            PlusIcon { class: format!("size-3.5 mr-1.5 {}", if *is_initializing_schema.read() { "animate-spin" } else { "" }) }
                                            if *is_initializing_schema.read() { "Initializing Google Sheet..." } else { "Auto-Initialize Schema" }
                                        }
                                    }
                                }

                                // --------------------------------------------
                                // Card 3: Data Synchronization Engine
                                // --------------------------------------------
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold flex items-center gap-2",
                                                RefreshIcon { class: "size-4 text-primary" }
                                                "Data Synchronization Engine"
                                            }
                                            CardDescription { class: "text-xs",
                                                "Manage automated background cadence and trigger manual two-way sync pulls"
                                            }
                                        }
                                        span {
                                            class: format!(
                                                "px-2.5 py-1 rounded-full text-xs font-semibold border shrink-0 {}",
                                                if *auto_sync_enabled.read() { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-zinc-500/10 text-zinc-500 border-zinc-500/30" }
                                            ),
                                            if *auto_sync_enabled.read() { "Background Sync: Active" } else { "Background Sync: Paused" }
                                        }
                                    }

                                    CardContent { class: "p-6 space-y-5",
                                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4 items-center",
                                            // Toggle Switch
                                            div { class: "p-4 rounded-xl border border-border/70 bg-card space-y-2 flex items-center justify-between",
                                                div { class: "space-y-0.5",
                                                    p { class: "text-xs font-bold text-foreground", "Automatic Background Sync" }
                                                    p { class: "text-[11px] text-muted-foreground", "Periodically pull tickets and push outbox in background" }
                                                }
                                                button {
                                                    r#type: "button",
                                                    class: format!(
                                                        "relative inline-flex h-6 w-11 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none {}",
                                                        if *auto_sync_enabled.read() { "bg-primary" } else { "bg-muted" }
                                                    ),
                                                    onclick: move |_| handle_toggle_auto_sync(()),
                                                    span {
                                                        class: format!(
                                                            "pointer-events-none inline-block size-5 transform rounded-full bg-background shadow ring-0 transition duration-200 ease-in-out {}",
                                                            if *auto_sync_enabled.read() { "translate-x-5" } else { "translate-x-0" }
                                                        ),
                                                    }
                                                }
                                            }

                                            // Sync Cadence Selector
                                            div { class: "p-4 rounded-xl border border-border/70 bg-card space-y-2",
                                                div { class: "flex items-center justify-between",
                                                    p { class: "text-xs font-bold text-foreground", "Sync Cycle Interval" }
                                                    span { class: "text-[11px] font-mono text-primary font-bold", "{sync_interval_secs.read()}s" }
                                                }
                                                div { class: "flex items-center gap-1.5 flex-wrap",
                                                    for (sec, label) in [("15", "15s"), ("30", "30s"), ("60", "60s"), ("120", "2m"), ("300", "5m")] {
                                                        {
                                                            let sec_str = sec.to_string();
                                                            let is_selected = *sync_interval_secs.read() == sec_str;
                                                            rsx! {
                                                                button {
                                                                    key: "{sec}",
                                                                    r#type: "button",
                                                                    class: format!(
                                                                        "px-2.5 py-1 rounded text-xs font-semibold cursor-pointer border transition-colors {}",
                                                                        if is_selected { "bg-primary text-primary-foreground border-primary" } else { "bg-muted/60 text-muted-foreground border-border/60 hover:bg-muted" }
                                                                    ),
                                                                    onclick: move |_| handle_change_sync_cadence(sec_str.clone()),
                                                                    "{label}"
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-3.5 flex items-center justify-between gap-3",
                                        Button {
                                            variant: "outline".to_string(),
                                            size: "sm".to_string(),
                                            class: "text-xs text-muted-foreground hover:text-foreground cursor-pointer h-9",
                                            onclick: move |_| handle_clear_sync_queue(()),
                                            TrashIcon { class: "size-3.5 mr-1.5" }
                                            "Clear Sync Queue"
                                        }
                                        Button {
                                            size: "sm".to_string(),
                                            disabled: *is_pulling_now.read(),
                                            class: "text-xs font-semibold cursor-pointer h-9 bg-primary text-primary-foreground hover:bg-primary/90",
                                            onclick: move |_| handle_force_pull_now(()),
                                            RefreshIcon { class: format!("size-3.5 mr-1.5 {}", if *is_pulling_now.read() { "animate-spin" } else { "" }) }
                                            if *is_pulling_now.read() { "Pulling Data..." } else { "Force Sync Now (Pull Tickets & Staff)" }
                                        }
                                    }
                                }

                                // --------------------------------------------
                                // Card 4: SQLite Database Administration
                                // --------------------------------------------
                                Card { class: "border-border/80 shadow-xs overflow-hidden",
                                    CardHeader { class: "border-b border-border/50 bg-muted/15 pb-4 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3",
                                        div { class: "space-y-1",
                                            CardTitle { class: "text-base font-semibold flex items-center gap-2",
                                                DatabaseIcon { class: "size-4 text-primary" }
                                                "SQLite Database & Local Storage"
                                            }
                                            CardDescription { class: "text-xs",
                                                "Monitor local database records and optimize disk storage"
                                            }
                                        }
                                        span { class: "inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-md text-[11px] font-mono bg-muted text-muted-foreground border border-border/60",
                                            "WAL Mode • Thread-Safe"
                                        }
                                    }

                                    CardContent { class: "p-6 space-y-5",
                                        if let Some((ok, msg)) = maintenance_msg.read().as_ref() {
                                            div {
                                                class: format!(
                                                    "flex items-center gap-2.5 p-3 rounded-lg text-xs border animate-in slide-in-from-top-1 {}",
                                                    if *ok { "bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border-emerald-500/30" } else { "bg-destructive/10 text-destructive border-destructive/30" }
                                                ),
                                                if *ok { CheckCircle2Icon { class: "size-4 shrink-0" } } else { AlertCircleIcon { class: "size-4 shrink-0" } }
                                                span { class: "font-medium", "{msg}" }
                                            }
                                        }

                                        // Metrics Grid
                                        div { class: "grid grid-cols-2 sm:grid-cols-6 gap-3",
                                            div { class: "p-3 rounded-xl border border-border/70 bg-card space-y-1",
                                                p { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Tickets" }
                                                p { class: "text-xl font-bold text-foreground", "{db_stats().0}" }
                                            }
                                            div { class: "p-3 rounded-xl border border-border/70 bg-card space-y-1",
                                                p { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Employees" }
                                                p { class: "text-xl font-bold text-foreground", "{db_stats().1}" }
                                            }
                                            div { class: "p-3 rounded-xl border border-border/70 bg-card space-y-1",
                                                p { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Reminders" }
                                                p { class: "text-xl font-bold text-foreground", "{db_stats().2}" }
                                            }
                                            div { class: "p-3 rounded-xl border border-border/70 bg-card space-y-1",
                                                p { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Notes" }
                                                p { class: "text-xl font-bold text-foreground", "{db_stats().3}" }
                                            }
                                            div { class: "p-3 rounded-xl border border-border/70 bg-card space-y-1",
                                                p { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Users" }
                                                p { class: "text-xl font-bold text-foreground", "{db_stats().4}" }
                                            }
                                            div { class: "p-3 rounded-xl border border-border/70 bg-card space-y-1",
                                                p { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider", "Sync Outbox" }
                                                p {
                                                    class: format!(
                                                        "text-xl font-bold {}",
                                                        if db_stats().5 > 0 { "text-amber-600 dark:text-amber-400" } else { "text-emerald-600 dark:text-emerald-400" }
                                                    ),
                                                    "{db_stats().5}"
                                                }
                                            }
                                        }
                                    }

                                    CardFooter { class: "border-t border-border/60 bg-muted/15 px-6 py-3.5 flex items-center justify-between gap-3",
                                        p { class: "text-[11px] text-muted-foreground",
                                            "Vacuuming defragments table pages, shrinks file size, and flushes journal checkpoints."
                                        }
                                        Button {
                                            variant: "outline".to_string(),
                                            size: "sm".to_string(),
                                            disabled: *is_vacuuming.read(),
                                            class: "text-xs font-semibold cursor-pointer h-9 bg-background hover:bg-muted",
                                            onclick: move |_| handle_vacuum_db(()),
                                            DatabaseIcon { class: format!("size-3.5 mr-1.5 {}", if *is_vacuuming.read() { "animate-spin" } else { "" }) }
                                            if *is_vacuuming.read() { "Optimizing Database..." } else { "Optimize & Vacuum Database" }
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
