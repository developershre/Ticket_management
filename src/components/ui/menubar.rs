#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
struct MenubarContext {
    active_menu: Signal<Option<String>>,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Menubar(props: MenubarProps) -> Element {
    let active_menu = use_signal(|| None);
    use_context_provider(|| MenubarContext { active_menu });

    let class = format!(
        "flex h-9 items-center space-x-1 rounded-md border border-border bg-background p-1 shadow-xs {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "menubar",
            role: "menubar",
            class: class,
            {props.children}
        }
    }
}

#[derive(Clone, PartialEq)]
struct MenubarMenuContext {
    id: String,
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarMenuProps {
    pub id: String,
    pub children: Element,
}

#[component]
pub fn MenubarMenu(props: MenubarMenuProps) -> Element {
    let id = props.id.clone();
    use_context_provider(|| MenubarMenuContext { id });

    rsx! {
        div {
            "data-slot": "menubar-menu",
            class: "relative inline-block",
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarTriggerProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn MenubarTrigger(props: MenubarTriggerProps) -> Element {
    let mut bar_ctx = use_context::<MenubarContext>();
    let menu_ctx = use_context::<MenubarMenuContext>();

    let is_open = (bar_ctx.active_menu)().as_deref() == Some(&menu_ctx.id);
    let menu_id = menu_ctx.id.clone();

    let open_cls = if is_open {
        "bg-accent text-accent-foreground"
    } else {
        ""
    };

    let class = format!(
        "flex cursor-pointer select-none items-center rounded-sm px-2.5 py-1 text-xs font-medium outline-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground transition-colors {} {}",
        open_cls,
        props.class
    );

    rsx! {
        button {
            "data-slot": "menubar-trigger",
            r#type: "button",
            role: "menuitem",
            "aria-expanded": if is_open { "true" } else { "false" },
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if is_open {
                    bar_ctx.active_menu.set(None);
                } else {
                    bar_ctx.active_menu.set(Some(menu_id.clone()));
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarContentProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn MenubarContent(props: MenubarContentProps) -> Element {
    let mut bar_ctx = use_context::<MenubarContext>();
    let menu_ctx = use_context::<MenubarMenuContext>();

    let is_open = (bar_ctx.active_menu)().as_deref() == Some(&menu_ctx.id);

    if !is_open {
        return rsx! {};
    }

    let class = format!(
        "absolute left-0 top-full z-50 min-w-48 mt-1 overflow-hidden rounded-md border border-border bg-popover p-1 text-popover-foreground shadow-md backdrop-blur-sm animate-in fade-in-0 zoom-in-95 {}",
        props.class
    );

    rsx! {
        div { class: "contents",
            // Backdrop dismiss overlay
            div {
                class: "fixed inset-0 z-40 bg-transparent",
                onclick: move |e| {
                    e.stop_propagation();
                    bar_ctx.active_menu.set(None);
                },
            }
            div {
                "data-slot": "menubar-content",
                role: "menu",
                class: class,
                onclick: move |e| e.stop_propagation(),
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarItemProps {
    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub class: String,

    pub on_select: Option<EventHandler<()>>,

    pub children: Element,
}

#[component]
pub fn MenubarItem(props: MenubarItemProps) -> Element {
    let mut bar_ctx = use_context::<MenubarContext>();

    let disabled_cls = if props.disabled {
        "pointer-events-none opacity-50"
    } else {
        "cursor-pointer"
    };

    let class = format!(
        "relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-xs outline-none hover:bg-accent hover:text-accent-foreground focus:bg-accent focus:text-accent-foreground transition-colors {} {}",
        disabled_cls,
        props.class
    );

    rsx! {
        div {
            "data-slot": "menubar-item",
            role: "menuitem",
            class: class,
            onclick: move |e| {
                e.stop_propagation();
                if !props.disabled {
                    if let Some(handler) = &props.on_select {
                        handler.call(());
                    }
                    bar_ctx.active_menu.set(None);
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarSeparatorProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn MenubarSeparator(props: MenubarSeparatorProps) -> Element {
    rsx! {
        div {
            "data-slot": "menubar-separator",
            role: "separator",
            class: "-mx-1 my-1 h-px bg-muted {props.class}",
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MenubarShortcutProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn MenubarShortcut(props: MenubarShortcutProps) -> Element {
    rsx! {
        span {
            "data-slot": "menubar-shortcut",
            class: "ml-auto text-[10px] tracking-widest text-muted-foreground {props.class}",
            {props.children}
        }
    }
}
