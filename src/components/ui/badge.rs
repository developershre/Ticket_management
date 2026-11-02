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
            "border-border/60 bg-muted/80 text-muted-foreground hover:bg-muted"
        }
        "destructive" | "red" => {
            "border-rose-500/35 bg-rose-500/15 text-rose-500 dark:text-rose-400 hover:bg-rose-500/25"
        }
        "warning" | "amber" => {
            "border-amber-500/35 bg-amber-500/15 text-amber-600 dark:text-amber-400 hover:bg-amber-500/25"
        }
        "success" | "emerald" | "green" => {
            "border-emerald-500/35 bg-emerald-500/15 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-500/25"
        }
        "info" | "blue" | "sky" => {
            "border-sky-500/35 bg-sky-500/15 text-sky-600 dark:text-sky-400 hover:bg-sky-500/25"
        }
        "purple" | "violet" => {
            "border-purple-500/35 bg-purple-500/15 text-purple-600 dark:text-purple-400 hover:bg-purple-500/25"
        }
        "cyan" | "teal" => {
            "border-cyan-500/35 bg-cyan-500/15 text-cyan-600 dark:text-cyan-400 hover:bg-cyan-500/25"
        }
        "orange" => {
            "border-orange-500/35 bg-orange-500/15 text-orange-600 dark:text-orange-400 hover:bg-orange-500/25"
        }
        "outline" => "border-border text-foreground hover:bg-muted/40",
        _ => "border-border/70 bg-muted/60 text-foreground hover:bg-muted",
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
