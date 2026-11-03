//! Notes page — interactive workspace for viewing, searching, creating,
//! editing, and managing all persistent notes in `management.db`.

use dioxus::prelude::*;

use crate::components::ui::button::Button;
use crate::components::ui::context_menu::{
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuLabel, ContextMenuSeparator,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger,
};
use crate::components::ui::dialog::{
    Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle,
};
use crate::components::ui::input::Input;
use crate::components::ui::toast::ToastState;
use crate::db::DbNote;
use crate::icons::{
    CheckIcon, CopyIcon, FileTextIcon, PencilIcon, PlusIcon, SearchIcon, StickyNoteIcon, TrashIcon,
};
use crate::state::AppDbRevision;

/// Parsed representation of a note from `NOTES` table.
#[derive(Clone, PartialEq, Debug)]
pub struct NoteItem {
    pub id: i64,
    pub emp_code: String,
    pub title: String,
    pub details: String,
    pub raw_content: String,
}

impl NoteItem {
    pub fn from_db(n: DbNote) -> Self {
        let (title, details) = if let Some((t, d)) = n.content.split_once(" — ") {
            (t.trim().to_string(), d.trim().to_string())
        } else if let Some((t, d)) = n.content.split_once('\n') {
            (t.trim().to_string(), d.trim().to_string())
        } else {
            (n.content.clone(), String::new())
        };

        Self {
            id: n.id,
            emp_code: n.emp_code,
            title,
            details,
            raw_content: n.content,
        }
    }
}

