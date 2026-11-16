use dioxus::prelude::*;
use crate::icons::{ExternalLinkIcon, FileTextIcon, PaperclipIcon, XIcon};

/// Extract direct viewable image preview URL (supporting Google Drive links and standard image formats).
pub fn get_image_preview_url(filename: &str, url: &str) -> Option<String> {
    let lower_name = filename.to_lowercase();
    let lower_url = url.to_lowercase();

    let is_img = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".svg", ".bmp"]
        .iter()
        .any(|ext| lower_name.ends_with(ext) || lower_url.contains(ext))
        || lower_name.contains("screenshot")
        || lower_url.contains("drive.google.com");

    if !is_img && url.trim().is_empty() {
        return None;
    }

    let url_trimmed = url.trim();

    // If it's a Google Drive link, extract file ID and convert to direct image preview URL
    if url_trimmed.contains("drive.google.com") {
        if let Some(pos) = url_trimmed.find("/file/d/") {
            let after = &url_trimmed[pos + 8..];
            let id = after
                .split('/')
                .next()
                .unwrap_or(after)
                .split('?')
                .next()
                .unwrap_or(after);
            if !id.is_empty() {
                return Some(format!("https://lh3.googleusercontent.com/d/{id}"));
            }
        } else if let Some(pos) = url_trimmed.find("id=") {
            let after = &url_trimmed[pos + 3..];
            let id = after.split('&').next().unwrap_or(after);
            if !id.is_empty() {
                return Some(format!("https://lh3.googleusercontent.com/d/{id}"));
            }
        }
    }

    if is_img && !url_trimmed.is_empty() {
        let local_path = std::path::Path::new(url_trimmed);
        if local_path.is_file() {
            if let Ok(bytes) = std::fs::read(local_path) {
                let mime = crate::db::guess_mime_type(url_trimmed);
                let b64 = crate::db::encode_base64(&bytes);
                return Some(format!("data:{mime};base64,{b64}"));
            }
        }
        return Some(url_trimmed.to_string());
    }

    None
}

