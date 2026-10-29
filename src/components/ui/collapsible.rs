use dioxus::prelude::*;

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
pub struct CollapsibleContext {
    pub open: Signal<bool>,
    pub disabled: Signal<bool>,
}

impl CollapsibleContext {
    pub fn is_open(&self) -> bool {
        *self.open.read()
    }

    pub fn is_disabled(&self) -> bool {
        *self.disabled.read()
    }

    pub fn toggle(&mut self) {
        if !self.is_disabled() {
            let current = self.is_open();
            self.open.set(!current);
        }
    }
}

// ============================================================
// Collapsible Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleProps {
    #[props(default = false)]
    pub default_open: bool,

    #[props(default)]
    pub open: Option<Signal<bool>>,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Collapsible(props: CollapsibleProps) -> Element {
    let default_val = props.default_open;
    let local_open = use_signal(move || default_val);
    let open = props.open.unwrap_or(local_open);

    let dis_val = props.disabled;
    let disabled = use_signal(move || dis_val);

    use_context_provider(|| CollapsibleContext { open, disabled });

    let is_open = *open.read();
    let state_str = if is_open { "open" } else { "closed" };

    rsx! {
        div {
            "data-slot": "collapsible",
            "data-state": state_str,
            "data-disabled": if props.disabled { "true" } else { "false" },
            class: props.class,
            {props.children}
        }
    }
}

// ============================================================
// Collapsible Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleTriggerProps {
    #[props(default)]
    pub class: String,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn CollapsibleTrigger(props: CollapsibleTriggerProps) -> Element {
    let mut ctx = use_context::<CollapsibleContext>();
    let is_open = ctx.is_open();
    let is_disabled = ctx.is_disabled();
    let state_str = if is_open { "open" } else { "closed" };

    rsx! {
        button {
            "data-slot": "collapsible-trigger",
            "data-state": state_str,
            "aria-expanded": if is_open { "true" } else { "false" },
            r#type: "button",
            disabled: is_disabled,
            class: props.class,
            onclick: move |e| {
                if !is_disabled {
                    ctx.toggle();
                    if let Some(handler) = &props.onclick {
                        handler.call(e);
                    }
                }
            },
            {props.children}
        }
    }
}

// ============================================================
// Collapsible Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CollapsibleContentProps {
    #[props(default = false)]
    pub force_mount: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn CollapsibleContent(props: CollapsibleContentProps) -> Element {
    let ctx = use_context::<CollapsibleContext>();
    let is_open = ctx.is_open();

    if !is_open && !props.force_mount {
        return rsx! {};
    }

    let state_str = if is_open { "open" } else { "closed" };
    let hidden_class = if !is_open { "hidden" } else { "" };

    rsx! {
        div {
            "data-slot": "collapsible-content",
            "data-state": state_str,
            class: format!(
                "overflow-hidden {} {}",
                hidden_class, props.class
            ),
            {props.children}
        }
    }
}
