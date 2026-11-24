#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct AccordionContext {
    value: Signal<Vec<String>>,
    is_multiple: bool,
    collapsible: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct AccordionProps {
    #[props(default = "single".to_string())]
    pub r#type: String,

    #[props(default = true)]
    pub collapsible: bool,

    #[props(default)]
    pub value: Option<Signal<Vec<String>>>,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Accordion(props: AccordionProps) -> Element {
    let is_multiple = props.r#type == "multiple";
    let internal_val = use_signal(Vec::new);
    let val = props.value.unwrap_or(internal_val);

    use_context_provider(|| AccordionContext {
        value: val,
        is_multiple,
        collapsible: props.collapsible,
    });

    let class = format!("w-full divide-y divide-border/60 {}", props.class);

    rsx! {
        div {
            "data-slot": "accordion",
            class: class,
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq)]
struct AccordionItemContext {
    value: String,
    disabled: bool,
}

#[derive(Props, Clone, PartialEq)]
pub struct AccordionItemProps {
    pub value: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn AccordionItem(props: AccordionItemProps) -> Element {
    let val = props.value.clone();
    use_context_provider(|| AccordionItemContext {
        value: val,
        disabled: props.disabled,
    });

    let class = format!("border-b border-border/60 last:border-b-0 {}", props.class);

    rsx! {
        div {
            "data-slot": "accordion-item",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AccordionTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AccordionTrigger(props: AccordionTriggerProps) -> Element {
    let mut acc_ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();

    let current_values = (acc_ctx.value)();
    let is_open = current_values.contains(&item_ctx.value);
    let item_val = item_ctx.value.clone();
    let is_disabled = item_ctx.disabled;

    let chevron_rotate = if is_open { "rotate-180" } else { "rotate-0" };

    let class = format!(
        "flex w-full items-center justify-between py-3 text-xs font-medium transition-all hover:underline cursor-pointer select-none disabled:pointer-events-none disabled:opacity-50 {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "accordion-trigger",
            r#type: "button",
            "aria-expanded": if is_open { "true" } else { "false" },
            disabled: is_disabled,
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if is_disabled {
                    return;
                }
                let mut vals = (acc_ctx.value)();
                if is_open {
                    if acc_ctx.collapsible || acc_ctx.is_multiple {
                        vals.retain(|v| v != &item_val);
                    }
                } else {
                    if acc_ctx.is_multiple {
                        vals.push(item_val.clone());
                    } else {
                        vals = vec![item_val.clone()];
                    }
                }
                acc_ctx.value.set(vals);
            },
            span { class: "text-left font-medium", {props.children} }
            svg {
                class: "size-3.5 shrink-0 transition-transform duration-200 text-muted-foreground {chevron_rotate}",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                polyline { points: "6 9 12 15 18 9" }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct AccordionContentProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn AccordionContent(props: AccordionContentProps) -> Element {
    let acc_ctx = use_context::<AccordionContext>();
    let item_ctx = use_context::<AccordionItemContext>();

    let is_open = (acc_ctx.value)().contains(&item_ctx.value);

    if !is_open {
        return rsx! {};
    }

    let class = format!(
        "overflow-hidden pb-3 pt-0 text-xs text-muted-foreground leading-relaxed animate-accordion-down {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "accordion-content",
            class: class,
            {props.children}
        }
    }
}
