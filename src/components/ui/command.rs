use dioxus::prelude::*;

// ============================================================
// Command
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Command(props: CommandProps) -> Element {
    let class = format!(
        "flex size-full flex-col overflow-hidden rounded-xl \
         bg-popover p-1 text-popover-foreground {}",
        props.class
    );

    rsx! {
        div {
            class: class,
            "data-slot": "command",

            {props.children}
        }
    }
}

// ============================================================
// CommandDialog
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandDialogProps {
    #[props(default = "Command Palette".to_string())]
    pub title: String,

    #[props(default = "Search for a command to run...".to_string())]
    pub description: String,

    #[props(default)]
    pub class: String,

    #[props(default = false)]
    pub show_close_button: bool,

    pub open: Signal<bool>,

    pub children: Element,
}

#[component]
pub fn CommandDialog(mut props: CommandDialogProps) -> Element {
    if !*props.open.read() {
        return rsx! {};
    }

    let top_class = if props.class.contains("top-") {
        ""
    } else {
        "top-2 "
    };

    let class = format!(
        "fixed left-1/2 {top_class}z-20 w-[calc(100%-2rem)] max-w-xl \
         -translate-x-1/2 overflow-hidden rounded-xl \
         border bg-popover p-0 shadow-2xl {}",
        props.class
    );

    rsx! {
        // Backdrop - clicking or pressing outside closes the command dialog
        div {
            class: "fixed inset-0 z-20 bg-black/10 backdrop-blur-[1px]",

            onmousedown: move |e| {
                e.stop_propagation();
                props.open.set(false);
            },

            onclick: move |e| {
                e.stop_propagation();
                props.open.set(false);
            },
        }

        // Dialog container
        div {
            class: class,
            role: "dialog",
            "aria-modal": "true",

            onmousedown: move |e| {
                e.stop_propagation();
            },

            onclick: move |e| {
                e.stop_propagation();
            },

            onkeydown: move |e| {
                if e.key() == Key::Escape {
                    props.open.set(false);
                }
            },

            div {
                class: "sr-only",

                h2 {
                    "{props.title}"
                }

                p {
                    "{props.description}"
                }
            }

            {props.children}
        }
    }
}

// ============================================================
// CommandInput
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandInputProps {
    #[props(default)]
    pub class: String,

    #[props(default = "Search...".to_string())]
    pub placeholder: String,

    pub value: Signal<String>,

    #[props(default)]
    pub oninput: Option<EventHandler<FormEvent>>,

    #[props(default)]
    pub onkeydown: Option<EventHandler<KeyboardEvent>>,
}

#[component]
pub fn CommandInput(mut props: CommandInputProps) -> Element {
    let class = format!(
        "w-full bg-transparent text-sm outline-none \
         disabled:cursor-not-allowed disabled:opacity-50 {}",
        props.class
    );

    rsx! {
        div {
            class: "p-1 pb-0",

            div {
                class: "flex h-8 w-full items-center rounded-lg \
                        border border-input/30 bg-input/30",

                // Search icon
                div {
                    class: "flex shrink-0 items-center px-2",

                    SearchIcon {
                        class: "size-4 shrink-0 opacity-50",
                    }
                }

                input {
                    class: class,
                    value: props.value.read().clone(),
                    placeholder: props.placeholder.clone(),
                    autofocus: true,

                    oninput: move |event| {
                        props.value.set(event.value().clone());

                        if let Some(handler) = &props.oninput {
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
    }
}
// ============================================================
// CommandList
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandListProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn CommandList(props: CommandListProps) -> Element {
    let class = format!(
        "max-h-72 scroll-py-1 overflow-x-hidden \
         overflow-y-auto outline-none {}",
        props.class
    );

    rsx! {
        div {
            class: class,
            "data-slot": "command-list",

            {props.children}
        }
    }
}

// ============================================================
// CommandEmpty
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandEmptyProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn CommandEmpty(props: CommandEmptyProps) -> Element {
    let class = format!("py-6 text-center text-sm {}", props.class);

    rsx! {
        div {
            class: class,
            "data-slot": "command-empty",

            {props.children}
        }
    }
}

// ============================================================
// CommandGroup
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandGroupProps {
    #[props(default)]
    pub class: String,

    #[props(default)]
    pub heading: String,

    pub children: Element,
}

#[component]
pub fn CommandGroup(props: CommandGroupProps) -> Element {
    let class = format!("overflow-hidden p-1 text-foreground {}", props.class);

    rsx! {
        div {
            class: class,
            "data-slot": "command-group",

            if !props.heading.is_empty() {
                div {
                    class: "px-2 py-1.5 text-xs font-medium \
                            text-muted-foreground",

                    "{props.heading}"
                }
            }

            {props.children}
        }
    }
}

// ============================================================
// CommandSeparator
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandSeparatorProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn CommandSeparator(props: CommandSeparatorProps) -> Element {
    let class = format!("-mx-1 h-px bg-border {}", props.class);

    rsx! {
        div {
            class: class,
            "data-slot": "command-separator",
        }
    }
}

// ============================================================
// CommandItem
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandItemProps {
    #[props(default)]
    pub class: String,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default = false)]
    pub selected: bool,

    #[props(default = false)]
    pub checked: bool,

    pub children: Element,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
}

#[component]
pub fn CommandItem(props: CommandItemProps) -> Element {
    let mut class = String::from(
        "group/command-item relative flex cursor-pointer \
         items-center gap-2 rounded-md px-2 py-1.5 \
         text-sm outline-none select-none hover:bg-muted \
         transition-colors \
         [&_svg]:pointer-events-none \
         [&_svg]:shrink-0 \
         [&_svg:not([class*='size-'])]:size-4",
    );

    if props.selected {
        class.push_str(" bg-muted text-foreground");
    }

    if props.disabled {
        class.push_str(" pointer-events-none opacity-50");
    }

    if !props.class.is_empty() {
        class.push(' ');
        class.push_str(&props.class);
    }

    rsx! {
        div {
            class: class,
            "data-slot": "command-item",

            "data-selected": if props.selected {
                "true"
            } else {
                "false"
            },

            "data-disabled": if props.disabled {
                "true"
            } else {
                "false"
            },

            "data-checked": if props.checked {
                "true"
            } else {
                "false"
            },

            onclick: move |event| {
                if !props.disabled {
                    if let Some(handler) = &props.onclick {
                        handler.call(event);
                    }
                }
            },

            {props.children}

            if props.checked {
                div {
                    class: "ml-auto",

                    CheckIcon {
                        class: "size-4",
                    }
                }
            }
        }
    }
}

// ============================================================
// CommandShortcut
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CommandShortcutProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn CommandShortcut(props: CommandShortcutProps) -> Element {
    let class = format!(
        "ml-auto text-xs tracking-widest \
         text-muted-foreground {}",
        props.class
    );

    rsx! {
        span {
            class: class,
            "data-slot": "command-shortcut",

            {props.children}
        }
    }
}

// ============================================================
// SearchIcon
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SearchIconProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn SearchIcon(props: SearchIconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",

            path {
                d: "M21 21L16.6569 16.6569M16.6569 16.6569C18.1046 15.2091 19 13.2091 19 11C19 6.58172 15.4183 3 11 3C6.58172 3 3 6.58172 3 11C3 15.4183 6.58172 19 11 19C13.209 19 15.209 18.1041 16.6569 16.6569Z",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
        }
    }
}

// ============================================================
// CheckIcon
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct CheckIconProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn CheckIcon(props: CheckIconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",

            path {
                d: "M20 6L9 17L4 12",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
            }
        }
    }
}
