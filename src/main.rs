use dioxus::prelude::*;

mod components;
mod icons;
mod pages;
mod state;

use components::app_sidebar::AppSidebar;
use components::fab::Fab;
use components::titlebar::Titlebar;
use components::ui::sidebar::SidebarProvider;
use components::ui::toast::ToastProvider;
use pages::calendar_page::CalendarPage;
use pages::reminders::Reminders;
use pages::tickets::Tickets;

/// All app routes.
///
/// There is intentionally no default dioxus navbar: `AppShell` renders a
/// completely custom chrome (custom draggable titlebar, sidebar navigation,
/// FAB) around the router outlet.
#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Tickets {},
    #[route("/reminders")]
    Reminders {},
    #[route("/calendar")]
    CalendarPage {},
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

fn main() {
    // Desktop: the native window frame (navbar + OS action buttons) is
    // disabled entirely — the custom `Titlebar` component provides the
    // drag region and the minimize/maximize/close buttons instead.
    #[cfg(feature = "desktop")]
    {
        use components::ui::theme::fetch_system_is_dark;
        use dioxus::desktop::tao::window::Theme;
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        let is_sys_dark = fetch_system_is_dark();
        let initial_theme = if is_sys_dark {
            Theme::Dark
        } else {
            Theme::Light
        };

        dioxus::LaunchBuilder::new()
            .with_cfg(
                Config::new().with_window(
                    WindowBuilder::new()
                        .with_title("TaskFlow")
                        .with_decorations(false)
                        .with_theme(Some(initial_theme))
                        .with_inner_size(LogicalSize::new(1200.0, 800.0))
                        .with_min_inner_size(LogicalSize::new(940.0, 600.0)),
                ),
            )
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(App);
    }
}

use components::ui::theme::{use_theme, ThemeProvider};

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Title { "TaskFlow" }
        ThemeProvider {
            Router::<Route> {}
        }
    }
}

/// App shell: custom titlebar + sidebar + page outlet + FAB.
#[component]
fn AppShell() -> Element {
    let theme = use_theme();
    let dark_cls = if theme.is_dark() { "dark" } else { "" };

    rsx! {
        ToastProvider {
            SidebarProvider {
                div { class: "{dark_cls} flex h-screen w-full flex-col overflow-hidden bg-background text-foreground transition-colors duration-200",
                    Titlebar { app_name: "TaskFlow".to_string() }
                    div { class: "flex flex-1 overflow-hidden",
                        AppSidebar {}
                        main { class: "relative flex-1 overflow-y-auto",
                            Outlet::<Route> {}
                        }
                    }
                    Fab {}
                }
            }
        }
    }
}
