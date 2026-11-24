#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct SheetContext {
    pub open: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetProps {
    #[props(default)]
    pub open: Option<Signal<bool>>,

    pub children: Element,
}

#[component]
pub fn Sheet(props: SheetProps) -> Element {
    let internal_open = use_signal(|| false);
    let open = props.open.unwrap_or(internal_open);

    use_context_provider(|| SheetContext { open });

    rsx! {
        div {
            "data-slot": "sheet",
            class: "contents",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SheetTrigger(props: SheetTriggerProps) -> Element {
    let mut ctx = use_context::<SheetContext>();

    rsx! {
        div {
            "data-slot": "sheet-trigger",
            class: "inline-flex cursor-pointer {props.class}",
            onclick: move |e| {
                e.stop_propagation();
                ctx.open.set(true);
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetContentProps {
    #[props(default = "right".to_string())]
    pub side: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SheetContent(props: SheetContentProps) -> Element {
    let mut ctx = use_context::<SheetContext>();
    let open = (ctx.open)();

    if !open {
        return rsx! {};
    }

    let side_cls = match props.side.as_str() {
        "top" => "inset-x-0 top-0 border-b animate-in slide-in-from-top-full duration-300",
        "bottom" => "inset-x-0 bottom-0 border-t animate-in slide-in-from-bottom-full duration-300",
        "left" => "inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm animate-in slide-in-from-left-full duration-300",
        _ => "inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm animate-in slide-in-from-right-full duration-300",
    };

    let class = format!(
        "fixed z-50 gap-4 bg-background p-6 shadow-xl transition ease-in-out border-border flex flex-col {} {}",
        side_cls,
        props.class
    );

    rsx! {
        div { class: "contents",
            // Backdrop Overlay
            div {
                class: "fixed inset-0 z-50 bg-black/60 backdrop-blur-xs animate-in fade-in-0 duration-200",
                onclick: move |e| {
                    e.stop_propagation();
                    ctx.open.set(false);
                },
            }
            div {
                "data-slot": "sheet-content",
                role: "dialog",
                class: class,
                onclick: move |e| e.stop_propagation(),
                // Close button top right
                button {
                    r#type: "button",
                    class: "absolute right-4 top-4 rounded-sm opacity-70 transition-opacity hover:opacity-100 focus:outline-none focus:ring-1 focus:ring-ring cursor-pointer",
                    onclick: move |_| ctx.open.set(false),
                    svg {
                        class: "size-4",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        line { x1: "18", y1: "6", x2: "6", y2: "18" }
                        line { x1: "6", y1: "6", x2: "18", y2: "18" }
                    }
                }
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetHeaderProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SheetHeader(props: SheetHeaderProps) -> Element {
    let class = format!("flex flex-col gap-1.5 text-left {}", props.class);

    rsx! {
        div {
            "data-slot": "sheet-header",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetFooterProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SheetFooter(props: SheetFooterProps) -> Element {
    let class = format!("flex flex-col-reverse sm:flex-row sm:justify-end sm:gap-2 mt-auto {}", props.class);

    rsx! {
        div {
            "data-slot": "sheet-footer",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetTitleProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SheetTitle(props: SheetTitleProps) -> Element {
    let class = format!("text-base font-semibold text-foreground {}", props.class);

    rsx! {
        h2 {
            "data-slot": "sheet-title",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetDescriptionProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SheetDescription(props: SheetDescriptionProps) -> Element {
    let class = format!("text-xs text-muted-foreground leading-relaxed {}", props.class);

    rsx! {
        p {
            "data-slot": "sheet-description",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SheetCloseProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SheetClose(props: SheetCloseProps) -> Element {
    let mut ctx = use_context::<SheetContext>();

    rsx! {
        div {
            "data-slot": "sheet-close",
            class: "inline-flex cursor-pointer {props.class}",
            onclick: move |e| {
                e.stop_propagation();
                ctx.open.set(false);
            },
            {props.children}
        }
    }
}
