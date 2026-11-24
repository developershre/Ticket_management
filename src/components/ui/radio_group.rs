#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct RadioGroupContext {
    value: Signal<String>,
    disabled: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupProps {
    pub value: Signal<String>,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn RadioGroup(props: RadioGroupProps) -> Element {
    use_context_provider(|| RadioGroupContext {
        value: props.value,
        disabled: props.disabled,
    });

    let class = format!("grid gap-2 {}", props.class);

    rsx! {
        div {
            "data-slot": "radio-group",
            role: "radiogroup",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct RadioGroupItemProps {
    pub value: String,

    #[props(default)]
    pub id: Option<String>,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub aria_label: Option<String>,
}

#[component]
pub fn RadioGroupItem(props: RadioGroupItemProps) -> Element {
    let ctx = use_context::<RadioGroupContext>();
    let selected = (ctx.value)() == props.value;
    let is_disabled = ctx.disabled || props.disabled;

    let border_cls = if selected {
        "border-primary text-primary"
    } else {
        "border-input hover:border-primary/50 text-transparent"
    };

    let disabled_cls = if is_disabled {
        "cursor-not-allowed opacity-50 pointer-events-none"
    } else {
        "cursor-pointer"
    };

    let class = format!(
        "aspect-square size-4 rounded-full border shadow-xs transition-colors focus:outline-none focus-visible:ring-1 focus-visible:ring-ring flex items-center justify-center {} {} {}",
        border_cls,
        disabled_cls,
        props.class
    );

    let val = props.value.clone();
    let mut current_val = ctx.value;

    rsx! {
        button {
            "data-slot": "radio-group-item",
            r#type: "button",
            role: "radio",
            id: props.id,
            "aria-checked": if selected { "true" } else { "false" },
            "aria-label": props.aria_label,
            disabled: is_disabled,
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if !is_disabled {
                    current_val.set(val.clone());
                }
            },
            span {
                class: if selected {
                    "size-2 rounded-full bg-primary"
                } else {
                    "hidden"
                },
            }
        }
    }
}
