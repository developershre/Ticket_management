#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SliderProps {
    pub value: Signal<f64>,

    #[props(default = 0.0)]
    pub min: f64,

    #[props(default = 100.0)]
    pub max: f64,

    #[props(default = 1.0)]
    pub step: f64,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub on_change: Option<EventHandler<f64>>,
}

#[component]
pub fn Slider(props: SliderProps) -> Element {
    let mut val = props.value;
    let cur = val();
    let range = (props.max - props.min).max(0.0001);
    let pct = ((cur - props.min) / range * 100.0).clamp(0.0, 100.0);

    let class = format!(
        "relative flex w-full touch-none select-none items-center group {}",
        props.class
    );

    let disabled_cls = if props.disabled {
        "opacity-50 pointer-events-none"
    } else {
        ""
    };

    rsx! {
        div {
            "data-slot": "slider",
            class: "{class} {disabled_cls}",
            div {
                class: "relative h-1.5 w-full grow overflow-hidden rounded-full bg-secondary",
                div {
                    class: "absolute h-full bg-primary transition-all duration-75",
                    style: "width: {pct}%;",
                }
            }
            input {
                r#type: "range",
                min: "{props.min}",
                max: "{props.max}",
                step: "{props.step}",
                value: "{cur}",
                disabled: props.disabled,
                class: "absolute inset-0 w-full opacity-0 cursor-pointer h-full",
                oninput: move |e| {
                    if let Ok(new_val) = e.value().parse::<f64>() {
                        val.set(new_val);
                        if let Some(handler) = &props.on_change {
                            handler.call(new_val);
                        }
                    }
                },
            }
            div {
                class: "absolute block size-4 rounded-full border border-primary/50 bg-background shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring pointer-events-none",
                style: "left: calc({pct}% - 8px);",
            }
        }
    }
}
