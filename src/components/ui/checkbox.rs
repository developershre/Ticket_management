#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CheckboxProps {
    #[props(default = false)]
    pub checked: bool,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default = false)]
    pub required: bool,

    #[props(default)]
    pub id: Option<String>,

    #[props(default)]
    pub name: Option<String>,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub aria_label: Option<String>,

    pub on_checked_change: Option<EventHandler<bool>>,
}

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let checked = props.checked;
    let disabled = props.disabled;

    let state_cls = if checked {
        "bg-primary text-primary-foreground border-primary"
    } else {
        "bg-background text-transparent border-input hover:border-primary/50"
    };

    let disabled_cls = if disabled {
        "cursor-not-allowed opacity-50 pointer-events-none"
    } else {
        "cursor-pointer"
    };

    let class = format!(
        "peer size-4 shrink-0 rounded-sm border shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring select-none flex items-center justify-center {} {} {}",
        state_cls,
        disabled_cls,
        props.class
    );

    rsx! {
        button {
            "data-slot": "checkbox",
            r#type: "button",
            role: "checkbox",
            id: props.id,
            name: props.name,
            "aria-checked": if checked { "true" } else { "false" },
            "aria-required": if props.required { "true" } else { "false" },
            "aria-label": props.aria_label,
            disabled: disabled,
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if !disabled {
                    if let Some(handler) = &props.on_checked_change {
                        handler.call(!checked);
                    }
                }
            },
            svg {
                class: if checked { "size-3 stroke-[2.5]" } else { "hidden" },
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                polyline { points: "20 6 9 17 4 12" }
            }
        }
    }
}
