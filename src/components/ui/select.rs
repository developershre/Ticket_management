use dioxus::prelude::*;

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct SelectContext {
    pub value: Signal<String>,
    pub display: Signal<String>,
    pub open: Signal<bool>,
    pub placeholder: Signal<String>,
    pub onchange: Option<EventHandler<String>>,
}

// ============================================================
// Select Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectProps {
    #[props(default)]
    pub value: Option<Signal<String>>,

    #[props(default)]
    pub default_value: String,

    #[props(default = "Select an option...".to_string())]
    pub placeholder: String,

    #[props(default)]
    pub onchange: Option<EventHandler<String>>,

    pub children: Element,
}

#[component]
pub fn Select(props: SelectProps) -> Element {
    let def_val = props.default_value;
    let local_val = use_signal(move || def_val);
    let value = props.value.unwrap_or(local_val);
    let display = use_signal(String::new);
    let open = use_signal(|| false);
    let placeholder_str = props.placeholder;
    let placeholder = use_signal(move || placeholder_str);
    let onchange = props.onchange;

    use_context_provider(|| SelectContext {
        value,
        display,
        open,
        placeholder,
        onchange,
    });

    rsx! {
        div {
            "data-slot": "select",
            class: "relative inline-block w-full",
            {props.children}
        }
    }
}

// ============================================================
// Select Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectTriggerProps {
    #[props(default)]
    pub class: String,

    #[props(default = false)]
    pub disabled: bool,

    pub children: Element,
}

#[component]
pub fn SelectTrigger(props: SelectTriggerProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let is_open = *ctx.open.read();

    let class = format!(
        "flex h-10 w-full items-center justify-between rounded-md border border-input \
         bg-card px-3 py-2 text-sm text-foreground outline-none \
         ring-offset-background transition-colors \
         hover:bg-accent/40 focus:border-primary focus:ring-1 focus:ring-primary/20 \
         disabled:cursor-not-allowed disabled:opacity-50 \
         cursor-pointer select-none {}",
        props.class
    );

    let chevron_class = if is_open {
        "size-4 opacity-60 shrink-0 ml-2 transition-transform duration-200 rotate-180"
    } else {
        "size-4 opacity-60 shrink-0 ml-2 transition-transform duration-200"
    };

    rsx! {
        button {
            "data-slot": "select-trigger",
            r#type: "button",
            class: class,
            disabled: props.disabled,
            onmousedown: move |e| {
                e.stop_propagation();
            },
            onclick: move |e| {
                e.stop_propagation();
                let current = *ctx.open.read();
                ctx.open.set(!current);
            },
            {props.children}
            svg {
                class: chevron_class,
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "M6 9l6 6 6-6" }
            }
        }
    }
}

// ============================================================
// Select Value
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectValueProps {
    #[props(default)]
    pub placeholder: String,

    #[props(default)]
    pub class: String,
}

#[component]
pub fn SelectValue(props: SelectValueProps) -> Element {
    let ctx = use_context::<SelectContext>();
    let val = ctx.display.read().clone();
    let current_val = ctx.value.read().clone();
    let is_empty = val.is_empty() && current_val.is_empty();

    let display_text = if !val.is_empty() {
        val
    } else if !current_val.is_empty() {
        current_val
    } else if !props.placeholder.is_empty() {
        props.placeholder
    } else {
        ctx.placeholder.read().clone()
    };

    let color_class = if is_empty {
        "text-muted-foreground"
    } else {
        "text-foreground"
    };

    rsx! {
        span {
            "data-slot": "select-value",
            class: "block truncate {color_class} {props.class}",
            "{display_text}"
        }
    }
}

