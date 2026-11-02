use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct BadgeProps {
    #[props(default = "default".to_string())]
    pub variant: String,

    #[props(default)]
    pub class: String,

    #[props(default)]
    pub size: String,

    pub children: Element,
}

#[component]
pub fn Badge(props: BadgeProps) -> Element {
    let variant_class = match props.variant.as_str() {
        "secondary" => {
            "border-transparent bg-secondary text-secondary-foreground hover:bg-secondary/80"
        }
        "destructive" => {
            "border-transparent bg-destructive text-destructive-foreground hover:bg-destructive/90"
        }
        "warning" => {
            "border-transparent bg-warning text-warning-foreground hover:bg-warning/90"
        }
        "success" => {
            "border-transparent bg-success text-success-foreground hover:bg-success/90"
        }
        "info" => {
            "border-transparent bg-info text-info-foreground hover:bg-info/90"
        }
        "outline" => "border-border text-foreground",
        _ => "border-transparent bg-primary text-primary-foreground hover:bg-primary/90",
    };

    let size_class = match props.size.as_str() {
        "lg" => "text-sm px-3 py-1",
        "xl" => "text-base px-3.5 py-1.5",
        _ => "text-xs px-2.5 py-0.5",
    };

    let class = format!(
        "inline-flex items-center gap-1 rounded-md border font-medium \
         transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 \
         select-none {variant_class} {size_class} {}",
        props.class
    );

    rsx! {
        div {
            "data-slot": "badge",
            class: class,
            {props.children}
        }
    }
}
