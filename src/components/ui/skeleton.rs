#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct SkeletonProps {
    #[props(default)]
    pub class: String,
}

#[component]
pub fn Skeleton(props: SkeletonProps) -> Element {
    let class = format!("animate-pulse rounded-md bg-muted {}", props.class);

    rsx! {
        div {
            "data-slot": "skeleton",
            class: class,
        }
    }
}
