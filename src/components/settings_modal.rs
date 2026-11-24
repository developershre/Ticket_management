//! Advanced Settings Panel for BPSL IT Support.
//!
//! Multi-tab settings dialog with:
//! - **Appearance & Theme**: Theme mode, OLED pure black, accent colors, typography, corner radius, density
//! - **Tickets & Workflow**: Default landing page, default view, priority, category, assignee, kanban filters
//! - **Alerts & Audio**: Sound chimes, sound volume, toast duration, reminder pre-alerts, sync alerts
//! - **API & Connection**: Base URL, protocol, API Key, connection testing, multi-sheet sources
//! - **Sync & Data**: Sync cadence, queue diagnostics, JSON export backup, SQLite optimize/vacuum
//! - **About & System**: App identity, database file stats, system diagnostics, keyboard shortcuts
//!
//! All preferences are persisted in SQLite `sync_meta` and survive app restarts.

use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::icons::{
    AlertCircleIcon, BellIcon, CheckIcon, ExternalLinkIcon, EyeIcon, KeyIcon, RefreshIcon, SettingsIcon,
    TrashIcon, XIcon,
};

// ============================================================
// Settings Tab Enum
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SettingsTab {
    Appearance,
    Workflow,
    Notifications,
    Api,
    SyncData,
    About,
}

impl SettingsTab {
    fn label(&self) -> &'static str {
        match self {
            Self::Appearance => "Appearance & Theme",
            Self::Workflow => "Tickets & Workflow",
            Self::Notifications => "Alerts & Audio",
            Self::Api => "API & Connection",
            Self::SyncData => "Sync & Data",
            Self::About => "About & System",
        }
    }

    fn icon_path(&self) -> &'static str {
        match self {
            Self::Appearance => "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707 M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8z",
            Self::Workflow => "M4 6h16M4 12h16M4 18h7",
            Self::Notifications => "M18 8A6 6 0 0 0 6 8c0 7-3 9-3 9h18s-3-2-3-9 M13.73 21a2 2 0 0 1-3.46 0",
            Self::Api => "M15 7h3a5 5 0 0 1 0 10h-3m-6 0H6a5 5 0 0 1 0-10h3 M8 12h8",
            Self::SyncData => "M21 2v6h-6 M3 12a9 9 0 0 1 15-6.7L21 8 M3 22v-6h6 M21 12a9 9 0 0 1-15 6.7L3 16",
            Self::About => "M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10z M12 16v-4 M12 8h.01",
        }
    }
}

const TABS: [SettingsTab; 6] = [
    SettingsTab::Appearance,
    SettingsTab::Workflow,
    SettingsTab::Notifications,
    SettingsTab::Api,
    SettingsTab::SyncData,
    SettingsTab::About,
];

// ============================================================
// Accent Colors Definition
// ============================================================

struct AccentOption {
    id: &'static str,
    name: &'static str,
    color_hex: &'static str,
}

