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
pub fn BellIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
            path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
        }
    }
}

#[component]
pub fn TicketIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" }
            path { d: "M13 5v2" }
            path { d: "M13 11v2" }
            path { d: "M13 17v2" }
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

#[component]
pub fn StickyNoteIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M16 3H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V8Z" }
            path { d: "M15 3v5a1 1 0 0 0 1 1h5" }
        }
    }
}

#[component]
pub fn FileTextIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
            path { d: "M14 2v4a2 2 0 0 0 2 2h4" }
            path { d: "M10 9H8" }
            path { d: "M16 13H8" }
            path { d: "M16 17H8" }
        }
    }
}

#[component]
pub fn PaperclipIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m21.44 11.05-9.19 9.19a6 6 0 0 1-8.49-8.49l8.57-8.57A4 4 0 1 1 18 8.84l-8.59 8.57a2 2 0 0 1-2.83-2.83l8.49-8.48" }
        }
    }
}

#[component]
pub fn UploadCloudIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242" }
            path { d: "m12 12-4 4h3v4h2v-4h3z" }
        }
    }
}

#[component]
pub fn AppLogo(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 292 319",
            fill: "none",
            xmlns: "http://www.w3.org/2000/svg",
            path {
                d: "M197.916 0C198.037 4.70667e-06 198.161 0.00923442 198.279 0.0263672C221.137 3.35361 237.799 12.053 249.324 23.7217C260.846 35.3883 267.203 49.9943 269.498 65.0918C274.083 95.2514 262.47 127.486 243.276 142.321C242.328 143.054 242.47 144.604 243.538 145.154C322.222 185.703 300.653 310.076 198.101 318.498L198.081 318.5H91.5727L91.56 318.014C90.3111 270.804 90.2957 243.414 101.048 221.405C111.805 199.388 133.301 182.823 174.767 157.093C207.145 131.182 215.917 103.846 213.813 83.0508C212.759 72.6395 208.98 63.8562 204.06 57.6865C199.13 51.5047 193.103 48 187.56 48H82.56C81.7316 48 81.06 48.6716 81.06 49.5V143.685C81.0598 144.684 80.4679 145.574 79.5737 145.989C70.0465 150.416 61.0448 158.442 53.2866 167.335C45.5332 176.222 39.0489 185.943 34.5512 193.723C33.8877 194.87 32.6222 195.187 31.5698 194.921C30.5156 194.655 29.56 193.774 29.56 192.439V44.1543C29.56 43.8355 29.4583 43.5247 29.27 43.2676L0.487732 3.97754C-0.722056 2.32608 0.457157 0 2.50433 0H197.916ZM188.56 175.5C166.475 175.5 148.107 191.409 144.289 212.393C143.817 213.784 143.56 215.275 143.56 216.826V265.5H192.234C193.785 265.5 195.275 265.242 196.666 264.77C217.65 260.952 233.56 242.585 233.56 220.5C233.56 195.647 213.413 175.5 188.56 175.5Z",
                fill: "#02254D",
            }
            path {
                d: "M22.996 315.051C22.2427 315.79 22.7408 317.5 23.7961 317.5H67.06C70.06 317.5 78.06 311.5 78.06 298C76.56 245 78.56 220 111.06 187.5C142.324 156.236 207.476 135.471 191.336 76.2445C191.056 75.2178 189.445 75.269 189.143 76.2893C180.667 104.915 126.671 129.323 88.56 156C68.8933 169.5 31.56 201.5 29.06 249L28.56 301C28.1529 304.664 27.2156 310.912 22.996 315.051Z",
                fill: "url(#bpsl_logo_gradient)",
            }
            defs {
                linearGradient {
                    id: "bpsl_logo_gradient",
                    x1: "106.713",
                    y1: "72",
                    x2: "106.713",
                    y2: "317.5",
                    gradient_units: "userSpaceOnUse",
                    stop { stop_color: "#EC8741" }
                    stop { offset: "0.520739", stop_color: "#DF6222" }
                    stop { offset: "1", stop_color: "#952F09" }
                }
            }
        }
    }
}

#[component]
pub fn SettingsIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

#[component]
pub fn KeyIcon(props: IconProps) -> Element {
    rsx! {
        svg {
            class: props.class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "7.5", cy: "15.5", r: "5.5" }
            path { d: "m21 2-9.6 9.6" }
            path { d: "m15.5 7.5 3 3L22 7l-3-3" }
        }
    }
}
