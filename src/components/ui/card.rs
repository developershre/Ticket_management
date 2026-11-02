#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CardProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Card(props: CardProps) -> Element {
    let class = format!(
        "rounded-xl border border-border bg-card text-card-foreground shadow-xs {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "card",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardHeaderProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn CardHeader(props: CardHeaderProps) -> Element {
    let class = format!("flex flex-col gap-1.5 p-6 {}", props.class);

    rsx! {
        div {
            "data-slot": "card-header",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardTitleProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn CardTitle(props: CardTitleProps) -> Element {
    let class = format!(
        "font-semibold leading-none tracking-tight text-foreground {}",
        props.class
    );

    rsx! {
        h3 {
            "data-slot": "card-title",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardDescriptionProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn CardDescription(props: CardDescriptionProps) -> Element {
    let class = format!("text-sm text-muted-foreground {}", props.class);

    rsx! {
        p {
            "data-slot": "card-description",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardContentProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn CardContent(props: CardContentProps) -> Element {
    let class = format!("p-6 pt-0 {}", props.class);

    rsx! {
        div {
            "data-slot": "card-content",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct CardFooterProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn CardFooter(props: CardFooterProps) -> Element {
    let class = format!("flex items-center p-6 pt-0 {}", props.class);

    rsx! {
        div {
            "data-slot": "card-footer",
            class: class,
            {props.children}
        }
    }
}
