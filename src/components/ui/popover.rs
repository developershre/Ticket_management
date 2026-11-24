#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct PopoverContext {
    pub open: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct PopoverProps {
    #[props(default)]
    pub open: Option<Signal<bool>>,

    pub children: Element,
}

#[component]
pub fn Popover(props: PopoverProps) -> Element {
    let internal_open = use_signal(|| false);
    let open = props.open.unwrap_or(internal_open);

    use_context_provider(|| PopoverContext { open });

    rsx! {
        div {
            "data-slot": "popover",
            class: "relative inline-block",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PopoverTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn PopoverTrigger(props: PopoverTriggerProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();

    rsx! {
        div {
            "data-slot": "popover-trigger",
            class: "inline-flex cursor-pointer {props.class}",
            onclick: move |e| {
                e.stop_propagation();
                let now = (ctx.open)();
                ctx.open.set(!now);
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PopoverContentProps {
    #[props(default = "bottom".to_string())]
    pub side: String,

    #[props(default = "center".to_string())]
    pub align: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn PopoverContent(props: PopoverContentProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();
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

    let align_cls = match props.align.as_str() {
        "start" => "left-0",
        "end" => "right-0",
        _ => "left-1/2 -translate-x-1/2",
    };

    let class = format!(
        "absolute z-50 w-72 rounded-md border border-border bg-popover p-4 text-popover-foreground shadow-md outline-none animate-in fade-in-0 zoom-in-95 backdrop-blur-sm ring-1 ring-foreground/5 {} {} {}",
        side_cls,
        align_cls,
        props.class
    );

    rsx! {
        div { class: "contents",
            // Backdrop dismiss overlay
            div {
                class: "fixed inset-0 z-40 bg-transparent",
                onclick: move |e| {
                    e.stop_propagation();
                    ctx.open.set(false);
                },
            }
            div {
                "data-slot": "popover-content",
                role: "dialog",
                class: class,
                onclick: move |e| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct PopoverCloseProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn PopoverClose(props: PopoverCloseProps) -> Element {
    let mut ctx = use_context::<PopoverContext>();

    rsx! {
        div {
            "data-slot": "popover-close",
            class: "inline-flex cursor-pointer {props.class}",
            onclick: move |e| {
                e.stop_propagation();
                ctx.open.set(false);
            },
            {props.children}
        }
    }
}
