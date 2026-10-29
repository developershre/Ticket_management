use dioxus::prelude::*;

// ============================================================
// Dropdown Menu Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuProps {
    pub children: Element,
}

#[component]
pub fn DropdownMenu(props: DropdownMenuProps) -> Element {
    let open = use_signal(|| false);

    use_context_provider(|| DropdownMenuContext { open });

    rsx! {
        div {
            "data-slot": "dropdown-menu",
            class: "relative inline-block",

            {props.children}
        }
    }
}

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct DropdownMenuContext {
    pub open: Signal<bool>,
}

// ============================================================
// Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuTriggerProps {
    #[props(default)]
    pub class: String,

    #[props(default)]
    pub disabled: bool,

    pub children: Element,
}

#[component]
pub fn DropdownMenuTrigger(props: DropdownMenuTriggerProps) -> Element {
    let mut menu = use_context::<DropdownMenuContext>();

    let classes = format!(
        "
        inline-flex
        items-center
        justify-center
        gap-2
        rounded-md
        text-sm
        font-medium

        outline-none

        hover:bg-accent
        hover:text-accent-foreground

        focus-visible:ring-2
        focus-visible:ring-ring

        disabled:pointer-events-none
        disabled:opacity-50

        {}
        ",
        props.class
    );

    rsx! {
        button {
            "data-slot": "dropdown-menu-trigger",

            type: "button",

            class: classes,

            disabled: props.disabled,

            onmousedown: move |e| {
                e.stop_propagation();
            },

            onclick: move |e| {
                e.stop_propagation();
                let current = (menu.open)();
                menu.open.set(!current);
            },

            {props.children}
        }
    }
}

