use crate::components::ui::tooltip::{Tooltip, TooltipContent, TooltipTrigger};
use dioxus::prelude::*;

// ============================================================
// Collapsible Modes
// ============================================================

#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[allow(dead_code)]
pub enum SidebarCollapsible {
    #[default]
    Icon,
    Offcanvas,
    None,
}

// ============================================================
// Context
// ============================================================

#[derive(Clone, Copy, PartialEq)]
pub struct SidebarContext {
    pub open: Signal<bool>,
    pub collapsible: Signal<SidebarCollapsible>,
}

impl SidebarContext {
    pub fn is_open(&self) -> bool {
        *self.open.read()
    }

    #[allow(dead_code)]
    pub fn is_collapsed(&self) -> bool {
        !self.is_open()
    }

    pub fn toggle(&mut self) {
        let current = self.is_open();
        self.open.set(!current);
    }
}

// ============================================================
// Sidebar Provider
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProviderProps {
    #[props(default = true)]
    pub default_open: bool,

    #[props(default)]
    pub open: Option<Signal<bool>>,

    #[props(default)]
    pub collapsible: SidebarCollapsible,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarProvider(props: SidebarProviderProps) -> Element {
    let default_val = props.default_open;
    let local_open = use_signal(move || default_val);
    let open = props.open.unwrap_or(local_open);
    let coll_val = props.collapsible;
    let collapsible = use_signal(move || coll_val);

    use_context_provider(|| SidebarContext { open, collapsible });

    let is_open = *open.read();

    rsx! {
        div {
            "data-slot": "sidebar-wrapper",
            "data-state": if is_open { "expanded" } else { "collapsed" },
            class: format!("group/sidebar-wrapper flex h-full w-full overflow-hidden {}", props.class),
            {props.children}
        }
    }
}

// ============================================================
// Sidebar Container
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarProps {
    #[props(default)]
    pub collapsible: SidebarCollapsible,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Sidebar(props: SidebarProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    let width_class = match (props.collapsible, is_open) {
        (SidebarCollapsible::None, _) => "w-52 sm:w-60",
        (SidebarCollapsible::Offcanvas, true) => "w-52 sm:w-60",
        (SidebarCollapsible::Offcanvas, false) => "w-0 -ml-60",
        (SidebarCollapsible::Icon, true) => "w-52 sm:w-60",
        (SidebarCollapsible::Icon, false) => "w-12",
    };

    let state_str = if is_open { "expanded" } else { "collapsed" };
    let collapsible_str = match props.collapsible {
        SidebarCollapsible::Icon => "icon",
        SidebarCollapsible::Offcanvas => "offcanvas",
        SidebarCollapsible::None => "none",
    };

    let mobile_class = if is_open {
        "max-md:fixed max-md:inset-y-0 max-md:left-0 max-md:z-10 max-md:bg-sidebar max-md:border-r max-md:border-sidebar-border max-md:shadow-2xl"
    } else {
        "max-md:hidden"
    };

    let class = format!(
        "group group/sidebar peer relative flex h-full flex-col bg-sidebar \
         text-sidebar-foreground shrink-0 overflow-hidden \
         {} {} {}",
        width_class, mobile_class, props.class
    );

    rsx! {
        aside {
            "data-slot": "sidebar",
            "data-state": state_str,
            "data-collapsible": if !is_open { collapsible_str } else { "" },
            class,
            {props.children}
        }
    }
}

// ============================================================
// Sidebar Header
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarHeaderProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarHeader(props: SidebarHeaderProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    let layout_class = if is_open {
        "w-full flex h-14 items-center justify-between px-3"
    } else {
        "w-full flex h-14 items-center justify-center p-2 [&_.sidebar-text]:hidden"
    };

    let class = format!(
        "{} shrink-0 border-b border-sidebar-border/40 \
         group-data-[collapsible=icon]:justify-center group-data-[collapsible=icon]:p-2 \
         group-data-[collapsible=icon]:[&_.sidebar-text]:hidden {}",
        layout_class, props.class
    );

    rsx! {
        div { "data-slot": "sidebar-header", class, {props.children} }
    }
}

// ============================================================
// Sidebar Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarContentProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarContent(props: SidebarContentProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    let pad_class = if is_open { "p-2" } else { "px-1.5 py-2" };

    let class = format!(
        "flex flex-1 flex-col gap-2 overflow-y-auto overflow-x-hidden {} \
         group-data-[collapsible=icon]:overflow-hidden group-data-[collapsible=icon]:px-1.5 {}",
        pad_class, props.class
    );

    rsx! {
        div { "data-slot": "sidebar-content", class, {props.children} }
    }
}

// ============================================================
// Sidebar Group
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarGroupProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarGroup(props: SidebarGroupProps) -> Element {
    let class = format!(
        "relative flex flex-col w-full min-w-0 p-0 py-1 {}",
        props.class
    );

    rsx! {
        div { "data-slot": "sidebar-group", class, {props.children} }
    }
}

// ============================================================
// Sidebar Group Label
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarGroupLabelProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarGroupLabel(props: SidebarGroupLabelProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    if !ctx.is_open() {
        return rsx! {};
    }

    let class = format!(
        "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-semibold uppercase tracking-wider \
         text-sidebar-foreground/70 outline-none select-none {}",
        props.class
    );

    rsx! {
        div { "data-slot": "sidebar-group-label", class, {props.children} }
    }
}

// ============================================================
// Sidebar Group Content
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarGroupContentProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarGroupContent(props: SidebarGroupContentProps) -> Element {
    let class = format!("w-full text-sm {}", props.class);

    rsx! {
        div { "data-slot": "sidebar-group-content", class, {props.children} }
    }
}

// ============================================================
// Sidebar Menu
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarMenu(props: SidebarMenuProps) -> Element {
    let class = format!(
        "flex w-full min-w-0 flex-col gap-1 list-none p-0 m-0 {}",
        props.class
    );

    rsx! {
        ul { "data-slot": "sidebar-menu", class, {props.children} }
    }
}

// ============================================================
// Sidebar Menu Item
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuItemProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarMenuItem(props: SidebarMenuItemProps) -> Element {
    let class = format!(
        "group/menu-item relative list-none flex flex-col {}",
        props.class
    );

    rsx! {
        li { "data-slot": "sidebar-menu-item", class, {props.children} }
    }
}

// ============================================================
// Sidebar Menu Button
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuButtonProps {
    #[props(default = false)]
    pub active: bool,

    #[props(default = false)]
    pub disabled: bool,

    #[props(default)]
    pub tooltip: Option<String>,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn SidebarMenuButton(props: SidebarMenuButtonProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    let active_class = if props.active {
        "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
    } else {
        "text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
    };

    let button_class = if is_open {
        format!(
            "peer/menu-button flex w-full items-center gap-2 overflow-hidden rounded-md p-2 \
             text-left text-sm outline-none transition-colors \
             focus-visible:ring-2 focus-visible:ring-sidebar-ring \
             disabled:pointer-events-none disabled:opacity-50 cursor-pointer select-none \
             [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 {} {}",
            active_class, props.class
        )
    } else {
        format!(
            "peer/menu-button flex size-8 items-center justify-center mx-auto rounded-md p-0 \
             outline-none transition-colors \
             focus-visible:ring-2 focus-visible:ring-sidebar-ring \
             disabled:pointer-events-none disabled:opacity-50 cursor-pointer select-none \
             [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0 \
             [&>*:not(svg):not(.sidebar-icon)]:hidden {} {}",
            active_class, props.class
        )
    };

    let click_handler = props.onclick;

    let button_element = rsx! {
        button {
            "data-slot": "sidebar-menu-button",
            "data-active": if props.active { "true" } else { "false" },
            r#type: "button",
            class: button_class,
            disabled: props.disabled,
            onclick: move |e| {
                if let Some(handler) = &click_handler {
                    handler.call(e);
                }
            },
            {props.children}
        }
    };

    // When collapsed and tooltip is provided, wrap with Tooltip
    if !is_open {
        if let Some(tooltip_text) = props.tooltip {
            return rsx! {
                Tooltip {
                    TooltipTrigger { {button_element} }
                    TooltipContent { side: "right".to_string(), "{tooltip_text}" }
                }
            };
        }
    }

    button_element
}

// ============================================================
// Sidebar Menu Badge
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuBadgeProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarMenuBadge(props: SidebarMenuBadgeProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    if !ctx.is_open() {
        return rsx! {};
    }

    let class = format!(
        "ml-auto flex h-5 min-w-5 items-center justify-center rounded-md px-1.5 text-[10px] \
         font-medium tabular-nums bg-sidebar-accent text-sidebar-accent-foreground select-none pointer-events-none {}",
        props.class
    );

    rsx! {
        div { "data-slot": "sidebar-menu-badge", class, {props.children} }
    }
}

// ============================================================
// Sidebar Menu Action
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuActionProps {
    #[props(default)]
    pub class: String,

    #[props(default)]
    pub show_on_hover: bool,

    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,

    pub children: Element,
}

#[component]
pub fn SidebarMenuAction(props: SidebarMenuActionProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    if !ctx.is_open() {
        return rsx! {};
    }

    let hover_class = if props.show_on_hover {
        "opacity-0 group-hover/menu-item:opacity-100 transition-opacity"
    } else {
        ""
    };

    let class = format!(
        "absolute right-1 top-1.5 flex size-5 items-center justify-center rounded-md \
         text-sidebar-foreground/70 hover:bg-sidebar-accent hover:text-sidebar-accent-foreground \
         outline-none cursor-pointer {} {}",
        hover_class, props.class
    );

    rsx! {
        button {
            "data-slot": "sidebar-menu-action",
            r#type: "button",
            class,
            onclick: move |e| {
                if let Some(h) = &props.onclick {
                    h.call(e);
                }
            },
            {props.children}
        }
    }
}

// ============================================================
// Sidebar Separator
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarSeparatorProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn SidebarSeparator(props: SidebarSeparatorProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    let class = format!(
        "mx-2 my-1 h-px bg-sidebar-border/60 {} {}",
        if is_open { "mx-2" } else { "mx-1" },
        props.class
    );

    rsx! {
        div { "data-slot": "sidebar-separator", class }
    }
}

// ============================================================
// Sidebar Footer
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarFooterProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn SidebarFooter(props: SidebarFooterProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    let layout_class = if is_open {
        "flex flex-col gap-2 p-2"
    } else {
        "flex flex-col items-center justify-center p-2 [&_.sidebar-text]:hidden"
    };

    let class = format!(
        "{} border-t border-sidebar-border/40 shrink-0 \
         group-data-[collapsible=icon]:items-center group-data-[collapsible=icon]:p-2 \
         group-data-[collapsible=icon]:[&_.sidebar-text]:hidden {}",
        layout_class, props.class
    );

    rsx! {
        div { "data-slot": "sidebar-footer", class, {props.children} }
    }
}

// ============================================================
// Sidebar Trigger
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarTriggerProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn SidebarTrigger(props: SidebarTriggerProps) -> Element {
    let mut ctx = use_context::<SidebarContext>();

    let class = format!(
        "inline-flex size-7 items-center justify-center rounded-md text-sidebar-foreground \
         hover:bg-sidebar-accent hover:text-sidebar-accent-foreground transition-colors \
         focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-sidebar-ring cursor-pointer {}",
        props.class
    );

    rsx! {
        button {
            "data-slot": "sidebar-trigger",
            r#type: "button",
            class,
            onclick: move |_| {
                ctx.toggle();
            },
            svg {
                class: "size-4",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                rect {
                    width: "18",
                    height: "18",
                    x: "3",
                    y: "3",
                    rx: "2",
                }
                path { d: "M9 3v18" }
            }
            span { class: "sr-only", "Toggle Sidebar" }
        }
    }
}

// ============================================================
// Sidebar Rail
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarRailProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn SidebarRail(props: SidebarRailProps) -> Element {
    let mut ctx = use_context::<SidebarContext>();

    rsx! {
        button {
            "data-slot": "sidebar-rail",
            "aria-label": "Toggle Sidebar",
            tabindex: -1,
            r#type: "button",
            class: format!(
                "absolute inset-y-0 right-0 z-10 hidden w-1 -mr-0.5 cursor-ew-resize \
                         hover:bg-sidebar-border transition-colors sm:flex {}",
                props.class,
            ),
            onclick: move |_| {
                ctx.toggle();
            },
        }
    }
}

// ============================================================
// Sidebar Menu Sub (for nested collapsible trees)
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuSubProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SidebarMenuSub(props: SidebarMenuSubProps) -> Element {
    let ctx = use_context::<SidebarContext>();
    let is_open = ctx.is_open();

    if !is_open {
        return rsx! {};
    }

    rsx! {
        ul {
            "data-slot": "sidebar-menu-sub",
            class: format!(
                "mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-sidebar-border/60 px-2.5 py-0.5 group-data-[collapsible=icon]:hidden {}",
                props.class,
            ),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuSubItemProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn SidebarMenuSubItem(props: SidebarMenuSubItemProps) -> Element {
    rsx! {
        li {
            "data-slot": "sidebar-menu-sub-item",
            class: format!("relative {}", props.class),
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct SidebarMenuSubButtonProps {
    #[props(default = false)]
    pub active: bool,
    #[props(default)]
    pub class: String,
    #[props(default)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn SidebarMenuSubButton(props: SidebarMenuSubButtonProps) -> Element {
    let active_class = if props.active {
        "bg-sidebar-accent text-sidebar-accent-foreground font-medium"
    } else {
        "text-sidebar-foreground/70 hover:bg-sidebar-accent/50 hover:text-sidebar-accent-foreground"
    };

    let class = format!(
        "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-xs outline-none cursor-pointer transition-colors focus-visible:ring-2 focus-visible:ring-sidebar-ring disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 {} {}",
        active_class, props.class
    );

    rsx! {
        button {
            "data-slot": "sidebar-menu-sub-button",
            "data-active": if props.active { "true" } else { "false" },
            r#type: "button",
            class,
            onclick: move |e| {
                if let Some(h) = &props.onclick {
                    h.call(e);
                }
            },
            {props.children}
        }
    }
}
