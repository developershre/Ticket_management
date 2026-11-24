#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct HoverCardContext {
    open: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct HoverCardProps {
    pub children: Element,
}

#[component]
pub fn HoverCard(props: HoverCardProps) -> Element {
    let open = use_signal(|| false);
    use_context_provider(|| HoverCardContext { open });

    rsx! {
        div {
            "data-slot": "hover-card",
            class: "relative inline-block",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct HoverCardTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn HoverCardTrigger(props: HoverCardTriggerProps) -> Element {
    let mut ctx = use_context::<HoverCardContext>();

    rsx! {
        div {
            "data-slot": "hover-card-trigger",
            class: "inline-flex cursor-pointer {props.class}",
            onmouseenter: move |_| ctx.open.set(true),
            onmouseleave: move |_| ctx.open.set(false),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct HoverCardContentProps {
    #[props(default = "bottom".to_string())]
    pub side: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn HoverCardContent(props: HoverCardContentProps) -> Element {
    let mut ctx = use_context::<HoverCardContext>();
    let open = (ctx.open)();

    if !open {
        return rsx! {};
    }

    let side_cls = match props.side.as_str() {
        "top" => "bottom-full mb-2",
        "left" => "right-full mr-2 top-0",
        "right" => "left-full ml-2 top-0",
        _ => "top-full mt-2",
    };

    let class = format!(
        "absolute z-50 w-64 rounded-md border border-border bg-popover p-4 text-popover-foreground shadow-md outline-none animate-in fade-in-0 zoom-in-95 left-0 {} {}",
        side_cls,
        props.class
    );

    rsx! {
        div {
            "data-slot": "hover-card-content",
            class: class,
            onmouseenter: move |_| ctx.open.set(true),
            onmouseleave: move |_| ctx.open.set(false),
            {props.children}
        }
    }
}
