use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct InputProps {
    #[props(default = "text".to_string())]
    pub r#type: String,

    #[props(default)]
    pub placeholder: String,

    #[props(default)]
    pub value: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default = false)]
    pub required: bool,

    #[props(default = false)]
    pub invalid: bool,

    #[props(default)]
    pub name: String,

    #[props(default)]
    pub id: String,

    /// Additional Tailwind classes
    #[props(default)]
    pub class: String,

    pub oninput: Option<EventHandler<FormEvent>>,
    pub onfocus: Option<EventHandler<FocusEvent>>,
    pub onblur: Option<EventHandler<FocusEvent>>,
    pub onkeydown: Option<EventHandler<KeyboardEvent>>,
}

#[component]
pub fn Input(props: InputProps) -> Element {
    let base_class = r#"
        flex
        h-9
        w-full
        min-w-0
        rounded-md
        border
        border-input
        bg-transparent
        px-3
        py-1
        text-base
        shadow-xs
        outline-none
        transition-[color,box-shadow]
        selection:bg-primary
        selection:text-primary-foreground

        placeholder:text-muted-foreground

        file:inline-flex
        file:h-7
        file:border-0
        file:bg-transparent
        file:text-sm
        file:font-medium

        disabled:pointer-events-none
        disabled:cursor-not-allowed
        disabled:opacity-50

        focus-visible:border-ring
        focus-visible:ring-ring/50
        focus-visible:ring-[3px]

        aria-invalid:border-destructive
        aria-invalid:ring-destructive/20

        md:text-sm
        dark:aria-invalid:ring-destructive/40
    "#;

    let classes = format!("{} {}", base_class, props.class);

    rsx! {
        input {
            class: classes,

            id: if props.id.is_empty() {
                None
            } else {
                Some(props.id.clone())
            },

            name: if props.name.is_empty() {
                None
            } else {
                Some(props.name.clone())
            },

            r#type: props.r#type.clone(),

            placeholder: if props.placeholder.is_empty() {
                None
            } else {
                Some(props.placeholder.clone())
            },

            value: props.value.clone(),

            disabled: props.disabled,
            required: props.required,

            aria_invalid: if props.invalid {
                Some("true")
            } else {
                None
            },

            oninput: move |event| {
                if let Some(handler) = &props.oninput {
                    handler.call(event);
                }
            },

            onfocus: move |event| {
                if let Some(handler) = &props.onfocus {
                    handler.call(event);
                }
            },

            onblur: move |event| {
                if let Some(handler) = &props.onblur {
                    handler.call(event);
                }
            },

            onkeydown: move |event| {
                if let Some(handler) = &props.onkeydown {
                    handler.call(event);
                }
            },
        }
    }
}
