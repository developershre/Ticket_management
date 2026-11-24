#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct ToggleGroupContext {
    value: Signal<String>,
    variant: &'static str,
    size: &'static str,
    disabled: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupProps {
    pub value: Signal<String>,

    #[props(default = "default".to_string())]
    pub variant: String,

    #[props(default = "default".to_string())]
    pub size: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ToggleGroup(props: ToggleGroupProps) -> Element {
    let variant: &'static str = match props.variant.as_str() {
        "outline" => "outline",
        _ => "default",
    };
    let size: &'static str = match props.size.as_str() {
        "sm" => "sm",
        "lg" => "lg",
        _ => "default",
    };

    use_context_provider(|| ToggleGroupContext {
        value: props.value,
        variant,
        size,
        disabled: props.disabled,
    });

    let class = format!("inline-flex items-center justify-center gap-1 {}", props.class);

    rsx! {
        div {
            "data-slot": "toggle-group",
            role: "group",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ToggleGroupItemProps {
    pub value: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub aria_label: Option<String>,

    pub children: Element,
}

#[component]
pub fn ToggleGroupItem(props: ToggleGroupItemProps) -> Element {
    let ctx = use_context::<ToggleGroupContext>();
    let selected = (ctx.value)() == props.value;
    let is_disabled = ctx.disabled || props.disabled;

    let size_cls = match ctx.size {
        "sm" => "h-8 px-2 min-w-8 text-xs",
        "lg" => "h-10 px-3 min-w-10",
        _ => "h-9 px-3 min-w-9 text-xs",
    };

    let variant_cls = if ctx.variant == "outline" {
        "border border-input bg-transparent shadow-xs hover:bg-accent hover:text-accent-foreground"
    } else {
        "bg-transparent hover:bg-muted hover:text-muted-foreground"
    };

    let active_cls = if selected {
        "bg-accent text-accent-foreground font-semibold"
    } else {
        "text-muted-foreground"
    };

    let disabled_cls = if is_disabled {
        "pointer-events-none opacity-50"
    } else {
        ""
    };

    let class = format!(
        "inline-flex items-center justify-center gap-2 rounded-md font-medium transition-colors focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring select-none {} {} {} {} {} {}",
        size_cls,
        variant_cls,
        active_cls,
        disabled_cls,
        props.class,
        if selected { "data-[state=on]" } else { "data-[state=off]" }
    );

    let val = props.value.clone();
    let mut current_val = ctx.value;

    rsx! {
        button {
            "data-slot": "toggle-group-item",
            r#type: "button",
            "aria-pressed": if selected { "true" } else { "false" },
            "aria-label": props.aria_label,
            disabled: is_disabled,
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if !is_disabled {
                    current_val.set(val.clone());
                }
            },
            {props.children}
        }
    }
}
