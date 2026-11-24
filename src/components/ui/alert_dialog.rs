#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub struct AlertDialogContext {
    pub open: Signal<bool>,
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogProps {
    #[props(default)]
    pub open: Option<Signal<bool>>,

    pub children: Element,
}

#[component]
pub fn AlertDialog(props: AlertDialogProps) -> Element {
    let internal_open = use_signal(|| false);
    let open = props.open.unwrap_or(internal_open);

    use_context_provider(|| AlertDialogContext { open });

    rsx! {
        div {
            "data-slot": "alert-dialog",
            class: "contents",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDialogTrigger(props: AlertDialogTriggerProps) -> Element {
    let mut ctx = use_context::<AlertDialogContext>();

    rsx! {
        div {
            "data-slot": "alert-dialog-trigger",
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
pub struct AlertDialogContentProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDialogContent(props: AlertDialogContentProps) -> Element {
    let ctx = use_context::<AlertDialogContext>();
    let open = (ctx.open)();

    if !open {
        return rsx! {};
    }

    let class = format!(
        "fixed left-1/2 top-1/2 z-50 grid w-full max-w-lg -translate-x-1/2 -translate-y-1/2 gap-4 border border-border bg-background p-6 shadow-lg duration-200 rounded-lg sm:rounded-lg animate-in fade-in-0 zoom-in-95 {}",
        props.class
    );

    rsx! {
        div { class: "contents",
            // Overlay with backdrop blur
            div {
                class: "fixed inset-0 z-50 bg-black/60 backdrop-blur-xs animate-in fade-in-0 duration-200",
                onclick: move |e| {
                    e.stop_propagation();
                    // Alert dialogs typically do not close on backdrop click by default to enforce confirmation
                },
            }
            div {
                "data-slot": "alert-dialog-content",
                role: "alertdialog",
                class: class,
                onclick: move |e| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogHeaderProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDialogHeader(props: AlertDialogHeaderProps) -> Element {
    let class = format!("flex flex-col gap-2 text-center sm:text-left {}", props.class);

    rsx! {
        div {
            "data-slot": "alert-dialog-header",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogFooterProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDialogFooter(props: AlertDialogFooterProps) -> Element {
    let class = format!(
        "flex flex-col-reverse sm:flex-row sm:justify-end sm:gap-2 mt-2 {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "alert-dialog-footer",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogTitleProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDialogTitle(props: AlertDialogTitleProps) -> Element {
    let class = format!("text-base font-semibold text-foreground {}", props.class);

    rsx! {
        h2 {
            "data-slot": "alert-dialog-title",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogDescriptionProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDialogDescription(props: AlertDialogDescriptionProps) -> Element {
    let class = format!("text-xs text-muted-foreground leading-relaxed {}", props.class);

    rsx! {
        p {
            "data-slot": "alert-dialog-description",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogActionProps {
    #[props(default)]
    pub class: String,
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn AlertDialogAction(props: AlertDialogActionProps) -> Element {
    let mut ctx = use_context::<AlertDialogContext>();
    let class = format!(
        "inline-flex h-9 items-center justify-center rounded-md bg-destructive text-destructive-foreground px-4 py-2 text-xs font-medium shadow-xs hover:bg-destructive/90 transition-colors select-none cursor-pointer {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "alert-dialog-action",
            r#type: "button",
            class: class,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
                ctx.open.set(false);
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDialogCancelProps {
    #[props(default)]
    pub class: String,
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn AlertDialogCancel(props: AlertDialogCancelProps) -> Element {
    let mut ctx = use_context::<AlertDialogContext>();
    let class = format!(
        "inline-flex h-9 items-center justify-center rounded-md border border-input bg-background px-4 py-2 text-xs font-medium shadow-xs hover:bg-accent hover:text-accent-foreground transition-colors select-none cursor-pointer mt-2 sm:mt-0 {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "alert-dialog-cancel",
            r#type: "button",
            class: class,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
                ctx.open.set(false);
            },
            {props.children}
        }
    }
}