const ACCENT_OPTIONS: [AccentOption; 11] = [
    AccentOption { id: "default", name: "Monochrome", color_hex: "#71717a" },
    AccentOption { id: "blue", name: "Ocean Blue", color_hex: "#3b82f6" },
    AccentOption { id: "indigo", name: "Indigo Deep", color_hex: "#6366f1" },
    AccentOption { id: "emerald", name: "Emerald Green", color_hex: "#10b981" },
    AccentOption { id: "teal", name: "Mint Teal", color_hex: "#14b8a6" },
    AccentOption { id: "violet", name: "Royal Purple", color_hex: "#8b5cf6" },
    AccentOption { id: "amber", name: "Amber Gold", color_hex: "#f59e0b" },
    AccentOption { id: "orange", name: "Sunset Orange", color_hex: "#f97316" },
    AccentOption { id: "rose", name: "Crimson Rose", color_hex: "#f43f5e" },
    AccentOption { id: "cyan", name: "Cyber Cyan", color_hex: "#06b6d4" },
    AccentOption { id: "slate", name: "Slate Steel", color_hex: "#64748b" },
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
    let app_settings = try_use_context::<Signal<crate::state::AppSettings>>();
    let db_sync = try_use_context::<Signal<crate::state::AppDbRevision>>();
    let open_changes = try_use_context::<Signal<crate::state::OpenSheetChangesDialog>>();

    // Active tab
    let mut active_tab = use_signal(|| SettingsTab::Appearance);

    // Instant search filter across all settings
    let mut search_query = use_signal(String::new);

    // ========================================================
    // TAB 1: Appearance State
    // ========================================================
    let initial_sidebar_pos = crate::db::sync_meta_get("ui_sidebar_position").unwrap_or_else(|| "left".to_string());
    let mut sidebar_position = use_signal(|| initial_sidebar_pos);

    let initial_density = crate::db::sync_meta_get("ui_card_density").unwrap_or_else(|| "default".to_string());
    let mut card_density = use_signal(|| initial_density);

    let initial_animations = crate::db::sync_meta_get("ui_animations_enabled").unwrap_or_else(|| "true".to_string());
    let mut animations_enabled = use_signal(|| initial_animations == "true");

    let initial_date_fmt = crate::db::sync_meta_get("ui_date_format").unwrap_or_else(|| "12hr".to_string());
    let mut date_format = use_signal(|| initial_date_fmt);

    let initial_date_pat = crate::db::sync_meta_get("ui_date_pattern").unwrap_or_else(|| "medium".to_string());
    let mut date_pattern = use_signal(|| initial_date_pat);

    let initial_high_contrast = crate::db::sync_meta_get("ui_high_contrast").map(|v| v == "true").unwrap_or(false);
    let mut high_contrast = use_signal(|| initial_high_contrast);

    let initial_glass = crate::db::sync_meta_get("ui_glassmorphism").map(|v| v != "false").unwrap_or(true);
    let mut glassmorphism = use_signal(|| initial_glass);

    let initial_font_scale = crate::db::sync_meta_get("ui_font_scale").unwrap_or_else(|| "auto".to_string());
    let mut font_scale = use_signal(|| initial_font_scale);

    let initial_font_fam = crate::db::sync_meta_get("ui_font_family").unwrap_or_else(|| "roboto".to_string());
    let mut font_family = use_signal(|| initial_font_fam);

    let initial_accent = crate::db::sync_meta_get("ui_accent_color").unwrap_or_else(|| "default".to_string());
    let mut accent_color = use_signal(|| initial_accent);

    let initial_radius = crate::db::sync_meta_get("ui_corner_radius").unwrap_or_else(|| "rounded".to_string());
    let mut corner_radius = use_signal(|| initial_radius);

    let initial_oled = crate::db::sync_meta_get("ui_oled_dark").map(|v| v == "true").unwrap_or(false);
    let mut oled_dark = use_signal(|| initial_oled);

    // ========================================================
    // TAB 2: Workflow & Tickets State
    // ========================================================
    let initial_landing = crate::db::sync_meta_get("ui_default_landing_page").unwrap_or_else(|| "tickets".to_string());
    let mut default_landing = use_signal(|| initial_landing);

    let initial_ticket_view = crate::db::sync_meta_get("ui_default_ticket_view").unwrap_or_else(|| "kanban".to_string());
    let mut default_ticket_view = use_signal(|| initial_ticket_view);

    let initial_priority = crate::db::sync_meta_get("ui_default_priority").unwrap_or_else(|| "Medium".to_string());
    let mut default_priority = use_signal(|| initial_priority);

    let initial_category = crate::db::sync_meta_get("ui_default_category").unwrap_or_else(|| "General".to_string());
    let mut default_category = use_signal(|| initial_category);

    let initial_assignee = crate::db::sync_meta_get("ui_default_assignee").unwrap_or_else(|| "IT Support".to_string());
    let mut default_assignee = use_signal(|| initial_assignee);

    let initial_dept = crate::db::sync_meta_get("ui_default_department").unwrap_or_else(|| "General".to_string());
    let mut default_department = use_signal(|| initial_dept);

    let initial_page_size = crate::db::sync_meta_get("ui_table_page_size").unwrap_or_else(|| "25".to_string());
    let mut table_page_size = use_signal(|| initial_page_size);

    let initial_hide_cancelled = crate::db::sync_meta_get("ui_kanban_hide_cancelled").map(|v| v == "true").unwrap_or(false);
    let mut kanban_hide_cancelled = use_signal(|| initial_hide_cancelled);

    let initial_hide_solved = crate::db::sync_meta_get("ui_kanban_hide_solved").map(|v| v == "true").unwrap_or(false);
    let mut kanban_hide_solved = use_signal(|| initial_hide_solved);

    let initial_kanban_compact = crate::db::sync_meta_get("ui_kanban_compact_cards").map(|v| v == "true").unwrap_or(false);
    let mut kanban_compact = use_signal(|| initial_kanban_compact);

    let initial_confirm_del = crate::db::sync_meta_get("ui_confirm_deletions").map(|v| v != "false").unwrap_or(true);
    let mut confirm_deletions = use_signal(|| initial_confirm_del);

    let initial_focus_refresh = crate::db::sync_meta_get("ui_focus_refresh").map(|v| v == "true").unwrap_or(false);
    let mut focus_refresh = use_signal(|| initial_focus_refresh);

    // ========================================================
    // TAB 3: Alerts & Audio State
    // ========================================================
    let initial_sound_on = crate::db::sync_meta_get("ui_sound_enabled").map(|v| v != "false").unwrap_or(true);
    let mut sound_enabled = use_signal(|| initial_sound_on);

    let initial_tone = crate::db::sync_meta_get("ui_chime_tone").unwrap_or_else(|| "arpeggio".to_string());
    let mut chime_tone = use_signal(|| initial_tone);

    let initial_sound_vol = crate::db::sync_meta_get("ui_sound_volume").unwrap_or_else(|| "medium".to_string());
    let mut sound_volume = use_signal(|| initial_sound_vol);

    let initial_dnd = crate::db::sync_meta_get("ui_dnd_silent_mode").map(|v| v == "true").unwrap_or(false);
    let mut dnd_mode = use_signal(|| initial_dnd);

    let initial_toast_dur = crate::db::sync_meta_get("ui_toast_duration_secs").unwrap_or_else(|| "5".to_string());
    let mut toast_duration = use_signal(|| initial_toast_dur);

    let initial_rem_adv = crate::db::sync_meta_get("ui_reminder_advance_mins").unwrap_or_else(|| "15".to_string());
    let mut reminder_advance = use_signal(|| initial_rem_adv);

    let initial_sla_warn = crate::db::sync_meta_get("ui_sla_warning_hours").unwrap_or_else(|| "2".to_string());
    let mut sla_warning_hours = use_signal(|| initial_sla_warn);

    let initial_notify_sync = crate::db::sync_meta_get("sync_notify_on_pull").unwrap_or_else(|| "false".to_string());
    let mut notify_on_pull = use_signal(|| initial_notify_sync == "true");

    let initial_win_notif = crate::db::sync_meta_get("ui_windows_notifications").map(|v| v != "false").unwrap_or(true);
    let mut windows_notifications = use_signal(|| initial_win_notif);

    // ========================================================
    // TAB 4: API & Connection State
    // ========================================================
    let all_init_sources = crate::db::get_all_sheet_sources();
    let initial_active_id = crate::db::get_active_sheet_source_id();
    let initial_active_src = all_init_sources
        .iter()
        .find(|s| s.id == initial_active_id)
        .cloned()
        .unwrap_or_else(|| all_init_sources[0].clone());

    let mut sheet_sources = use_signal(move || all_init_sources);
    let active_profile = use_signal(move || initial_active_src.id.clone());
    let mut current_profile_name = use_signal(move || initial_active_src.name.clone());
    let mut base_url = use_signal(move || initial_active_src.base_url.clone());
    let mut api_key = use_signal(move || initial_active_src.api_key.clone());
    let mut api_protocol = use_signal(move || initial_active_src.protocol.clone());

    let initial_emp = crate::db::sync_meta_get("system_employee_code").unwrap_or_default();
    let mut employee_code = use_signal(|| initial_emp);
    let mut show_key = use_signal(|| false);
    let mut test_status = use_signal(|| TestStatus::Idle);
    let mut is_saving = use_signal(|| false);

    let initial_timeout = crate::db::sync_meta_get("sync_network_timeout_secs").unwrap_or_else(|| "25".to_string());
    let mut network_timeout = use_signal(|| initial_timeout);

    let initial_offline = crate::db::sync_meta_get("sync_offline_mode").map(|v| v == "true").unwrap_or(false);
    let mut work_offline = use_signal(|| initial_offline);

    let mut schema_report = use_signal(|| Option::<crate::sheet_crud::SchemaVerificationReport>::None);
    let mut is_verifying_schema = use_signal(|| false);
    let mut is_initializing_schema = use_signal(|| false);

    // ========================================================
    // TAB 5: Sync & Data State
    // ========================================================
    let initial_sync_interval = crate::db::sync_meta_get("sync_cycle_secs").unwrap_or_else(|| "25".to_string());
    let mut sync_interval_str = use_signal(|| initial_sync_interval);

    let initial_auto_sync = crate::db::sync_meta_get("sync_auto_enabled").unwrap_or_else(|| "true".to_string());
    let mut auto_sync_enabled = use_signal(|| initial_auto_sync == "true");

    // ========================================================
    // Handlers
    // ========================================================

    let handle_close = move |_| {
        on_close.call(());
    };

    // Save Appearance
    let save_appearance = move |_| {
        let pos = sidebar_position.read().clone();
        let density = card_density.read().clone();
        let anim_val = *animations_enabled.read();
        let fmt = date_format.read().clone();
        let pat = date_pattern.read().clone();
        let hc = *high_contrast.read();
        let gm = *glassmorphism.read();
        let scale = font_scale.read().clone();
        let fam = font_family.read().clone();
        let acc = accent_color.read().clone();
        let rad = corner_radius.read().clone();
        let oled = *oled_dark.read();

        let _ = crate::db::sync_meta_set("ui_sidebar_position", &pos);
        let _ = crate::db::sync_meta_set("ui_card_density", &density);
        let _ = crate::db::sync_meta_set("ui_animations_enabled", if anim_val { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_date_format", &fmt);
        let _ = crate::db::sync_meta_set("ui_date_pattern", &pat);
        let _ = crate::db::sync_meta_set("ui_high_contrast", if hc { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_glassmorphism", if gm { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_font_scale", &scale);
        let _ = crate::db::sync_meta_set("ui_font_family", &fam);
        let _ = crate::db::sync_meta_set("ui_accent_color", &acc);
        let _ = crate::db::sync_meta_set("ui_corner_radius", &rad);
        let _ = crate::db::sync_meta_set("ui_oled_dark", if oled { "true" } else { "false" });

        if let Some(mut s) = app_settings {
            s.with_mut(|settings| {
                settings.sidebar_position = pos.clone();
                settings.card_density = density.clone();
                settings.animations_enabled = anim_val;
                settings.date_format = fmt.clone();
                settings.font_scale = scale.clone();
                settings.font_family = fam.clone();
                settings.accent_color = acc.clone();
                settings.corner_radius = rad.clone();
                settings.oled_dark = oled;
            });
        }
        crate::state::set_time_format_24hr(fmt == "24hr");
        crate::state::set_date_display_style(&pat);
        crate::state::apply_appearance_to_dom(&acc, &scale, &density, &rad, &fam, oled, anim_val);
        crate::state::play_sound("success");

        if let Some(mut r) = db_sync {
            r.with_mut(|rev| rev.0 += 1);
        }
        toast.success("Appearance Saved", "Theme, accent color, typography, and styling updated.");
    };

    // Reset Appearance to Defaults
    let reset_appearance = move |_| {
        sidebar_position.set("left".to_string());
        card_density.set("default".to_string());
        animations_enabled.set(true);
        date_format.set("12hr".to_string());
        date_pattern.set("medium".to_string());
        high_contrast.set(false);
        glassmorphism.set(true);
        font_scale.set("auto".to_string());
        font_family.set("roboto".to_string());
        accent_color.set("default".to_string());
        corner_radius.set("rounded".to_string());
        oled_dark.set(false);

        let _ = crate::db::sync_meta_set("ui_sidebar_position", "left");
        let _ = crate::db::sync_meta_set("ui_card_density", "default");
        let _ = crate::db::sync_meta_set("ui_animations_enabled", "true");
        let _ = crate::db::sync_meta_set("ui_date_format", "12hr");
        let _ = crate::db::sync_meta_set("ui_date_pattern", "medium");
        let _ = crate::db::sync_meta_set("ui_high_contrast", "false");
        let _ = crate::db::sync_meta_set("ui_glassmorphism", "true");
        let _ = crate::db::sync_meta_set("ui_font_scale", "auto");
        let _ = crate::db::sync_meta_set("ui_font_family", "roboto");
        let _ = crate::db::sync_meta_set("ui_accent_color", "default");
        let _ = crate::db::sync_meta_set("ui_corner_radius", "rounded");
        let _ = crate::db::sync_meta_set("ui_oled_dark", "false");

        crate::state::set_time_format_24hr(false);
        crate::state::set_date_display_style("medium");
        crate::state::apply_appearance_to_dom("default", "auto", "default", "rounded", "roboto", false, true);
        crate::state::play_sound("click");
        if let Some(mut r) = db_sync {
            r.with_mut(|rev| rev.0 += 1);
        }
        toast.info("Defaults Restored", "Appearance settings reset to standard system defaults.");
    };

    // Save Workflow
    let save_workflow = move |_| {
        let landing = default_landing.read().clone();
        let view = default_ticket_view.read().clone();
        let prio = default_priority.read().clone();
        let cat = default_category.read().clone();
        let ass = default_assignee.read().clone();
        let dept = default_department.read().clone();
        let p_size = table_page_size.read().clone();
        let hide_canc = *kanban_hide_cancelled.read();
        let hide_solv = *kanban_hide_solved.read();
        let k_compact = *kanban_compact.read();
        let conf_del = *confirm_deletions.read();
        let f_refresh = *focus_refresh.read();

        let _ = crate::db::sync_meta_set("ui_default_landing_page", &landing);
        let _ = crate::db::sync_meta_set("ui_default_ticket_view", &view);
        let _ = crate::db::sync_meta_set("ui_default_priority", &prio);
        let _ = crate::db::sync_meta_set("ui_default_category", &cat);
        let _ = crate::db::sync_meta_set("ui_default_assignee", &ass);
        let _ = crate::db::sync_meta_set("ui_default_department", &dept);
        let _ = crate::db::sync_meta_set("ui_table_page_size", &p_size);
        let _ = crate::db::sync_meta_set("ui_kanban_hide_cancelled", if hide_canc { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_kanban_hide_solved", if hide_solv { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_kanban_compact_cards", if k_compact { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_confirm_deletions", if conf_del { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_focus_refresh", if f_refresh { "true" } else { "false" });

        if let Some(mut s) = app_settings {
            s.with_mut(|settings| {
                settings.default_landing_page = landing;
                settings.default_ticket_view = view;
                settings.default_priority = prio;
                settings.default_category = cat;
                settings.default_assignee = ass;
                settings.kanban_hide_cancelled = hide_canc;
                settings.kanban_hide_solved = hide_solv;
            });
        }
        crate::state::play_sound("success");
        if let Some(mut r) = db_sync {
            r.with_mut(|rev| rev.0 += 1);
        }
        toast.success("Workflow Saved", "Ticket defaults and Kanban column preferences saved.");
    };

    // Reset Workflow to Defaults
    let reset_workflow = move |_: MouseEvent| {
        default_landing.set("tickets".to_string());
        default_ticket_view.set("kanban".to_string());
        default_priority.set("Medium".to_string());
        default_category.set("General".to_string());
        default_assignee.set("IT Support".to_string());
        default_department.set("General".to_string());
        table_page_size.set("25".to_string());
        kanban_hide_cancelled.set(false);
        kanban_hide_solved.set(false);
        kanban_compact.set(false);
        confirm_deletions.set(true);
        focus_refresh.set(false);

        let _ = crate::db::sync_meta_set("ui_default_landing_page", "tickets");
        let _ = crate::db::sync_meta_set("ui_default_ticket_view", "kanban");
        let _ = crate::db::sync_meta_set("ui_default_priority", "Medium");
        let _ = crate::db::sync_meta_set("ui_default_category", "General");
        let _ = crate::db::sync_meta_set("ui_default_assignee", "IT Support");
        let _ = crate::db::sync_meta_set("ui_default_department", "General");
        let _ = crate::db::sync_meta_set("ui_table_page_size", "25");
        let _ = crate::db::sync_meta_set("ui_kanban_hide_cancelled", "false");
        let _ = crate::db::sync_meta_set("ui_kanban_hide_solved", "false");
        let _ = crate::db::sync_meta_set("ui_kanban_compact_cards", "false");
        let _ = crate::db::sync_meta_set("ui_confirm_deletions", "true");
        let _ = crate::db::sync_meta_set("ui_focus_refresh", "false");

        crate::state::play_sound("click");
        if let Some(mut r) = db_sync {
            r.with_mut(|rev| rev.0 += 1);
        }
        toast.info("Defaults Restored", "Workflow and ticket settings reset to defaults.");
    };

    // Save Notifications & Audio
    let save_notifications = move |_: MouseEvent| {
        let snd_on = *sound_enabled.read();
        let tone = chime_tone.read().clone();
        let snd_vol = sound_volume.read().clone();
        let dnd = *dnd_mode.read();
        let dur_str = toast_duration.read().clone();
        let dur_val: u32 = dur_str.parse().unwrap_or(5);
        let adv_str = reminder_advance.read().clone();
        let adv_val: u32 = adv_str.parse().unwrap_or(15);
        let sla_str = sla_warning_hours.read().clone();
        let notif_pull = *notify_on_pull.read();

        let _ = crate::db::sync_meta_set("ui_sound_enabled", if snd_on && !dnd { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_chime_tone", &tone);
        let _ = crate::db::sync_meta_set("ui_sound_volume", &snd_vol);
        let _ = crate::db::sync_meta_set("ui_dnd_silent_mode", if dnd { "true" } else { "false" });
        let _ = crate::db::sync_meta_set("ui_toast_duration_secs", &dur_val.to_string());
        let _ = crate::db::sync_meta_set("ui_reminder_advance_mins", &adv_val.to_string());
        let _ = crate::db::sync_meta_set("ui_sla_warning_hours", &sla_str);
        let _ = crate::db::sync_meta_set("sync_notify_on_pull", if notif_pull { "true" } else { "false" });
        let win_notif = *windows_notifications.read();
        let _ = crate::db::sync_meta_set("ui_windows_notifications", if win_notif { "true" } else { "false" });

        crate::state::set_sound_enabled(snd_on && !dnd);
        crate::state::set_sound_volume(&snd_vol);

        if let Some(mut s) = app_settings {
            s.with_mut(|settings| {
                settings.sound_enabled = snd_on && !dnd;
                settings.sound_volume = snd_vol;
                settings.toast_duration_secs = dur_val;
                settings.reminder_advance_mins = adv_val;
                settings.notify_on_remote_pull = notif_pull;
                settings.windows_notifications = win_notif;
            });
        }
        crate::state::play_sound(&tone);
        toast.success("Alerts Saved", "Audio chime, notification duration, and reminder preferences updated.");
    };

    // Reset Notifications
    let reset_notifications = move |_: MouseEvent| {
        sound_enabled.set(true);
        chime_tone.set("arpeggio".to_string());
        sound_volume.set("medium".to_string());
        dnd_mode.set(false);
        toast_duration.set("5".to_string());
        reminder_advance.set("15".to_string());
        sla_warning_hours.set("2".to_string());
        notify_on_pull.set(false);
        windows_notifications.set(true);

        let _ = crate::db::sync_meta_set("ui_sound_enabled", "true");
        let _ = crate::db::sync_meta_set("ui_chime_tone", "arpeggio");
        let _ = crate::db::sync_meta_set("ui_sound_volume", "medium");
        let _ = crate::db::sync_meta_set("ui_dnd_silent_mode", "false");
        let _ = crate::db::sync_meta_set("ui_toast_duration_secs", "5");
        let _ = crate::db::sync_meta_set("ui_reminder_advance_mins", "15");
        let _ = crate::db::sync_meta_set("ui_sla_warning_hours", "2");
        let _ = crate::db::sync_meta_set("sync_notify_on_pull", "false");
        let _ = crate::db::sync_meta_set("ui_windows_notifications", "true");

        crate::state::set_sound_enabled(true);
        crate::state::set_sound_volume("medium");
        crate::state::play_sound("click");
        toast.info("Defaults Restored", "Alert and audio preferences restored to defaults.");
    };

    // Play test chime
    let handle_test_sound = move |_: MouseEvent| {
        let snd_vol = sound_volume.read().clone();
        let tone = chime_tone.read().clone();
        crate::state::set_sound_volume(&snd_vol);
        crate::state::set_sound_enabled(true);
        crate::state::play_sound(&tone);
        toast.info("Audio Test", format!("Played '{tone}' tone preview."));
    };

    // Export Tickets CSV
    let handle_export_tickets_csv = move |_: MouseEvent| {
        let tickets = crate::db::get_all_tickets();
        let mut csv = String::from("Ticket ID,Employee Code,Email,Category,Subject,Priority,Status,Assigned To,Customer,Time Spent,Created On\n");
        for t in &tickets {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                t.ticket_id,
                t.employee_code.replace('"', "\"\""),
                t.email_address.replace('"', "\"\""),
                t.category.replace('"', "\"\""),
                t.subject.replace('"', "\"\""),
                t.priority.replace('"', "\"\""),
                t.status.replace('"', "\"\""),
                t.assigned_to.replace('"', "\"\""),
                t.customer.replace('"', "\"\""),
                t.time_spent,
                t.created_on.replace('"', "\"\""),
            ));
        }
        let escaped = csv.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            r#"
            (function() {{
                try {{
                    const blob = new Blob([`{escaped}`], {{ type: 'text/csv;charset=utf-8;' }});
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'bpsl_tickets_' + new Date().toISOString().slice(0,10) + '.csv';
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                }} catch(e) {{ console.error('CSV export failed', e); }}
            }})();
            "#
        );
        let _ = document::eval(&js);
        crate::state::play_sound("success");
        toast.success("CSV Exported", format!("Exported {} tickets to CSV file.", tickets.len()));
    };

    // Export Employees CSV
    let handle_export_employees_csv = move |_: MouseEvent| {
        let emps = crate::db::get_all_employees();
        let mut csv = String::from("Employee ID,Employee Code,Full Name,Email,Department,Division\n");
        for e in &emps {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                e.emp_id,
                e.emp_code.replace('"', "\"\""),
                e.emp_name.replace('"', "\"\""),
                e.emp_email.replace('"', "\"\""),
                e.department.replace('"', "\"\""),
                e.division.replace('"', "\"\""),
            ));
        }
        let escaped = csv.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            r#"
            (function() {{
                try {{
                    const blob = new Blob([`{escaped}`], {{ type: 'text/csv;charset=utf-8;' }});
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'bpsl_employees_' + new Date().toISOString().slice(0,10) + '.csv';
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                }} catch(e) {{ console.error('CSV export failed', e); }}
            }})();
            "#
        );
        let _ = document::eval(&js);
        crate::state::play_sound("success");
        toast.success("CSV Exported", format!("Exported {} employees to CSV file.", emps.len()));
    };

    // Export Reminders CSV
    let handle_export_reminders_csv = move |_: MouseEvent| {
        let rems = crate::db::get_all_reminders();
        let mut csv = String::from("Reminder ID,Employee Code,Employee Name,Email,Service Category,Message,Send At,Status\n");
        for r in &rems {
            csv.push_str(&format!(
                "\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
                r.reminder_id,
                r.employee_code.replace('"', "\"\""),
                r.employee_name.replace('"', "\"\""),
                r.email_address.replace('"', "\"\""),
                r.service_category.replace('"', "\"\""),
                r.reminder_message.replace('"', "\"\""),
                r.send_at.replace('"', "\"\""),
                r.status.replace('"', "\"\""),
            ));
        }
        let escaped = csv.replace('\\', "\\\\").replace('`', "\\`").replace('$', "\\$");
        let js = format!(
            r#"
            (function() {{
                try {{
                    const blob = new Blob([`{escaped}`], {{ type: 'text/csv;charset=utf-8;' }});
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = 'bpsl_reminders_' + new Date().toISOString().slice(0,10) + '.csv';
                    document.body.appendChild(a);
                    a.click();
                    document.body.removeChild(a);
                    URL.revokeObjectURL(url);
                }} catch(e) {{ console.error('CSV export failed', e); }}
            }})();
            "#
        );
        let _ = document::eval(&js);
        crate::state::play_sound("success");
        toast.success("CSV Exported", format!("Exported {} reminders to CSV file.", rems.len()));
    };

    // Clear Transient Sync Errors
    let handle_clear_sync_errors_ui = move |_: MouseEvent| {
        match crate::db::clear_sync_errors() {
            Ok(_) => {
                crate::state::play_sound("success");
                toast.success("Errors Cleared", "Cleared transient sync error records and refreshed status.");
                if let Some(mut r) = db_sync {
                    r.with_mut(|rev| rev.0 += 1);
                }
            }
            Err(e) => {
                toast.error("Clear Failed", format!("{e}"));
            }
        }
    };

    // Open Sheet URL in External Browser
    let handle_open_sheet_browser = {
        let base_url = base_url;
        move |_: MouseEvent| {
            let url = base_url.read().trim().to_string();
            if url.is_empty() {
                toast.warning("No URL Configured", "Please enter a valid Google Apps Script or Spreadsheet URL first.");
                return;
            }
            let escaped = url.replace('\\', "\\\\").replace('\'', "\\'");
            let js = format!("window.open('{escaped}', '_blank');");
            let _ = document::eval(&js);
            toast.info("Browser Launched", "Opened Google Sheet endpoint in default browser.");
        }
    };

    // Export Data JSON
    let handle_export_json = move |_| {
        let tickets = crate::db::get_all_tickets();
        let reminders = crate::db::get_all_reminders();
        let notes = crate::db::get_all_notes();
        let employees = crate::db::get_all_employees();
        let export_payload = serde_json::json!({
            "application": "BPSL IT Support",
            "version": "1.1.2",
            "exported_at": crate::state::format_utc(crate::sync::now_epoch()),
            "ticket_count": tickets.len(),
            "reminder_count": reminders.len(),
            "note_count": notes.len(),
            "employee_count": employees.len(),
            "tickets": tickets,
            "reminders": reminders,
            "notes": notes,
            "employees": employees,
        });

        if let Ok(json_string) = serde_json::to_string_pretty(&export_payload) {
            let escaped = json_string
                .replace('\\', "\\\\")
                .replace('`', "\\`")
                .replace('$', "\\$");
            let js = format!(
                r#"
                (function() {{
                    try {{
                        const data = `{escaped}`;
                        const blob = new Blob([data], {{ type: 'application/json' }});
                        const url = URL.createObjectURL(blob);
                        const a = document.createElement('a');
                        a.href = url;
                        a.download = 'bpsl_backup_' + new Date().toISOString().slice(0,10) + '.json';
                        document.body.appendChild(a);
                        a.click();
                        document.body.removeChild(a);
                        URL.revokeObjectURL(url);
                    }} catch(e) {{
                        console.error('Export error', e);
                    }}
                }})();
                "#
            );
            let _ = document::eval(&js);
            crate::state::play_sound("success");
            toast.success("Backup Exported", format!("Exported {} tickets, {} reminders, {} notes, {} employees to JSON.", tickets.len(), reminders.len(), notes.len(), employees.len()));
        } else {
            toast.error("Export Error", "Failed to generate JSON string.");
        }
    };

    // Optimize / Compact DB
    let handle_optimize_db = move |_| {
        let before_bytes = crate::db::get_database_file_size();
        let _ = crate::db::purge_resolved_conflicts();
        match crate::db::optimize_database() {
            Ok(_) => {
                let after_bytes = crate::db::get_database_file_size();
                crate::state::play_sound("success");
                let kb = (after_bytes as f64) / 1024.0;
                let saved = before_bytes.saturating_sub(after_bytes);
                toast.success(
                    "Database Optimized",
                    format!("Cleaned and defragmented SQLite tables. Database size: {kb:.1} KB (freed {saved} bytes)."),
                );
                if let Some(mut r) = db_sync {
                    r.with_mut(|rev| rev.0 += 1);
                }
            }
            Err(e) => {
                toast.error("Optimization Error", format!("{e}"));
            }
        }
    };

    // API Handlers
    let handle_add_sheet = {
        let mut sheet_sources = sheet_sources;
        let mut active_profile = active_profile;
        let mut current_profile_name = current_profile_name;
        let mut base_url = base_url;
        let mut api_key = api_key;
        let mut api_protocol = api_protocol;
        let mut test_status = test_status;
        move |_| {
            let mut list = sheet_sources.write();
            let count = list.len() + 1;
            let new_id = format!("sheet_{}", crate::sync::now_epoch());
            let new_name = format!("Sheet Source {count}");
            let new_src = crate::db::SheetSource {
                id: new_id.clone(),
                name: new_name.clone(),
                base_url: String::new(),
                api_key: String::new(),
                protocol: "auto".to_string(),
            };
            list.push(new_src);
            drop(list);
            active_profile.set(new_id);
            current_profile_name.set(new_name);
            base_url.set(String::new());
            api_key.set(String::new());
            api_protocol.set("auto".to_string());
            test_status.set(TestStatus::Idle);
        }
    };

    let mut handle_delete_sheet = {
        let mut sheet_sources = sheet_sources;
        let mut active_profile = active_profile;
        let mut current_profile_name = current_profile_name;
        let mut base_url = base_url;
        let mut api_key = api_key;
        let mut api_protocol = api_protocol;
        let mut test_status = test_status;
        move |id_to_del: String| {
            if sheet_sources.read().len() <= 1 {
                toast.warning("Cannot Delete", "You must keep at least one Google Sheet source.");
                return;
            }
            let mut list = sheet_sources.write();
            list.retain(|s| s.id != id_to_del);
            let remaining_first = list.first().cloned();
            let list_copy = list.clone();
            drop(list);
            let _ = crate::db::save_all_sheet_sources(&list_copy);
            if *active_profile.read() == id_to_del {
                if let Some(first) = remaining_first {
                    active_profile.set(first.id.clone());
                    current_profile_name.set(first.name.clone());
                    base_url.set(first.base_url.clone());
                    api_key.set(first.api_key.clone());
                    api_protocol.set(first.protocol.clone());
                    let _ = crate::sync::switch_sheet_profile_and_sync(&first.id);
                }
            }
            test_status.set(TestStatus::Idle);
        }
    };

    let mut handle_select_sheet = {
        let mut sheet_sources = sheet_sources;
        let mut active_profile = active_profile;
        let mut current_profile_name = current_profile_name;
        let mut base_url = base_url;
        let mut api_key = api_key;
        let mut api_protocol = api_protocol;
        let mut test_status = test_status;
        move |src: crate::db::SheetSource| {
            let cur_id = active_profile.read().clone();
            {
                let mut list = sheet_sources.write();
                if let Some(item) = list.iter_mut().find(|s| s.id == cur_id) {
                    item.name = current_profile_name.read().trim().to_string();
                    item.base_url = base_url.read().trim().to_string();
                    item.api_key = api_key.read().trim().to_string();
                    item.protocol = api_protocol.read().trim().to_string();
                }
            }
            active_profile.set(src.id.clone());
            current_profile_name.set(src.name.clone());
            base_url.set(src.base_url.clone());
            api_key.set(src.api_key.clone());
            api_protocol.set(src.protocol.clone());
            test_status.set(TestStatus::Idle);
        }
    };

    let handle_test = move |_| {
        let url = base_url.read().trim().to_string();
        let key = api_key.read().trim().to_string();
        test_status.set(TestStatus::Testing);

        spawn(async move {
            let start = std::time::Instant::now();
            match crate::sync::test_connection(&url, &key).await {
                Ok(msg) => {
                    let elapsed = start.elapsed().as_millis();
                    crate::state::play_sound("success");
                    test_status.set(TestStatus::Success(format!("{msg} · Latency: {elapsed} ms")));
                }
                Err(err) => {
                    let elapsed = start.elapsed().as_millis();
                    crate::state::play_sound("alert");
                    test_status.set(TestStatus::Error(format!("{err} ({elapsed} ms)")));
                }
            }
        });
    };

    let handle_verify_schema = move |_| {
        let url = base_url.read().trim().to_string();
        let key = api_key.read().trim().to_string();
        let proto = api_protocol.read().trim().to_string();

        if url.is_empty() {
            toast.error("Validation Error", "Please provide a Google Apps Script Web App URL first.");
            return;
        }

        is_verifying_schema.set(true);

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
                        if report.valid {
                            toast.success("Schema Valid", "Connected Google Sheet has all required tabs and columns!");
                        } else {
                            toast.warning("Schema Incomplete", "Some required tabs or columns are missing in the Google Sheet.");
                        }
                        schema_report.set(Some(report));
                        is_verifying_schema.set(false);
                    }
                    Err(e) => {
                        toast.error("Verification Error", format!("Failed to verify schema: {e}"));
                        is_verifying_schema.set(false);
                    }
                },
                Err(e) => {
                    toast.error("Client Error", format!("Failed to initialize client: {e}"));
                    is_verifying_schema.set(false);
                }
            }
        });
    };

    let handle_init_schema = move |_| {
        let url = base_url.read().trim().to_string();
        let key = api_key.read().trim().to_string();
        let proto = api_protocol.read().trim().to_string();

        if url.is_empty() {
            toast.error("Validation Error", "Please provide a Google Apps Script Web App URL first.");
            return;
        }

        is_initializing_schema.set(true);

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
                        toast.success("Initialized", "Google Sheet tabs and standard headers initialized successfully!");
                        if let Ok(report) = client.crud_verify_schema().await {
                            schema_report.set(Some(report));
                        }
                        is_initializing_schema.set(false);
                    }
                    Err(e) => {
                        toast.error("Initialization Failed", format!("Failed to auto-init schema: {e}"));
                        is_initializing_schema.set(false);
                    }
                },
                Err(e) => {
                    toast.error("Client Error", format!("Failed to initialize client: {e}"));
                    is_initializing_schema.set(false);
                }
            }
        });
    };

    let handle_save_api = move |_| {
        let url = base_url.read().trim().to_string();
        let key = api_key.read().trim().to_string();
        let emp = employee_code.read().trim().to_string();
        let prof = active_profile.read().clone();
        let proto = api_protocol.read().trim().to_string();
        let name = current_profile_name.read().trim().to_string();
        let timeout_str = network_timeout.read().trim().to_string();
        let offline = *work_offline.read();

        if url.is_empty() {
            toast.error("Validation Error", "Google Apps Script Web App URL cannot be empty.");
            return;
        }

        is_saving.set(true);

        let _ = crate::db::sync_meta_set("sync_network_timeout_secs", &timeout_str);
        let _ = crate::db::sync_meta_set("sync_offline_mode", if offline { "true" } else { "false" });

        let mut list = sheet_sources.write();
        if let Some(item) = list.iter_mut().find(|s| s.id == prof) {
            item.name = if name.is_empty() { format!("Sheet Source {}", prof) } else { name.clone() };
            item.base_url = url.clone();
            item.api_key = key.clone();
            item.protocol = proto.clone();
        }
        let sources_copy = list.clone();
        drop(list);

        if let Err(e) = crate::db::save_all_sheet_sources(&sources_copy) {
            is_saving.set(false);
            toast.error("Failed to Save Settings", format!("Database error: {e}"));
            return;
        }

        if !emp.is_empty() {
            let _ = crate::db::sync_meta_set("system_employee_code", &emp);
        }

        match crate::sync::switch_sheet_profile_and_sync(&prof) {
            Ok(_) => {
                is_saving.set(false);
                crate::sync::kick();
                crate::state::play_sound("success");
                spawn(async move {
                    let _ = crate::sync::pull_tickets_now().await;
                    let _ = crate::sync::pull_employees_now().await;
                });
                if let Some(mut r) = db_sync {
                    r.with_mut(|rev| rev.0 += 1);
                }
                let disp_name = if name.is_empty() { "Google Sheet".to_string() } else { name };
                toast.success("Settings Saved & Connected", format!("Successfully connected to {disp_name}."));
                on_close.call(());
            }
            Err(e) => {
                is_saving.set(false);
                toast.warning("Settings Saved", format!("Saved in database, but connection reload warned: {e}"));
                on_close.call(());
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

    let save_sync_settings = move |_| {
        let interval = sync_interval_str.read().trim().to_string();
        let interval_val: u64 = interval.parse().unwrap_or(25);
        let clamped = interval_val.clamp(5, 300);
        sync_interval_str.set(clamped.to_string());
        let _ = crate::db::sync_meta_set("sync_cycle_secs", &clamped.to_string());
        let _ = crate::db::sync_meta_set("sync_auto_enabled", if *auto_sync_enabled.read() { "true" } else { "false" });
        crate::sync::kick();
        crate::state::play_sound("success");
        toast.success("Sync Settings Saved", format!("Sync interval set to {clamped}s and background worker refreshed."));
    };

    let handle_reset_sync_queue = move |_| {
        let _ = crate::db::reset_failed_sync_ops();
        crate::state::play_sound("click");
        toast.info("Sync Queue Reset", "All failed operations have been reset for retry.");
    };

    let handle_force_sync = move |_| {
        crate::sync::kick();
        crate::state::play_sound("click");
        toast.info("Sync Triggered", "A background sync cycle has been initiated.");
    };

    // Diagnostics Data
    let db_path_display = crate::db::db_path().display().to_string();
    let db_size_bytes = crate::db::get_database_file_size();
    let db_size_display = format!("{:.1} KB", (db_size_bytes as f64) / 1024.0);
    let pending_ops = crate::db::pending_op_count();
    let ticket_count = crate::db::get_all_tickets().len();
    let reminder_count = crate::db::get_all_reminders().len();
    let employee_count = crate::db::employee_count();
    let note_count = crate::db::get_all_notes().len();
    let conflict_count = crate::db::unresolved_conflict_count();
    let latest_error = crate::db::latest_active_error().unwrap_or_else(|| "None".to_string());

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex min-h-full items-center justify-center overflow-y-auto bg-black/60 backdrop-blur-xs p-3 sm:p-6 animate-dialog-overlay",
            onclick: handle_close,

            div {
                class: "relative z-50 w-full max-w-5xl my-auto rounded-2xl border border-border/80 bg-card text-card-foreground shadow-2xl animate-modal-pop flex flex-col max-h-[92vh] overflow-hidden outline-none",
                onclick: move |e: MouseEvent| e.stop_propagation(),

                // HEADER
                div { class: "flex items-center justify-between border-b border-border/60 px-6 py-4 shrink-0",
                    div { class: "flex items-center gap-3",
                        div { class: "flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary shadow-xs",
                            SettingsIcon { class: "size-5" }
                        }
                        div { class: "flex flex-col",
                            h2 { class: "text-lg font-bold text-foreground leading-none", "Preferences & Customization" }
                            span { class: "text-xs text-muted-foreground mt-0.5", "Customize themes, workflows, alerts, API connection, and data storage" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "rounded-lg p-1.5 text-muted-foreground hover:bg-muted hover:text-foreground active:scale-95 transition-all duration-150 cursor-pointer",
                        onclick: handle_close,
                        XIcon { class: "size-5" }
                    }
                }

                // INSTANT SETTINGS SEARCH BAR
                div { class: "border-b border-border/50 px-6 py-2.5 bg-muted/15 flex items-center gap-3 shrink-0",
                    div { class: "relative flex-1 flex items-center",
                        div { class: "absolute left-3 flex items-center text-muted-foreground pointer-events-none",
                            crate::icons::SearchIcon { class: "size-3.5" }
                        }
                        input {
                            r#type: "text",
                            class: "w-full pl-9 pr-8 h-8 rounded-lg border border-border/70 bg-background text-xs font-medium placeholder:text-muted-foreground/60 focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/20 transition-all",
                            placeholder: "Search all settings (e.g. sound, accent, sync, department, date, csv, backup, timeout, kanban)...",
                            value: search_query.read().clone(),
                            oninput: move |e: FormEvent| search_query.set(e.value()),
                        }
                        if !search_query.read().is_empty() {
                            button {
                                r#type: "button",
                                title: "Clear search filter",
                                class: "absolute right-2 p-1 text-muted-foreground hover:text-foreground cursor-pointer rounded",
                                onclick: move |_| search_query.set(String::new()),
                                XIcon { class: "size-3.5" }
                            }
                        }
                    }
                    if !search_query.read().is_empty() {
                        span { class: "text-[11px] font-medium text-primary bg-primary/10 px-2.5 py-1 rounded-md shrink-0 border border-primary/20",
                            "Search Active"
                        }
                    }
                }

                // BODY: TAB NAV + CONTENT
                div { class: "flex flex-1 overflow-hidden min-h-0",

                    // Left Tab Navigation Rail
                    nav { class: "w-52 shrink-0 border-r border-border/50 bg-muted/20 p-2.5 flex flex-col gap-1 overflow-y-auto",
                        for tab in TABS.iter() {
                            {
                                let t = *tab;
                                let is_active = *active_tab.read() == t;
                                let q = search_query.read().to_lowercase().trim().to_string();
                                let is_searching = !q.is_empty();
                                let tab_matches = if is_searching {
                                    match t {
                                        SettingsTab::Appearance => "appearance theme color accent dark light oled radius corner font scale density time date sidebar animation contrast glass blur".contains(&q),
                                        SettingsTab::Workflow => "workflow landing ticket kanban priority category assignee technician department table page delete confirm focus refresh".contains(&q),
                                        SettingsTab::Notifications => "alert audio sound chime volume tone bell ping toast reminder advance pull notify sla breach dnd silent".contains(&q),
                                        SettingsTab::Api => "api url key sheet source protocol connection test employee code timeout offline".contains(&q),
                                        SettingsTab::SyncData => "sync cadence interval queue conflict export backup csv json import restore vacuum optimize cache clean reset".contains(&q),
                                        SettingsTab::About => "about version system shortcut keyboard diagnostic cpu os database stats".contains(&q),
                                    }
                                } else {
                                    false
                                };
                                let cls = if is_active {
                                    "bg-primary/10 text-primary font-semibold border-primary/30 shadow-2xs"
                                } else if tab_matches {
                                    "bg-primary/5 text-primary/90 font-medium border-primary/20"
                                } else {
                                    "text-muted-foreground hover:bg-muted hover:text-foreground border-transparent"
                                };
                                rsx! {
                                    button {
                                        key: "{t.label()}",
                                        r#type: "button",
                                        class: "flex items-center justify-between w-full text-left rounded-lg px-3 py-2 text-xs border transition-all duration-150 cursor-pointer active:scale-[0.98] {cls}",
                                        onclick: move |_| {
                                            crate::state::play_sound("click");
                                            active_tab.set(t);
                                        },
                                        div { class: "flex items-center gap-2.5 min-w-0 flex-1",
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
                                            span { class: "truncate", "{t.label()}" }
                                        }
                                        if tab_matches {
                                            span { class: "size-2 rounded-full bg-primary shrink-0 animate-pulse" }
                                        }
                                    }
                                }
                            }
                        }

                        // Version badge at bottom of rail
                        div { class: "mt-auto pt-4 px-2 flex items-center justify-between",
                            span { class: "text-[10px] text-muted-foreground/60 font-mono", "v1.1.2" }
                            span { class: "text-[10px] text-primary/70 font-medium", "Dioxus 0.7" }
                        }
                    }

                    // Right Content Area
                    div { class: "flex-1 overflow-y-auto p-6",

                        // ========================================================
                        // TAB 1: Appearance & Theme
                        // ========================================================
                        if *active_tab.read() == SettingsTab::Appearance {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Theme Mode Section
                                div { class: "flex flex-col gap-2.5",
                                    span { class: "text-xs font-semibold text-foreground uppercase tracking-wider", "Theme & Palette" }
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-sm font-semibold text-foreground", "Theme Mode" }
                                            span { class: "text-[11px] text-muted-foreground", "Switch between Light, Dark, or automatic OS detection." }
                                        }
                                        crate::components::ui::theme::ThemeSegmented {}
                                    }
                                }

                                // OLED Pure Black Toggle
                                SettingToggle {
                                    title: "OLED Pitch Black (Dark Mode)",
                                    description: "Use pure #000000 black background for OLED screens and maximum contrast.",
                                    checked: *oled_dark.read(),
                                    on_toggle: move |_| {
                                        let next = !*oled_dark.read();
                                        oled_dark.set(next);
                                        let cur_acc = accent_color.read().clone();
                                        let cur_scale = font_scale.read().clone();
                                        let cur_density = card_density.read().clone();
                                        let cur_rad = corner_radius.read().clone();
                                        let cur_fam = font_family.read().clone();
                                        let cur_anim = *animations_enabled.read();
                                        crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &cur_density, &cur_rad, &cur_fam, next, cur_anim);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // Accent Color Swatches
                                div { class: "flex flex-col gap-2.5",
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-sm font-semibold text-foreground", "Primary Accent Color" }
                                        span { class: "text-[11px] text-muted-foreground", "Applies to active buttons, badges, selection rings, and highlight borders." }
                                    }
                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2",
                                        for opt in ACCENT_OPTIONS.iter() {
                                            {
                                                let is_active = *accent_color.read() == opt.id;
                                                let id_val = opt.id.to_string();
                                                let border_cls = if is_active { "border-primary ring-2 ring-primary/30 bg-primary/10 font-bold" } else { "border-border/70 hover:bg-muted/50" };
                                                rsx! {
                                                    button {
                                                        key: "{opt.id}",
                                                        r#type: "button",
                                                        class: "flex items-center gap-2 rounded-lg border px-3 py-2 text-xs transition-all duration-150 cursor-pointer text-left {border_cls}",
                                                        onclick: {
                                                            let v = id_val.clone();
                                                            move |_| {
                                                                accent_color.set(v.clone());
                                                                let cur_scale = font_scale.read().clone();
                                                                let cur_density = card_density.read().clone();
                                                                let cur_rad = corner_radius.read().clone();
                                                                let cur_fam = font_family.read().clone();
                                                                let cur_oled = *oled_dark.read();
                                                                let cur_anim = *animations_enabled.read();
                                                                crate::state::apply_appearance_to_dom(&v, &cur_scale, &cur_density, &cur_rad, &cur_fam, cur_oled, cur_anim);
                                                            }
                                                        },
                                                        span {
                                                            class: "size-3.5 rounded-full shrink-0 shadow-2xs border border-white/20",
                                                            style: "background-color: {opt.color_hex};",
                                                        }
                                                        span { class: "truncate text-foreground text-[11px]", "{opt.name}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Corner Radius Presets
                                SettingRow {
                                    title: "Window & Element Corner Radius",
                                    description: "Adjust how rounded buttons, cards, and modal windows feel.",
                                    div { class: "flex gap-1.5 flex-wrap",
                                        for (val, label) in [("sharp", "Sharp (0px)"), ("rounded", "Modern (6px)"), ("smooth", "Smooth (10px)"), ("pill", "Pill (16px)")] {
                                            {
                                                let is_active = *corner_radius.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                             "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                corner_radius.set(v.clone());
                                                                let cur_acc = accent_color.read().clone();
                                                                let cur_scale = font_scale.read().clone();
                                                                let cur_density = card_density.read().clone();
                                                                let cur_fam = font_family.read().clone();
                                                                let cur_oled = *oled_dark.read();
                                                                let cur_anim = *animations_enabled.read();
                                                                crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &cur_density, &v, &cur_fam, cur_oled, cur_anim);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Font Family Selection
                                SettingRow {
                                    title: "Font Family",
                                    description: "Typeface styling used across tables, tickets, notes, and metrics.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("roboto", "Roboto (Default)"), ("system", "System UI / Sans"), ("mono", "Roboto Mono")] {
                                            {
                                                let is_active = *font_family.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                font_family.set(v.clone());
                                                                let cur_acc = accent_color.read().clone();
                                                                let cur_scale = font_scale.read().clone();
                                                                let cur_density = card_density.read().clone();
                                                                let cur_rad = corner_radius.read().clone();
                                                                let cur_oled = *oled_dark.read();
                                                                let cur_anim = *animations_enabled.read();
                                                                crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &cur_density, &cur_rad, &v, cur_oled, cur_anim);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Typography Scale
                                SettingRow {
                                    title: "Typography & Font Scaling",
                                    description: "Calibrated font scaling for 19-inch displays, laptops, and 4K screens.",
                                    div { class: "flex flex-wrap gap-1.5",
                                        for (val, label) in [
                                            ("auto", "Auto (Fluid)"),
                                            ("compact", "Compact (90%)"),
                                            ("default", "Default (100%)"),
                                            ("comfortable", "Comfortable (110%)"),
                                            ("large", "Large (120%)"),
                                        ] {
                                            {
                                                let is_active = *font_scale.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                font_scale.set(v.clone());
                                                                let cur_acc = accent_color.read().clone();
                                                                let cur_density = card_density.read().clone();
                                                                let cur_rad = corner_radius.read().clone();
                                                                let cur_fam = font_family.read().clone();
                                                                let cur_oled = *oled_dark.read();
                                                                let cur_anim = *animations_enabled.read();
                                                                crate::state::apply_appearance_to_dom(&cur_acc, &v, &cur_density, &cur_rad, &cur_fam, cur_oled, cur_anim);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Card Density
                                SettingRow {
                                    title: "Card Density",
                                    description: "Control compact or spacious padding across tickets and reminder cards.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("compact", "Compact"), ("default", "Default"), ("comfortable", "Comfortable")] {
                                            {
                                                let is_active = *card_density.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                card_density.set(v.clone());
                                                                let cur_acc = accent_color.read().clone();
                                                                let cur_scale = font_scale.read().clone();
                                                                let cur_rad = corner_radius.read().clone();
                                                                let cur_fam = font_family.read().clone();
                                                                let cur_oled = *oled_dark.read();
                                                                let cur_anim = *animations_enabled.read();
                                                                crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &v, &cur_rad, &cur_fam, cur_oled, cur_anim);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Time Display Format
                                SettingRow {
                                    title: "Time Display Format",
                                    description: "How timestamps appear in tickets, reminders, and calendar.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("12hr", "12-Hour (AM/PM)"), ("24hr", "24-Hour")] {
                                            {
                                                let is_active = *date_format.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                date_format.set(v.clone());
                                                                crate::state::set_time_format_24hr(v == "24hr");
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Date Display Pattern
                                SettingRow {
                                    title: "Date Display Pattern",
                                    description: "Preferred arrangement for civil dates across tickets and tables.",
                                    div { class: "flex gap-1.5 flex-wrap",
                                        for (val, label) in [
                                            ("medium", "Nov 20, 2026 (Default)"),
                                            ("dmy", "20/11/2026 (Indian)"),
                                            ("iso", "2026-11-20 (ISO)"),
                                            ("d-mmm-y", "20-Nov-2026"),
                                        ] {
                                            {
                                                let is_active = *date_pattern.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                date_pattern.set(v.clone());
                                                                crate::state::set_date_display_style(&v);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // High Contrast Borders Toggle
                                SettingToggle {
                                    title: "High-Contrast Element Borders",
                                    description: "Enhance visual separation with darker and higher-contrast element borders.",
                                    checked: *high_contrast.read(),
                                    on_toggle: move |_| {
                                        let next = !*high_contrast.read();
                                        high_contrast.set(next);
                                        let _ = crate::db::sync_meta_set("ui_high_contrast", if next { "true" } else { "false" });
                                        let cur_acc = accent_color.read().clone();
                                        let cur_scale = font_scale.read().clone();
                                        let cur_density = card_density.read().clone();
                                        let cur_rad = corner_radius.read().clone();
                                        let cur_fam = font_family.read().clone();
                                        let cur_oled = *oled_dark.read();
                                        let cur_anim = *animations_enabled.read();
                                        crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &cur_density, &cur_rad, &cur_fam, cur_oled, cur_anim);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // Glassmorphism Toggle
                                SettingToggle {
                                    title: "Translucent Glassmorphism & Blur",
                                    description: "Enable frosted backdrop-blur effects (disable for low-end graphics acceleration).",
                                    checked: *glassmorphism.read(),
                                    on_toggle: move |_| {
                                        let next = !*glassmorphism.read();
                                        glassmorphism.set(next);
                                        let _ = crate::db::sync_meta_set("ui_glassmorphism", if next { "true" } else { "false" });
                                        let cur_acc = accent_color.read().clone();
                                        let cur_scale = font_scale.read().clone();
                                        let cur_density = card_density.read().clone();
                                        let cur_rad = corner_radius.read().clone();
                                        let cur_fam = font_family.read().clone();
                                        let cur_oled = *oled_dark.read();
                                        let cur_anim = *animations_enabled.read();
                                        crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &cur_density, &cur_rad, &cur_fam, cur_oled, cur_anim);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // Sidebar Position
                                SettingRow {
                                    title: "Sidebar Position",
                                    description: "Where the primary navigation rail is anchored in the app shell.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("left", "Left Hand Side"), ("right", "Right Hand Side")] {
                                            {
                                                let is_active = *sidebar_position.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| sidebar_position.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Animations Toggle
                                SettingToggle {
                                    title: "Smooth Animations & Transitions",
                                    description: "Enable micro-interactions, modal transitions, and card drag visuals.",
                                    checked: *animations_enabled.read(),
                                    on_toggle: move |_| {
                                        let next = !*animations_enabled.read();
                                        animations_enabled.set(next);
                                        let cur_acc = accent_color.read().clone();
                                        let cur_scale = font_scale.read().clone();
                                        let cur_density = card_density.read().clone();
                                        let cur_rad = corner_radius.read().clone();
                                        let cur_fam = font_family.read().clone();
                                        let cur_oled = *oled_dark.read();
                                        crate::state::apply_appearance_to_dom(&cur_acc, &cur_scale, &cur_density, &cur_rad, &cur_fam, cur_oled, next);
                                    },
                                }

                                // Live Preview Sample Card
                                div { class: "rounded-xl border border-primary/20 bg-primary/5 p-4 flex flex-col gap-2 mt-1",
                                    span { class: "text-[10px] font-bold text-primary uppercase tracking-wider", "Live Theme Preview" }
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex items-center gap-2",
                                            span { class: "size-2.5 rounded-full bg-primary" }
                                            span { class: "text-xs font-semibold text-foreground", "Sample Support Ticket" }
                                            span { class: "rounded-full bg-primary text-primary-foreground text-[10px] px-2 py-0.5 font-bold", "High Priority" }
                                        }
                                        Button {
                                            variant: "default",
                                            class: "h-7 text-xs px-3 shadow-xs",
                                            "Sample Action"
                                        }
                                    }
                                }

                                // Save & Reset Appearance
                                div { class: "flex items-center justify-between pt-3 border-t border-border/50",
                                    button {
                                        r#type: "button",
                                        class: "text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer underline-offset-4 hover:underline",
                                        onclick: reset_appearance,
                                        "Reset Defaults"
                                    }
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

                        // ========================================================
                        // TAB 2: Tickets & Workflow
                        // ========================================================
                        if *active_tab.read() == SettingsTab::Workflow {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Default Landing Page
                                SettingRow {
                                    title: "Default Landing Page",
                                    description: "The page that opens automatically when launching the application.",
                                    div { class: "flex flex-wrap gap-1.5",
                                        for (val, label) in [
                                            ("tickets", "Tickets"),
                                            ("notes", "Notes"),
                                            ("reminders", "Reminders"),
                                            ("calendar", "Calendar"),
                                            ("analytics", "Analytics"),
                                        ] {
                                            {
                                                let is_active = *default_landing.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| default_landing.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Default Ticket View
                                SettingRow {
                                    title: "Default Tickets View",
                                    description: "Preferred presentation layout when browsing Support Tickets.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("kanban", "Kanban Board"), ("table", "IT Data Table")] {
                                            {
                                                let is_active = *default_ticket_view.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| default_ticket_view.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Default Priority
                                SettingRow {
                                    title: "Default Priority for New Tickets",
                                    description: "Initial priority selected when creating a new support request.",
                                    div { class: "flex gap-1.5",
                                        for val in ["Low", "Medium", "High", "Urgent"] {
                                            {
                                                let is_active = *default_priority.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| default_priority.set(v.clone())
                                                        },
                                                        "{val}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Default Category
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Default Ticket Category" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "CATEGORY" }
                                    }
                                    select {
                                        value: "{default_category}",
                                        onchange: move |e: FormEvent| default_category.set(e.value()),
                                        class: "w-full font-medium text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary focus:outline-none transition-colors",
                                        option { value: "General", "General" }
                                        option { value: "Hardware", "Hardware Support" }
                                        option { value: "Software", "Software / OS Application" }
                                        option { value: "Network", "Network / Wi-Fi / LAN" }
                                        option { value: "Access & Accounts", "Access & User Accounts" }
                                        option { value: "Email / Outlook", "Email / Outlook / Office 365" }
                                        option { value: "Printer / Peripherals", "Printer / Peripherals" }
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Pre-filled category when launching the New Ticket modal dialog."
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Default Assigned Technician
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Default Support Technician" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "ASSIGNED_TO" }
                                    }
                                    Input {
                                        r#type: "text",
                                        placeholder: "e.g. IT Support, Shreyansh, GR...",
                                        value: "{default_assignee}",
                                        oninput: move |e: FormEvent| default_assignee.set(e.value()),
                                        class: "font-medium text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary",
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Technician or support group assigned by default on new tickets."
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Default Department
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Default Ticket Department" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "DEPARTMENT" }
                                    }
                                    select {
                                        value: "{default_department}",
                                        onchange: move |e: FormEvent| default_department.set(e.value()),
                                        class: "w-full font-medium text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary focus:outline-none transition-colors",
                                        option { value: "General", "General / Any Department" }
                                        option { value: "OPERATION", "OPERATION" }
                                        option { value: "STORES", "STORES" }
                                        option { value: "QUALITY CONTROL", "QUALITY CONTROL" }
                                        option { value: "MECHANICAL", "MECHANICAL" }
                                        option { value: "ELECTRICAL DEPARTMENT(E&I)", "ELECTRICAL DEPARTMENT(E&I)" }
                                        option { value: "FINANCE AND ACCOUNTS", "FINANCE AND ACCOUNTS" }
                                        option { value: "CORPORATE", "CORPORATE" }
                                        option { value: "IT SUPPORT", "IT SUPPORT" }
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Pre-selected department when logging support issues."
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Table Pagination Rows
                                SettingRow {
                                    title: "Table Pagination Rows per Page",
                                    description: "Maximum tickets and employees shown before paging in data tables.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("15", "15 Rows"), ("25", "25 Rows"), ("50", "50 Rows"), ("100", "100 Rows"), ("all", "All Rows")] {
                                            {
                                                let is_active = *table_page_size.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| table_page_size.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Kanban Compact Cards
                                SettingToggle {
                                    title: "Compact Kanban Card Layout",
                                    description: "Display condensed ticket cards with tighter paddings to fit more items on screen.",
                                    checked: *kanban_compact.read(),
                                    on_toggle: move |_| {
                                        let next = !*kanban_compact.read();
                                        kanban_compact.set(next);
                                    },
                                }

                                // Confirm Deletions Safety Toggle
                                SettingToggle {
                                    title: "Confirm Before Permanent Deletions",
                                    description: "Prompt with a safety confirmation dialog before deleting tickets, reminders, or notes.",
                                    checked: *confirm_deletions.read(),
                                    on_toggle: move |_| {
                                        let next = !*confirm_deletions.read();
                                        confirm_deletions.set(next);
                                    },
                                }

                                // Auto Refresh on Window Focus
                                SettingToggle {
                                    title: "Auto-Refresh on Window Focus",
                                    description: "Silently check for remote sheet changes when switching back to this application window.",
                                    checked: *focus_refresh.read(),
                                    on_toggle: move |_| {
                                        let next = !*focus_refresh.read();
                                        focus_refresh.set(next);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // Kanban Column Visibility Toggles
                                SettingToggle {
                                    title: "Hide 'Cancelled' Column on Kanban",
                                    description: "Keeps the Kanban board clean by omitting cancelled tickets from display.",
                                    checked: *kanban_hide_cancelled.read(),
                                    on_toggle: move |_| {
                                        let next = !*kanban_hide_cancelled.read();
                                        kanban_hide_cancelled.set(next);
                                    },
                                }

                                SettingToggle {
                                    title: "Hide 'Solved (Closed)' Column on Kanban",
                                    description: "Omits resolved and closed tickets to focus exclusively on active support work.",
                                    checked: *kanban_hide_solved.read(),
                                    on_toggle: move |_| {
                                        let next = !*kanban_hide_solved.read();
                                        kanban_hide_solved.set(next);
                                    },
                                }

                                // Save Workflow & Reset Row
                                div { class: "flex items-center justify-between pt-3 border-t border-border/50",
                                    button {
                                        r#type: "button",
                                        class: "text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer underline-offset-4 hover:underline",
                                        onclick: reset_workflow,
                                        "Reset Defaults"
                                    }
                                    Button {
                                        variant: "default",
                                        class: "h-9 text-xs px-4 font-semibold cursor-pointer shadow-xs gap-1.5",
                                        onclick: save_workflow,
                                        CheckIcon { class: "size-3.5" }
                                        "Save Workflow Settings"
                                    }
                                }
                            }
                        }

                        // ========================================================
                        // TAB 3: Alerts & Audio
                        // ========================================================
                        if *active_tab.read() == SettingsTab::Notifications {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Audio Sound Effects Toggle
                                SettingToggle {
                                    title: "Synthesized Audio Chimes",
                                    description: "Play subtle harmonic sound effects on ticket creation, sync completion, and alarms.",
                                    checked: *sound_enabled.read(),
                                    on_toggle: move |_| {
                                        let next = !*sound_enabled.read();
                                        sound_enabled.set(next);
                                        crate::state::set_sound_enabled(next);
                                    },
                                }

                                // Chime Melody Tone Picker
                                SettingRow {
                                    title: "Notification Chime Melody Tone",
                                    description: "Select the audio synthesizer melody played on success and updates.",
                                    div { class: "flex gap-1.5 flex-wrap",
                                        for (val, label) in [
                                            ("arpeggio", "Arpeggio (Harmonic)"),
                                            ("ping", "Subtle Ping"),
                                            ("bell", "Classic Bell"),
                                            ("marimba", "Warm Marimba"),
                                            ("digital", "Digital Beep"),
                                        ] {
                                            {
                                                let is_active = *chime_tone.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                chime_tone.set(v.clone());
                                                                crate::state::play_sound(&v);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Audio Volume Selector
                                SettingRow {
                                    title: "Chime Volume Level",
                                    description: "Adjust audio output volume for application sound effects.",
                                    div { class: "flex items-center gap-2",
                                        for (val, label) in [("low", "Gentle (8%)"), ("medium", "Standard (15%)"), ("high", "Clear (30%)")] {
                                            {
                                                let is_active = *sound_volume.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| {
                                                                sound_volume.set(v.clone());
                                                                crate::state::set_sound_volume(&v);
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                        Button {
                                            variant: "outline",
                                            class: "h-8 text-xs px-3 shadow-2xs gap-1 cursor-pointer ml-1",
                                            onclick: handle_test_sound,
                                            BellIcon { class: "size-3.5" }
                                            "Test Sound"
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Do Not Disturb / Silent Mode Toggle
                                SettingToggle {
                                    title: "Do Not Disturb (Silent Mode)",
                                    description: "Mute all audio chimes and sound effects while keeping desktop toast visuals active.",
                                    checked: *dnd_mode.read(),
                                    on_toggle: move |_| {
                                        let next = !*dnd_mode.read();
                                        dnd_mode.set(next);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // SLA Breach Advance Warning
                                SettingRow {
                                    title: "SLA Due Date Warning Horizon",
                                    description: "Warn in advance when tickets approach their resolution deadline.",
                                    div { class: "flex gap-1.5 flex-wrap",
                                        for (val, label) in [
                                            ("0", "Disabled"),
                                            ("1", "1h Before"),
                                            ("2", "2h Before"),
                                            ("4", "4h Before"),
                                            ("8", "8h Before"),
                                            ("24", "24h Before"),
                                        ] {
                                            {
                                                let is_active = *sla_warning_hours.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| sla_warning_hours.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Toast Duration
                                SettingRow {
                                    title: "Toast Notification Duration",
                                    description: "How long in-app status toast messages remain visible before fading.",
                                    div { class: "flex gap-1.5",
                                        for (val, label) in [("3", "Fast (3s)"), ("5", "Standard (5s)"), ("8", "Extended (8s)")] {
                                            {
                                                let is_active = *toast_duration.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-3 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| toast_duration.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Reminder Advance Alert
                                SettingRow {
                                    title: "Default Reminder Advance Alert",
                                    description: "Time before due date when upcoming reminders show high-priority badges.",
                                    div { class: "flex gap-1.5 flex-wrap",
                                        for (val, label) in [
                                            ("0", "At Due Time"),
                                            ("5", "5m Before"),
                                            ("15", "15m Before"),
                                            ("30", "30m Before"),
                                            ("60", "1h Before"),
                                            ("1440", "1d Before"),
                                        ] {
                                            {
                                                let is_active = *reminder_advance.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| reminder_advance.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // Windows Desktop Notifications Toggle
                                SettingToggle {
                                    title: "Windows Desktop Notifications",
                                    description: "Receive native Windows notifications for new tickets, status changes, and due reminders.",
                                    checked: *windows_notifications.read(),
                                    on_toggle: move |_| {
                                        let next = !*windows_notifications.read();
                                        windows_notifications.set(next);
                                    },
                                }

                                // Test Windows Notification
                                div { class: "flex items-center justify-between p-3 rounded-lg bg-muted/30 border border-border/50",
                                    div { class: "space-y-0.5",
                                        div { class: "text-xs font-medium text-foreground", "Test Desktop Notification" }
                                        div { class: "text-[11px] text-muted-foreground", "Send a sample Windows notification to test your system settings." }
                                    }
                                    button {
                                        r#type: "button",
                                        class: "h-8 text-xs px-3 rounded-md bg-secondary text-secondary-foreground hover:bg-secondary/80 font-medium transition-colors cursor-pointer",
                                        onclick: move |_| {
                                            crate::notifications::test_notification();
                                            toast.info("Notification Sent", "Check your Windows Action Center or notification banner.");
                                        },
                                        "Send Test Alert"
                                    }
                                }

                                // Notify on Pull
                                SettingToggle {
                                    title: "Notify on Remote Sheet Sync",
                                    description: "Display an in-app banner whenever background sync pulls new updates.",
                                    checked: *notify_on_pull.read(),
                                    on_toggle: move |_| {
                                        let next = !*notify_on_pull.read();
                                        notify_on_pull.set(next);
                                    },
                                }

                                // Save Notifications & Reset
                                div { class: "flex items-center justify-between pt-3 border-t border-border/50",
                                    button {
                                        r#type: "button",
                                        class: "text-xs text-muted-foreground hover:text-foreground transition-colors cursor-pointer underline-offset-4 hover:underline",
                                        onclick: reset_notifications,
                                        "Reset Defaults"
                                    }
                                    Button {
                                        variant: "default",
                                        class: "h-9 text-xs px-4 font-semibold cursor-pointer shadow-xs gap-1.5",
                                        onclick: save_notifications,
                                        CheckIcon { class: "size-3.5" }
                                        "Save Alert Settings"
                                    }
                                }
                            }
                        }

                        // ========================================================
                        // TAB 4: API & Connection
                        // ========================================================
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

                                // Google Sheet Sources Manager
                                div { class: "flex flex-col gap-2.5 rounded-xl border border-border/70 bg-muted/30 p-3.5",
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex items-center gap-1.5",
                                            span { class: "text-xs font-semibold text-foreground", "Google Sheet Sources" }
                                            span { class: "text-[10px] text-muted-foreground px-1.5 py-0.5 rounded-full bg-muted font-mono",
                                                "{sheet_sources.read().len()}"
                                            }
                                        }
                                        button {
                                            r#type: "button",
                                            class: "cursor-pointer flex items-center gap-1 rounded-md bg-primary/10 hover:bg-primary/20 text-primary border border-primary/20 px-2 py-1 text-[11px] font-medium transition-colors shadow-2xs",
                                            onclick: handle_add_sheet,
                                            svg { class: "size-3", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                path { d: "M12 5v14M5 12h14" }
                                            }
                                            "Add Sheet Source"
                                        }
                                    }
                                    div { class: "flex flex-col gap-1.5 max-h-48 overflow-y-auto pr-1",
                                        for src in sheet_sources.read().clone() {
                                            {
                                                let is_selected = *active_profile.read() == src.id;
                                                let src_id = src.id.clone();
                                                let src_name = src.name.clone();
                                                let url_display = if src.base_url.is_empty() { "No URL configured".to_string() } else { src.base_url.clone() };
                                                let can_delete = sheet_sources.read().len() > 1;
                                                rsx! {
                                                    div {
                                                        key: "{src.id}",
                                                        class: if is_selected {
                                                            "group flex items-center justify-between rounded-lg border border-primary bg-primary/10 px-3 py-2 transition-all shadow-2xs cursor-pointer"
                                                        } else {
                                                            "group flex items-center justify-between rounded-lg border border-border/70 bg-background px-3 py-2 hover:bg-muted/50 transition-all cursor-pointer"
                                                        },
                                                        onclick: {
                                                            let src_c = src.clone();
                                                            move |_| handle_select_sheet(src_c.clone())
                                                        },
                                                        div { class: "flex items-center gap-2.5 min-w-0 flex-1",
                                                            div {
                                                                class: if is_selected { "size-2 rounded-full bg-primary shrink-0" } else { "size-2 rounded-full bg-muted-foreground/40 shrink-0" },
                                                            }
                                                            div { class: "flex flex-col min-w-0 flex-1",
                                                                span { class: "text-xs font-semibold text-foreground truncate", "{src_name}" }
                                                                span { class: "text-[10px] text-muted-foreground font-mono truncate", "{url_display}" }
                                                            }
                                                        }
                                                        if can_delete {
                                                            button {
                                                                r#type: "button",
                                                                title: "Delete this sheet source",
                                                                class: "opacity-40 group-hover:opacity-100 hover:text-destructive hover:bg-destructive/10 p-1 rounded transition-all cursor-pointer ml-2",
                                                                onclick: {
                                                                    let id = src_id.clone();
                                                                    move |e: MouseEvent| {
                                                                        e.stop_propagation();
                                                                        handle_delete_sheet(id.clone());
                                                                    }
                                                                },
                                                                svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                                    path { d: "M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Sheet Name / Label input
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Sheet Source Name" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "LABEL" }
                                    }
                                    Input {
                                        r#type: "text",
                                        placeholder: "e.g. Primary Sheet, Production Support...",
                                        value: current_profile_name.read().clone(),
                                        oninput: move |e: FormEvent| {
                                            let val = e.value();
                                            current_profile_name.set(val.clone());
                                            let cur_id = active_profile.read().clone();
                                            let mut list = sheet_sources.write();
                                            if let Some(item) = list.iter_mut().find(|s| s.id == cur_id) {
                                                item.name = val;
                                            }
                                        },
                                        class: "font-medium text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary",
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Custom name to identify this Google Sheet link in the sidebar selector and tabs."
                                    }
                                }

                                // Base URL
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Google Apps Script Web App URL" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "TMS_API_BASE_URL" }
                                    }
                                    div { class: "flex items-center gap-2",
                                        Input {
                                            r#type: "text",
                                            placeholder: "https://script.google.com/macros/s/AKfycb.../exec",
                                            value: "{base_url}",
                                            oninput: move |e: FormEvent| {
                                                let val = e.value();
                                                base_url.set(val.clone());
                                                let cur_id = active_profile.read().clone();
                                                let mut list = sheet_sources.write();
                                                if let Some(item) = list.iter_mut().find(|s| s.id == cur_id) {
                                                    item.base_url = val;
                                                }
                                            },
                                            class: "font-mono text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary flex-1",
                                        }
                                        Button {
                                            variant: "outline",
                                            class: "h-9 text-xs px-3 shadow-2xs gap-1.5 shrink-0 cursor-pointer",
                                            onclick: handle_open_sheet_browser,
                                            title: "Open script endpoint in default external browser",
                                            ExternalLinkIcon { class: "size-3.5" }
                                            "Open Sheet"
                                        }
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Paste the Web App URL deployed from your Google Apps Script editor (must end in /exec)."
                                    }
                                }

                                // API Protocol
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "API Protocol" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "TMS_API_PROTOCOL" }
                                    }
                                    select {
                                        value: "{api_protocol}",
                                        onchange: move |e: FormEvent| {
                                            let val = e.value();
                                            api_protocol.set(val.clone());
                                            let cur_id = active_profile.read().clone();
                                            let mut list = sheet_sources.write();
                                            if let Some(item) = list.iter_mut().find(|s| s.id == cur_id) {
                                                item.protocol = val.clone();
                                            }
                                            let _ = crate::db::save_system_api_protocol(&val);
                                        },
                                        class: "w-full font-mono text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary focus:outline-none focus:ring-1 focus:ring-primary/20 transition-colors",
                                        option { value: "auto", "Auto-detect (recommended)" }
                                        option { value: "rest", "REST API (Direct / Query parameters)" }
                                        option { value: "v1", "JSON-RPC Envelope (Standard v1)" }
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Select 'Auto-detect' to automatically select the protocol supported by your Google Apps Script endpoint."
                                    }
                                }

                                // API Key
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "API Secret Key (Optional)" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "TMS_API_KEY" }
                                    }
                                    div { class: "relative flex items-center",
                                        input {
                                            r#type: if *show_key.read() { "text" } else { "password" },
                                            placeholder: "Enter API authentication key (optional for unauthenticated REST)",
                                            value: "{api_key}",
                                            oninput: move |e: FormEvent| {
                                                let val = e.value();
                                                api_key.set(val.clone());
                                                let cur_id = active_profile.read().clone();
                                                let mut list = sheet_sources.write();
                                                if let Some(item) = list.iter_mut().find(|s| s.id == cur_id) {
                                                    item.api_key = val;
                                                }
                                            },
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
                                        "Leave blank if your Google Apps Script Web App does not require an API key."
                                    }
                                }

                                // Employee Code
                                div { class: "flex flex-col gap-1.5",
                                    label { class: "text-xs font-semibold text-foreground flex items-center justify-between",
                                        span { "Default Employee Code" }
                                        span { class: "text-[10px] text-muted-foreground font-mono", "TMS_EMPLOYEE_CODE" }
                                    }
                                    Input {
                                        r#type: "text",
                                        placeholder: "e.g. 10081",
                                        value: "{employee_code}",
                                        oninput: move |e: FormEvent| employee_code.set(e.value()),
                                        class: "font-mono text-xs h-9 rounded-md border border-input bg-background px-3 focus:border-primary",
                                    }
                                    span { class: "text-[11px] text-muted-foreground",
                                        "Your company employee code used for ticket authorship and reminder notifications."
                                    }
                                }

                                div { class: "border-t border-border/50" }

                                // HTTP Network Request Timeout
                                SettingRow {
                                    title: "HTTP Request Timeout",
                                    description: "Maximum seconds to await response from Google Sheets before failing gracefully.",
                                    div { class: "flex gap-1.5 flex-wrap",
                                        for (val, label) in [
                                            ("10", "10s (Fast)"),
                                            ("15", "15s (Normal)"),
                                            ("25", "25s (Recommended)"),
                                            ("45", "45s (Slow net)"),
                                            ("60", "60s (Long)"),
                                        ] {
                                            {
                                                let is_active = *network_timeout.read() == val;
                                                let val_str = val.to_string();
                                                rsx! {
                                                    button {
                                                        key: "{val}",
                                                        r#type: "button",
                                                        class: format!(
                                                            "rounded-md px-2.5 py-1.5 text-xs font-medium border transition-all duration-150 cursor-pointer active:scale-95 {}",
                                                            if is_active { "bg-primary text-primary-foreground border-primary shadow-xs font-semibold" } else { "bg-card text-foreground border-border hover:border-primary/50" }
                                                        ),
                                                        onclick: {
                                                            let v = val_str.clone();
                                                            move |_| network_timeout.set(v.clone())
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Offline / Local Mode Toggle
                                SettingToggle {
                                    title: "Offline / Low-Bandwidth Mode",
                                    description: "Operate entirely from the local SQLite cache without triggering background HTTP polling.",
                                    checked: *work_offline.read(),
                                    on_toggle: move |_| {
                                        let next = !*work_offline.read();
                                        work_offline.set(next);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // Test Connection Box
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

                                // Universal Google Sheet Schema & CRUD Manager
                                div { class: "flex flex-col gap-3 rounded-xl border border-border/70 bg-muted/20 p-3.5",
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex items-center gap-2",
                                            span { class: "text-sm", "📊" }
                                            div { class: "flex flex-col",
                                                span { class: "text-xs font-semibold text-foreground", "Google Sheet Schema & Universal CRUD" }
                                                span { class: "text-[11px] text-muted-foreground", "Validate tabs (Tickets, Employees, Reminders, Notes) or auto-setup" }
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            Button {
                                                variant: "outline",
                                                class: "h-7 text-xs px-2.5 font-medium cursor-pointer shadow-2xs hover:bg-background",
                                                disabled: *is_verifying_schema.read() || *is_initializing_schema.read(),
                                                onclick: handle_verify_schema,
                                                if *is_verifying_schema.read() {
                                                    span { class: "flex items-center gap-1",
                                                        span { class: "size-3 border-2 border-primary border-t-transparent rounded-full animate-spin mr-1" }
                                                        "Checking…"
                                                    }
                                                }
                                                if !*is_verifying_schema.read() {
                                                    "Verify Schema"
                                                }
                                            }
                                            Button {
                                                variant: "default",
                                                class: "h-7 text-xs px-2.5 font-medium bg-blue-600 hover:bg-blue-700 text-white cursor-pointer shadow-2xs",
                                                disabled: *is_verifying_schema.read() || *is_initializing_schema.read(),
                                                onclick: handle_init_schema,
                                                if *is_initializing_schema.read() {
                                                    span { class: "flex items-center gap-1",
                                                        span { class: "size-3 border-2 border-white border-t-transparent rounded-full animate-spin mr-1" }
                                                        "Setting up…"
                                                    }
                                                }
                                                if !*is_initializing_schema.read() {
                                                    "Auto-Setup Sheet"
                                                }
                                            }
                                        }
                                    }

                                    if let Some(ref rep) = *schema_report.read() {
                                        div { class: "flex flex-col gap-2 rounded-lg border border-border/60 bg-background/50 p-2.5 text-xs animate-in fade-in-0",
                                            div { class: "flex items-center justify-between font-medium",
                                                span { class: if rep.valid { "text-emerald-500 font-semibold" } else { "text-amber-500 font-semibold" },
                                                    if rep.valid { "✓ All standard tabs and columns verified" } else { "⚠ Schema incomplete - click Auto-Setup to fix" }
                                                }
                                                if let Some(ref title) = rep.spreadsheet_title {
                                                    span { class: "text-[10px] text-muted-foreground truncate max-w-[180px]", "{title}" }
                                                }
                                            }
                                            div { class: "grid grid-cols-2 sm:grid-cols-4 gap-2 pt-1",
                                                for (name , val) in rep.sheets.iter() {
                                                    div { class: "p-2 rounded border border-border/40 bg-card flex flex-col gap-0.5",
                                                        div { class: "flex items-center justify-between",
                                                            span { class: "font-semibold text-foreground text-[11px]", "{name}" }
                                                            if val.exists && val.is_valid {
                                                                span { class: "text-emerald-500 text-[10px] font-bold", "✓ OK" }
                                                            } else if val.exists {
                                                                span { class: "text-amber-500 text-[10px] font-bold", "⚠ cols" }
                                                            } else {
                                                                span { class: "text-destructive text-[10px] font-bold", "✗ Missing" }
                                                            }
                                                        }
                                                        span { class: "text-[10px] text-muted-foreground", "{val.total_rows} rows" }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "text-[11px] text-muted-foreground bg-muted/40 rounded p-2 flex items-center justify-between border border-border/40",
                                        span { "Universal Apps Script backend script located at:" }
                                        code { class: "px-1.5 py-0.5 rounded bg-muted text-[10px] font-mono text-primary", "scripts/GoogleAppsScript_Universal_TMS.js" }
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

                        // ========================================================
                        // TAB 5: Sync & Data Management
                        // ========================================================
                        if *active_tab.read() == SettingsTab::SyncData {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // Auto Sync Toggle
                                SettingToggle {
                                    title: "Automatic Background Sync",
                                    description: "Continuously synchronize tickets, reminders, and employee records in the background.",
                                    checked: *auto_sync_enabled.read(),
                                    on_toggle: move |_| {
                                        let next = !*auto_sync_enabled.read();
                                        auto_sync_enabled.set(next);
                                    },
                                }

                                div { class: "border-t border-border/50" }

                                // Sync Interval
                                SettingRow {
                                    title: "Sync Cycle Cadence (seconds)",
                                    description: "Frequency of periodic polling to Google Sheets. Range: 5–300 seconds.",
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

                                div { class: "border-t border-border/50" }

                                // Sync Queue Status
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Sync Queue & Conflict Status" }
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

                                // Sync Actions row
                                div { class: "flex items-center gap-2 flex-wrap",
                                    Button {
                                        variant: "outline",
                                        class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5",
                                        onclick: handle_force_sync,
                                        RefreshIcon { class: "size-3.5" }
                                        "Force Sync Now"
                                    }
                                    Button {
                                        variant: "outline",
                                        class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5",
                                        onclick: handle_reset_sync_queue,
                                        AlertCircleIcon { class: "size-3.5" }
                                        "Retry Failed Ops"
                                    }
                                    Button {
                                        variant: "outline",
                                        class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5 hover:text-destructive",
                                        onclick: handle_clear_sync_errors_ui,
                                        title: "Clear sticky error messages from sync logs and status badge",
                                        TrashIcon { class: "size-3.5" }
                                        "Clear Error Cache"
                                    }
                                    Button {
                                        variant: "default",
                                        class: "h-8 text-xs px-3 font-semibold cursor-pointer shadow-xs gap-1.5 ml-auto",
                                        onclick: save_sync_settings,
                                        CheckIcon { class: "size-3.5" }
                                        "Save Sync Interval"
                                    }
                                }

                                div { class: "border-t border-border/50 my-1" }

                                // Google Sheet Changes & Values Log
                                {
                                    let sheet_changes_list = crate::db::get_recent_sheet_changes(8);
                                    let total_changes = sheet_changes_list.len();
                                    rsx! {
                                        div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                            div { class: "flex items-center justify-between",
                                                div { class: "flex flex-col gap-0.5",
                                                    div { class: "flex items-center gap-2",
                                                        span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Recent Google Sheet Changes" }
                                                        span { class: "text-[10px] font-mono px-1.5 py-0.5 rounded bg-primary/10 text-primary font-semibold", "{total_changes} recent" }
                                                    }
                                                    span { class: "text-[11px] text-muted-foreground", "Live values and field diffs captured when syncing from your Google Sheet." }
                                                }

                                                div { class: "flex items-center gap-1.5",
                                                    Button {
                                                        variant: "outline",
                                                        size: "sm",
                                                        class: "h-7 text-xs px-2.5 cursor-pointer shadow-2xs gap-1",
                                                        onclick: move |_| {
                                                            if let Some(mut sig) = open_changes {
                                                                sig.set(crate::state::OpenSheetChangesDialog(true));
                                                            }
                                                        },
                                                        "View All Details"
                                                    }
                                                }
                                            }

                                            if sheet_changes_list.is_empty() {
                                                div { class: "py-4 text-center text-xs text-muted-foreground bg-muted/20 rounded-lg border border-border/40",
                                                    "No remote changes logged yet. Any cell edits in Google Sheets will display their before and after values here."
                                                }
                                            } else {
                                                div { class: "flex flex-col gap-2 max-h-60 overflow-y-auto pr-1",
                                                    for item in sheet_changes_list.iter() {
                                                        div {
                                                            key: "{item.change_id}",
                                                            class: "flex items-center justify-between gap-3 p-2 rounded-lg bg-card border border-border/60 text-xs",
                                                            div { class: "flex items-center gap-2 min-w-0 flex-1",
                                                                span { class: "font-mono font-bold text-[11px] text-foreground shrink-0", "{item.entity_id}" }
                                                                span { class: "text-[11px] text-muted-foreground truncate max-w-[140px]", "{item.entity_title}" }
                                                                span { class: "rounded bg-muted px-1.5 py-0.5 text-[10px] font-semibold text-muted-foreground shrink-0", "{item.field_name}" }
                                                                span { class: "line-through text-rose-500 bg-rose-500/10 px-1.5 py-0.5 rounded text-[11px] font-mono shrink-0",
                                                                    if item.old_value.is_empty() { "—" } else { "{item.old_value}" }
                                                                }
                                                                span { class: "text-muted-foreground font-bold shrink-0", "→" }
                                                                span { class: "text-emerald-500 bg-emerald-500/10 px-1.5 py-0.5 rounded text-[11px] font-semibold font-mono shrink-0",
                                                                    if item.new_value.is_empty() { "—" } else { "{item.new_value}" }
                                                                }
                                                            }
                                                            span { class: "text-[10px] font-mono text-muted-foreground shrink-0", "{item.pulled_at}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50 my-1" }

                                // CSV Selective Data Exports
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    div { class: "flex flex-col gap-0.5",
                                        span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Selective CSV Data Exports" }
                                        span { class: "text-[11px] text-muted-foreground", "Download table-specific spreadsheets for reporting, auditing, or Excel import." }
                                    }
                                    div { class: "flex items-center gap-2.5 pt-1 flex-wrap",
                                        Button {
                                            variant: "outline",
                                            class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5 bg-card hover:bg-muted",
                                            onclick: handle_export_tickets_csv,
                                            svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" }
                                            }
                                            "Tickets CSV"
                                        }
                                        Button {
                                            variant: "outline",
                                            class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5 bg-card hover:bg-muted",
                                            onclick: handle_export_employees_csv,
                                            svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" }
                                            }
                                            "Employees CSV"
                                        }
                                        Button {
                                            variant: "outline",
                                            class: "h-8 text-xs px-3 cursor-pointer shadow-2xs gap-1.5 bg-card hover:bg-muted",
                                            onclick: handle_export_reminders_csv,
                                            svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" }
                                            }
                                            "Reminders CSV"
                                        }
                                    }
                                }

                                div { class: "border-t border-border/50 my-1" }

                                // Database Maintenance & Backups
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    div { class: "flex items-center justify-between",
                                        div { class: "flex flex-col gap-0.5",
                                            span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Database Maintenance & Backup" }
                                            span { class: "text-[11px] text-muted-foreground", "Export your full SQLite dataset or compact the database file on disk." }
                                        }
                                        span { class: "text-xs font-mono font-semibold text-primary px-2 py-0.5 rounded bg-primary/10", "{db_size_display}" }
                                    }
                                    div { class: "flex items-center gap-3 pt-1 flex-wrap",
                                        Button {
                                            variant: "outline",
                                            class: "h-8 text-xs px-3.5 cursor-pointer shadow-2xs gap-1.5 bg-card hover:bg-muted",
                                            onclick: handle_export_json,
                                            svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                path { d: "M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3" }
                                            }
                                            "Export Full Backup (JSON)"
                                        }
                                        Button {
                                            variant: "outline",
                                            class: "h-8 text-xs px-3.5 cursor-pointer shadow-2xs gap-1.5 bg-card hover:bg-muted",
                                            onclick: handle_optimize_db,
                                            svg { class: "size-3.5", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                                path { d: "M12 2v4M12 18v4M4.93 4.93l2.83 2.83M16.24 16.24l2.83 2.83M2 12h4M18 12h4M4.93 19.07l2.83-2.83M16.24 7.76l2.83-2.83" }
                                            }
                                            "Vacuum & Optimize Database"
                                        }
                                    }
                                }
                            }
                        }

                        // ========================================================
                        // TAB 6: About & Diagnostics
                        // ========================================================
                        if *active_tab.read() == SettingsTab::About {
                            div { class: "flex flex-col gap-5 animate-in fade-in-0 duration-150",

                                // App identity
                                div { class: "flex items-center gap-4 pb-4 border-b border-border/50",
                                    div { class: "flex size-14 items-center justify-center rounded-2xl bg-primary/10 text-primary shadow-xs",
                                        crate::icons::AppLogo { class: "size-8" }
                                    }
                                    div { class: "flex flex-col gap-1",
                                        h3 { class: "text-lg font-bold text-foreground", "BPSL IT Support" }
                                        span { class: "text-xs text-muted-foreground", "Enterprise Ticket & Reminder Management System" }
                                        div { class: "flex items-center gap-2 mt-1",
                                            span { class: "text-[10px] font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/60", "v1.1.0" }
                                            span { class: "text-[10px] font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/60", "Dioxus 0.7.1" }
                                            span { class: "text-[10px] font-mono text-muted-foreground bg-muted px-2 py-0.5 rounded-md border border-border/60", "Rust" }
                                        }
                                    }
                                }

                                // Database Statistics
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-3",
                                    div { class: "flex items-center justify-between",
                                        span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Database Entity Statistics" }
                                        span { class: "text-xs font-mono text-muted-foreground", "File size: {db_size_display}" }
                                    }
                                    div { class: "grid grid-cols-2 sm:grid-cols-4 gap-3",
                                        StatCard { label: "Tickets", value: "{ticket_count}", accent: "text-blue-600" }
                                        StatCard { label: "Reminders", value: "{reminder_count}", accent: "text-amber-600" }
                                        StatCard { label: "Employees", value: "{employee_count}", accent: "text-emerald-600" }
                                        StatCard { label: "Notes", value: "{note_count}", accent: "text-purple-600" }
                                    }
                                }

                                // Keyboard Shortcuts Reference
                                div { class: "rounded-xl border border-border/70 bg-muted/20 p-4 flex flex-col gap-2.5",
                                    span { class: "text-xs font-bold text-foreground uppercase tracking-wider", "Keyboard Shortcuts" }
                                    div { class: "grid grid-cols-1 sm:grid-cols-2 gap-2 text-xs",
                                        ShortcutRow { key_label: "Ctrl + ,", description: "Open Settings Dialog" }
                                        ShortcutRow { key_label: "Ctrl + D", description: "Toggle Light / Dark Theme" }
                                        ShortcutRow { key_label: "Ctrl + Shift + S", description: "Trigger Force Sync" }
                                        ShortcutRow { key_label: "Esc", description: "Dismiss Active Dialog" }
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
                                        label: "Operating System",
                                        value: std::env::consts::OS.to_string(),
                                        mono: false,
                                    }
                                    DiagRow {
                                        label: "CPU Architecture",
                                        value: std::env::consts::ARCH.to_string(),
                                        mono: false,
                                    }
                                }

                                // Credits
                                div { class: "text-center py-2",
                                    span { class: "text-[11px] text-muted-foreground",
                                        "BPSL IT Support Team • Fast, Offline-First, Powered by Rust & Dioxus"
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
            div { class: "flex flex-col gap-0.5 min-w-0 flex-1",
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
    let (track_bg, knob_bg, knob_pos) = if props.checked {
        ("bg-primary", "bg-primary-foreground", "translate-x-4")
    } else {
        ("bg-muted-foreground/30 dark:bg-muted/80 border border-border/60", "bg-white dark:bg-zinc-200", "translate-x-0.5")
    };

    rsx! {
        div { class: "flex items-center justify-between gap-4",
            div { class: "flex flex-col gap-0.5 min-w-0 flex-1",
                span { class: "text-sm font-semibold text-foreground", "{props.title}" }
                span { class: "text-[11px] text-muted-foreground leading-relaxed", "{props.description}" }
            }
            button {
                r#type: "button",
                role: "switch",
                "aria-checked": if props.checked { "true" } else { "false" },
                class: "relative inline-flex h-6 w-10 shrink-0 cursor-pointer items-center rounded-full transition-colors duration-200 ease-in-out focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary {track_bg}",
                onclick: move |e| props.on_toggle.call(e),
                span {
                    class: "pointer-events-none inline-block size-5 rounded-full {knob_bg} shadow-md border border-black/10 dark:border-white/10 ring-0 transition-all duration-200 ease-in-out {knob_pos}",
                }
            }
        }
    }
}

// ============================================================
// Shortcut Row
// ============================================================

#[derive(Props, Clone, PartialEq)]
struct ShortcutRowProps {
    key_label: &'static str,
    description: &'static str,
}

#[component]
fn ShortcutRow(props: ShortcutRowProps) -> Element {
    rsx! {
        div { class: "flex items-center justify-between p-2 rounded-lg bg-background border border-border/60",
            span { class: "text-muted-foreground text-[11px]", "{props.description}" }
            span { class: "px-2 py-0.5 rounded font-mono text-[10px] font-bold bg-muted border border-border/80 text-foreground",
                "{props.key_label}"
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
