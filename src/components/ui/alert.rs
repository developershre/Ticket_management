#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum AlertVariant {
    #[default]
    Default,
    Destructive,
    Warning,
    Success,
    Info,
}

impl AlertVariant {
    pub fn from_str_name(s: &str) -> Self {
        match s {
            "destructive" | "danger" | "error" => Self::Destructive,
            "warning" | "amber" => Self::Warning,
            "success" | "emerald" => Self::Success,
            "info" | "blue" => Self::Info,
            _ => Self::Default,
        }
    }

    pub fn class_name(&self) -> &'static str {
        match self {
            Self::Default => "bg-background text-foreground border-border",
            Self::Destructive => "border-destructive/50 text-destructive dark:border-destructive [&>svg]:text-destructive bg-destructive/10",
            Self::Warning => "border-warning/50 text-warning-foreground dark:text-warning [&>svg]:text-warning bg-warning/10",
            Self::Success => "border-success/50 text-success [&>svg]:text-success bg-success/10",
            Self::Info => "border-info/50 text-info [&>svg]:text-info bg-info/10",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertProps {
    #[props(default = "default".to_string())]
    pub variant: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Alert(props: AlertProps) -> Element {
    let variant = AlertVariant::from_str_name(&props.variant);
    let class = format!(
        "relative w-full rounded-lg border p-4 text-sm [&>svg+div]:translate-y-[-3px] [&>svg]:absolute [&>svg]:left-4 [&>svg]:top-4 [&>svg]:size-4 [&>svg~*]:pl-7 transition-colors duration-200 {} {}",
        variant.class_name(),
        props.class
    );

    rsx! {
        div {
            "data-slot": "alert",
            role: "alert",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertTitleProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertTitle(props: AlertTitleProps) -> Element {
    let class = format!(
        "mb-1 font-semibold leading-none tracking-tight text-foreground {}",
        props.class
    );

    rsx! {
        h5 {
            "data-slot": "alert-title",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AlertDescriptionProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AlertDescription(props: AlertDescriptionProps) -> Element {
    let class = format!("text-sm [&_p]:leading-relaxed opacity-90 {}", props.class);

    rsx! {
        div {
            "data-slot": "alert-description",
            class: class,
            {props.children}
        }
    }
}
