#![allow(dead_code)]
use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TableProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn Table(props: TableProps) -> Element {
    let class = format!("w-full caption-bottom text-xs {}", props.class);

    rsx! {
        div {
            class: "relative w-full overflow-auto rounded-md border border-border/60",
            table {
                "data-slot": "table",
                class: class,
                {props.children}
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableHeaderProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn TableHeader(props: TableHeaderProps) -> Element {
    let class = format!("[&_tr]:border-b bg-muted/30 {}", props.class);

    rsx! {
        thead {
            "data-slot": "table-header",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableBodyProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn TableBody(props: TableBodyProps) -> Element {
    let class = format!("[&_tr:last-child]:border-0 {}", props.class);

    rsx! {
        tbody {
            "data-slot": "table-body",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableFooterProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn TableFooter(props: TableFooterProps) -> Element {
    let class = format!("border-t bg-muted/50 font-medium [&>tr]:last:border-b-0 {}", props.class);

    rsx! {
        tfoot {
            "data-slot": "table-footer",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableRowProps {
    #[props(default)]
    pub class: String,
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}

#[component]
pub fn TableRow(props: TableRowProps) -> Element {
    let class = format!(
        "border-b border-border/60 transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted {}",
        props.class
    );

    rsx! {
        tr {
            "data-slot": "table-row",
            class: class,
            onclick: move |e| {
                if let Some(handler) = &props.onclick {
                    handler.call(e);
                }
            },
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableHeadProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn TableHead(props: TableHeadProps) -> Element {
    let class = format!(
        "h-9 px-3 text-left align-middle font-semibold text-muted-foreground [&:has([role=checkbox])]:pr-0 {}",
        props.class
    );

    rsx! {
        th {
            "data-slot": "table-head",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableCellProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn TableCell(props: TableCellProps) -> Element {
    let class = format!("p-3 align-middle [&:has([role=checkbox])]:pr-0 {}", props.class);

    rsx! {
        td {
            "data-slot": "table-cell",
            class: class,
            {props.children}
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct TableCaptionProps {
    #[props(default)]
    pub class: String,
    pub children: Element,
}

#[component]
pub fn TableCaption(props: TableCaptionProps) -> Element {
    let class = format!("mt-4 text-xs text-muted-foreground {}", props.class);

    rsx! {
        caption {
            "data-slot": "table-caption",
            class: class,
            {props.children}
        }
    }
}
