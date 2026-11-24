#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ProgressProps {
    #[props(default = 0.0)]
    pub value: f64,

    #[props(default = 100.0)]
    pub max: f64,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub indicator_class: String,
}

#[component]
pub fn Progress(props: ProgressProps) -> Element {
    let max = if props.max <= 0.0 { 100.0 } else { props.max };
    let clamped_val = props.value.clamp(0.0, max);
    let pct = (clamped_val / max) * 100.0;
    let translate_x = format!("transform: translateX(-{:.2}%);", 100.0 - pct);

    let container_class = format!(
        "relative h-2 w-full overflow-hidden rounded-full bg-muted {}",
        props.class
    );
    let ind_class = format!(
        "h-full w-full flex-1 bg-primary transition-all duration-300 ease-in-out {}",
        props.indicator_class
    );

    rsx! {
        div {
            "data-slot": "progress",
            role: "progressbar",
            "aria-valuemin": "0",
            "aria-valuemax": "{max}",
            "aria-valuenow": "{clamped_val}",
            class: container_class,
            div {
                "data-slot": "progress-indicator",
                class: ind_class,
                style: "{translate_x}",
            }
        }
    }
}
