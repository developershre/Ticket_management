//! Inline SVG icons (lucide-style) shared by the UI kit.
//!
//! Every icon accepts an optional `class` prop so callers can size/color
//! them with Tailwind utilities, e.g. `SearchIcon { class: "size-4" }`.

use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct IconProps {
    #[props(default = "size-4".to_string())]
    pub class: String,
}

#[component]
pub fn ChevronLeftIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m15 18-6-6 6-6" }
        }
    }
}

#[component]
pub fn ChevronRightIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m9 18 6-6-6-6" }
        }
    }
}

#[component]
pub fn SearchIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "11", cy: "11", r: "8" }
            path { d: "m21 21-4.3-4.3" }
        }
    }
}

#[component]
pub fn CalendarIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { width: "18", height: "18", x: "3", y: "4", rx: "2", ry: "2" }
            path { d: "M16 2v4" }
            path { d: "M8 2v4" }
            path { d: "M3 10h18" }
        }
    }
}