/// Reusable attachment upload & preview widget used by both Create and Modify Ticket modals.
/// Saves picked files into the temporary attachments folder so they can be previewed locally
/// and synced to Google Apps Script.
#[component]
pub fn AttachmentPicker(
    mut attachment_name: Signal<String>,
    mut attachment_url: Signal<String>,
) -> Element {
    let has_attachment =
        !attachment_name.read().is_empty() || !attachment_url.read().is_empty();

    rsx! {
        div { class: "rounded-md border border-border/70 bg-muted/20 p-3.5 flex flex-col gap-2.5",
            div { class: "flex items-center justify-between",
                span { class: "text-[10px] font-semibold text-muted-foreground uppercase tracking-wider flex items-center gap-1.5",
                    PaperclipIcon { class: "size-3.5 text-primary" }
                    "Attachment (Optional)"
                }
                if has_attachment {
                    button {
                        r#type: "button",
                        class: "text-[11px] text-destructive hover:underline flex items-center gap-1 cursor-pointer",
                        onclick: move |_| {
                            let old_url = attachment_url.read().clone();
                            crate::db::remove_temp_attachment_if_managed(&old_url);
                            attachment_name.set(String::new());
                            attachment_url.set(String::new());
                        },
                        XIcon { class: "size-3" }
                        "Clear Attachment"
                    }
                }
            }

            if !has_attachment {
                div { class: "flex flex-col gap-2",
                    label { class: "flex flex-col items-center justify-center border-2 border-dashed border-border/80 hover:border-primary/60 bg-card/60 hover:bg-card p-4 rounded-lg cursor-pointer transition-all duration-150 group",
                        input {
                            r#type: "file",
                            class: "hidden",
                            onchange: move |e: FormEvent| {
                                let raw_val = e.value();
                                let files = e.files();
                                spawn(async move {
                                    if let Some(file_data) = files.into_iter().next() {
                                        let raw_name = file_data.name();
                                        let clean_name = std::path::Path::new(&raw_name)
                                            .file_name()
                                            .map(|n| n.to_string_lossy().to_string())
                                            .unwrap_or_else(|| raw_name.clone());
                                        if let Ok(bytes) = file_data.read_bytes().await {
                                            if let Ok(saved_path) = crate::db::save_temp_attachment(&clean_name, &bytes) {
                                                attachment_name.set(clean_name);
                                                attachment_url.set(saved_path.to_string_lossy().to_string());
                                                return;
                                            }
                                        }
                                        if !clean_name.is_empty() {
                                            attachment_name.set(clean_name.clone());
                                            attachment_url.set(format!("/attachments/{}", clean_name));
                                        }
                                        return;
                                    }
                                    let clean = raw_val
                                        .replace("C:\\fakepath\\", "")
                                        .replace('\\', "/");
                                    let filename = clean.split('/').next_back().unwrap_or(&clean).to_string();
                                    if !filename.is_empty() {
                                        attachment_name.set(filename.clone());
                                        attachment_url.set(format!("/attachments/{}", filename));
                                    }
                                });
                            },
                        }
                        div { class: "flex items-center gap-2 text-muted-foreground group-hover:text-primary transition-colors",
                            PaperclipIcon { class: "size-4" }
                            span { class: "text-xs font-medium", "Choose file or screenshot..." }
                        }
                        span { class: "text-[10px] text-muted-foreground/70 mt-1",
                            "Saved locally in temp cache · Synced to Google Apps Script"
                        }
                    }

                    div { class: "flex items-center gap-2",
                        div { class: "h-px flex-1 bg-border/60" }
                        span { class: "text-[10px] text-muted-foreground font-mono uppercase tracking-wider", "or paste cloud URL" }
                        div { class: "h-px flex-1 bg-border/60" }
                    }

                    input {
                        r#type: "url",
                        class: "w-full text-xs h-8 px-2.5 rounded border border-border/80 bg-card font-mono text-foreground placeholder:text-muted-foreground focus:border-primary focus:outline-none",
                        placeholder: "https://drive.google.com/file/d/... or image URL",
                        value: attachment_url.read().clone(),
                        oninput: move |e: FormEvent| {
                            let val = e.value();
                            attachment_url.set(val.clone());
                            if attachment_name.read().is_empty() && !val.trim().is_empty() {
                                let guessed = val.split('/').next_back().unwrap_or("cloud_attachment").to_string();
                                attachment_name.set(guessed);
                            }
                        },
                    }
                }
            } else {
                {
                    let att_name = attachment_name.read().clone();
                    let att_url = attachment_url.read().clone();
                    let img_preview = get_image_preview_url(&att_name, &att_url);
                    let is_temp = crate::db::is_temp_attachment_path(&att_url);

                    rsx! {
                        div { class: "flex flex-col gap-2 rounded-md bg-muted/40 border border-border/70 p-3",
                            div { class: "flex items-center justify-between",
                                div { class: "flex items-center gap-2.5 min-w-0",
                                    div { class: "flex size-8 shrink-0 items-center justify-center rounded bg-primary/10 text-primary",
                                        FileTextIcon { class: "size-4" }
                                    }
                                    div { class: "flex flex-col min-w-0",
                                        span { class: "font-mono font-semibold text-foreground text-xs truncate",
                                            "{att_name}"
                                        }
                                        span { class: "text-[10px] text-muted-foreground font-mono truncate",
                                            if att_url.is_empty() {
                                                "Local attachment"
                                            } else {
                                                "{att_url}"
                                            }
                                        }
                                    }
                                }
                                span { class: "text-[10px] bg-primary/20 text-primary px-2 py-0.5 rounded font-medium shrink-0",
                                    if is_temp { "Temp Cached" } else { "Attached" }
                                }
                            }

                            if let Some(preview_src) = img_preview {
                                div { class: "mt-1 flex flex-col items-center justify-center rounded-md border border-border/70 bg-card p-2 shadow-xs",
                                    img {
                                        src: "{preview_src}",
                                        alt: "{att_name}",
                                        class: "max-h-72 max-w-full rounded object-contain transition-all hover:scale-[1.01]",
                                        loading: "lazy",
                                    }
                                    if !att_url.is_empty() && !is_temp {
                                        a {
                                            href: "{att_url}",
                                            target: "_blank",
                                            class: "mt-2 inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                            ExternalLinkIcon { class: "size-3" }
                                            "Open full image in browser"
                                        }
                                    }
                                }
                            } else if !att_url.is_empty() && !is_temp {
                                div { class: "mt-1",
                                    a {
                                        href: "{att_url}",
                                        target: "_blank",
                                        class: "inline-flex items-center gap-1 text-[11px] font-medium text-primary hover:underline",
                                        ExternalLinkIcon { class: "size-3" }
                                        "Open attachment link"
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
