use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SwitchProps {
    #[props(default)]
    pub checked: Option<Signal<bool>>,

    #[props(default = false)]
    pub default_checked: bool,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub onchange: Option<EventHandler<bool>>,
}

#[component]
pub fn Switch(props: SwitchProps) -> Element {
    let def = props.default_checked;
    let local_checked = use_signal(move || def);
    let mut is_checked_sig = props.checked.unwrap_or(local_checked);
    let is_checked = *is_checked_sig.read();

    let bg_class = if is_checked { "bg-primary" } else { "bg-input" };
    let thumb_trans = if is_checked { "translate-x-4" } else { "translate-x-0" };

    let class = format!(
        "peer inline-flex h-5 w-9 shrink-0 items-center rounded-full border-2 border-transparent \
         shadow-xs transition-colors focus-visible:outline-none focus-visible:ring-2 \
         focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background \
         disabled:cursor-not-allowed disabled:opacity-50 cursor-pointer select-none {} {}",
        bg_class, props.class
    );

    rsx! {
        button {
            "data-slot": "switch",
            "data-state": if is_checked { "checked" } else { "unchecked" },
            r#type: "button",
            role: "switch",
            "aria-checked": if is_checked { "true" } else { "false" },
            disabled: props.disabled,
            class: class,
            onclick: move |_| {
                if !props.disabled {
                    let next = !is_checked;
                    is_checked_sig.set(next);
                    if let Some(handler) = &props.onchange {
                        handler.call(next);
                    }
                }
            },
            span {
                "data-slot": "switch-thumb",
                "data-state": if is_checked { "checked" } else { "unchecked" },
                class: "pointer-events-none block size-4 rounded-full bg-background shadow-lg ring-0 transition-transform {thumb_trans}",
            }
        }
    }
}
