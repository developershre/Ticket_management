//! Floating action button with a speed-dial menu.
//!
//! The main FAB expands into quick actions (note / ticket / reminder). Picking
//! one opens a quick-add dialog; saving fires a toast via the `ToastProvider`
//! higher in the tree.

use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;

// ============================================================
// Quick-add kinds
// ============================================================

#[derive(Clone, Copy, PartialEq, Eq)]
enum QuickKind {
    Note,
    Ticket,
    Reminder,
}

impl QuickKind {
    fn label(&self) -> &'static str {
        match self {
            QuickKind::Note => "Note",
            QuickKind::Ticket => "Ticket",
            QuickKind::Reminder => "Reminder",
        }
    }

    fn title(&self) -> &'static str {
        match self {
            QuickKind::Note => "New note",
            QuickKind::Ticket => "New ticket",
            QuickKind::Reminder => "New reminder",
        }
    }

    fn placeholder(&self) -> &'static str {
        match self {
            QuickKind::Note => "Note title, e.g. \"Ideas for the v2 launch\"",
            QuickKind::Ticket => "IT ticket subject, e.g. \"VPN gateway disconnects every 30m\"",
            QuickKind::Reminder => "Reminder title, e.g. \"Call the dentist\"",
        }
    }

    fn icon(&self) -> Element {
        let paths = match self {
            // File text
            QuickKind::Note => rsx! {
                path { d: "M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" }
                path { d: "M14 2v4a2 2 0 0 0 2 2h4" }
                path { d: "M10 9H8" }
                path { d: "M16 13H8" }
                path { d: "M16 17H8" }
            },
            // Ticket
            QuickKind::Ticket => rsx! {
                path { d: "M2 9a3 3 0 0 1 0 6v2a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-2a3 3 0 0 1 0-6V7a2 2 0 0 0-2-2H4a2 2 0 0 0-2 2Z" }
                path { d: "M13 5v2" }
                path { d: "M13 11v2" }
                path { d: "M13 17v2" }
            },
            // Bell
            QuickKind::Reminder => rsx! {
                path { d: "M10.268 21a2 2 0 0 0 3.464 0" }
                path { d: "M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" }
            },
        };

        rsx! {
            svg {
                class: "size-4.5",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                {paths}
            }
        }
    }
}

const ALL_KINDS: [QuickKind; 3] = [QuickKind::Note, QuickKind::Ticket, QuickKind::Reminder];

// ============================================================
// FAB
// ============================================================

