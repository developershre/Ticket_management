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
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        dioxus::LaunchBuilder::new()
            .with_cfg(
                Config::new().with_window(
                    WindowBuilder::new()
                        .with_title("TaskFlow")
                        .with_decorations(false)
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

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        document::Title { "TaskFlow" }
        Router::<Route> {}
    }
}

/// App shell: custom titlebar + sidebar + page outlet + FAB.
#[component]
fn AppShell() -> Element {
    rsx! {
        ToastProvider {
            SidebarProvider {
                div { class: "flex h-screen w-full flex-col overflow-hidden bg-background text-foreground",
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
