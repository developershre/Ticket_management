#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct InputOTPContext {
    value: Signal<String>,
    max_length: usize,
    disabled: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct InputOTPProps {
    pub value: Signal<String>,

    #[props(default = 6)]
    pub max_length: usize,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub on_change: Option<EventHandler<String>>,

    pub children: Element,
}

#[component]
pub fn InputOTP(props: InputOTPProps) -> Element {
    use_context_provider(|| InputOTPContext {
        value: props.value,
        max_length: props.max_length,
        disabled: props.disabled,
    });

    let mut val = props.value;
    let max_len = props.max_length;
    let is_disabled = props.disabled;

    let class = format!(
        "relative flex items-center gap-2 select-none {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "input-otp",
            class: class,
            // Hidden text input capturing typed keystrokes
            input {
                r#type: "text",
                inputmode: "numeric",
                maxlength: "{max_len}",
                value: "{val}",
                disabled: is_disabled,
                class: "absolute inset-0 size-full opacity-0 cursor-pointer z-10",
                oninput: move |e| {
                    let text = e.value();
                    let trimmed: String = text.chars().take(max_len).collect();
                    val.set(trimmed.clone());
                    if let Some(handler) = &props.on_change {
                        handler.call(trimmed);
                    }
                },
            }
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InputOTPGroupProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn InputOTPGroup(props: InputOTPGroupProps) -> Element {
    let class = format!("flex items-center {}", props.class);

    rsx! {
        div {
            "data-slot": "input-otp-group",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InputOTPSlotProps {
    pub index: usize,

    #[props(default)]
    pub class: String,
}

#[component]
pub fn InputOTPSlot(props: InputOTPSlotProps) -> Element {
    let ctx = use_context::<InputOTPContext>();
    let current = (ctx.value)();
    let char_opt = current.chars().nth(props.index);
    let is_focused = current.len() == props.index && !ctx.disabled;

    let ring_cls = if is_focused {
        "z-10 ring-1 ring-ring border-ring"
    } else {
        "border-input"
    };

    let class = format!(
        "relative flex size-9 items-center justify-center border-y border-r text-xs shadow-xs transition-all first:rounded-l-md first:border-l last:rounded-r-md bg-background font-mono font-medium {} {}",
        ring_cls,
        props.class
    );

    rsx! {
        div {
            "data-slot": "input-otp-slot",
            class: class,
            if let Some(ch) = char_opt {
                span { "{ch}" }
            } else if is_focused {
                div { class: "pointer-events-none absolute inset-0 flex items-center justify-center",
                    div { class: "h-4 w-px animate-pulse bg-foreground duration-1000" }
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct InputOTPSeparatorProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn InputOTPSeparator(props: InputOTPSeparatorProps) -> Element {
    let class = format!("text-muted-foreground flex items-center justify-center px-1 {}", props.class);

    rsx! {
        div {
            "data-slot": "input-otp-separator",
            role: "separator",
            class: class,
            svg {
                class: "size-3",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                circle { cx: "12", cy: "12", r: "2" }
            }
        }
    }
}
