use dioxus::prelude::*;

#[component]
pub fn InputGroup(children: Element) -> Element {
    rsx! {
        div {
            "data-slot": "input-group",

            class: "
                flex
                w-full
                min-w-0
                items-stretch
                rounded-md
                border
                border-input
                bg-transparent
                shadow-xs
                transition-[color,box-shadow]

                focus-within:border-ring
                focus-within:ring-ring/50
                focus-within:ring-[3px]

                has-[[data-slot=input-group-control][aria-invalid=true]]:
                    border-destructive

                has-[[data-slot=input-group-control][aria-invalid=true]]:
                    ring-destructive/20

                dark:has-[[data-slot=input-group-control][aria-invalid=true]]:
                    ring-destructive/40
            ",

            {children}
        }
    }
}

// ============================================================
// Input Group Addon
// ============================================================

#[derive(Clone, PartialEq)]
#[allow(dead_code)]
pub enum InputGroupAddonPosition {
    InlineStart,
    InlineEnd,
    BlockStart,
    BlockEnd,
}

#[derive(Props, Clone, PartialEq)]
pub struct InputGroupAddonProps {
    pub position: InputGroupAddonPosition,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn InputGroupAddon(props: InputGroupAddonProps) -> Element {
    let position_class = match props.position {
        InputGroupAddonPosition::InlineStart => "border-e border-input pl-3",

        InputGroupAddonPosition::InlineEnd => "border-s border-input pr-3",

        InputGroupAddonPosition::BlockStart => "border-b border-input px-3 py-2",

        InputGroupAddonPosition::BlockEnd => "border-t border-input px-3 py-2",
    };

    let classes = format!(
        "
        flex
        items-center
        gap-2
        text-sm
        text-muted-foreground
        [&>svg]:pointer-events-none
        [&>svg]:size-4
        [&>svg]:shrink-0

        {}
        {}
        ",
        position_class, props.class
    );

    rsx! {
        div {
            "data-slot": "input-group-addon",
            class: classes,

            {props.children}
        }
    }
}

// ============================================================
// Input Group Button
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct InputGroupButtonProps {
    #[props(default = "outline".to_string())]
    pub variant: String,

    #[props(default = "sm".to_string())]
    pub size: String,

    #[props(default)]
    pub class: String,

    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn InputGroupButton(props: InputGroupButtonProps) -> Element {
    let variant_class = match props.variant.as_str() {
        "ghost" => {
            "
            hover:bg-accent
            hover:text-accent-foreground
        "
        }

        "outline" => {
            "
            border
            border-input
            bg-background
            shadow-xs
            hover:bg-accent
            hover:text-accent-foreground
        "
        }

        _ => {
            "
            bg-primary
            text-primary-foreground
            hover:bg-primary/90
        "
        }
    };

    let size_class = match props.size.as_str() {
        "icon" => {
            "
            size-8
            p-0
        "
        }

        "lg" => {
            "
            h-10
            px-4
        "
        }

        _ => {
            "
            h-8
            px-3
        "
        }
    };

    let classes = format!(
        "
        inline-flex
        shrink-0
        items-center
        justify-center
        gap-2
        rounded-md

        text-sm
        font-medium
        whitespace-nowrap

        transition-colors
        outline-none

        disabled:pointer-events-none
        disabled:opacity-50

        focus-visible:border-ring
        focus-visible:ring-ring/50
        focus-visible:ring-[3px]

        {}

        {}

        {}
        ",
        variant_class, size_class, props.class
    );

    rsx! {
        button {
            "data-slot": "input-group-button",

            type: "button",

            class: classes,

            onclick: move |event| {
                if let Some(handler) = &props.onclick {
                    handler.call(event);
                }
            },

            {props.children}
        }
    }
}

// ============================================================
// Input Group Text
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct InputGroupTextProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn InputGroupText(props: InputGroupTextProps) -> Element {
    let classes = format!(
        "
        flex
        items-center
        gap-2

        text-sm
        text-muted-foreground

        [&>svg]:pointer-events-none
        [&>svg]:size-4
        [&>svg]:shrink-0

        {}
        ",
        props.class
    );

    rsx! {
        span {
            "data-slot": "input-group-text",

            class: classes,

            {props.children}
        }
    }
}

// ============================================================
// Input Group Input Wrapper
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct InputGroupControlProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn InputGroupControl(props: InputGroupControlProps) -> Element {
    let classes = format!(
        "
        flex
        min-w-0
        flex-1

        border-0
        bg-transparent

        shadow-none
        outline-none

        focus-visible:ring-0

        {}
        ",
        props.class
    );

    rsx! {
        div {
            "data-slot": "input-group-control",

            class: classes,

            {props.children}
        }
    }
}
