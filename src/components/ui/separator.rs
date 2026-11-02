#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SeparatorProps {
    #[props(default = "horizontal".to_string())]
    pub orientation: String,

    #[props(default)]
    pub class: String,
}

#[component]
pub fn Separator(props: SeparatorProps) -> Element {
    let orientation_class = if props.orientation == "vertical" {
        "h-full w-[1px]"
    } else {
        "h-[1px] w-full"
    };

    let class = format!("shrink-0 bg-border {} {}", orientation_class, props.class);

    rsx! {
        div {
            "data-slot": "separator",
            role: "none",
            class: class,
        }
    }
}
