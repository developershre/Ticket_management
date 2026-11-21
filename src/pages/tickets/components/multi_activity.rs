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

fn remove_tag(mut value: Signal<String>, idx: usize) {
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
}

fn add_tag(mut value: Signal<String>, mut tag_input: Signal<String>) {
    let text = tag_input.read().trim().to_uppercase();
    if text.is_empty() {
        return;
    }
    let parts: Vec<&str> = text
        .split(|c| c == ',' || c == '\n' || c == ';')
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

#[derive(Props, Clone, PartialEq)]
pub struct MultiActivitySelectorProps {
    pub value: Signal<String>,
}

#[component]
pub fn MultiActivitySelector(props: MultiActivitySelectorProps) -> Element {
    let mut tag_input = use_signal(String::new);
    let value = props.value;

    let raw_val = value.read().clone();
    let selected_items: Vec<String> = raw_val
        .split(',')
        .map(|s| clean_badge_item(s))
        .filter(|s| !s.is_empty())
        .collect();

    rsx! {
        div { class: "flex flex-col gap-1.5 w-full",
            // Single unified tag input container: monochromatic tags rendered directly inside the input field
            div { class: "flex items-center w-full min-h-[42px] rounded-md border border-input bg-card shadow-xs focus-within:ring-1 focus-within:ring-primary focus-within:border-primary overflow-hidden transition-all",
                // Inline tags and typing input
                div { class: "flex flex-wrap items-center gap-1.5 flex-1 p-2 min-w-0 cursor-text",
                    for (idx, item) in selected_items.iter().enumerate() {
                        {
                            let item_str = item.clone();
                            rsx! {
                                span {
                                    key: "{item_str}_{idx}",
                                    class: "inline-flex items-center gap-1.5 rounded bg-neutral-200 dark:bg-neutral-800 text-neutral-900 dark:text-neutral-100 border border-neutral-300 dark:border-neutral-700 px-2.5 py-0.5 text-xs font-bold uppercase tracking-wider select-none shrink-0 shadow-2xs",
                                    "{item_str}"
                                    button {
                                        r#type: "button",
                                        class: "cursor-pointer rounded-full p-0.5 text-neutral-500 hover:text-neutral-900 dark:text-neutral-400 dark:hover:text-neutral-100 hover:bg-neutral-300 dark:hover:bg-neutral-700 transition-colors focus:outline-none flex items-center justify-center",
                                        title: "Remove tag {item_str}",
                                        onclick: move |e: MouseEvent| {
                                            e.stop_propagation();
                                            remove_tag(value, idx);
                                        },
                                        svg {
                                            width: "12",
                                            height: "12",
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

                    // Text input right inside the field
                    input {
                        r#type: "text",
                        class: "flex-1 min-w-[120px] h-7 px-1 bg-transparent text-xs text-foreground placeholder:text-muted-foreground focus:outline-none uppercase font-mono",
                        placeholder: if selected_items.is_empty() { "ADD A NEW TAG..." } else { "ADD TAG..." },
                        value: tag_input.read().clone(),
                        oninput: move |e: FormEvent| {
                            let val = e.value();
                            if val.ends_with(',') {
                                let trimmed = val.trim_end_matches(',').trim().to_uppercase();
                                if !trimmed.is_empty() {
                                    tag_input.set(trimmed);
                                    add_tag(value, tag_input);
                                } else {
                                    tag_input.set(String::new());
                                }
                            } else {
                                tag_input.set(val.to_uppercase());
                            }
                        },
                        onkeydown: move |e: KeyboardEvent| {
                            if e.key() == Key::Enter {
                                e.prevent_default();
                                add_tag(value, tag_input);
                            } else if e.key() == Key::Backspace && tag_input.read().is_empty() && !selected_items.is_empty() {
                                remove_tag(value, selected_items.len() - 1);
                            }
                        },
                    }
                }

                // Add button attached to the right side of the input field
                button {
                    r#type: "button",
                    class: "self-stretch px-4 bg-transparent border-l border-input text-foreground hover:bg-muted text-xs font-semibold cursor-pointer transition-colors focus:outline-none shrink-0 flex items-center justify-center",
                    title: "Add tag",
                    onclick: move |_| add_tag(value, tag_input),
                    "Add"
                }
            }

            if !selected_items.is_empty() {
                div { class: "flex items-center justify-between text-[10px] text-muted-foreground px-0.5",
                    span { "Press Enter or comma to add tag · Click ⊗ or press Backspace to remove" }
                    span { class: "font-mono font-medium", "{selected_items.len()} tag(s)" }
                }
            }
        }
    }
}
