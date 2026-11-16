use dioxus::prelude::*;

fn remove_activity(mut value: Signal<String>, item: &str) {
    let item_upper = item.trim().to_uppercase();
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    list.retain(|x| x != &item_upper);
    value.set(list.join(", ").to_uppercase());
}

fn add_custom_activity(mut value: Signal<String>, mut custom_input: Signal<String>) {
    let text = custom_input.read().trim().to_uppercase();
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
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    for p in parts {
        let p_upper = p.to_uppercase();
        if !list.contains(&p_upper) {
            list.push(p_upper);
        }
    }
    value.set(list.join(", ").to_uppercase());
    custom_input.set(String::new());
}

fn pop_last_activity(mut value: Signal<String>) {
    let raw = value.read().clone();
    let mut list: Vec<String> = raw
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    if list.pop().is_some() {
        value.set(list.join(", ").to_uppercase());
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct MultiActivitySelectorProps {
    pub value: Signal<String>,
}

#[component]
pub fn MultiActivitySelector(props: MultiActivitySelectorProps) -> Element {
    let mut custom_input = use_signal(String::new);
    let value = props.value;

    let raw_val = value.read().clone();
    let selected_items: Vec<String> = raw_val
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();

    rsx! {
        div { class: "min-h-[42px] w-full rounded-md border border-border/80 bg-card px-2.5 py-1.5 flex flex-wrap items-center gap-1.5 focus-within:border-primary focus-within:ring-1 focus-within:ring-primary/20 transition-all cursor-text",
            for item in selected_items.iter() {
                {
                    let item_str = item.clone();
                    rsx! {
                        div {
                            class: "inline-flex items-center gap-1 rounded bg-primary/15 px-2 py-0.5 text-[11px] font-bold tracking-wide text-primary border border-primary/30 uppercase select-none shadow-2xs",
                            "{item_str}"
                            button {
                                r#type: "button",
                                onclick: {
                                    let to_remove = item_str.clone();
                                    move |_| remove_activity(value, &to_remove)
                                },
                                svg {
                                    width: "24",
                                    height: "24",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    class: "stroke-black hover:stroke-red-700 cursor-pointer font-bold leading-none select-none text-xs",
                                    xmlns: "http://www.w3.org/2000/svg",

                                    path {
                                        d: "M16 16L12 12M12 12L8 8M12 12L16 8M12 12L8 16",
                                        stroke_width: "2",
                                        stroke_linecap: "round",
                                        stroke_linejoin: "round",
                                    }
                                }
                            }
                        }
                    }
                }
            }

            input {
                r#type: "text",
                class: "flex-1 min-w-[160px] bg-transparent text-xs text-foreground placeholder:text-muted-foreground focus:outline-none uppercase font-mono py-0.5",
                placeholder: if selected_items.is_empty() { "Type activity and press Enter..." } else { "Type and press Enter..." },
                value: custom_input.read().clone(),
                oninput: move |e: FormEvent| {
                    let val = e.value();
                    if val.ends_with(',') {
                        custom_input.set(val[..val.len() - 1].to_uppercase());
                        add_custom_activity(value, custom_input);
                    } else {
                        custom_input.set(val.to_uppercase());
                    }
                },
                onkeydown: move |e: KeyboardEvent| {
                    if e.key() == Key::Enter {
                        add_custom_activity(value, custom_input);
                    } else if e.key() == Key::Backspace && custom_input.read().is_empty() {
                        pop_last_activity(value);
                    }
                },
            }
        }
    }
}
