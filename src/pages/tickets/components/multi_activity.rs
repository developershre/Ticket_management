use dioxus::prelude::*;

pub fn clean_badge_item(s: &str) -> String {
    let trimmed = s.trim();
    let lower = trimmed.to_lowercase();
    if lower.starts_with("actioned:") {
        trimmed[9..].trim().to_uppercase()
    } else if lower.starts_with("actioned :") {
        trimmed[10..].trim().to_uppercase()
    } else {
        trimmed.to_uppercase()
    }
}

fn remove_tag(
    mut value: Signal<String>,
    idx: usize,
    mut editing_index: Signal<Option<usize>>,
    mut inline_edit_idx: Signal<Option<usize>>,
) {
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| clean_badge_item(s))
        .filter(|s| !s.is_empty())
        .collect();

    if idx < list.len() {
        list.remove(idx);
        value.set(list.join(", "));
    }

    let curr_opt = *editing_index.read();
    if curr_opt == Some(idx) {
        editing_index.set(None);
    } else if let Some(curr_idx) = curr_opt {
        if idx < curr_idx {
            editing_index.set(Some(curr_idx - 1));
        }
    }

    if *inline_edit_idx.read() == Some(idx) {
        inline_edit_idx.set(None);
    }
}

fn add_or_update_tag(
    mut value: Signal<String>,
    mut tag_input: Signal<String>,
    mut editing_index: Signal<Option<usize>>,
) {
    let text = tag_input.read().trim().to_uppercase();

    let edit_target = *editing_index.read();
    // Check if we are in modify mode for an existing tag
    if let Some(idx) = edit_target {
        let raw = value.read().clone();
        let mut list: Vec<String> = raw
            .split(',')
            .map(|s| clean_badge_item(s))
            .filter(|s| !s.is_empty())
            .collect();

        if text.is_empty() {
            // If modified to empty, remove that tag
            if idx < list.len() {
                list.remove(idx);
            }
        } else {
            let clean = clean_badge_item(&text);
            if idx < list.len() {
                list[idx] = clean;
            }
        }
        value.set(list.join(", "));
        tag_input.set(String::new());
        editing_index.set(None);
        return;
    }

    // Normal mode: add new tag(s)
    if text.is_empty() {
        return;
    }
    let parts: Vec<&str> = text
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.is_empty() {
        return;
    }

    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| clean_badge_item(s))
        .filter(|s| !s.is_empty())
        .collect();

    for p in parts {
        let p_clean = clean_badge_item(p);
        if !p_clean.is_empty() && !list.iter().any(|x| x.eq_ignore_ascii_case(&p_clean)) {
            list.push(p_clean);
        }
    }
    value.set(list.join(", "));
    tag_input.set(String::new());
}

fn save_inline_edit(
    idx: usize,
    mut value: Signal<String>,
    mut inline_edit_idx: Signal<Option<usize>>,
    inline_edit_val: Signal<String>,
) {
    let text = inline_edit_val.read().trim().to_uppercase();
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| clean_badge_item(s))
        .filter(|s| !s.is_empty())
        .collect();

    if text.is_empty() {
        if idx < list.len() {
            list.remove(idx);
        }
    } else {
        let clean = clean_badge_item(&text);
        if idx < list.len() {
            list[idx] = clean;
        }
    }
    value.set(list.join(", "));
    inline_edit_idx.set(None);
}

#[derive(Props, Clone, PartialEq)]
pub struct MultiActivitySelectorProps {
    pub value: Signal<String>,
}

