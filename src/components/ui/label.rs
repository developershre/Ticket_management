#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    #[props(default)]
    pub r#for: Option<String>,

    #[props(default)]
    pub class: String,

    #[props(default = false)]
    pub required: bool,

    pub children: Element,
}

#[component]
pub fn Label(props: LabelProps) -> Element {
    let class = format!(
        "text-xs font-medium leading-none text-foreground select-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 {}",
        props.class
    );

    rsx! {
        label {
            "data-slot": "label",
            r#for: props.r#for,
            class: class,
            {props.children}
            if props.required {
                span { class: "ml-0.5 text-destructive font-bold", "*" }
            }
        }
    }
}
