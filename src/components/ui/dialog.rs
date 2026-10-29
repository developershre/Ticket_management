use dioxus::prelude::*;

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct DialogContext {
    pub open: Signal<bool>,
}

// ============================================================
// Dialog Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogProps {
    #[props(default)]
    pub open: Option<Signal<bool>>,

    pub children: Element,
}

#[component]
pub fn Dialog(props: DialogProps) -> Element {
    let local_open = use_signal(|| false);
    let open = props.open.unwrap_or(local_open);

    use_context_provider(|| DialogContext { open });

    rsx! {
        div {
            "data-slot": "dialog",
            class: "contents",
            {props.children}
        }
    }
}

// ============================================================
// Dialog Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogTriggerProps {
    #[props(default)]
    pub class: String,

    #[props(default = false)]
    pub disabled: bool,

    pub children: Element,
}

#[component]
pub fn DialogTrigger(props: DialogTriggerProps) -> Element {
    let mut ctx = use_context::<DialogContext>();

    let classes = format!(
        "inline-flex items-center justify-center rounded-md font-medium text-sm transition-colors \
         focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring \
         disabled:pointer-events-none disabled:opacity-50 cursor-pointer {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "dialog-trigger",
            r#type: "button",
            class: classes,
            disabled: props.disabled,
            onmousedown: move |e| {
                e.stop_propagation();
            },
            onclick: move |e| {
                e.stop_propagation();
                ctx.open.set(true);
            },
            {props.children}
        }
    }
}

// ============================================================
// Dialog Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogContentProps {
    #[props(default)]
    pub class: String,

    #[props(default = true)]
    pub show_close_button: bool,

    pub children: Element,
}

#[component]
pub fn DialogContent(props: DialogContentProps) -> Element {
    let mut ctx = use_context::<DialogContext>();

    if !*ctx.open.read() {
        return rsx! {};
    }

    let class = format!(
        "fixed left-1/2 top-1/2 z-20 grid w-[calc(100%-2rem)] max-w-lg -translate-x-1/2 -translate-y-1/2 \
         gap-4 border bg-background p-4 sm:p-6 shadow-lg rounded-xl duration-200 \
         animate-in fade-in-0 zoom-in-95 outline-none {}",
        props.class
    );

    rsx! {
        // Backdrop overlay
        div {
            class: "fixed inset-0 z-20 bg-black/60 backdrop-blur-xs animate-in fade-in-0",
            onmousedown: move |e| {
                e.stop_propagation();
                ctx.open.set(false);
            },
            onclick: move |e| {
                e.stop_propagation();
                ctx.open.set(false);
            },
        }

        // Dialog container
        div {
            class: class,
            role: "dialog",
            "aria-modal": "true",
            tabindex: "-1",
            onmousedown: move |e| {
                e.stop_propagation();
            },
            onclick: move |e| {
                e.stop_propagation();
            },
            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    ctx.open.set(false);
                }
            },

            {props.children}

            if props.show_close_button {
                button {
                    r#type: "button",
                    class: "absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background \
                            transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 \
                            focus:ring-ring focus:ring-offset-2 cursor-pointer",
                    onclick: move |e| {
                        e.stop_propagation();
                        ctx.open.set(false);
                    },
                    svg {
                        class: "size-4",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M18 6L6 18M6 6l12 12" }
                    }
                    span { class: "sr-only", "Close" }
                }
            }
        }
    }
}

// ============================================================
// Dialog Header
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogHeaderProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DialogHeader(props: DialogHeaderProps) -> Element {
    let class = format!(
        "flex flex-col space-y-1.5 text-center sm:text-left {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "dialog-header",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Dialog Footer
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogFooterProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DialogFooter(props: DialogFooterProps) -> Element {
    let class = format!(
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:space-x-2 gap-2 sm:gap-0 {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "dialog-footer",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Dialog Title
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogTitleProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DialogTitle(props: DialogTitleProps) -> Element {
    let class = format!(
        "text-lg font-semibold leading-none tracking-tight text-foreground {}",
        props.class
    );

    rsx! {
        h2 {
            "data-slot": "dialog-title",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Dialog Description
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogDescriptionProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DialogDescription(props: DialogDescriptionProps) -> Element {
    let class = format!("text-sm text-muted-foreground {}", props.class);

    rsx! {
        p {
            "data-slot": "dialog-description",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Dialog Close
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DialogCloseProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DialogClose(props: DialogCloseProps) -> Element {
    let mut ctx = use_context::<DialogContext>();

    rsx! {
        div {
            "data-slot": "dialog-close",
            class: props.class,
            onclick: move |e| {
                e.stop_propagation();
                ctx.open.set(false);
            },
            {props.children}
        }
    }
}
