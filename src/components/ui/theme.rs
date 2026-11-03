//! Theme management system for BPSL IT Support.
//!
//! Fetches the system theme directly from the operating system (GNOME/FreeDesktop,
//! KDE, macOS, Windows, and Tao desktop window) and applies it properly to the
//! native desktop window, DOM `<html>`/`<body>` classes, and Dioxus reactive UI tree.

use dioxus::prelude::*;

use crate::components::ui::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use crate::icons::{MonitorIcon, MoonIcon, SunIcon};

// ============================================================
// Direct System Theme Detection
// ============================================================

/// Queries the host operating system directly (bypassing browser media queries)
/// to detect whether the user's OS is currently configured for dark mode.
pub fn fetch_system_is_dark() -> bool {
    // ----------------------------------------------------
    // Linux desktop detection (GNOME, FreeDesktop portal, KDE, GTK)
    // ----------------------------------------------------
    #[cfg(target_os = "linux")]
    {
        // 1. GNOME / FreeDesktop portal standard dark mode preference
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "color-scheme"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("prefer-dark") {
                    return true;
                }
                if stdout.contains("prefer-light") {
                    return false;
                }
            }
        }

        // 2. GTK theme name check (e.g. Adwaita-dark, Yaru-dark, etc.)
        if let Ok(output) = std::process::Command::new("gsettings")
            .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
                if stdout.contains("dark") || stdout.contains("black") {
                    return true;
                }
            }
        }

        // 3. KDE Plasma check via kreadconfig6 / kreadconfig5
        for cmd in ["kreadconfig6", "kreadconfig5"] {
            if let Ok(output) = std::process::Command::new(cmd)
                .args(["--group", "General", "--key", "ColorScheme"])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_lowercase();
                    if stdout.contains("dark") || stdout.contains("black") {
                        return true;
                    }
                }
            }
        }

        // 4. Tao desktop window fallback (safe if called within Dioxus runtime)
        #[cfg(feature = "desktop")]
        {
            if let Ok(dark) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                matches!(
                    dioxus::desktop::window().theme(),
                    dioxus::desktop::tao::window::Theme::Dark
                )
            })) {
                if dark {
                    return true;
                }
            }
        }
    }

    // ----------------------------------------------------
    // macOS native detection
    // ----------------------------------------------------
    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleInterfaceStyle"])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.trim().eq_ignore_ascii_case("dark") {
                    return true;
                }
                return false;
            }
        }
    }

    // ----------------------------------------------------
    // Windows native detection
    // ----------------------------------------------------
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let mut cmd = std::process::Command::new("reg");
        cmd.creation_flags(CREATE_NO_WINDOW);
        if let Ok(output) = cmd
            .args([
                "query",
                r"HKCU\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "/v",
                "AppsUseLightTheme",
            ])
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if stdout.contains("0x0") {
                    return true;
                }
                if stdout.contains("0x1") {
                    return false;
                }
            }
        }
    }

    // ----------------------------------------------------
    // Cross-platform desktop fallback (safe if called within Dioxus runtime)
    // ----------------------------------------------------
    #[cfg(all(feature = "desktop", not(target_os = "linux")))]
    {
        if let Ok(dark) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            matches!(
                dioxus::desktop::window().theme(),
                dioxus::desktop::tao::window::Theme::Dark
            )
        })) {
            if dark {
                return true;
            }
        }
    }

    false
}

// ============================================================
// Theme Mode
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    Light,
    Dark,
    #[default]
    System,
}

impl ThemeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
            ThemeMode::System => "system",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            ThemeMode::Light => "Light",
            ThemeMode::Dark => "Dark",
            ThemeMode::System => "System",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }

    /// Cycles Light -> Dark -> System -> Light
    #[allow(dead_code)]
    pub fn next(&self) -> Self {
        match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
            ThemeMode::System => ThemeMode::Light,
        }
    }
}

// ============================================================
// Theme Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
pub struct ThemeContext {
    pub mode: Signal<ThemeMode>,
    pub system_dark: Signal<bool>,
}

impl ThemeContext {
    pub fn mode(&self) -> ThemeMode {
        *self.mode.read()
    }

    /// Returns whether the effective active theme is dark mode.
    pub fn is_dark(&self) -> bool {
        match self.mode() {
            ThemeMode::Light => false,
            ThemeMode::Dark => true,
            ThemeMode::System => *self.system_dark.read(),
        }
    }

    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.mode.set(mode);
    }

    /// Quickly flips between Light and Dark modes.
    pub fn toggle(&mut self) {
        if self.is_dark() {
            self.set_mode(ThemeMode::Light);
        } else {
            self.set_mode(ThemeMode::Dark);
        }
    }

    /// Cycles through Light -> Dark -> System.
    #[allow(dead_code)]
    pub fn cycle(&mut self) {
        let next = self.mode().next();
        self.set_mode(next);
    }
}

/// Consumes the nearest `ThemeContext`.
pub fn use_theme() -> ThemeContext {
    use_context::<ThemeContext>()
}

// ============================================================
// Theme Provider
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ThemeProviderProps {
    #[props(default)]
    pub default_mode: ThemeMode,
    pub children: Element,
}

