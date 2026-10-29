use dioxus::prelude::*;

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct TabsContext {
    pub value: Signal<String>,
    pub orientation: Signal<String>,
}

// ============================================================
// Tabs
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TabsProps {
    #[props(default)]
    pub value: String,

    #[props(default = "horizontal".to_string())]
    pub orientation: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Tabs(props: TabsProps) -> Element {
    let value = use_signal(|| props.value.clone());
    let orientation = use_signal(|| props.orientation.clone());

    use_context_provider(|| TabsContext { value, orientation });

    let orientation_class = if orientation() == "vertical" {
        "flex"
    } else {
        "flex flex-col"
    };

    rsx! {
        div {
            "data-slot": "tabs",
            "data-orientation": "{orientation()}",
            class: "group/tabs {orientation_class} gap-2 {props.class}",

            {props.children}
        }
    }
}

// ============================================================
// Tabs List
// ============================================================

#[derive(Clone, PartialEq, Default)]
#[allow(dead_code)]
pub enum TabsListVariant {
    #[default]
    Default,
    Line,
}

impl TabsListVariant {
    #[allow(dead_code)]
    fn as_str(&self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Line => "line",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TabsListProps {
    #[props(default)]
    pub variant: TabsListVariant,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn TabsList(props: TabsListProps) -> Element {
    let tabs = use_context::<TabsContext>();

    let variant_class = match props.variant {
        TabsListVariant::Default => "bg-muted",
        TabsListVariant::Line => "gap-1 bg-transparent",
    };

    let orientation_class = if (tabs.orientation)() == "vertical" {
        "h-fit flex-col"
    } else {
        "h-8"
    };

    rsx! {
        div {
            "data-slot": "tabs-list",
            "data-variant": "{props.variant.as_str()}",

            class: "
                group/tabs-list
                inline-flex
                w-fit
                items-center
                justify-center
                rounded-lg
                p-[3px]
                text-muted-foreground
                {orientation_class}
                {variant_class}
                {props.class}
            ",

            {props.children}
        }
    }
}

// ============================================================
// Tabs Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TabsTriggerProps {
    pub value: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn TabsTrigger(props: TabsTriggerProps) -> Element {
    let mut tabs = use_context::<TabsContext>();

    let is_active = (tabs.value)() == props.value;

    let orientation_class = if (tabs.orientation)() == "vertical" {
        "w-full justify-start"
    } else {
        ""
    };

    let active_class = if is_active {
        "bg-background text-foreground"
    } else {
        "text-foreground/60"
    };

    let value = props.value.clone();

    rsx! {
        button {
            r#type: "button",

            "data-slot": "tabs-trigger",
            "data-state": if is_active { "active" } else { "inactive" },
            "data-active": if is_active { "true" } else { "false" },

            class: "
                relative
                inline-flex
                h-[calc(100%-1px)]
                flex-1
                items-center
                justify-center
                gap-1.5
                rounded-md
                border
                border-transparent
                px-1.5
                py-0.5
                text-sm
                font-medium
                whitespace-nowrap
                transition-all

                hover:text-foreground

                focus-visible:border-ring
                focus-visible:outline-1
                focus-visible:outline-ring

                disabled:pointer-events-none
                disabled:opacity-50

                {orientation_class}
                {active_class}
                {props.class}
            ",

            onclick: move |_| {
                tabs.value.set(value.clone());
            },

            {props.children}
        }
    }
}

// ============================================================
// Tabs Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct TabsContentProps {
    pub value: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn TabsContent(props: TabsContentProps) -> Element {
    let tabs = use_context::<TabsContext>();

    if (tabs.value)() != props.value {
        return rsx! {};
    }

    rsx! {
        div {
            "data-slot": "tabs-content",

            class: "
                flex-1
                text-sm
                outline-none
                {props.class}
            ",

            {props.children}
        }
    }
}
