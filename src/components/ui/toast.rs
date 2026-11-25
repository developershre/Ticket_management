#![allow(dead_code)]

use dioxus::prelude::*;

use crate::icons::{
    AlertCircleIcon, BellIcon, CheckCircle2Icon, ChevronDownIcon, ChevronUpIcon, XIcon,
};

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
        if list.len() >= 5 {
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

    pub fn dismiss_all(&self) {
        let mut toasts = self.toasts;
        toasts.write().clear();
    }
}

#[component]
pub fn ToastProvider(children: Element) -> Element {
    let toasts = use_signal(Vec::<ToastItem>::new);
    let toast = use_context_provider(|| ToastState { toasts });

    // Interactive expansion state: ONLY expands when user explicitly clicks the stack
    let mut is_pinned = use_signal(|| false);
    let mut is_hovered = use_signal(|| false);

    // Swipe to clear state: tracks (active_toast_id, start_client_x)
    let mut drag_state = use_signal(|| Option::<(u32, f64)>::None);
    let mut drag_offset = use_signal(|| 0.0_f64);

    let toast_list = toasts.read();
    let total = toast_list.len();

    // Auto-reset pin if down to 0 or 1 toast
    if total <= 1 && *is_pinned.peek() {
        is_pinned.set(false);
    }

    // Only show list view if user clicked on the stack toast
    let is_expanded = *is_pinned.read() && total > 1;

    // Toast dimensions for smooth pixel-based transitions
    let card_height: i32 = 72;
    let card_gap: i32 = 10;

    // Compute container wrapper height so layout smoothly expands/collapses
    let wrapper_height = if !is_expanded {
        if total <= 1 {
            card_height
        } else if total == 2 {
            card_height + 14
        } else {
            card_height + 26
        }
    } else {
        (total as i32) * card_height + ((total as i32) - 1).max(0) * card_gap
    };

    // Toasts displayed newest first (idx = 0 is front card in stack / top card in list)
    let reversed_toasts: Vec<ToastItem> = toast_list.iter().cloned().rev().collect();
    drop(toast_list);

    let current_drag = *drag_state.read();
    let cur_offset = *drag_offset.read();

    rsx! {
        {children}
        if total > 0 {
            div {
                class: format!(
                    "pointer-events-none fixed top-14 right-6 z-[80] flex w-88 max-w-[calc(100vw-2.5rem)] flex-col items-end {}",
                    if *is_hovered.read() || is_expanded { "toast-paused" } else { "" }
                ),
                onmouseenter: move |_| is_hovered.set(true),
                onmouseleave: move |_| {
                    is_hovered.set(false);
                    // If mouse leaves while dragging, complete swipe or snap back
                    let active_drag = *drag_state.peek();
                    if let Some((id, _)) = active_drag {
                        let off = *drag_offset.peek();
                        drag_state.set(None);
                        drag_offset.set(0.0);
                        if off.abs() >= 60.0 {
                            toast.dismiss(id);
                        }
                    }
                },
                onmousemove: move |e: MouseEvent| {
                    if let Some((_, start_x)) = *drag_state.peek() {
                        let diff = e.client_coordinates().x - start_x;
                        drag_offset.set(diff);
                    }
                },
                onmouseup: move |_| {
                    let active_drag = *drag_state.peek();
                    if let Some((id, _)) = active_drag {
                        let off = *drag_offset.peek();
                        drag_state.set(None);
                        drag_offset.set(0.0);
                        if off.abs() >= 60.0 {
                            toast.dismiss(id);
                        }
                    }
                },

                // Top Toolbar: shown when there are 2+ toasts to control stack vs list
                if total > 1 {
                    div { class: "pointer-events-auto mb-2 flex w-full items-center justify-between gap-2 px-1 select-none animate-in fade-in-50 duration-200",
                        // Stack badge & toggle button
                        button {
                            r#type: "button",
                            class: "inline-flex items-center gap-1.5 rounded-full border border-border/80 bg-card/95 px-2.5 py-1 text-[11px] font-semibold text-foreground shadow-xs backdrop-blur-md transition-all hover:bg-muted cursor-pointer",
                            onclick: move |_| {
                                let cur = *is_pinned.read();
                                is_pinned.set(!cur);
                            },
                            span { class: "flex size-2 rounded-full bg-primary animate-pulse" }
                            span { "{total} notifications" }
                            if is_expanded {
                                ChevronUpIcon { class: "size-3 text-muted-foreground" }
                            } else {
                                ChevronDownIcon { class: "size-3 text-muted-foreground" }
                            }
                        }

                        // Right actions (Expand / Collapse & Dismiss All)
                        div { class: "flex items-center gap-1.5",
                            button {
                                r#type: "button",
                                class: "inline-flex items-center gap-1 rounded-md border border-border/60 bg-card/90 px-2 py-0.5 text-[11px] font-medium text-muted-foreground shadow-2xs backdrop-blur-md transition-colors hover:bg-muted hover:text-foreground cursor-pointer",
                                onclick: move |_| {
                                    let cur = *is_pinned.read();
                                    is_pinned.set(!cur);
                                },
                                if is_expanded { "Collapse Stack" } else { "Expand List" }
                            }
                            if is_expanded {
                                button {
                                    r#type: "button",
                                    class: "inline-flex items-center gap-1 rounded-md border border-destructive/30 bg-destructive/10 px-2 py-0.5 text-[11px] font-medium text-destructive shadow-2xs backdrop-blur-md transition-colors hover:bg-destructive/20 cursor-pointer",
                                    onclick: move |_| toast.dismiss_all(),
                                    "Clear all"
                                }
                            }
                        }
                    }
                }

                // Interactive Stacking Container
                div {
                    class: "relative w-full",
                    style: format!("height: {wrapper_height}px; transition: height 0.35s cubic-bezier(0.16, 1, 0.3, 1);"),

                    for (idx, item) in reversed_toasts.iter().enumerate() {
                        {
                            let id = item.id;
                            let is_front = idx == 0;
                            let is_being_dragged = current_drag.map(|(d_id, _)| d_id == id).unwrap_or(false);

                            // Calculate geometry:
                            // In stacked mode: Card 0 is top (y=0, scale=1), Card 1 is shifted down (y=12px, scale=0.95), Card 2 is (y=24px, scale=0.90)
                            // In expanded list mode: Each card is shifted down by idx * (height + gap) with scale=1
                            let (y_offset, scale, base_opacity, z_index, pointer_events) = if is_expanded {
                                (
                                    idx as i32 * (card_height + card_gap),
                                    1.0_f32,
                                    1.0_f32,
                                    50 - (idx as i32 * 2),
                                    "pointer-events-auto",
                                )
                            } else {
                                let clamped_idx = idx.min(3) as i32;
                                let y = clamped_idx * 12;
                                let s = 1.0 - (clamped_idx as f32 * 0.05);
                                let op = match clamped_idx {
                                    0 => 1.0_f32,
                                    1 => 0.90_f32,
                                    2 => 0.75_f32,
                                    _ => 0.0_f32,
                                };
                                let pe = if idx <= 2 { "pointer-events-auto" } else { "pointer-events-none" };
                                (y, s, op, 50 - (idx as i32 * 5), pe)
                            };

                            // Apply swipe drag translation and fading
                            let (x_offset, opacity, transition_style) = if is_being_dragged {
                                let drag_fade = (1.0 - (cur_offset.abs() / 180.0)).clamp(0.2, 1.0) as f32;
                                (cur_offset, base_opacity * drag_fade, "transition: none;")
                            } else {
                                (0.0_f64, base_opacity, "transition: transform 0.35s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.3s ease;")
                            };

                            let (border_class, icon_class) = match item.variant {
                                ToastVariant::Default => ("border-border/80", "bg-muted text-foreground"),
                                ToastVariant::Success => (
                                    "border-emerald-500/40",
                                    "bg-emerald-500/15 text-emerald-600 dark:text-emerald-400",
                                ),
                                ToastVariant::Error => (
                                    "border-destructive/40",
                                    "bg-destructive/15 text-destructive",
                                ),
                                ToastVariant::Info => ("border-sky-500/40", "bg-sky-500/15 text-sky-600 dark:text-sky-400"),
                            };

                            let title = item.title.clone();
                            let description = item.description.clone();

                            rsx! {
                                div {
                                    key: "{id}",
                                    class: format!(
                                        "absolute top-0 right-0 w-full origin-top {pointer_events} select-none"
                                    ),
                                    style: format!(
                                        "transform: translateY({y_offset}px) translateX({x_offset:.1}px) scale({scale:.2}); opacity: {opacity:.2}; z-index: {z_index}; {transition_style}"
                                    ),
                                    onmousedown: move |e: MouseEvent| {
                                        // Start swipe tracking on mouse down
                                        drag_state.set(Some((id, e.client_coordinates().x)));
                                        drag_offset.set(0.0);
                                    },
                                    onclick: move |_| {
                                        // Only expand if user clicked (not swiped) on the stack
                                        let offset = drag_offset.peek().abs();
                                        if offset < 6.0 && !is_expanded && total > 1 {
                                            is_pinned.set(true);
                                        }
                                    },

                                    // Inner Card Container
                                    div {
                                        role: "status",
                                        class: format!(
                                            "relative overflow-hidden flex w-full items-start gap-3 rounded-xl border {border_class} bg-card/95 p-3.5 shadow-lg backdrop-blur-md transition-shadow select-none {} {}",
                                            if is_being_dragged { "cursor-grabbing ring-1 ring-primary/30" } else { "cursor-grab hover:border-primary/50" },
                                            if !is_expanded && !is_front { "hover:scale-[1.01]" } else { "" }
                                        ),

                                        // Icon
                                        div { class: "flex size-7 shrink-0 items-center justify-center rounded-lg {icon_class}",
                                            match item.variant {
                                                ToastVariant::Success => rsx! {
                                                    CheckCircle2Icon { class: "size-4" }
                                                },
                                                ToastVariant::Error => rsx! {
                                                    AlertCircleIcon { class: "size-4" }
                                                },
                                                _ => rsx! {
                                                    BellIcon { class: "size-4" }
                                                },
                                            }
                                        }

                                        // Text Content
                                        div { class: "flex min-w-0 flex-1 flex-col gap-0.5 pt-0.5",
                                            span { class: "text-xs font-bold leading-tight text-foreground truncate",
                                                "{title}"
                                            }
                                            if let Some(desc) = description.as_ref() {
                                                span { class: "text-[11px] leading-snug text-muted-foreground line-clamp-2",
                                                    "{desc}"
                                                }
                                            }
                                        }

                                        // Dismiss Button
                                        button {
                                            r#type: "button",
                                            class: "shrink-0 cursor-pointer rounded-md p-1 text-muted-foreground transition-colors hover:bg-muted hover:text-foreground",
                                            onmousedown: move |e: MouseEvent| e.stop_propagation(),
                                            onclick: move |e: MouseEvent| {
                                                e.stop_propagation();
                                                toast.dismiss(id);
                                            },
                                            title: "Dismiss notification",
                                            XIcon { class: "size-3.5" }
                                        }

                                        // Countdown Timer Bar (auto-dismisses when animation completes)
                                        div {
                                            class: "absolute bottom-0 left-0 h-0.5 bg-primary/30 toast-timer-bar",
                                            onanimationend: move |_| toast.dismiss(id),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