// ============================================================
// Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuContentProps {
    #[props(default = "start".to_string())]
    pub align: String,

    #[props(default = "bottom".to_string())]
    pub side: String,

    #[props(default = 4)]
    pub side_offset: i32,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DropdownMenuContent(props: DropdownMenuContentProps) -> Element {
    let mut menu = use_context::<DropdownMenuContext>();
    let uid = use_hook(super::popup::next_popup_id);
    let open = menu.open;
    let side = props.side.clone();
    let align = props.align.clone();
    let offset = props.side_offset;
    let effect_uid = uid.clone();
    let listener_uid = uid.clone();

    // Position the popup when it opens
    use_effect(move || {
        if open() {
            super::popup::position_popup(
                &effect_uid,
                "[data-slot=\"dropdown-menu\"]",
                "[data-slot=\"dropdown-menu-trigger\"]",
                &side,
                &align,
                offset,
                false,
            );
        }
    });

    // Global click-outside listener using document event — avoids z-index stacking context issues
    use_effect(move || {
        if open() {
            let close_id = format!("__dd_close_{}", listener_uid);
            let js = format!(
                r#"
                (function() {{
                    var handlerName = "{}";
                    // Remove previous handler if any
                    if (window[handlerName]) {{
                        document.removeEventListener("pointerdown", window[handlerName], true);
                    }}
                    window[handlerName] = function(e) {{
                        var popup = document.querySelector('[data-popup="{}"]');
                        if (!popup) return;
                        var content = popup.querySelector('[data-slot="dropdown-menu-content"]');
                        if (!content) return;
                        // Check if click is inside the dropdown content or a trigger
                        var target = e.target;
                        if (content.contains(target)) return;
                        // Check if it's a trigger button
                        var trigger = target.closest('[data-slot="dropdown-menu-trigger"]');
                        if (trigger && popup.closest('[data-slot="dropdown-menu"]') &&
                            popup.closest('[data-slot="dropdown-menu"]').contains(trigger)) return;
                        // Click was outside — close via a synthetic event
                        var closeBtn = document.getElementById("{}");
                        if (closeBtn) closeBtn.click();
                    }};
                    // Use capture phase so we get the event before anything else
                    document.addEventListener("pointerdown", window[handlerName], true);
                }})();
                "#,
                close_id, listener_uid, close_id
            );
            let _ = document::eval(&js);
        } else {
            // Cleanup listener when closed
            let close_id = format!("__dd_close_{}", listener_uid);
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

    if !(menu.open)() {
        return rsx! {};
    }

    let close_btn_id = format!("__dd_close_{}", uid);

    let classes = format!(
        "
        min-w-32

        overflow-x-hidden
        overflow-y-auto

        rounded-lg
        border
        border-border

        bg-popover
        p-1

        text-popover-foreground

        shadow-md
        ring-1
        ring-foreground/10

        outline-none

        animate-in
        fade-in-0
        zoom-in-95

        {}
        ",
        props.class
    );

    rsx! {
        // Hidden close button triggered by the global pointerdown listener
        button {
            id: close_btn_id,
            class: "hidden",
            onclick: move |_| {
                menu.open.set(false);
            },
        }

        div {
            "data-popup": uid,
            class: "fixed left-0 top-0 z-50 invisible",

            div {
                "data-slot": "dropdown-menu-content",

                class: classes,

                onmousedown: move |event| {
                    event.stop_propagation();
                },

                onclick: move |event| {
                    event.stop_propagation();
                },

                {props.children}
            }
        }
    }
}

// ============================================================
// Group
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuGroupProps {
    pub children: Element,
}

#[component]
pub fn DropdownMenuGroup(props: DropdownMenuGroupProps) -> Element {
    rsx! {
        div {
            "data-slot": "dropdown-menu-group",

            {props.children}
        }
    }
}

// ============================================================
// Label
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuLabelProps {
    #[props(default = false)]
    pub inset: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DropdownMenuLabel(props: DropdownMenuLabelProps) -> Element {
    let inset_class = if props.inset { "pl-7" } else { "" };

    let classes = format!(
        "
        px-1.5
        py-1

        text-xs
        font-medium
        text-muted-foreground

        {}

        {}
        ",
        inset_class, props.class
    );

    rsx! {
        div {
            "data-slot": "dropdown-menu-label",

            class: classes,

            {props.children}
        }
    }
}

// ============================================================
// Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuItemProps {
    #[props(default = "default".to_string())]
    pub variant: String,

    #[props(default = false)]
    pub inset: bool,

    #[props(default)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn DropdownMenuItem(props: DropdownMenuItemProps) -> Element {
    let mut menu = use_context::<DropdownMenuContext>();

    let variant_class = if props.variant == "destructive" {
        "
        text-destructive
        focus:bg-destructive/10
        focus:text-destructive
        dark:focus:bg-destructive/20
        "
    } else {
        "
        focus:bg-accent
        focus:text-accent-foreground
        "
    };

    let inset_class = if props.inset { "pl-7" } else { "" };

    let classes = format!(
        "
        group/dropdown-menu-item

        relative
        flex
        w-full

        cursor-pointer
        items-center

        gap-1.5

        rounded-md

        px-2
        py-1.5

        text-xs

        outline-none
        select-none

        hover:bg-accent
        hover:text-accent-foreground

        {}

        {}

        disabled:pointer-events-none
        disabled:opacity-50

        [&_svg]:pointer-events-none
        [&_svg]:shrink-0
        [&_svg:not([class*='size-'])]:size-4

        {}
        ",
        variant_class, inset_class, props.class
    );

    rsx! {
        button {
            "data-slot": "dropdown-menu-item",

            type: "button",

            class: classes,

            disabled: props.disabled,

            onmousedown: move |e| {
                e.stop_propagation();
            },

            onclick: move |event| {
                event.stop_propagation();
                if let Some(handler) = &props.onclick {
                    handler.call(event);
                }

                menu.open.set(false);
            },

            {props.children}
        }
    }
}

// ============================================================
// Separator
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuSeparatorProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn DropdownMenuSeparator(props: DropdownMenuSeparatorProps) -> Element {
    let classes = format!(
        "
        -mx-1
        my-1
        h-px
        bg-border

        {}
        ",
        props.class
    );

    rsx! {
        div {
            "data-slot": "dropdown-menu-separator",
            class: classes,
        }
    }
}

// ============================================================
// Shortcut
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuShortcutProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DropdownMenuShortcut(props: DropdownMenuShortcutProps) -> Element {
    let classes = format!(
        "
        ml-auto

        text-xs
        tracking-widest

        text-muted-foreground

        group-focus/dropdown-menu-item:text-accent-foreground

        {}

        ",
        props.class
    );

    rsx! {
        span {
            "data-slot": "dropdown-menu-shortcut",

            class: classes,

            {props.children}
        }
    }
}

// ============================================================
// Checkbox Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuCheckboxItemProps {
    #[props(default = false)]
    pub checked: bool,

    #[props(default = false)]
    pub inset: bool,

    #[props(default)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub onchange: Option<EventHandler<bool>>,

    pub children: Element,
}

#[component]
pub fn DropdownMenuCheckboxItem(props: DropdownMenuCheckboxItemProps) -> Element {
    let mut checked = use_signal(|| props.checked);

    let inset_class = if props.inset { "pl-7" } else { "" };

    let classes = format!(
        "
        relative
        flex
        w-full

        cursor-default
        items-center

        gap-1.5

        rounded-md

        py-1
        pr-8
        pl-1.5

        text-sm

        outline-none
        select-none

        focus:bg-accent
        focus:text-accent-foreground

        disabled:pointer-events-none
        disabled:opacity-50

        {}

        {}
        ",
        inset_class, props.class
    );

    rsx! {
        button {
            "data-slot": "dropdown-menu-checkbox-item",

            type: "button",

            class: classes,

            disabled: props.disabled,

            onclick: move |_| {
                let new_value = !checked();
                checked.set(new_value);

                if let Some(handler) = &props.onchange {
                    handler.call(new_value);
                }
            },

            span {
                class: "
                    pointer-events-none
                    absolute
                    right-2

                    flex
                    items-center
                    justify-center

                    size-4
                ",

                if checked() {
                    span {
                        class: "text-current",

                        "✓"
                    }
                }
            }

            {props.children}
        }
    }
}

// ============================================================
// Radio Group
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuRadioGroupProps {
    pub value: Signal<String>,

    pub children: Element,
}

#[component]
pub fn DropdownMenuRadioGroup(props: DropdownMenuRadioGroupProps) -> Element {
    use_context_provider(|| props.value);

    rsx! {
        div {
            "data-slot": "dropdown-menu-radio-group",

            {props.children}
        }
    }
}

// ============================================================
// Radio Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuRadioItemProps {
    pub value: String,

    #[props(default = false)]
    pub inset: bool,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DropdownMenuRadioItem(props: DropdownMenuRadioItemProps) -> Element {
    let mut selected = use_context::<Signal<String>>();

    let is_selected = selected() == props.value;

    let inset_class = if props.inset { "pl-7" } else { "" };

    let classes = format!(
        "
        relative
        flex
        w-full

        cursor-default
        items-center

        gap-1.5

        rounded-md

        py-1
        pr-8
        pl-1.5

        text-sm

        outline-none
        select-none

        focus:bg-accent
        focus:text-accent-foreground

        disabled:pointer-events-none
        disabled:opacity-50

        {}

        {}
        ",
        inset_class, props.class
    );

    rsx! {
        button {
            "data-slot": "dropdown-menu-radio-item",

            type: "button",

            class: classes,

            disabled: props.disabled,

            onclick: move |_| {
                selected.set(props.value.clone());
            },

            span {
                class: "
                    pointer-events-none
                    absolute
                    right-2

                    flex
                    items-center
                    justify-center

                    size-4
                ",

                if is_selected {
                    span {
                        class: "
                            size-2
                            rounded-full
                            bg-current
                        "
                    }
                }
            }

            {props.children}
        }
    }
}

// ============================================================
// Sub Menu
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuSubProps {
    pub children: Element,
}

#[component]
pub fn DropdownMenuSub(props: DropdownMenuSubProps) -> Element {
    let open = use_signal(|| false);

    use_context_provider(|| open);

    rsx! {
        div {
            "data-slot": "dropdown-menu-sub",
            class: "relative",

            {props.children}
        }
    }
}

// ============================================================
// Sub Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuSubTriggerProps {
    #[props(default = false)]
    pub inset: bool,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DropdownMenuSubTrigger(props: DropdownMenuSubTriggerProps) -> Element {
    let mut sub_open = use_context::<Signal<bool>>();

    let inset_class = if props.inset { "pl-7" } else { "" };

    let classes = format!(
        "
        flex
        w-full

        cursor-default
        items-center

        gap-1.5

        rounded-md

        px-1.5
        py-1

        text-sm

        outline-none
        select-none

        focus:bg-accent
        focus:text-accent-foreground

        {}

        {}

        [&_svg]:pointer-events-none
        [&_svg]:shrink-0
        [&_svg:not([class*='size-'])]:size-4
        ",
        inset_class, props.class
    );

    rsx! {
        button {
            "data-slot": "dropdown-menu-sub-trigger",

            type: "button",

            class: classes,

            "aria-expanded": if (sub_open)() { "true" } else { "false" },

            onclick: move |_| {
                let current = sub_open();
                sub_open.set(!current);
            },

            {props.children}

            // Chevron
            span {
                class: "ml-auto",

                "›"
            }
        }
    }
}

// ============================================================
// Sub Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuSubContentProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn DropdownMenuSubContent(props: DropdownMenuSubContentProps) -> Element {
    let sub_open = use_context::<Signal<bool>>();

    if !(sub_open)() {
        return rsx! {};
    }

    let classes = format!(
        "
        absolute
        left-full
        top-0

        z-30

        min-w-[96px]

        rounded-lg

        border
        border-border

        bg-popover

        p-1

        text-popover-foreground

        shadow-lg

        ring-1
        ring-foreground/10

        animate-in
        fade-in-0
        zoom-in-95

        {}

        ",
        props.class
    );

    rsx! {
        div {
            "data-slot": "dropdown-menu-sub-content",

            class: classes,

            {props.children}
        }
    }
}

// ============================================================
// Portal
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct DropdownMenuPortalProps {
    pub children: Element,
}

#[component]
pub fn DropdownMenuPortal(props: DropdownMenuPortalProps) -> Element {
    rsx! {
        {props.children}
    }
}