#[component]
pub fn ThemeProvider(props: ThemeProviderProps) -> Element {
    // 1. Fetch system theme DIRECTLY from the operating system synchronously at launch
    let initial_sys_dark = fetch_system_is_dark();
    let mut system_dark = use_signal(move || initial_sys_dark);
    let mut mode = use_signal(|| props.default_mode);

    let ctx = ThemeContext { mode, system_dark };
    use_context_provider(|| ctx);

    // Initial load: check localStorage for any user explicit choice ('light', 'dark', 'system')
    use_hook(|| {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                const saved = localStorage.getItem('bpsl-theme') || localStorage.getItem('taskflow-theme') || '';
                dioxus.send(saved);
                "#,
            );

            if let Ok(saved) = eval.recv::<String>().await {
                if !saved.is_empty() {
                    let m = ThemeMode::from_str_lossy(&saved);
                    mode.set(m);
                }
            }
        });
    });

    // Background listener: poll OS system theme gracefully every 30 seconds so if the user
    // switches their OS theme, the app adapts without consuming background CPU.
    #[cfg(feature = "desktop")]
    use_hook(|| {
        spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let current_sys_dark = fetch_system_is_dark();
                if *system_dark.peek() != current_sys_dark {
                    system_dark.set(current_sys_dark);
                }
            }
        });
    });

    // Reactive sync: apply theme properly to native desktop window & DOM
    use_effect(move || {
        let is_dark = ctx.is_dark();
        let mode_str = ctx.mode().as_str();

        // 1. Update native desktop window theme in Tao
        #[cfg(feature = "desktop")]
        {
            let w = dioxus::desktop::window();
            let tao_theme = if is_dark {
                dioxus::desktop::tao::window::Theme::Dark
            } else {
                dioxus::desktop::tao::window::Theme::Light
            };
            w.set_theme(Some(tao_theme));
        }

        // 2. Update HTML/DOM documentElement & body classes and colorScheme, plus persist
        let js = format!(
            r#"
            (function() {{
                const dark = {is_dark};
                const mode = "{mode_str}";
                try {{
                    localStorage.setItem("bpsl-theme", mode);
                }} catch (e) {{}}

                const root = document.documentElement;
                const body = document.body;
                if (dark) {{
                    root.classList.add("dark");
                    if (body) body.classList.add("dark");
                    root.style.colorScheme = "dark";
                }} else {{
                    root.classList.remove("dark");
                    if (body) body.classList.remove("dark");
                    root.style.colorScheme = "light";
                }}
            }})();
            "#
        );
        let _ = document::eval(&js);
    });

    rsx! {
        {props.children}
    }
}

// ============================================================
// Interactive Theme Controls
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ThemeToggleProps {
    #[props(default)]
    pub class: String,
    #[props(default = false)]
    pub show_label: bool,
}

/// A compact theme toggle button.
/// Clicking it toggles between Light and Dark mode.
#[component]
pub fn ThemeToggle(props: ThemeToggleProps) -> Element {
    let mut theme = use_theme();
    let is_dark = theme.is_dark();
    let mode = theme.mode();

    let tooltip_text = format!("Theme: {} (Click to switch)", mode.label());

    rsx! {
        Tooltip {
            TooltipTrigger {
                button {
                    r#type: "button",
                    title: "{tooltip_text}",
                    class: "group relative inline-flex size-8 items-center justify-center rounded-md \
                            border border-border/60 bg-background/80 text-foreground/80 \
                            hover:bg-accent hover:text-accent-foreground \
                            focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring \
                            transition-all duration-200 cursor-pointer {props.class}",
                    onclick: move |e| {
                        e.stop_propagation();
                        theme.toggle();
                    },
                    if is_dark {
                        SunIcon { class: "size-4 text-foreground transition-transform duration-300 group-hover:rotate-45" }
                    } else {
                        MoonIcon { class: "size-4 text-foreground transition-transform duration-300 group-hover:-rotate-12" }
                    }
                    if props.show_label {
                        span { class: "ml-2 text-xs font-medium", "{mode.label()}" }
                    }
                }
            }
            TooltipContent {
                p { "{tooltip_text}" }
            }
        }
    }
}

/// Segmented 3-option control (Light / Dark / System).
#[derive(Props, Clone, PartialEq)]
pub struct ThemeSegmentedProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn ThemeSegmented(props: ThemeSegmentedProps) -> Element {
    let mut theme = use_theme();
    let active_mode = theme.mode();

    let item_btn = |mode: ThemeMode, label: &'static str, icon: Element| {
        let is_active = active_mode == mode;
        let active_cls = if is_active {
            "bg-background text-foreground font-semibold"
        } else {
            "text-muted-foreground hover:text-foreground hover:bg-background/50 font-normal"
        };

        rsx! {
            button {
                r#type: "button",
                class: "flex flex-1 items-center justify-center gap-1.5 rounded-md px-2.5 py-1 text-xs \
                        transition-all duration-150 cursor-pointer outline-none {active_cls}",
                onclick: move |e| {
                    e.stop_propagation();
                    theme.set_mode(mode);
                },
                {icon}
                span { "{label}" }
            }
        }
    };

    rsx! {
        div {
            class: "inline-flex items-center gap-1 rounded-lg border border-border/60 bg-muted/60 p-0.5 text-muted-foreground {props.class}",
            {item_btn(ThemeMode::Light, "Light", rsx! { SunIcon { class: "size-3.5" } })}
            {item_btn(ThemeMode::Dark, "Dark", rsx! { MoonIcon { class: "size-3.5" } })}
            {item_btn(ThemeMode::System, "System", rsx! { MonitorIcon { class: "size-3.5" } })}
        }
    }
}
