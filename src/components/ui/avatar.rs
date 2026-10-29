use dioxus::prelude::*;

// ============================================================
// Avatar Root
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct AvatarProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn Avatar(props: AvatarProps) -> Element {
    let class = format!(
        "relative flex shrink-0 overflow-hidden rounded-full {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "avatar",
            class: class,
            {props.children}
        }
    }
}

// ============================================================
// Avatar Image
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct AvatarImageProps {
    pub src: String,

    #[props(default)]
    pub alt: String,

    #[props(default)]
    pub class: String,
}

#[component]
pub fn AvatarImage(props: AvatarImageProps) -> Element {
    let class = format!("aspect-square size-full object-cover {}", props.class);

    rsx! {
        img {
            "data-slot": "avatar-image",
            src: props.src,
            alt: props.alt,
            class: class,
        }
    }
}

// ============================================================
// Avatar Fallback
// ============================================================

#[derive(Props, Clone, PartialEq)]
pub struct AvatarFallbackProps {
    #[props(default)]
    pub class: String,

    pub children: Element,
}

#[component]
pub fn AvatarFallback(props: AvatarFallbackProps) -> Element {
    let class = format!(
        "flex size-full items-center justify-center rounded-full bg-muted \
         font-medium text-xs text-muted-foreground select-none uppercase {}",
        props.class
    );

    rsx! {
        span {
            "data-slot": "avatar-fallback",
            class: class,
            {props.children}
        }
    }
}