#[component]
pub fn MultiActivitySelector(props: MultiActivitySelectorProps) -> Element {
    let mut tag_input = use_signal(String::new);
    let mut editing_index = use_signal(|| Option::<usize>::None);
    let mut inline_edit_idx = use_signal(|| Option::<usize>::None);
    let mut inline_edit_val = use_signal(String::new);
    let mut value = props.value;

    let raw_val = value.read().clone();
    let selected_items: Vec<String> = raw_val
        .split(',')
        .map(|s| clean_badge_item(s))
        .filter(|s| !s.is_empty())
        .collect();

    let is_modifying = editing_index.read().is_some();

    rsx! {
        div { class: "flex flex-col gap-2 w-full",
            // Header stats: Count of already added tags & actions
            div { class: "flex items-center justify-between text-xs",
                div { class: "flex items-center gap-2",
                    span { class: "font-semibold text-foreground/90", "Added Activity Tags" }
                    span {
                        class: if selected_items.is_empty() {
                            "px-2 py-0.5 rounded-full bg-muted text-muted-foreground font-bold text-[10px]"
                        } else {
                            "px-2 py-0.5 rounded-full bg-[#00c5eb]/20 text-[#008ba8] dark:text-[#00c5eb] font-bold text-[10px] border border-[#00c5eb]/40"
                        },
                        "{selected_items.len()} added"
                    }
                }

                div { class: "flex items-center gap-2",
                    if is_modifying {
                        span { class: "text-[11px] font-medium text-amber-500",
                            "Editing tag #{editing_index.read().unwrap() + 1}"
                        }
                        button {
                            r#type: "button",
                            class: "text-[11px] text-muted-foreground hover:text-foreground underline cursor-pointer",
                            onclick: move |_| {
                                editing_index.set(None);
                                tag_input.set(String::new());
                            },
                            "Cancel edit"
                        }
                    } else if !selected_items.is_empty() {
                        button {
                            r#type: "button",
                            class: "text-[10px] text-destructive hover:underline cursor-pointer",
                            onclick: move |_| {
                                value.set(String::new());
                                editing_index.set(None);
                                inline_edit_idx.set(None);
                            },
                            "Remove all ({selected_items.len()})"
                        }
                    }
                }
            }

            // Top input group matching user screenshot (Image 4)
            // Switches styling dynamically when modifying an existing tag
            div {
                class: if is_modifying {
                    "flex items-center w-full rounded-md border-2 border-amber-500/80 shadow-xs overflow-hidden bg-amber-500/5 transition-all"
                } else {
                    "flex items-center w-full rounded-md border border-input shadow-xs overflow-hidden focus-within:ring-1 focus-within:ring-primary focus-within:border-primary bg-card transition-all"
                },
                input {
                    r#type: "text",
                    class: "flex-1 h-9 px-3 bg-transparent text-xs text-foreground placeholder:text-muted-foreground focus:outline-none uppercase font-mono",
                    placeholder: if is_modifying { "Modify tag and press Update..." } else { "Add a new tag..." },
                    value: tag_input.read().clone(),
                    oninput: move |e: FormEvent| tag_input.set(e.value().to_uppercase()),
                    onkeydown: move |e: KeyboardEvent| {
                        if e.key() == Key::Enter {
                            e.prevent_default();
                            add_or_update_tag(value, tag_input, editing_index);
                        } else if e.key() == Key::Escape && is_modifying {
                            editing_index.set(None);
                            tag_input.set(String::new());
                        }
                    },
                }

                if is_modifying {
                    button {
                        r#type: "button",
                        class: "h-9 px-3 bg-amber-500 hover:bg-amber-600 text-slate-950 text-xs font-bold cursor-pointer transition-colors focus:outline-none shrink-0",
                        onclick: move |_| add_or_update_tag(value, tag_input, editing_index),
                        "Update"
                    }
                    button {
                        r#type: "button",
                        class: "h-9 px-2.5 bg-transparent border-l border-amber-500/40 text-muted-foreground hover:text-foreground text-xs cursor-pointer transition-colors focus:outline-none shrink-0",
                        onclick: move |_| {
                            editing_index.set(None);
                            tag_input.set(String::new());
                        },
                        "✕"
                    }
                } else {
                    button {
                        r#type: "button",
                        class: "h-9 px-4 bg-transparent border-l border-input text-primary hover:bg-primary/10 text-xs font-semibold cursor-pointer transition-colors focus:outline-none shrink-0",
                        onclick: move |_| add_or_update_tag(value, tag_input, editing_index),
                        "Add"
                    }
                }
            }

            // Container box displaying cyan badges with Modify (✎) and Remove (⊗) actions
            div { class: "min-h-[52px] w-full rounded-md border border-border/80 bg-muted/20 p-2.5 flex flex-wrap items-center gap-2",
                if selected_items.is_empty() {
                    span { class: "text-xs text-muted-foreground italic px-1",
                        "No activity tags added yet. Enter a tag above and click Add or press Enter."
                    }
                } else {
                    for (idx, item) in selected_items.iter().enumerate() {
                        {
                            let item_str = item.clone();
                            let is_inline_editing = *inline_edit_idx.read() == Some(idx);
                            let is_being_modified = *editing_index.read() == Some(idx);

                            rsx! {
                                span {
                                    key: "{item_str}_{idx}",
                                    class: if is_being_modified {
                                        "inline-flex items-center gap-1.5 rounded bg-amber-400 text-slate-950 font-bold text-xs px-2.5 py-1 uppercase shadow-sm ring-2 ring-amber-500 select-none transition-all"
                                    } else {
                                        "inline-flex items-center gap-1.5 rounded bg-[#00c5eb] dark:bg-[#00c5eb] text-slate-950 font-bold text-xs px-2.5 py-1 uppercase shadow-2xs select-none transition-all"
                                    },

                                    if is_inline_editing {
                                        // Inline quick-edit mode
                                        input {
                                            r#type: "text",
                                            class: "w-28 px-1.5 py-0.5 rounded bg-white text-slate-950 font-bold text-xs uppercase focus:outline-none shadow-inner",
                                            autofocus: true,
                                            value: inline_edit_val.read().clone(),
                                            oninput: move |e: FormEvent| inline_edit_val.set(e.value().to_uppercase()),
                                            onkeydown: move |e: KeyboardEvent| {
                                                if e.key() == Key::Enter {
                                                    e.prevent_default();
                                                    save_inline_edit(idx, value, inline_edit_idx, inline_edit_val);
                                                } else if e.key() == Key::Escape {
                                                    inline_edit_idx.set(None);
                                                }
                                            },
                                        }
                                        button {
                                            r#type: "button",
                                            class: "cursor-pointer rounded-full p-0.5 hover:bg-black/20 text-slate-950 transition-colors focus:outline-none font-bold text-xs",
                                            title: "Save changes",
                                            onclick: move |e: MouseEvent| {
                                                e.stop_propagation();
                                                save_inline_edit(idx, value, inline_edit_idx, inline_edit_val);
                                            },
                                            "✓"
                                        }
                                        button {
                                            r#type: "button",
                                            class: "cursor-pointer rounded-full p-0.5 hover:bg-black/20 text-slate-950 transition-colors focus:outline-none font-bold text-xs",
                                            title: "Cancel",
                                            onclick: move |e: MouseEvent| {
                                                e.stop_propagation();
                                                inline_edit_idx.set(None);
                                            },
                                            "✕"
                                        }
                                    } else {
                                        // Standard badge display with Modify and Remove buttons
                                        span {
                                            class: "cursor-pointer hover:underline",
                                            title: "Click to edit inline",
                                            onclick: {
                                                let str_clone = item_str.clone();
                                                move |_| {
                                                    inline_edit_idx.set(Some(idx));
                                                    inline_edit_val.set(str_clone.clone());
                                                }
                                            },
                                            "{item_str}"
                                        }

                                        // Modify button (pencil icon)
                                        button {
                                            r#type: "button",
                                            class: "cursor-pointer rounded-full p-0.5 hover:bg-black/20 text-slate-950 transition-colors focus:outline-none flex items-center justify-center",
                                            title: "Modify this tag",
                                            onclick: {
                                                let str_clone = item_str.clone();
                                                move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    editing_index.set(Some(idx));
                                                    tag_input.set(str_clone.clone());
                                                }
                                            },
                                            svg {
                                                width: "13",
                                                height: "13",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2.3",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",

                                                path { d: "M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" }
                                                path { d: "m15 5 4 4" }
                                            }
                                        }

                                        // Remove button (⊗ x-circle) matching Image 4
                                        button {
                                            r#type: "button",
                                            class: "cursor-pointer rounded-full p-0.5 hover:bg-black/20 text-slate-950 transition-colors focus:outline-none flex items-center justify-center",
                                            title: "Remove tag",
                                            onclick: move |e: MouseEvent| {
                                                e.stop_propagation();
                                                remove_tag(value, idx, editing_index, inline_edit_idx);
                                            },
                                            svg {
                                                width: "14",
                                                height: "14",
                                                view_box: "0 0 24 24",
                                                fill: "none",
                                                stroke: "currentColor",
                                                stroke_width: "2.2",
                                                stroke_linecap: "round",
                                                stroke_linejoin: "round",

                                                circle { cx: "12", cy: "12", r: "10" }
                                                line { x1: "15", y1: "9", x2: "9", y2: "15" }
                                                line { x1: "9", y1: "9", x2: "15", y2: "15" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Bottom helper text
            div { class: "flex items-center justify-between text-[10px] text-muted-foreground px-0.5",
                span { "Click ✎ or click badge text to modify it · Click ⊗ to remove" }
                span { class: "font-mono font-medium", "{selected_items.len()} tag(s)" }
            }
        }
    }
}
