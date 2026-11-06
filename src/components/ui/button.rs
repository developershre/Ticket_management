#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonVariant {
    #[default]
    Default,
    Destructive,
    Outline,
    Secondary,
    Ghost,
    Link,
}

impl ButtonVariant {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "destructive" => Self::Destructive,
            "outline" => Self::Outline,
            "secondary" => Self::Secondary,
            "ghost" => Self::Ghost,
            "link" => Self::Link,
            _ => Self::Default,
        }
    }

    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Default => "bg-primary text-primary-foreground shadow-xs hover:bg-primary/90",
            Self::Destructive => {
                "bg-destructive text-destructive-foreground shadow-xs hover:bg-destructive/90"
            }
            Self::Outline => {
                "border border-input bg-background shadow-xs hover:bg-accent hover:text-accent-foreground"
            }
            Self::Secondary => {
                "bg-secondary text-secondary-foreground shadow-xs hover:bg-secondary/80"
            }
            Self::Ghost => "hover:bg-accent hover:text-accent-foreground",
            Self::Link => "text-primary underline-offset-4 hover:underline",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum ButtonSize {
    #[default]
    Default,
    Sm,
    Lg,
    Icon,
    IconSm,
}

impl ButtonSize {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "sm" => Self::Sm,
            "lg" => Self::Lg,
            "icon" => Self::Icon,
            "icon-sm" => Self::IconSm,
            _ => Self::Default,
        }
    }

    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Default => "h-9 px-4 py-2",
            Self::Sm => "h-8 rounded-md px-3 text-xs",
            Self::Lg => "h-10 rounded-md px-8",
            Self::Icon => "size-9 p-0",
            Self::IconSm => "size-8 p-0",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ButtonProps {
    #[props(default = "default".to_string())]
    pub variant: String,

    #[props(default = "default".to_string())]
    pub size: String,

    #[props(default)]
    pub class: String,

    #[props(default = "button".to_string())]
    pub r#type: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub title: Option<String>,

    #[props(default)]
    pub aria_label: Option<String>,

    pub onclick: Option<EventHandler<MouseEvent>>,
    pub onmousedown: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    let variant = ButtonVariant::from_str_name(&props.variant);
    let size = ButtonSize::from_str_name(&props.size);

    let base_class = "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-md \
                      text-sm font-medium transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer select-none \
                      focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring \
                      disabled:pointer-events-none disabled:opacity-50 disabled:active:scale-100 \
                      [&_svg]:pointer-events-none [&_svg:not([class*='size-'])]:size-4 shrink-0";

    let classes = format!(
        "{} {} {} {}",
        base_class,
        variant.class_name(),
        size.class_name(),
        props.class
    );

    rsx! {
        button {
            "data-slot": "button",
            r#type: props.r#type,
            class: classes,
            disabled: props.disabled,
            title: props.title,
            aria_label: props.aria_label,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            onmousedown: move |e| {
                if let Some(handler) = &props.onmousedown {
                    handler.call(e);
                }
            },
            {props.children}
        }
    }
}