#[component]
pub fn Fab() -> Element {
    // ToastProvider (AppShell) must be an ancestor.
    let toast = use_context::<ToastState>();

    let mut menu_open = use_signal(|| false);
    let mut dialog_open = use_signal(|| false);
    let mut kind = use_signal(|| QuickKind::Note);
    let mut title_text = use_signal(String::new);
    let mut details = use_signal(String::new);
    // Bumped every time the dialog opens so inputs remount empty.
    let mut generation = use_signal(|| 0u32);

    let mut open_kind = move |k: QuickKind| {
        kind.set(k);
        title_text.write().clear();
        details.write().clear();
        *generation.write() += 1;
        menu_open.set(false);
        dialog_open.set(true);
    };

    rsx! {
        // ------------------------------------------------------------
        // Click-away layer that closes the speed dial
        // ------------------------------------------------------------
        if menu_open() {
            div {
                class: "fixed inset-0 z-30 bg-background/20 backdrop-blur-[2px] transition-all",
                onclick: move |_| menu_open.set(false),
            }
        }

        // ------------------------------------------------------------
        // Speed-dial actions
        // ------------------------------------------------------------
        if menu_open() {
            div {
                class: "fixed bottom-22 right-6 z-40 flex flex-col items-end gap-3 animate-in fade-in slide-in-from-bottom-3 duration-200",
                for (action_idx, kind_item) in ALL_KINDS.iter().enumerate() {
                    div {
                        key: "quick-action-{action_idx}",
                        class: "flex items-center gap-2.5",
                        span { class: "rounded-xl border border-border/80 bg-popover/95 px-3 py-1 text-xs font-semibold text-popover-foreground backdrop-blur-md",
                            "{kind_item.label()}"
                        }
                        button {
                            r#type: "button",
                            title: "New {kind_item.label()}",
                            class: "flex size-11 items-center justify-center rounded-full border border-border/70 bg-popover text-foreground transition-all duration-150 hover:scale-105 hover:border-primary/50 hover:bg-accent cursor-pointer",
                            onclick: move |_| open_kind(*kind_item),
                            {kind_item.icon()}
                        }
                    }
                }
            }
        }

        // ------------------------------------------------------------
        // Main FAB
        // ------------------------------------------------------------
        button {
            r#type: "button",
            title: "Quick add",
            "aria-label": "Quick add",
            class: "fixed bottom-6 right-6 z-40 flex size-13 items-center justify-center rounded-full bg-primary text-primary-foreground transition-all duration-200 hover:scale-105 active:scale-95 cursor-pointer",
            onclick: move |_| {
                let next = !*menu_open.read();
                menu_open.set(next);
            },
            svg {
                class: if menu_open() { "size-6 rotate-45 transition-transform duration-300" } else { "size-6 transition-transform duration-300" },
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2.5",
                stroke_linecap: "round",
                path { d: "M5 12h14" }
                path { d: "M12 5v14" }
            }
        }

        // ------------------------------------------------------------
        // Quick-add dialog
        // ------------------------------------------------------------
        Dialog {
            open: Some(dialog_open),
            DialogContent {
                DialogHeader {
                    DialogTitle { "{kind().title()}" }
                    DialogDescription { "Capture it now — you can organize it later." }
                }

                div { class: "grid gap-3",
                    div { class: "grid gap-1.5",
                        label { class: "text-xs font-medium text-foreground", "Title" }
                        Input {
                            key: "title-{generation()}",
                            placeholder: kind().placeholder().to_string(),
                            value: title_text.read().clone(),
                            oninput: move |e: FormEvent| title_text.set(e.value()),
                        }
                    }
                    div { class: "grid gap-1.5",
                        label { class: "text-xs font-medium text-foreground", "Details" }
                        textarea {
                            key: "details-{generation()}",
                            rows: "4",
                            class: "flex w-full resize-none rounded-md border border-input bg-transparent px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                            placeholder: "Optional details...".to_string(),
                            oninput: move |e: FormEvent| details.set(e.value()),
                        }
                    }
                }

                DialogFooter {
                    Button {
                        variant: "outline",
                        onclick: move |_| dialog_open.set(false),
                        "Cancel"
                    }
                    Button {
                        variant: "default",
                        onclick: move |_| {
                            let title = title_text.read().trim().to_string();
                            if title.is_empty() {
                                toast.error("Missing title", format!("Give your {} a title first.", kind().label().to_lowercase()));
                                return;
                            }
                            let detail = details.read().trim().to_string();
                            let current_kind = kind();

                            match current_kind {
                                QuickKind::Ticket => {
                                    match crate::db::create_ticket(&title, "General", "Medium", "EMP-1001 (Admin)", "IT", "Pending", &detail) {
                                        Ok(new_id) => {
                                            toast.success("Ticket Saved to DB", format!("IT-{new_id}: \"{title}\" persisted to management.db"));
                                        }
                                        Err(e) => {
                                            toast.error("Database Error", format!("Failed to save ticket: {:?}", e));
                                        }
                                    }
                                }
                                QuickKind::Note | QuickKind::Reminder => {
                                    let content = if detail.is_empty() {
                                        title.clone()
                                    } else {
                                        format!("{title} — {detail}")
                                    };
                                    match crate::db::create_note("EMP-1001", &content) {
                                        Ok(new_id) => {
                                            let label = current_kind.label();
                                            toast.success("Saved to DB", format!("{label} #{new_id} persisted to management.db"));
                                        }
                                        Err(e) => {
                                            toast.error("Database Error", format!("Failed to save: {:?}", e));
                                        }
                                    }
                                }
                            }
                            dialog_open.set(false);
                        },
                        "Create"
                    }
                }
            }
        }
    }
}
