#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct PaginationProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Pagination(props: PaginationProps) -> Element {
    let class = format!("mx-auto flex w-full justify-center {}", props.class);

    rsx! {
        nav {
            "data-slot": "pagination",
            role: "navigation",
            "aria-label": "pagination",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PaginationContentProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn PaginationContent(props: PaginationContentProps) -> Element {
    let class = format!("flex flex-row items-center gap-1 {}", props.class);

    rsx! {
        ul {
            "data-slot": "pagination-content",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PaginationItemProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn PaginationItem(props: PaginationItemProps) -> Element {
    rsx! {
        li {
            "data-slot": "pagination-item",
            class: "{props.class}",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PaginationLinkProps {
    #[props(default = false)]
    pub is_active: bool,

    #[props(default = "default".to_string())]
    pub size: String,

    #[props(default)]
    pub href: Option<String>,

    #[props(default)]
    pub class: String,

    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn PaginationLink(props: PaginationLinkProps) -> Element {
    let active_cls = if props.is_active {
        "border border-input bg-background shadow-xs hover:bg-accent hover:text-accent-foreground font-semibold"
    } else {
        "hover:bg-accent hover:text-accent-foreground text-muted-foreground"
    };

    let size_cls = match props.size.as_str() {
        "sm" => "h-8 px-2 text-xs",
        "lg" => "h-10 px-4",
        "icon" => "size-9 p-0",
        _ => "h-9 w-9 p-0 text-xs",
    };

    let class = format!(
        "inline-flex items-center justify-center rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring select-none cursor-pointer {} {} {}",
        size_cls,
        active_cls,
        props.class
    );

    if let Some(href) = &props.href {
        rsx! {
            a {
                "data-slot": "pagination-link",
                "aria-current": if props.is_active { "page" } else { "false" },
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
            button {
                "data-slot": "pagination-link",
                r#type: "button",
                "aria-current": if props.is_active { "page" } else { "false" },
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

#[derive(Props, Clone, PartialEq)]
pub struct PaginationPreviousProps {
    #[props(default)]
    pub href: Option<String>,

    #[props(default)]
    pub class: String,

    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn PaginationPrevious(props: PaginationPreviousProps) -> Element {
    let class = format!(
        "inline-flex items-center gap-1 h-9 px-3 text-xs rounded-md font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "pagination-previous",
            r#type: "button",
            "aria-label": "Go to previous page",
            class: class,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            svg {
                class: "size-3.5",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                polyline { points: "15 18 9 12 15 6" }
            }
            span { "Previous" }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PaginationNextProps {
    #[props(default)]
    pub href: Option<String>,

    #[props(default)]
    pub class: String,

    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn PaginationNext(props: PaginationNextProps) -> Element {
    let class = format!(
        "inline-flex items-center gap-1 h-9 px-3 text-xs rounded-md font-medium text-muted-foreground hover:bg-accent hover:text-accent-foreground transition-colors cursor-pointer {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "pagination-next",
            r#type: "button",
            "aria-label": "Go to next page",
            class: class,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            span { "Next" }
            svg {
                class: "size-3.5",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                polyline { points: "9 18 15 12 9 6" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PaginationEllipsisProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn PaginationEllipsis(props: PaginationEllipsisProps) -> Element {
    let class = format!("flex size-9 items-center justify-center text-muted-foreground {}", props.class);

    rsx! {
        span {
            "data-slot": "pagination-ellipsis",
            "aria-hidden": "true",
            class: class,
            svg {
                class: "size-4",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                circle { cx: "12", cy: "12", r: "1" }
                circle { cx: "19", cy: "12", r: "1" }
                circle { cx: "5", cy: "12", r: "1" }
            }
            span { class: "sr-only", "More pages" }
        }
    }
}
