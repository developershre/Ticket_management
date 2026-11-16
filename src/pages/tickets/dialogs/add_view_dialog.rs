use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::icons::{CheckIcon, PanelRightIcon};

#[derive(Props, Clone, PartialEq)]
pub struct AddCustomViewDialogProps {
    pub open: Signal<bool>,
    pub title: Signal<String>,
    pub subtitle: Signal<String>,
    pub color: Signal<String>,
    pub on_create: EventHandler<()>,
}

#[component]
pub fn AddCustomViewDialog(mut props: AddCustomViewDialogProps) -> Element {
    let mut title = props.title;
    let mut subtitle = props.subtitle;
    let mut color = props.color;

    rsx! {
        Dialog { open: props.open,
            DialogContent {
                class: "max-w-md w-full p-5 flex flex-col gap-4 border-border/80 bg-background/95 backdrop-blur-md shadow-2xl rounded-lg",
                DialogHeader {
                    DialogTitle {
                        class: "text-base font-bold flex items-center gap-2",
                        PanelRightIcon { class: "size-4 text-primary" }
                        "Add Custom Kanban View"
                    }
                    DialogDescription {
                        class: "text-xs text-muted-foreground",
                        "Create a new custom Kanban stage or column to categorize tickets according to your support workflow."
                    }
                }

                div { class: "flex flex-col gap-3.5 py-1",
                    div { class: "flex flex-col gap-1",
                        label { class: "text-xs font-semibold text-foreground", "View Title *" }
                        Input {
                            class: "text-xs h-9 w-full",
                            placeholder: "e.g. Under Review, QA Testing, Waiting Vendor".to_string(),
                            value: title.read().clone(),
                            oninput: move |e: FormEvent| title.set(e.value()),
                        }
                    }

                    div { class: "flex flex-col gap-1",
                        label { class: "text-xs font-semibold text-foreground", "Subtitle / Description" }
                        Input {
                            class: "text-xs h-9 w-full",
                            placeholder: "e.g. Tickets under vendor investigation".to_string(),
                            value: subtitle.read().clone(),
                            oninput: move |e: FormEvent| subtitle.set(e.value()),
                        }
                    }

                    div { class: "flex flex-col gap-1.5",
                        label { class: "text-xs font-semibold text-foreground", "Accent Color" }
                        div { class: "flex items-center gap-2 flex-wrap",
                            for (col_cls, col_name) in [
                                ("bg-sky-500", "Sky Blue"),
                                ("bg-indigo-500", "Indigo"),
                                ("bg-purple-500", "Purple"),
                                ("bg-pink-500", "Pink"),
                                ("bg-rose-500", "Rose"),
                                ("bg-amber-500", "Amber"),
                                ("bg-emerald-500", "Emerald"),
                                ("bg-teal-500", "Teal"),
                            ] {
                                {
                                    let is_sel = *color.read() == col_cls;
                                    let cls_c = col_cls.to_string();
                                    let ring_cls = if is_sel { "ring-2 ring-primary ring-offset-2 scale-110" } else { "opacity-80 hover:opacity-100" };
                                    rsx! {
                                        button {
                                            r#type: "button",
                                            key: "color-{col_cls}",
                                            title: "{col_name}",
                                            class: "size-6 rounded-full {col_cls} cursor-pointer transition-transform hover:scale-110 flex items-center justify-center {ring_cls}",
                                            onclick: move |_| color.set(cls_c.clone()),
                                            if is_sel {
                                                CheckIcon { class: "size-3.5 text-white stroke-[3]" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                DialogFooter { class: "flex items-center justify-end gap-2 pt-3 border-t border-border/50",
                    Button {
                        variant: "outline",
                        size: "sm",
                        onclick: move |_| {
                            props.open.set(false);
                            title.set(String::new());
                            subtitle.set(String::new());
                        },
                        "Cancel"
                    }
                    Button {
                        variant: "default",
                        size: "sm",
                        onclick: move |_| props.on_create.call(()),
                        "Create View"
                    }
                }
            }
        }
    }
}
