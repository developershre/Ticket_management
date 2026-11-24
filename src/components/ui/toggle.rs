#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToggleVariant {
    #[default]
    Default,
    Outline,
}

impl ToggleVariant {
    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Default => "bg-transparent hover:bg-muted hover:text-muted-foreground data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
            Self::Outline => "border border-input bg-transparent shadow-xs hover:bg-accent hover:text-accent-foreground data-[state=on]:bg-accent data-[state=on]:text-accent-foreground",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ToggleSize {
    #[default]
    Default,
    Sm,
    Lg,
}

impl ToggleSize {
    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Default => "h-9 px-3 min-w-9",
            Self::Sm => "h-8 px-2 min-w-8 text-xs",
            Self::Lg => "h-10 px-3 min-w-10",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleProps {
    #[props(default = false)]
    pub pressed: bool,

    #[props(default = "default".to_string())]
    pub variant: String,

    #[props(default = "default".to_string())]
    pub size: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub aria_label: Option<String>,

    pub on_pressed_change: Option<EventHandler<bool>>,

    pub children: Element,
}

#[component]
pub fn Toggle(props: ToggleProps) -> Element {
    let variant = match props.variant.as_str() {
        "outline" => ToggleVariant::Outline,
        _ => ToggleVariant::Default,
    };
    let size = match props.size.as_str() {
        "sm" => ToggleSize::Sm,
        "lg" => ToggleSize::Lg,
        _ => ToggleSize::Default,
    };

    let state_str = if props.pressed { "on" } else { "off" };
    let active_cls = if props.pressed {
        "bg-accent text-accent-foreground"
    } else {
        ""
    };

    let disabled_cls = if props.disabled {
        "pointer-events-none opacity-50"
    } else {
        ""
    };

    let class = format!(
        "inline-flex items-center justify-center gap-2 rounded-md text-xs font-medium transition-colors hover:bg-muted hover:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 select-none {} {} {} {} {} {}",
        variant.class_name(),
        size.class_name(),
        active_cls,
        disabled_cls,
        props.class,
        if props.pressed { "data-[state=on]" } else { "data-[state=off]" }
    );

    let pressed = props.pressed;

    rsx! {
        button {
            "data-slot": "toggle",
            r#type: "button",
            "aria-pressed": if pressed { "true" } else { "false" },
            "data-state": state_str,
            "aria-label": props.aria_label,
            disabled: props.disabled,
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if !props.disabled {
                    if let Some(handler) = &props.on_pressed_change {
                        handler.call(!pressed);
                    }
                }
            },
            {props.children}
        }
    }
}