// ============================================================
// Select Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectContentProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SelectContent(props: SelectContentProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let uid = use_hook(super::popup::next_popup_id);
    let open = ctx.open;
    let effect_uid = uid.clone();
    let listener_uid = uid.clone();

    use_effect(move || {
        if open() {
            super::popup::position_popup(
                &effect_uid,
                "[data-slot=\"select\"]",
                "[data-slot=\"select-trigger\"]",
                "bottom",
                "start",
                4,
                true,
            );
        }
    });

    // Global click-outside listener
    use_effect(move || {
        if open() {
            let close_id = format!("__sel_close_{}", listener_uid);
            let js = format!(
                r#"
                (function() {{
                    var handlerName = "{}";
                    if (window[handlerName]) {{
                        document.removeEventListener("pointerdown", window[handlerName], true);
                    }}
                    window[handlerName] = function(e) {{
                        var popup = document.querySelector('[data-popup="{}"]');
                        if (!popup) return;
                        var content = popup.querySelector('[data-slot="select-content"]');
                        if (!content) return;
                        var target = e.target;
                        if (content.contains(target)) return;
                        var trigger = target.closest('[data-slot="select-trigger"]');
                        if (trigger && popup.closest('[data-slot="select"]') &&
                            popup.closest('[data-slot="select"]').contains(trigger)) return;
                        var closeBtn = document.getElementById("{}");
                        if (closeBtn) closeBtn.click();
                    }};
                    document.addEventListener("pointerdown", window[handlerName], true);
                }})();
                "#,
                close_id, listener_uid, close_id
            );
            let _ = document::eval(&js);
        } else {
            let close_id = format!("__sel_close_{}", listener_uid);
            let js = format!(
                r#"
                (function() {{
                    var handlerName = "{}";
                    if (window[handlerName]) {{
                        document.removeEventListener("pointerdown", window[handlerName], true);
                        delete window[handlerName];
                    }}
                }})();
                "#,
                close_id
            );
            let _ = document::eval(&js);
        }
    });

    if !*ctx.open.read() {
        return rsx! {};
    }

    let close_btn_id = format!("__sel_close_{}", uid);

    let class = format!(
        "w-full min-w-[8rem] max-h-64 \
         overflow-y-auto overflow-x-hidden rounded-md border border-border/80 \
         bg-popover/95 backdrop-blur-md p-1.5 text-popover-foreground outline-none \
         shadow-xl shadow-black/15 ring-1 ring-border/50 animate-in fade-in-0 zoom-in-95 duration-100 {}",
        props.class
    );

    rsx! {
        // Hidden close button triggered by the global pointerdown listener
        button {
            id: close_btn_id,
            class: "hidden",
            onclick: move |_| {
                ctx.open.set(false);
            },
        }

        div {
            "data-popup": uid,
            class: "fixed left-0 top-0 z-[100] invisible",

            div {
                "data-slot": "select-content",
                class: class,
                onmousedown: move |e| {
                    e.stop_propagation();
                },
                onclick: move |e| {
                    e.stop_propagation();
                },
                {props.children}
            }
        }
    }
}

// ============================================================
// Select Group
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectGroupProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SelectGroup(props: SelectGroupProps) -> Element {
    rsx! {
        div {
            "data-slot": "select-group",
            class: "p-1 {props.class}",
            {props.children}
        }
    }
}

// ============================================================
// Select Label
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectLabelProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SelectLabel(props: SelectLabelProps) -> Element {
    rsx! {
        div {
            "data-slot": "select-label",
            class: "px-2 py-1.5 text-xs font-semibold text-muted-foreground {props.class}",
            {props.children}
        }
    }
}

// ============================================================
// Select Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectItemProps {
    pub value: String,

    #[props(default)]
    pub text: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SelectItem(props: SelectItemProps) -> Element {
    let mut ctx = use_context::<SelectContext>();
    let is_selected = *ctx.value.read() == props.value;

    let class = format!(
        "relative flex w-full cursor-pointer select-none items-center rounded-sm py-2 pl-3 pr-8 \
         text-sm outline-none transition-colors \
         hover:bg-accent hover:text-accent-foreground \
         data-[state=checked]:bg-primary/10 data-[state=checked]:text-primary data-[state=checked]:font-medium \
         disabled:pointer-events-none disabled:opacity-50 {}",
        props.class
    );

    let val = props.value.clone();
    let text_val = if !props.text.is_empty() {
        props.text.clone()
    } else {
        props.value.clone()
    };

    rsx! {
        div {
            "data-slot": "select-item",
            "data-state": if is_selected { "checked" } else { "unchecked" },
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if !props.disabled {
                    ctx.value.set(val.clone());
                    ctx.display.set(text_val.clone());
                    ctx.open.set(false);
                    if let Some(h) = ctx.onchange {
                        h.call(val.clone());
                    }
                }
            },

            span {
                class: "truncate",
                {props.children}
            }

            if is_selected {
                span {
                    class: "absolute right-2.5 flex size-4 items-center justify-center text-primary",
                    svg {
                        class: "size-4",
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        stroke_width: "2.5",
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        path { d: "M20 6L9 17L4 12" }
                    }
                }
            }
        }
    }
}

// ============================================================
// Select Separator
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SelectSeparatorProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn SelectSeparator(props: SelectSeparatorProps) -> Element {
    rsx! {
        div {
            "data-slot": "select-separator",
            class: "-mx-1 my-1 h-px bg-muted {props.class}",
        }
    }
}

// ============================================================
// Simple Select (Convenience Component for Forms)
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SimpleSelectProps {
    pub value: Signal<String>,
    #[props(default)]
    pub placeholder: String,
    pub options: Vec<(String, String)>,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onchange: Option<EventHandler<String>>,
}

#[component]
pub fn SimpleSelect(props: SimpleSelectProps) -> Element {
    let val = props.value;
    let onchange = props.onchange;
    rsx! {
        Select {
            value: Some(val),
            placeholder: props.placeholder.clone(),
            onchange: move |v: String| {
                if let Some(h) = onchange {
                    h.call(v);
                }
            },
            SelectTrigger {
                class: format!("h-10 text-sm bg-card border-input {}", props.class),
                SelectValue { placeholder: props.placeholder }
            }
            SelectContent {
                for (opt_val, opt_label) in props.options {
                    SelectItem {
                        key: "{opt_val}",
                        value: opt_val.clone(),
                        text: opt_label.clone(),
                        "{opt_label}"
                    }
                }
            }
        }
    }
}
