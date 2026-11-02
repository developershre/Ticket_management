#![allow(dead_code)]
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

#[component]
pub fn SunIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2" }
            path { d: "M12 20v2" }
            path { d: "m4.93 4.93 1.41 1.41" }
            path { d: "m17.66 17.66 1.41 1.41" }
            path { d: "M2 12h2" }
            path { d: "M20 12h2" }
            path { d: "m6.34 17.66-1.41 1.41" }
            path { d: "m19.07 4.93-1.41 1.41" }
        }
    }
}

#[component]
pub fn MoonIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z" }
        }
    }
}

#[component]
pub fn MonitorIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { width: "20", height: "14", x: "2", y: "3", rx: "2" }
            line { x1: "8", x2: "16", y1: "21", y2: "21" }
            line { x1: "12", x2: "12", y1: "17", y2: "21" }
        }
    }
}

#[component]
pub fn GripVerticalIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "9", cy: "12", r: "1", fill: "currentColor" }
            circle { cx: "9", cy: "5", r: "1", fill: "currentColor" }
            circle { cx: "9", cy: "19", r: "1", fill: "currentColor" }
            circle { cx: "15", cy: "12", r: "1", fill: "currentColor" }
            circle { cx: "15", cy: "5", r: "1", fill: "currentColor" }
            circle { cx: "15", cy: "19", r: "1", fill: "currentColor" }
        }
    }
}

#[component]
pub fn PlusIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M5 12h14" }
            path { d: "M12 5v14" }
        }
    }
}

#[component]
pub fn ClockIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            polyline { points: "12 6 12 12 16 14" }
        }
    }
}

#[component]
pub fn MapPinIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0" }
            circle { cx: "12", cy: "10", r: "3" }
        }
    }
}

#[component]
pub fn VideoIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m16 13 5.223 3.482a.5.5 0 0 0 .777-.416V7.934a.5.5 0 0 0-.777-.416L16 11" }
            rect { x: "2", y: "6", width: "14", height: "12", rx: "2" }
        }
    }
}

#[component]
pub fn UsersIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" }
            circle { cx: "9", cy: "7", r: "4" }
            path { d: "M22 21v-2a4 4 0 0 0-3-3.87" }
            path { d: "M16 3.13a4 4 0 0 1 0 7.75" }
        }
    }
}

#[component]
pub fn XIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M18 6 6 18" }
            path { d: "m6 6 12 12" }
        }
    }
}

#[component]
pub fn FilterIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polygon { points: "22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3" }
        }
    }
}

#[component]
pub fn TrashIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M3 6h18" }
            path { d: "M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" }
            path { d: "M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" }
            line { x1: "10", x2: "10", y1: "11", y2: "17" }
            line { x1: "14", x2: "14", y1: "11", y2: "17" }
        }
    }
}

#[component]
pub fn ExternalLinkIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M15 3h6v6" }
            path { d: "M10 14 21 3" }
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
        }
    }
}

#[component]
pub fn GridIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { width: "7", height: "7", x: "3", y: "3", rx: "1" }
            rect { width: "7", height: "7", x: "14", y: "3", rx: "1" }
            rect { width: "7", height: "7", x: "14", y: "14", rx: "1" }
            rect { width: "7", height: "7", x: "3", y: "14", rx: "1" }
        }
    }
}

#[component]
pub fn ListIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            line { x1: "8", x2: "21", y1: "6", y2: "6" }
            line { x1: "8", x2: "21", y1: "12", y2: "12" }
            line { x1: "8", x2: "21", y1: "18", y2: "18" }
            line { x1: "3", x2: "3.01", y1: "6", y2: "6" }
            line { x1: "3", x2: "3.01", y1: "12", y2: "12" }
            line { x1: "3", x2: "3.01", y1: "18", y2: "18" }
        }
    }
}

#[component]
pub fn CheckIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

#[component]
pub fn AlertCircleIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
            line { x1: "12", x2: "12", y1: "8", y2: "12" }
            line { x1: "12", x2: "12.01", y1: "16", y2: "16" }
        }
    }
}

#[component]
pub fn CopyIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            rect { width: "14", height: "14", x: "8", y: "8", rx: "2", ry: "2" }
            path { d: "M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" }
        }
    }
}

#[component]
pub fn ArrowUpIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m5 12 7-7 7 7" }
            path { d: "M12 19V5" }
        }
    }
}

#[component]
pub fn ArrowDownIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12 5v14" }
            path { d: "m19 12-7 7-7-7" }
        }
    }
}

#[component]
pub fn EyeIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

#[component]
pub fn PencilIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" }
            path { d: "m15 5 4 4" }
        }
    }
}

#[component]
pub fn MoreHorizontalIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "1", fill: "currentColor" }
            circle { cx: "19", cy: "12", r: "1", fill: "currentColor" }
            circle { cx: "5", cy: "12", r: "1", fill: "currentColor" }
        }
    }
}

#[component]
pub fn ArrowUpDownIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m21 16-4 4-4-4" }
            path { d: "M17 20V4" }
            path { d: "m3 8 4-4 4 4" }
            path { d: "M7 4v16" }
        }
    }
}

#[component]
pub fn ChevronsUpDownIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m7 15 5 5 5-5" }
            path { d: "m7 9 5-5 5 5" }
        }
    }
}
