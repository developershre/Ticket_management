#![allow(dead_code)]

use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToastVariant {
    Default,
    Success,
    Error,
    Info,
}

#[derive(Clone, Debug, PartialEq)]
struct ToastItem {
    id: u32,
    title: String,
    description: Option<String>,
    variant: ToastVariant,
}

#[derive(Clone, Copy, PartialEq)]
pub struct ToastState {
    toasts: Signal<Vec<ToastItem>>,
}

#[allow(dead_code)]
impl ToastState {
    pub fn show(&self, title: impl Into<String>, description: impl Into<String>) {
        self.push(
            ToastVariant::Default,
            title.into(),
            Some(description.into()),
        );
    }

    pub fn success(&self, title: impl Into<String>, description: impl Into<String>) {
        self.push(
            ToastVariant::Success,
            title.into(),
            Some(description.into()),
        );
    }

    pub fn error(&self, title: impl Into<String>, description: impl Into<String>) {
        self.push(ToastVariant::Error, title.into(), Some(description.into()));
    }

    pub fn info(&self, title: impl Into<String>, description: impl Into<String>) {
        self.push(ToastVariant::Info, title.into(), Some(description.into()));
    }

    pub fn warning(&self, title: impl Into<String>, description: impl Into<String>) {
        self.push(ToastVariant::Info, title.into(), Some(description.into()));
    }

    fn push(&self, variant: ToastVariant, title: String, description: Option<String>) {
        let mut toasts = self.toasts;
        let id = {
            let list = toasts.read();
            list.iter().map(|t| t.id).max().unwrap_or(0) + 1
        };
        let mut list = toasts.write();
        if list.len() >= 4 {
            list.remove(0);
        }
        list.push(ToastItem {
            id,
            title,
            description,
            variant,
        });
    }

    pub fn dismiss(&self, id: u32) {
        let mut toasts = self.toasts;
        let mut list = toasts.write();
        list.retain(|t| t.id != id);
    }
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    let toasts = use_signal(Vec::<ToastItem>::new);
    let toast = use_context_provider(|| ToastState { toasts });

    rsx! {
        {children}
        div { class: "pointer-events-none fixed top-14 right-6 z-[70] flex w-84 max-w-[calc(100vw-3rem)] flex-col items-end gap-2.5",
            for item in toasts.read().iter() {
                {
                    let id = item.id;
                    let (border_class, icon_class) = match item.variant {
                        ToastVariant::Default => ("border-border/60", "bg-muted text-foreground"),
                        ToastVariant::Success => (
                            "border-success/40",
                            "bg-success/10 text-success",
                        ),
                        ToastVariant::Error => (
                            "border-destructive/40",
                            "bg-destructive/10 text-destructive",
                        ),
                        ToastVariant::Info => ("border-info/40", "bg-info/10 text-info"),
                    };
                    let title = item.title.clone();
                    let description = item.description.clone();
                    rsx! {
                        div {
                            key: "{id}",
                            role: "status",
                            class: "toast-item pointer-events-auto flex w-full items-start gap-3 rounded-lg border {border_class} bg-card p-3.5",
                            onanimationend: move |_| toast.dismiss(id),
                            div { class: "flex size-7 shrink-0 items-center justify-center rounded-md {icon_class}",
                                match item.variant {
                                    ToastVariant::Success => rsx! {
                                        svg {
                                            class: "size-4",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            path { d: "M22 11.08V12a10 10 0 1 1-5.93-9.14" }
                                            path { d: "m9 11 3 3L22 4" }
                                        }
                                    },
                                    ToastVariant::Error => rsx! {
                                        svg {
                                            class: "size-4",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            path { d: "m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3Z" }
                                            line { x1: "12", y1: "9", x2: "12", y2: "13" }
                                            line { x1: "12", y1: "17", x2: "12.01", y2: "17" }
                                        }
                                    },
                                    _ => rsx! {
                                        svg {
                                            class: "size-4",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "2",
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            circle { cx: "12", cy: "12", r: "10" }
                                            path { d: "M12 16v-4" }
                                            path { d: "M12 8h.01" }
                                        }
                                    },
                                }
                            }
                            div { class: "flex min-w-0 flex-1 flex-col gap-0.5",
                                span { class: "text-sm font-semibold leading-tight text-foreground",
                                    "{title}"
                                }
                                if let Some(desc) = description.as_ref() {
                                    span { class: "text-xs leading-snug text-muted-foreground", "{desc}" }
                                }
                            }
                            button {
                                r#type: "button",
                                class: "shrink-0 cursor-pointer rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted",
                                onclick: move |_| toast.dismiss(id),
                                svg {
                                    class: "size-3.5",
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
                    }
                }
            }
        }
    }
}
