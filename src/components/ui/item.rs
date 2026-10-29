use dioxus::prelude::*;

// ============================================================
// Item Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ItemProps {
    #[props(default)]
    pub class: String,

    #[props(default = false)]
    pub active: bool,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn Item(props: ItemProps) -> Element {
    let active_class = if props.active {
        "bg-accent border-accent-foreground/20"
    } else {
        "bg-card hover:bg-accent/50"
    };

    let disabled_class = if props.disabled {
        "opacity-50 pointer-events-none"
    } else {
        "cursor-pointer"
    };

    let class = format!(
        "group/item relative flex items-center justify-between gap-4 rounded-xl border \
         border-border p-3.5 text-card-foreground shadow-xs transition-colors outline-none \
         focus-visible:ring-2 focus-visible:ring-ring select-none {} {} {}",
        active_class, disabled_class, props.class
    );

    rsx! {
        div {
            "data-slot": "item",
            class: class,
            onclick: move |e| {
                if !props.disabled {
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
// Item Media / Icon
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ItemMediaProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ItemMedia(props: ItemMediaProps) -> Element {
    let class = format!(
        "flex size-10 shrink-0 items-center justify-center rounded-lg \
         bg-muted text-muted-foreground transition-colors \
         group-hover/item:text-foreground [&_svg]:size-5 {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "item-media",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Item Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ItemContentProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ItemContent(props: ItemContentProps) -> Element {
    let class = format!("flex flex-1 flex-col gap-1 min-w-0 {}", props.class);

    rsx! {
        div {
            "data-slot": "item-content",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Item Title
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ItemTitleProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ItemTitle(props: ItemTitleProps) -> Element {
    let class = format!(
        "text-sm font-medium leading-none text-foreground truncate {}",
        props.class
    );

    rsx! {
        h4 {
            "data-slot": "item-title",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Item Description
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ItemDescriptionProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ItemDescription(props: ItemDescriptionProps) -> Element {
    let class = format!("text-xs text-muted-foreground line-clamp-1 {}", props.class);

    rsx! {
        p {
            "data-slot": "item-description",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Item Actions
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct ItemActionsProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ItemActions(props: ItemActionsProps) -> Element {
    let class = format!("flex items-center gap-2 shrink-0 ml-auto {}", props.class);

    rsx! {
        div {
            "data-slot": "item-actions",
            class: class,
            {props.children}
        }
    }
}