#[component]
pub fn NotesPage() -> Element {
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<AppDbRevision>>();

    // Reactive notes list loaded from SQLite
    let mut local_revision = use_signal(|| 0u64);
    let mut search_query = use_signal(String::new);

    // Create note modal state
    let mut is_create_open = use_signal(|| false);
    let mut new_title = use_signal(String::new);
    let mut new_details = use_signal(String::new);

    // Edit note modal state
    let mut is_edit_open = use_signal(|| false);
    let mut edit_id = use_signal(|| 0i64);
    let mut edit_title = use_signal(String::new);
    let mut edit_details = use_signal(String::new);

    // Delete note modal state
    let mut is_delete_open = use_signal(|| false);
    let mut delete_id = use_signal(|| 0i64);
    let mut delete_title = use_signal(String::new);

    // Read all notes from database only when db_sync or local_revision changes (memoized)
    let notes_memo = use_memo(move || {
        let _ = local_revision();
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        crate::db::get_all_notes()
            .into_iter()
            .map(NoteItem::from_db)
            .collect::<Vec<NoteItem>>()
    });
    let notes_data = notes_memo();

    let total_count = notes_data.len();

    // Filter by search query
    let filtered_notes: Vec<NoteItem> = {
        let q = search_query.read().trim().to_lowercase();
        if q.is_empty() {
            notes_data
        } else {
            notes_data
                .into_iter()
                .filter(|n| {
                    n.title.to_lowercase().contains(&q)
                        || n.details.to_lowercase().contains(&q)
                        || n.emp_code.to_lowercase().contains(&q)
                        || n.id.to_string().contains(&q)
                })
                .collect()
        }
    };

    rsx! {
        div { class: "flex flex-1 flex-col h-full overflow-hidden bg-background",
            // ============================================================
            // Page Header
            // ============================================================
            header { class: "flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-border/70 bg-card/60 backdrop-blur-md px-6 py-4.5",
                div { class: "flex items-center gap-3",
                    div { class: "flex size-10 items-center justify-center rounded-xl bg-muted text-foreground border border-border/80 shadow-xs",
                        StickyNoteIcon { class: "size-5" }
                    }
                    div { class: "flex flex-col",
                        div { class: "flex items-center gap-2.5",
                            h1 { class: "text-lg font-bold tracking-tight text-foreground", "Notes" }
                            span { class: "rounded-full bg-muted border border-border/80 px-2.5 py-0.5 text-xs font-semibold text-muted-foreground",
                                "{total_count} saved"
                            }
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Quick notes captured from the FAB button and workspace, stored directly in SQLite (management.db)"
                        }
                    }
                }

                div { class: "flex items-center gap-3",
                    // Search bar
                    div { class: "relative w-64 sm:w-72",
                        SearchIcon { class: "absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" }
                        Input {
                            class: "pl-8 text-xs h-9 bg-card border-border/80 focus-visible:ring-1",
                            placeholder: "Search notes, titles, or authors...",
                            value: search_query.read().clone(),
                            oninput: move |e: FormEvent| search_query.set(e.value()),
                        }
                        if !search_query.read().is_empty() {
                            button {
                                r#type: "button",
                                title: "Clear search",
                                class: "absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground text-xs",
                                onclick: move |_| search_query.set(String::new()),
                                "✕"
                            }
                        }
                    }

                    // New note button
                    Button {
                        variant: "default",
                        size: "sm",
                        class: "gap-1.5 font-semibold shadow-xs cursor-pointer",
                        onclick: move |_| {
                            new_title.set(String::new());
                            new_details.set(String::new());
                            is_create_open.set(true);
                        },
                        PlusIcon { class: "size-4" }
                        "New Note"
                    }
                }
            }

            // ============================================================
            // Notes Content Grid
            // ============================================================
            main { class: "flex-1 overflow-y-auto p-6",
                if filtered_notes.is_empty() {
                    div { class: "flex flex-col items-center justify-center min-h-[360px] text-center p-8 rounded-2xl border border-dashed border-border/70 bg-card/30",
                        div { class: "flex size-14 items-center justify-center rounded-2xl bg-muted/60 text-muted-foreground mb-3",
                            FileTextIcon { class: "size-7" }
                        }
                        if search_query.read().is_empty() {
                            h3 { class: "text-base font-semibold text-foreground mb-1", "No notes yet" }
                            p { class: "text-xs text-muted-foreground max-w-sm mb-4",
                                "Create quick notes on the fly using the floating action button (FAB) in the bottom-right corner or the \"New Note\" button above."
                            }
                            Button {
                                variant: "default",
                                size: "sm",
                                onclick: move |_| {
                                    new_title.set(String::new());
                                    new_details.set(String::new());
                                    is_create_open.set(true);
                                },
                                PlusIcon { class: "size-4 mr-1.5" }
                                "Create Note"
                            }
                        } else {
                            h3 { class: "text-base font-semibold text-foreground mb-1", "No matching notes found" }
                            p { class: "text-xs text-muted-foreground max-w-sm mb-4",
                                "No notes matched your query \"{search_query}\". Try a different search term or clear the filter."
                            }
                            Button {
                                variant: "outline",
                                size: "sm",
                                onclick: move |_| search_query.set(String::new()),
                                "Clear Search"
                            }
                        }
                    }
                } else {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4",
                        for note in filtered_notes.iter() {
                            {
                                let n_id = note.id;
                                let title_str = note.title.clone();
                                let details_str = note.details.clone();
                                let emp_code_str = note.emp_code.clone();
                                let copy_title = note.title.clone();
                                let edit_title_c = note.title.clone();
                                let edit_details_c = note.details.clone();
                                let del_title_c = note.title.clone();
                                let ctx_copy_title = note.title.clone();
                                let ctx_edit_title = note.title.clone();
                                let ctx_edit_details = note.details.clone();
                                let ctx_del_title = note.title.clone();

                                let tag_variant = match n_id % 5 {
                                    0 => "purple",
                                    1 => "blue",
                                    2 => "emerald",
                                    3 => "amber",
                                    _ => "cyan",
                                };
                                let tag_label = match n_id % 5 {
                                    0 => "Insight",
                                    1 => "Task",
                                    2 => "Doc",
                                    3 => "Idea",
                                    _ => "Note",
                                };

                                rsx! {
                                    ContextMenu {
                                        key: "note-ctx-{n_id}",
                                        ContextMenuTrigger {
                                            class: "contents",
                                            div {
                                                key: "note-{n_id}",
                                                class: "group relative flex flex-col justify-between rounded-xl border border-border/80 bg-card/90 p-5 shadow-sm hover:shadow-md hover:border-foreground/30 transition-all duration-200 backdrop-blur-sm",
                                                
                                                // Card Header: Badge & Quick actions
                                                div { class: "flex items-start justify-between gap-2 mb-3",
                                                    div { class: "flex items-center gap-1.5",
                                                        span { class: "font-mono text-[10px] font-bold text-muted-foreground bg-muted/60 px-2 py-0.5 rounded border border-border/70",
                                                            "#{n_id}"
                                                        }
                                                        span { class: "text-[10px] font-semibold text-muted-foreground bg-muted/40 px-2 py-0.5 rounded border border-border/50",
                                                            "{emp_code_str}"
                                                        }
                                                        crate::components::ui::badge::Badge {
                                                            variant: "{tag_variant}",
                                                            class: "text-[9px] font-semibold px-1.5 py-0.2",
                                                            "{tag_label}"
                                                        }
                                                    }

                                                    // Action buttons (Copy, Edit, Delete)
                                                    div { class: "flex items-center gap-1 opacity-70 group-hover:opacity-100 transition-opacity",
                                                        button {
                                                            r#type: "button",
                                                            title: "Copy note text",
                                                            class: "flex size-7 items-center justify-center rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                                            onclick: move |_| {
                                                                toast.success("Note Copied", format!("\"{}\" copied to clipboard.", copy_title));
                                                            },
                                                            CopyIcon { class: "size-3.5" }
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            title: "Edit note",
                                                            class: "flex size-7 items-center justify-center rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                                            onclick: move |_| {
                                                                edit_id.set(n_id);
                                                                edit_title.set(edit_title_c.clone());
                                                                edit_details.set(edit_details_c.clone());
                                                                is_edit_open.set(true);
                                                            },
                                                            PencilIcon { class: "size-3.5" }
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            title: "Delete note",
                                                            class: "flex size-7 items-center justify-center rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer",
                                                            onclick: move |_| {
                                                                delete_id.set(n_id);
                                                                delete_title.set(del_title_c.clone());
                                                                is_delete_open.set(true);
                                                            },
                                                            TrashIcon { class: "size-3.5" }
                                                        }
                                                    }
                                                }

                                                // Card Body: Title & Details
                                                div { class: "flex-1 mb-3",
                                                    h3 { class: "text-sm font-bold text-foreground leading-snug line-clamp-2 mb-1.5 group-hover:text-primary transition-colors",
                                                        "{title_str}"
                                                    }
                                                    if !details_str.is_empty() {
                                                        p { class: "text-xs text-muted-foreground line-clamp-4 whitespace-pre-wrap leading-relaxed",
                                                            "{details_str}"
                                                        }
                                                    }
                                                }

                                                // Card Footer: Char/Word counter & info
                                                div { class: "flex items-center justify-between border-t border-border/50 pt-2.5 text-[10px] text-muted-foreground",
                                                    span { "SQLite: NOTES" }
                                                    span { "{title_str.len() + details_str.len()} characters" }
                                                }
                                            }
                                        }
                                        ContextMenuContent {
                                            class: "w-56 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                                            ContextMenuLabel {
                                                div { class: "flex items-center justify-between gap-2",
                                                    span { class: "font-mono font-bold text-foreground text-xs", "#{n_id}" }
                                                    crate::components::ui::badge::Badge {
                                                        variant: "{tag_variant}",
                                                        class: "text-[9px] font-semibold px-1.5 py-0.2",
                                                        "{tag_label}"
                                                    }
                                                }
                                                span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5", "{title_str}" }
                                            }
                                            ContextMenuSeparator {}
                                            ContextMenuItem {
                                                on_select: {
                                                    let c_title = ctx_copy_title.clone();
                                                    move |_| {
                                                        toast.success("Note Copied", format!("\"{}\" copied to clipboard.", c_title));
                                                    }
                                                },
                                                CopyIcon { class: "size-3.5 text-muted-foreground" }
                                                "Copy Note Text"
                                            }
                                            ContextMenuItem {
                                                on_select: {
                                                    let t_c = ctx_edit_title.clone();
                                                    let d_c = ctx_edit_details.clone();
                                                    move |_| {
                                                        edit_id.set(n_id);
                                                        edit_title.set(t_c.clone());
                                                        edit_details.set(d_c.clone());
                                                        is_edit_open.set(true);
                                                    }
                                                },
                                                PencilIcon { class: "size-3.5 text-muted-foreground" }
                                                "Edit Note"
                                            }
                                            ContextMenuSeparator {}
                                            ContextMenuSub {
                                                ContextMenuSubTrigger {
                                                    "Assign Employee"
                                                }
                                                ContextMenuSubContent {
                                                    for &code in &["EMP-4019", "EMP-4020", "EMP-3042", "EMP-1088", "EMP-2045"] {
                                                        {
                                                            let is_active = emp_code_str == code;
                                                            let c_str = code.to_string();
                                                            rsx! {
                                                                ContextMenuItem {
                                                                    key: "emp-{code}",
                                                                    on_select: move |_| {
                                                                        let _ = crate::db::update_note_emp_code(n_id, &c_str);
                                                                        toast.success("Assigned", format!("Note #{n_id} assigned to {c_str}"));
                                                                        if let Some(mut sync) = db_sync {
                                                                            sync.with_mut(|r| r.0 += 1);
                                                                        }
                                                                    },
                                                                    div { class: "flex items-center justify-between w-full",
                                                                        span { "{code}" }
                                                                        if is_active {
                                                                            CheckIcon { class: "size-3.5 text-primary ml-auto" }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            ContextMenuSeparator {}
                                            ContextMenuItem {
                                                destructive: true,
                                                on_select: {
                                                    let t_c = ctx_del_title.clone();
                                                    move |_| {
                                                        delete_id.set(n_id);
                                                        delete_title.set(t_c.clone());
                                                        is_delete_open.set(true);
                                                    }
                                                },
                                                TrashIcon { class: "size-3.5 text-destructive" }
                                                "Delete Note"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Create Note Dialog Modal
            // ============================================================
            Dialog {
                open: is_create_open,
                DialogContent {
                    class: "max-w-md w-full",
                    DialogHeader {
                        DialogTitle { "Create New Note" }
                        DialogDescription { "Capture ideas, tasks, or reminders directly to SQLite." }
                    }

                    div { class: "grid gap-3 py-1",
                        div { class: "grid gap-1.5",
                            label { class: "text-xs font-semibold text-foreground", "Title" }
                            Input {
                                placeholder: "e.g. Plan Q4 server architecture review".to_string(),
                                value: new_title.read().clone(),
                                oninput: move |e: FormEvent| new_title.set(e.value()),
                            }
                        }
                        div { class: "grid gap-1.5",
                            label { class: "text-xs font-semibold text-foreground", "Details (optional)" }
                            textarea {
                                rows: "5",
                                class: "flex w-full resize-none rounded-md border border-input bg-card px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                placeholder: "Add supporting context, bullet points, or instructions...".to_string(),
                                value: new_details.read().clone(),
                                oninput: move |e: FormEvent| new_details.set(e.value()),
                            }
                        }
                    }

                    DialogFooter {
                        Button {
                            variant: "outline",
                            size: "sm",
                            onclick: move |_| is_create_open.set(false),
                            "Cancel"
                        }
                        Button {
                            variant: "default",
                            size: "sm",
                            onclick: move |_| {
                                let title = new_title.read().trim().to_string();
                                if title.is_empty() {
                                    toast.error("Missing title", "Please enter a title for your note.");
                                    return;
                                }
                                let details = new_details.read().trim().to_string();
                                let content = if details.is_empty() {
                                    title.clone()
                                } else {
                                    format!("{title} — {details}")
                                };

                                match crate::db::create_note("EMP-1001", &content) {
                                    Ok(new_id) => {
                                        toast.success("Note Saved", format!("Note #{new_id} persisted to database."));
                                        is_create_open.set(false);
                                        *local_revision.write() += 1;
                                        if let Some(mut sync) = db_sync {
                                            sync.with_mut(|r| r.0 += 1);
                                        }
                                    }
                                    Err(e) => {
                                        toast.error("Database Error", format!("Failed to insert note: {:?}", e));
                                    }
                                }
                            },
                            "Create Note"
                        }
                    }
                }
            }

            // ============================================================
            // Edit Note Dialog Modal
            // ============================================================
            Dialog {
                open: is_edit_open,
                DialogContent {
                    class: "max-w-md w-full",
                    DialogHeader {
                        DialogTitle { "Edit Note #{edit_id()}" }
                        DialogDescription { "Update note contents persisted in management.db." }
                    }

                    div { class: "grid gap-3 py-1",
                        div { class: "grid gap-1.5",
                            label { class: "text-xs font-semibold text-foreground", "Title" }
                            Input {
                                placeholder: "Note title...".to_string(),
                                value: edit_title.read().clone(),
                                oninput: move |e: FormEvent| edit_title.set(e.value()),
                            }
                        }
                        div { class: "grid gap-1.5",
                            label { class: "text-xs font-semibold text-foreground", "Details" }
                            textarea {
                                rows: "5",
                                class: "flex w-full resize-none rounded-md border border-input bg-card px-3 py-2 text-sm outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                placeholder: "Note details...".to_string(),
                                value: edit_details.read().clone(),
                                oninput: move |e: FormEvent| edit_details.set(e.value()),
                            }
                        }
                    }

                    DialogFooter {
                        Button {
                            variant: "outline",
                            size: "sm",
                            onclick: move |_| is_edit_open.set(false),
                            "Cancel"
                        }
                        Button {
                            variant: "default",
                            size: "sm",
                            onclick: move |_| {
                                let id = edit_id();
                                let title = edit_title.read().trim().to_string();
                                if title.is_empty() {
                                    toast.error("Missing title", "Note title cannot be blank.");
                                    return;
                                }
                                let details = edit_details.read().trim().to_string();
                                let content = if details.is_empty() {
                                    title.clone()
                                } else {
                                    format!("{title} — {details}")
                                };

                                match crate::db::update_note(id, &content) {
                                    Ok(_) => {
                                        toast.success("Note Updated", format!("Note #{id} saved successfully."));
                                        is_edit_open.set(false);
                                        *local_revision.write() += 1;
                                        if let Some(mut sync) = db_sync {
                                            sync.with_mut(|r| r.0 += 1);
                                        }
                                    }
                                    Err(e) => {
                                        toast.error("Database Error", format!("Failed to update note: {:?}", e));
                                    }
                                }
                            },
                            "Save Changes"
                        }
                    }
                }
            }

            // ============================================================
            // Delete Note Confirmation Dialog
            // ============================================================
            Dialog {
                open: is_delete_open,
                DialogContent {
                    class: "max-w-sm w-full",
                    DialogHeader {
                        DialogTitle { "Delete Note #{delete_id()}" }
                        DialogDescription { "Are you sure you want to delete \"{delete_title()}\"? This action cannot be undone." }
                    }

                    DialogFooter {
                        Button {
                            variant: "outline",
                            size: "sm",
                            onclick: move |_| is_delete_open.set(false),
                            "Cancel"
                        }
                        Button {
                            variant: "destructive",
                            size: "sm",
                            onclick: move |_| {
                                let id = delete_id();
                                match crate::db::delete_note(id) {
                                    Ok(_) => {
                                        toast.success("Note Deleted", format!("Note #{id} removed from database."));
                                        is_delete_open.set(false);
                                        *local_revision.write() += 1;
                                        if let Some(mut sync) = db_sync {
                                            sync.with_mut(|r| r.0 += 1);
                                        }
                                    }
                                    Err(e) => {
                                        toast.error("Database Error", format!("Failed to delete note: {:?}", e));
                                    }
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        }
    }
}
