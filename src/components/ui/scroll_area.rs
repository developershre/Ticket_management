#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct ScrollAreaProps {
    #[props(default = "vertical".to_string())]
    pub orientation: String,

    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn ScrollArea(props: ScrollAreaProps) -> Element {
    let overflow_cls = match props.orientation.as_str() {
        "horizontal" => "overflow-x-auto overflow-y-hidden",
        "both" => "overflow-auto",
        _ => "overflow-y-auto overflow-x-hidden",
    };

    let class = format!(
        "relative rounded-md border border-border/40 scrollbar-thin {} {}",
        overflow_cls,
        props.class
    );

    rsx! {
        div {
            "data-slot": "scroll-area",
            class: class,
            div {
                "data-slot": "scroll-area-viewport",
                class: "h-full w-full rounded-[inherit]",
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct ScrollBarProps {
    #[props(default = "vertical".to_string())]
    pub orientation: String,

    #[props(default)]
    pub class: String,
}

#[component]
pub fn ScrollBar(props: ScrollBarProps) -> Element {
    let is_horizontal = props.orientation == "horizontal";
    let base_cls = if is_horizontal {
        "flex h-2.5 flex-col border-t border-t-transparent p-[1px]"
    } else {
        "flex h-full w-2.5 border-l border-l-transparent p-[1px]"
    };

    let class = format!("touch-none select-none transition-colors {} {}", base_cls, props.class);

    rsx! {
        div {
            "data-slot": "scroll-bar",
            class: class,
            div {
                class: "relative flex-1 rounded-full bg-border hover:bg-muted-foreground/60 transition-colors",
            }
        }
    }
}
