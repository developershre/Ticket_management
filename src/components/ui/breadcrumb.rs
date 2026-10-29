use dioxus::prelude::*;

// ============================================================
// Breadcrumb Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Breadcrumb(props: BreadcrumbProps) -> Element {
    rsx! {
        nav {
            "data-slot": "breadcrumb",
            "aria-label": "breadcrumb",
            class: props.class,
            {props.children}
        }
    }
}

// ============================================================
// Breadcrumb List
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbListProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn BreadcrumbList(props: BreadcrumbListProps) -> Element {
    let class = format!(
        "flex flex-wrap items-center gap-1.5 break-words text-sm text-muted-foreground sm:gap-2.5 list-none p-0 m-0 {}",
        props.class
    );

    rsx! {
        ol {
            "data-slot": "breadcrumb-list",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Breadcrumb Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbItemProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn BreadcrumbItem(props: BreadcrumbItemProps) -> Element {
    let class = format!("inline-flex items-center gap-1.5 {}", props.class);

    rsx! {
        li {
            "data-slot": "breadcrumb-item",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Breadcrumb Link
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbLinkProps {
    #[props(default)]
    pub href: String,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn BreadcrumbLink(props: BreadcrumbLinkProps) -> Element {
    let class = format!(
        "transition-colors hover:text-foreground cursor-pointer {}",
        props.class
    );

    rsx! {
        a {
            "data-slot": "breadcrumb-link",
            href: props.href,
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

// ============================================================
// Breadcrumb Page
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbPageProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn BreadcrumbPage(props: BreadcrumbPageProps) -> Element {
    let class = format!("font-normal text-foreground {}", props.class);

    rsx! {
        span {
            "data-slot": "breadcrumb-page",
            role: "link",
            "aria-disabled": "true",
            "aria-current": "page",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Breadcrumb Separator
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbSeparatorProps {
    #[props(default)]
    pub class: String,

    pub children: Option<Element>,
}

#[component]
pub fn BreadcrumbSeparator(props: BreadcrumbSeparatorProps) -> Element {
    let class = format!("size-3.5 text-muted-foreground/60 {}", props.class);

    rsx! {
        li {
            "data-slot": "breadcrumb-separator",
            role: "presentation",
            "aria-hidden": "true",
            class: "inline-flex items-center",
            if let Some(ch) = props.children {
                {ch}
            } else {
                svg {
                    class: class,
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "2",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M9 18l6-6-6-6" }
                }
            }
        }
    }
}

// ============================================================
// Breadcrumb Ellipsis
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct BreadcrumbEllipsisProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn BreadcrumbEllipsis(props: BreadcrumbEllipsisProps) -> Element {
    rsx! {
        span {
            "data-slot": "breadcrumb-ellipsis",
            role: "presentation",
            "aria-hidden": "true",
            class: "flex size-6 items-center justify-center text-muted-foreground {props.class}",
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
            span { class: "sr-only", "More" }
        }
    }
}
