#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct NavMenuContext {
    active_item: Signal<Option<String>>,
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn NavigationMenu(props: NavigationMenuProps) -> Element {
    let active_item = use_signal(|| None);
    use_context_provider(|| NavMenuContext { active_item });

    let class = format!(
        "relative z-10 flex max-w-max flex-1 items-center justify-center {}",
        props.class
    );

    rsx! {
        nav {
            "data-slot": "navigation-menu",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuListProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn NavigationMenuList(props: NavigationMenuListProps) -> Element {
    let class = format!(
        "group flex flex-1 list-none items-center justify-center space-x-1 {}",
        props.class
    );

    rsx! {
        ul {
            "data-slot": "navigation-menu-list",
            class: class,
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq)]
struct NavMenuItemContext {
    id: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuItemProps {
    pub id: String,
    pub children: Element,
}

#[component]
pub fn NavigationMenuItem(props: NavigationMenuItemProps) -> Element {
    let id = props.id.clone();
    use_context_provider(|| NavMenuItemContext { id });

    rsx! {
        li {
            "data-slot": "navigation-menu-item",
            class: "relative",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn NavigationMenuTrigger(props: NavigationMenuTriggerProps) -> Element {
    let mut nav_ctx = use_context::<NavMenuContext>();
    let item_ctx = use_context::<NavMenuItemContext>();

    let is_open = (nav_ctx.active_item)().as_deref() == Some(&item_ctx.id);
    let item_id = item_ctx.id.clone();
    let chevron_rotate = if is_open { "rotate-180" } else { "rotate-0" };

    let open_cls = if is_open {
        "bg-accent/50 text-accent-foreground"
    } else {
        ""
    };

    let class = format!(
        "group inline-flex h-9 w-max items-center justify-center rounded-md bg-background px-3 py-1.5 text-xs font-medium hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground transition-colors cursor-pointer select-none {} {}",
        open_cls,
        props.class
    );

    rsx! {
        button {
            "data-slot": "navigation-menu-trigger",
            r#type: "button",
            "aria-expanded": if is_open { "true" } else { "false" },
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if is_open {
                    nav_ctx.active_item.set(None);
                } else {
                    nav_ctx.active_item.set(Some(item_id.clone()));
                }
            },
            span { {props.children} }
            svg {
                class: "ml-1 size-3 transition-transform duration-200 text-muted-foreground {chevron_rotate}",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuContentProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn NavigationMenuContent(props: NavigationMenuContentProps) -> Element {
    let mut nav_ctx = use_context::<NavMenuContext>();
    let item_ctx = use_context::<NavMenuItemContext>();

    let is_open = (nav_ctx.active_item)().as_deref() == Some(&item_ctx.id);

    if !is_open {
        return rsx! {};
    }

    let class = format!(
        "absolute left-0 top-full z-50 w-64 md:w-80 mt-1.5 rounded-md border border-border bg-popover p-3 text-popover-foreground shadow-md backdrop-blur-sm animate-in fade-in-0 zoom-in-95 {}",
        props.class
    );

    rsx! {
        div { class: "contents",
            div {
                class: "fixed inset-0 z-40 bg-transparent",
                onclick: move |e| {
                    e.stop_propagation();
                    nav_ctx.active_item.set(None);
                },
            }
            div {
                "data-slot": "navigation-menu-content",
                class: class,
                onclick: move |e| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct NavigationMenuLinkProps {
    #[props(default)]
    pub href: Option<String>,

    #[props(default)]
    pub class: String,

    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn NavigationMenuLink(props: NavigationMenuLinkProps) -> Element {
    let class = format!(
        "block select-none rounded-md p-2 text-xs leading-none no-underline outline-none transition-colors hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground cursor-pointer {}",
        props.class
    );

    if let Some(href) = &props.href {
        rsx! {
            a {
                "data-slot": "navigation-menu-link",
                href: "{href}",
                class: class,
                onclick: move |e| {
                    if let Some(handler) = &props.onclick {
                        handler.call(e);
                    }
                },
                {props.children}
            }
        }
    } else {
        rsx! {
            span {
                "data-slot": "navigation-menu-link",
                class: class,
                onclick: move |e| {
                    if let Some(handler) = &props.onclick {
                        handler.call(e);
                    }
                },
                {props.children}
            }
        }
    }
}
