#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TextareaProps {
    #[props(default)]
    pub value: String,

    #[props(default)]
    pub placeholder: String,

    #[props(default = 3)]
    pub rows: u32,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default = false)]
    pub readonly: bool,

    #[props(default = false)]
    pub required: bool,

    #[props(default)]
    pub id: Option<String>,

    #[props(default)]
    pub name: Option<String>,

    #[props(default)]
    pub class: String,

    pub oninput: Option<EventHandler<FormEvent>>,
    pub onchange: Option<EventHandler<FormEvent>>,
    pub onkeydown: Option<EventHandler<KeyboardEvent>>,
}

#[component]
pub fn Textarea(props: TextareaProps) -> Element {
    let disabled_cls = if props.disabled {
        "cursor-not-allowed opacity-50"
    } else {
        ""
    };

    let class = format!(
        "flex min-h-[60px] w-full rounded-md border border-input bg-transparent px-3 py-2 text-xs shadow-xs placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 transition-colors resize-y {} {}",
        disabled_cls,
        props.class
    );

    rsx! {
        textarea {
            "data-slot": "textarea",
            id: props.id,
            name: props.name,
            value: "{props.value}",
            placeholder: props.placeholder,
            rows: "{props.rows}",
            disabled: props.disabled,
            readonly: props.readonly,
            required: props.required,
            class: class,
            oninput: move |e| {
                if let Some(handler) = &props.oninput {
                    handler.call(e);
                }
            },
            onchange: move |e| {
                if let Some(handler) = &props.onchange {
                    handler.call(e);
                }
            },
            onkeydown: move |e| {
                if let Some(handler) = &props.onkeydown {
                    handler.call(e);
                }
            },
        }
    }
}
