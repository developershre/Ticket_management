use dioxus::prelude::*;

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct TooltipContext {
    pub open: Signal<bool>,
}

// ============================================================
// Tooltip Provider
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TooltipProviderProps {
    pub children: Element,
}

#[component]
pub fn TooltipProvider(props: TooltipProviderProps) -> Element {
    rsx! {
        div {
            "data-slot": "tooltip-provider",
            class: "contents",
            {props.children}
        }
    }
}

// ============================================================
// Tooltip Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TooltipProps {
    #[props(default)]
    pub open: Option<Signal<bool>>,

    pub children: Element,
}

#[component]
pub fn Tooltip(props: TooltipProps) -> Element {
    let local_open = use_signal(|| false);
    let open = props.open.unwrap_or(local_open);

    use_context_provider(|| TooltipContext { open });

    rsx! {
        div {
            "data-slot": "tooltip",
            class: "relative inline-flex",
            {props.children}
        }
    }
}

// ============================================================
// Tooltip Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TooltipTriggerProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn TooltipTrigger(props: TooltipTriggerProps) -> Element {
    let mut ctx = use_context::<TooltipContext>();

    rsx! {
        div {
            "data-slot": "tooltip-trigger",
            class: "inline-flex cursor-pointer {props.class}",
            onmouseenter: move |_| {
                ctx.open.set(true);
            },
            onmouseleave: move |_| {
                ctx.open.set(false);
            },
            onfocus: move |_| {
                ctx.open.set(true);
            },
            onblur: move |_| {
                ctx.open.set(false);
            },
            {props.children}
        }
    }
}

// ============================================================
// Tooltip Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TooltipContentProps {
    #[props(default = "top".to_string())]
    pub side: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn TooltipContent(props: TooltipContentProps) -> Element {
    let ctx = use_context::<TooltipContext>();
    let uid = use_hook(super::popup::next_popup_id);
    let open = ctx.open;
    let side = props.side.clone();
    let effect_uid = uid.clone();

    use_effect(move || {
        if open() {
            super::popup::position_popup(
                &effect_uid,
                "[data-slot=\"tooltip\"]",
                "[data-slot=\"tooltip-trigger\"]",
                &side,
                "center",
                6,
                false,
            );
        }
    });

    if !*ctx.open.read() {
        return rsx! {};
    }

    let class = format!(
        "overflow-hidden rounded-md bg-primary px-3 py-1.5 text-xs \
         text-primary-foreground shadow-md animate-in fade-in-0 zoom-in-95 pointer-events-none \
         whitespace-nowrap select-none {}",
        props.class
    );

    rsx! {
        div {
            "data-popup": uid,
            class: "fixed left-0 top-0 z-30 invisible pointer-events-none",

            div {
                "data-slot": "tooltip-content",
                role: "tooltip",
                class: class,
                {props.children}
            }
        }
    }
}
