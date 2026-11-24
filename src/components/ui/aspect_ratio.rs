#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct AspectRatioProps {
    #[props(default = 16.0 / 9.0)]
    pub ratio: f64,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn AspectRatio(props: AspectRatioProps) -> Element {
    let ratio = if props.ratio <= 0.0 { 1.0 } else { props.ratio };
    let style = format!("aspect-ratio: {};", ratio);

    let class = format!("relative w-full overflow-hidden {}", props.class);

    rsx! {
        div {
            "data-slot": "aspect-ratio",
            class: class,
            style: "{style}",
            {props.children}
        }
    }
}
