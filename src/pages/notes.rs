//! Notes workspace — interactive, feature-rich note-taking environment
//! with categories, color coding, pin-to-top, search, sorting, multi-view modes
//! (Grid vs Compact List), clipboard export, and persistent SQLite storage.

use dioxus::prelude::*;

use crate::components::ui::badge::Badge;
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
    CheckIcon, CopyIcon, DownloadIcon, FileTextIcon, GridIcon, ListIcon, PencilIcon, PinIcon,
    PinOffIcon, PlusIcon, SearchIcon, StickyNoteIcon, TagIcon, TrashIcon,
};
use crate::state::AppDbRevision;

// ============================================================
// Constants & Color Helpers
// ============================================================

pub const NOTE_CATEGORIES: &[&str] = &[
    "General",
    "Task",
    "Idea",
    "Meeting",
    "Bug",
    "Doc",
    "Architecture",
    "Urgent",
];

pub const NOTE_COLORS: &[(&str, &str, &str)] = &[
    ("default", "Default", "bg-card border-border"),
    ("amber", "Amber", "bg-amber-500/10 border-amber-500/50 text-amber-500"),
    ("emerald", "Emerald", "bg-emerald-500/10 border-emerald-500/50 text-emerald-500"),
    ("blue", "Blue", "bg-blue-500/10 border-blue-500/50 text-blue-500"),
    ("purple", "Purple", "bg-purple-500/10 border-purple-500/50 text-purple-500"),
    ("rose", "Rose", "bg-rose-500/10 border-rose-500/50 text-rose-500"),
    ("cyan", "Cyan", "bg-cyan-500/10 border-cyan-500/50 text-cyan-500"),
];

fn category_badge_variant(cat: &str) -> &'static str {
    match cat.to_lowercase().as_str() {
        "task" => "blue",
        "idea" => "amber",
        "meeting" => "purple",
        "bug" => "destructive",
        "doc" => "emerald",
        "architecture" => "indigo",
        "urgent" => "destructive",
        _ => "secondary",
    }
}

fn note_card_color_classes(color: &str) -> &'static str {
    match color.to_lowercase().as_str() {
        "amber" => "border-amber-500/40 bg-amber-500/[0.04] dark:bg-amber-500/[0.08] hover:border-amber-500/70",
        "emerald" => "border-emerald-500/40 bg-emerald-500/[0.04] dark:bg-emerald-500/[0.08] hover:border-emerald-500/70",
        "blue" => "border-blue-500/40 bg-blue-500/[0.04] dark:bg-blue-500/[0.08] hover:border-blue-500/70",
        "purple" => "border-purple-500/40 bg-purple-500/[0.04] dark:bg-purple-500/[0.08] hover:border-purple-500/70",
        "rose" => "border-rose-500/40 bg-rose-500/[0.04] dark:bg-rose-500/[0.08] hover:border-rose-500/70",
        "cyan" => "border-cyan-500/40 bg-cyan-500/[0.04] dark:bg-cyan-500/[0.08] hover:border-cyan-500/70",
        _ => "border-border/80 bg-card hover:border-foreground/30",
    }
}

fn note_stripe_color(color: &str) -> &'static str {
    match color.to_lowercase().as_str() {
        "amber" => "bg-amber-500",
        "emerald" => "bg-emerald-500",
        "blue" => "bg-blue-500",
        "purple" => "bg-purple-500",
        "rose" => "bg-rose-500",
        "cyan" => "bg-cyan-500",
        _ => "bg-primary/40",
    }
}

fn copy_to_clipboard(text: &str) {
    let escaped = text
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "");
    let js = format!("navigator.clipboard.writeText('{escaped}');");
    let _ = dioxus::prelude::document::eval(&js);
}

// ============================================================
// Model
// ============================================================

/// Parsed representation of a note from `NOTES` table.
#[derive(Clone, PartialEq, Debug)]
pub struct NoteItem {
    pub id: i64,
    pub emp_code: String,
    pub title: String,
    pub details: String,
    pub raw_content: String,
    pub created_at: String,
    pub updated_at: String,
    pub is_pinned: bool,
    pub category: String,
    pub color: String,
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
            created_at: n.created_at,
            updated_at: n.updated_at,
            is_pinned: n.is_pinned,
            category: if n.category.is_empty() {
                "General".to_string()
            } else {
                n.category
            },
            color: n.color,
        }
    }

    pub fn full_export_text(&self) -> String {
        if self.details.is_empty() {
            format!("### [{}] {}\nCreated: {}\n", self.category, self.title, self.created_at)
        } else {
            format!(
                "### [{}] {}\nCreated: {}\nAuthor: {}\n\n{}\n",
                self.category, self.title, self.created_at, self.emp_code, self.details
            )
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortOption {
    Newest,
    Oldest,
    RecentlyUpdated,
    TitleAZ,
    TitleZA,
}

// ============================================================
// Modal & Action State Handler
// ============================================================

#[derive(Clone, Copy)]
pub struct NotesModalState {
    pub is_view_open: Signal<bool>,
    pub view_note: Signal<Option<NoteItem>>,
    pub is_edit_open: Signal<bool>,
    pub edit_id: Signal<i64>,
    pub edit_title: Signal<String>,
    pub edit_details: Signal<String>,
    pub edit_category: Signal<String>,
    pub edit_color: Signal<String>,
    pub edit_is_pinned: Signal<bool>,
    pub is_delete_open: Signal<bool>,
    pub delete_id: Signal<i64>,
    pub delete_title: Signal<String>,
    pub local_revision: Signal<u64>,
    pub db_sync: Option<Signal<AppDbRevision>>,
}

impl NotesModalState {
    pub fn open_view(&mut self, note: NoteItem) {
        self.view_note.set(Some(note));
        self.is_view_open.set(true);
    }
    pub fn open_edit(&mut self, note: &NoteItem) {
        self.edit_id.set(note.id);
        self.edit_title.set(note.title.clone());
        self.edit_details.set(note.details.clone());
        self.edit_category.set(note.category.clone());
        self.edit_color.set(note.color.clone());
        self.edit_is_pinned.set(note.is_pinned);
        self.is_edit_open.set(true);
    }
    pub fn open_delete(&mut self, id: i64, title: &str) {
        self.delete_id.set(id);
        self.delete_title.set(title.to_string());
        self.is_delete_open.set(true);
    }
    pub fn trigger_sync(&mut self) {
        *self.local_revision.write() += 1;
        if let Some(mut sync) = self.db_sync {
            sync.with_mut(|r| r.0 += 1);
        }
    }
}

// ============================================================
// Component
// ============================================================

#[component]
pub fn NotesPage() -> Element {
    let toast = use_context::<ToastState>();
    let db_sync = try_use_context::<Signal<AppDbRevision>>();

    // Reactive notes state
    let local_revision = use_signal(|| 0u64);
    let mut search_query = use_signal(String::new);
    let mut selected_category = use_signal(|| "All".to_string());
    let mut selected_author = use_signal(|| "All".to_string());
    let mut view_mode = use_signal(|| "grid".to_string()); // "grid" or "list"
    let mut sort_option = use_signal(|| SortOption::Newest);

    // View note details modal state
    let mut is_view_open = use_signal(|| false);
    let mut view_note = use_signal(|| None::<NoteItem>);

    // Create note modal state
    let mut is_create_open = use_signal(|| false);
    let mut new_title = use_signal(String::new);
    let mut new_details = use_signal(String::new);
    let mut new_category = use_signal(|| "General".to_string());
    let mut new_color = use_signal(|| "default".to_string());
    let mut new_is_pinned = use_signal(|| false);

    // Edit note modal state
    let mut is_edit_open = use_signal(|| false);
    let edit_id = use_signal(|| 0i64);
    let mut edit_title = use_signal(String::new);
    let mut edit_details = use_signal(String::new);
    let mut edit_category = use_signal(|| "General".to_string());
    let mut edit_color = use_signal(|| "default".to_string());
    let mut edit_is_pinned = use_signal(|| false);

    // Delete note modal state
    let mut is_delete_open = use_signal(|| false);
    let delete_id = use_signal(|| 0i64);
    let delete_title = use_signal(String::new);

    let mut modals = NotesModalState {
        is_view_open,
        view_note,
        is_edit_open,
        edit_id,
        edit_title,
        edit_details,
        edit_category,
        edit_color,
        edit_is_pinned,
        is_delete_open,
        delete_id,
        delete_title,
        local_revision,
        db_sync,
    };

    // Read all notes from database memoized
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

    let all_employees_memo = use_memo(move || {
        let _ = local_revision();
        if let Some(sync) = db_sync {
            let _ = sync();
        }
        crate::db::get_all_employees()
    });
    let all_employees = all_employees_memo();

    let emp_name_map = use_memo(move || {
        let emps = all_employees_memo();
        let mut map = std::collections::HashMap::new();
        for e in emps.iter() {
            if !e.emp_code.is_empty() {
                map.insert(e.emp_code.trim().to_string(), e.emp_name.trim().to_string());
            }
        }
        map
    });

    let total_count = notes_data.len();
    let pinned_count = notes_data.iter().filter(|n| n.is_pinned).count();

    // Unique categories present in data
    let mut present_categories: Vec<String> = NOTE_CATEGORIES.iter().map(|s| s.to_string()).collect();
    for n in &notes_data {
        if !present_categories.contains(&n.category) && !n.category.is_empty() {
            present_categories.push(n.category.clone());
        }
    }

    // Filter by search query, category, and author
    let filtered_notes = use_memo(move || {
        let notes_list = notes_memo();
        let q = search_query.read().trim().to_lowercase();
        let cat = selected_category.read().clone();
        let author = selected_author.read().clone();
        let sort = *sort_option.read();

        let mut list: Vec<NoteItem> = notes_list
            .into_iter()
            .filter(|n| {
                // Category filter
                if cat == "Pinned" {
                    if !n.is_pinned {
                        return false;
                    }
                } else if cat != "All" && !n.category.eq_ignore_ascii_case(&cat) {
                    return false;
                }

                // Author filter
                if author != "All" && n.emp_code != author {
                    return false;
                }

                // Search query
                if !q.is_empty() {
                    let matches_text = n.title.to_lowercase().contains(&q)
                        || n.details.to_lowercase().contains(&q)
                        || n.id.to_string().contains(&q)
                        || n.category.to_lowercase().contains(&q)
                        || crate::db::matches_emp_query(&n.emp_code, "", "", &q);
                    if !matches_text {
                        return false;
                    }
                }

                true
            })
            .collect();

        // Apply sorting
        match sort {
            SortOption::Newest => {
                list.sort_by_key(|a| std::cmp::Reverse(a.id));
            }
            SortOption::Oldest => {
                list.sort_by_key(|a| a.id);
            }
            SortOption::RecentlyUpdated => {
                list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
            }
            SortOption::TitleAZ => {
                list.sort_by_key(|a| a.title.to_lowercase());
            }
            SortOption::TitleZA => {
                list.sort_by_key(|a| std::cmp::Reverse(a.title.to_lowercase()));
            }
        }

        list
    });

    // Partition into Pinned vs Regular (when in All or non-pinned filter)
    let is_pinned_filter = *selected_category.read() == "Pinned";
    let (pinned_notes, other_notes): (Vec<NoteItem>, Vec<NoteItem>) = if is_pinned_filter {
        (filtered_notes.read().clone(), Vec::new())
    } else {
        filtered_notes.read().iter().cloned().partition(|n| n.is_pinned)
    };

    rsx! {
        div { class: "flex flex-1 flex-col h-full overflow-hidden bg-background",
            // ============================================================
            // Top Navigation & Action Header
            // ============================================================
            header { class: "flex flex-col gap-3 border-b border-border/70 bg-card/60 backdrop-blur-md px-4 sm:px-6 py-3.5 shrink-0 shadow-2xs",
                div { class: "flex flex-col lg:flex-row lg:items-center lg:justify-between gap-3 min-w-0",
                    // Left Branding & Metrics
                    div { class: "flex items-center gap-3 min-w-0",
                        div { class: "flex size-10 shrink-0 items-center justify-center rounded-xl bg-primary/10 text-primary border border-primary/20 shadow-2xs",
                            StickyNoteIcon { class: "size-5" }
                        }
                        div { class: "flex flex-col min-w-0",
                            div { class: "flex items-center gap-2",
                                h1 { class: "text-lg font-bold tracking-tight text-foreground shrink-0", "Notes" }
                                span { class: "rounded-full bg-muted border border-border px-2 py-0.5 text-xs font-semibold text-muted-foreground shrink-0",
                                    "{total_count} saved"
                                }
                                if pinned_count > 0 {
                                    span { class: "inline-flex items-center gap-1 rounded-full bg-amber-500/10 border border-amber-500/30 px-2 py-0.5 text-xs font-semibold text-amber-600 dark:text-amber-400 shrink-0",
                                        PinIcon { class: "size-3" }
                                        "{pinned_count} pinned"
                                    }
                                }
                            }
                            p { class: "text-xs text-muted-foreground truncate hidden sm:block",
                                "Fast, categorized workspace notes with pinning, color highlights, and instant clipboard sync."
                            }
                        }
                    }

                    // Search, View Switcher & Action Buttons
                    div { class: "flex flex-wrap items-center gap-2 shrink-0 justify-between sm:justify-end",
                        // Search bar
                        div { class: "relative w-full sm:w-64",
                            SearchIcon { class: "absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" }
                            Input {
                                class: "pl-8 text-xs h-8.5 bg-card border-border/80 focus-visible:ring-1 w-full rounded-lg shadow-2xs",
                                placeholder: "Search notes, tags, authors...",
                                value: search_query.read().clone(),
                                oninput: move |e: FormEvent| search_query.set(e.value()),
                            }
                            if !search_query.read().is_empty() {
                                button {
                                    r#type: "button",
                                    title: "Clear search",
                                    class: "absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground text-xs cursor-pointer p-0.5",
                                    onclick: move |_| search_query.set(String::new()),
                                    "✕"
                                }
                            }
                        }

                        // View mode switcher (Grid / List)
                        div { class: "inline-flex items-center rounded-lg border border-border/70 bg-muted/50 p-0.5 shadow-2xs",
                            button {
                                r#type: "button",
                                title: "Grid view",
                                class: format!(
                                    "p-1.5 rounded-md transition-colors cursor-pointer {}",
                                    if *view_mode.read() == "grid" { "bg-background text-foreground shadow-2xs" } else { "text-muted-foreground hover:text-foreground" }
                                ),
                                onclick: move |_| view_mode.set("grid".to_string()),
                                GridIcon { class: "size-3.5" }
                            }
                            button {
                                r#type: "button",
                                title: "Compact list view",
                                class: format!(
                                    "p-1.5 rounded-md transition-colors cursor-pointer {}",
                                    if *view_mode.read() == "list" { "bg-background text-foreground shadow-2xs" } else { "text-muted-foreground hover:text-foreground" }
                                ),
                                onclick: move |_| view_mode.set("list".to_string()),
                                ListIcon { class: "size-3.5" }
                            }
                        }

                        // Sort selector
                        select {
                            class: "h-8.5 rounded-lg border border-border/80 bg-card px-2.5 text-xs text-foreground font-medium outline-none shadow-2xs cursor-pointer hover:border-foreground/30 transition-colors",
                            value: match *sort_option.read() {
                                SortOption::Newest => "newest",
                                SortOption::Oldest => "oldest",
                                SortOption::RecentlyUpdated => "updated",
                                SortOption::TitleAZ => "az",
                                SortOption::TitleZA => "za",
                            },
                            onchange: move |e: FormEvent| {
                                match e.value().as_str() {
                                    "oldest" => sort_option.set(SortOption::Oldest),
                                    "updated" => sort_option.set(SortOption::RecentlyUpdated),
                                    "az" => sort_option.set(SortOption::TitleAZ),
                                    "za" => sort_option.set(SortOption::TitleZA),
                                    _ => sort_option.set(SortOption::Newest),
                                }
                            },
                            option { value: "newest", "Newest First" }
                            option { value: "oldest", "Oldest First" }
                            option { value: "updated", "Recently Updated" }
                            option { value: "az", "Title (A–Z)" }
                            option { value: "za", "Title (Z–A)" }
                        }

                        // Export button
                        Button {
                            variant: "outline",
                            size: "sm",
                            class: "h-8.5 px-2.5 text-xs gap-1.5 shadow-2xs cursor-pointer font-medium border-border/80",
                            title: "Copy all filtered notes as Markdown",
                            onclick: {
                                let f_notes = filtered_notes.read().clone();
                                move |_| {
                                    if f_notes.is_empty() {
                                        toast.warning("Nothing to export", "No notes match the current filters.");
                                        return;
                                    }
                                    let mut doc = format!("# Exported Notes ({})\n\n", f_notes.len());
                                    for n in &f_notes {
                                        doc.push_str(&n.full_export_text());
                                        doc.push_str("\n---\n\n");
                                    }
                                    copy_to_clipboard(&doc);
                                    toast.success("Exported to Clipboard", format!("{} notes copied as formatted Markdown.", f_notes.len()));
                                }
                            },
                            DownloadIcon { class: "size-3.5" }
                            span { class: "hidden sm:inline", "Export" }
                        }

                        // New Note button
                        Button {
                            variant: "default",
                            size: "sm",
                            class: "h-8.5 px-3 text-xs gap-1.5 font-semibold shadow-xs cursor-pointer",
                            onclick: move |_| {
                                new_title.set(String::new());
                                new_details.set(String::new());
                                new_category.set("General".to_string());
                                new_color.set("default".to_string());
                                new_is_pinned.set(false);
                                is_create_open.set(true);
                            },
                            PlusIcon { class: "size-3.5" }
                            "New Note"
                        }
                    }
                }

                // ============================================================
                // Category Filter Chips
                // ============================================================
                div { class: "flex items-center gap-1.5 overflow-x-auto pb-1 pt-0.5 no-scrollbar",
                    // All Pill
                    {
                        let is_sel = *selected_category.read() == "All";
                        rsx! {
                            button {
                                r#type: "button",
                                class: format!(
                                    "px-2.5 py-1 rounded-full text-xs font-semibold shrink-0 transition-all cursor-pointer border {}",
                                    if is_sel {
                                        "bg-foreground text-background border-foreground shadow-2xs"
                                    } else {
                                        "bg-card text-muted-foreground border-border/70 hover:text-foreground hover:bg-muted/70"
                                    }
                                ),
                                onclick: move |_| selected_category.set("All".to_string()),
                                "All ({total_count})"
                            }
                        }
                    }

                    // Pinned Pill
                    if pinned_count > 0 {
                        {
                            let is_sel = *selected_category.read() == "Pinned";
                            rsx! {
                                button {
                                    r#type: "button",
                                    class: format!(
                                        "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-semibold shrink-0 transition-all cursor-pointer border {}",
                                        if is_sel {
                                            "bg-amber-500 text-white border-amber-600 shadow-2xs"
                                        } else {
                                            "bg-amber-500/10 text-amber-600 dark:text-amber-400 border-amber-500/30 hover:bg-amber-500/20"
                                        }
                                    ),
                                    onclick: move |_| selected_category.set("Pinned".to_string()),
                                    PinIcon { class: "size-3" }
                                    "Pinned ({pinned_count})"
                                }
                            }
                        }
                    }

                    div { class: "h-4 w-px bg-border/60 mx-1 shrink-0" }

                    // Category Pills
                    for cat in present_categories.iter() {
                        {
                            let c_name = cat.clone();
                            let is_sel = *selected_category.read() == c_name;
                            let count = notes_data.iter().filter(|n| n.category.eq_ignore_ascii_case(&c_name)).count();
                            rsx! {
                                button {
                                    key: "cat-pill-{c_name}",
                                    r#type: "button",
                                    class: format!(
                                        "px-2.5 py-1 rounded-full text-xs font-medium shrink-0 transition-all cursor-pointer border {}",
                                        if is_sel {
                                            "bg-primary text-primary-foreground border-primary shadow-2xs font-semibold"
                                        } else {
                                            "bg-card text-muted-foreground border-border/70 hover:text-foreground hover:bg-muted/60"
                                        }
                                    ),
                                    onclick: move |_| selected_category.set(c_name.clone()),
                                    "{c_name}"
                                    if count > 0 {
                                        span { class: "ml-1 opacity-70 text-[11px]", "({count})" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Main Content Area
            // ============================================================
            main { class: "flex-1 overflow-y-auto p-4 sm:p-6",
                if filtered_notes.read().is_empty() {
                    div { class: "flex flex-col items-center justify-center min-h-[380px] text-center p-8 rounded-2xl border border-dashed border-border/70 bg-card/30",
                        div { class: "flex size-14 items-center justify-center rounded-2xl bg-muted/60 text-muted-foreground mb-3",
                            FileTextIcon { class: "size-7" }
                        }
                        if search_query.read().is_empty() && *selected_category.read() == "All" {
                            h3 { class: "text-base font-semibold text-foreground mb-1", "No notes yet" }
                            p { class: "text-xs text-muted-foreground max-w-sm mb-4",
                                "Create quick notes on the fly using the floating action button (FAB) or the \"New Note\" button above."
                            }
                            Button {
                                variant: "default",
                                size: "sm",
                                onclick: move |_| {
                                    new_title.set(String::new());
                                    new_details.set(String::new());
                                    new_category.set("General".to_string());
                                    new_color.set("default".to_string());
                                    new_is_pinned.set(false);
                                    is_create_open.set(true);
                                },
                                PlusIcon { class: "size-4 mr-1.5" }
                                "Create Note"
                            }
                        } else {
                            h3 { class: "text-base font-semibold text-foreground mb-1", "No matching notes found" }
                            p { class: "text-xs text-muted-foreground max-w-sm mb-4",
                                "No notes matched your current search and filter settings. Try adjusting your query."
                            }
                            Button {
                                variant: "outline",
                                size: "sm",
                                onclick: move |_| {
                                    search_query.set(String::new());
                                    selected_category.set("All".to_string());
                                    selected_author.set("All".to_string());
                                },
                                "Clear All Filters"
                            }
                        }
                    }
                } else if *view_mode.read() == "list" {
                    // ============================================================
                    // Compact List View
                    // ============================================================
                    div { class: "flex flex-col rounded-xl border border-border/80 bg-card divide-y divide-border/60 overflow-hidden shadow-2xs",
                        for note in filtered_notes.read().iter() {
                            {
                                let n = note.clone();
                                let n_id = n.id;
                                let is_p = n.is_pinned;
                                let title_str = n.title.clone();
                                let details_str = n.details.clone();
                                let cat_str = n.category.clone();
                                let color_stripe = note_stripe_color(&n.color);
                                let emp_code_str = n.emp_code.clone();
                                let emp_display = emp_name_map
                                    .read()
                                    .get(&emp_code_str)
                                    .cloned()
                                    .unwrap_or_else(|| emp_code_str.clone());

                                rsx! {
                                    div {
                                        key: "list-note-{n_id}",
                                        class: "group flex items-center justify-between gap-3 px-4 py-3 hover:bg-muted/40 transition-colors cursor-pointer relative",
                                        onclick: {
                                            let note_to_view = n.clone();
                                            move |_| modals.open_view(note_to_view.clone())
                                        },

                                        // Left color indicator
                                        div { class: "absolute left-0 top-0 bottom-0 w-1 {color_stripe}" }

                                        // Left info
                                        div { class: "flex items-center gap-3 min-w-0 flex-1 pl-1",
                                            // Pin Toggle Button
                                            button {
                                                r#type: "button",
                                                title: if is_p { "Unpin note" } else { "Pin to top" },
                                                class: format!(
                                                    "p-1 rounded hover:bg-muted cursor-pointer transition-colors {}",
                                                    if is_p { "text-amber-500 font-bold" } else { "text-muted-foreground/40 hover:text-muted-foreground" }
                                                ),
                                                onclick: move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    let _ = crate::db::toggle_note_pin(n_id);
                                                    modals.trigger_sync();
                                                },
                                                PinIcon { class: "size-3.5" }
                                            }

                                            span { class: "font-mono text-xs font-bold text-muted-foreground bg-muted/60 px-2 py-0.5 rounded border border-border/70 shrink-0",
                                                "#{n_id}"
                                            }

                                            Badge {
                                                variant: "{category_badge_variant(&cat_str)}",
                                                class: "text-[10px] font-semibold px-2 py-0.5 shrink-0",
                                                "{cat_str}"
                                            }

                                            div { class: "flex flex-col min-w-0 flex-1",
                                                span { class: "text-xs font-bold text-foreground truncate group-hover:text-primary transition-colors",
                                                    "{title_str}"
                                                }
                                                if !details_str.is_empty() {
                                                    span { class: "text-[11px] text-muted-foreground truncate",
                                                        "{details_str}"
                                                    }
                                                }
                                            }
                                        }

                                        // Right metadata & actions
                                        div { class: "flex items-center gap-3 shrink-0",
                                            if !emp_display.is_empty() {
                                                span { class: "text-[11px] text-muted-foreground bg-muted/40 px-2 py-0.5 rounded border border-border/50 hidden md:inline truncate max-w-[120px]",
                                                    "{emp_display}"
                                                }
                                            }

                                            span { class: "text-[11px] text-muted-foreground hidden sm:inline",
                                                "{crate::state::arrange_date_time(&n.created_at)}"
                                            }

                                            div { class: "flex items-center gap-1 opacity-80 group-hover:opacity-100 transition-opacity",
                                                // Copy button
                                                button {
                                                    r#type: "button",
                                                    title: "Copy text",
                                                    class: "size-7 inline-flex items-center justify-center rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                                    onclick: {
                                                        let full_text = n.raw_content.clone();
                                                        let t_title = n.title.clone();
                                                        move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            copy_to_clipboard(&full_text);
                                                            toast.success("Note Copied", format!("\"{}\" copied to clipboard.", t_title));
                                                        }
                                                    },
                                                    CopyIcon { class: "size-3.5" }
                                                }
                                                // Edit button
                                                button {
                                                    r#type: "button",
                                                    title: "Edit note",
                                                    class: "size-7 inline-flex items-center justify-center rounded hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                                    onclick: {
                                                        let item = n.clone();
                                                        move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            modals.open_edit(&item);
                                                        }
                                                    },
                                                    PencilIcon { class: "size-3.5" }
                                                }
                                                // Delete button
                                                button {
                                                    r#type: "button",
                                                    title: "Delete note",
                                                    class: "size-7 inline-flex items-center justify-center rounded hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer",
                                                    onclick: {
                                                        let item = n.clone();
                                                        move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            modals.open_delete(item.id, &item.title);
                                                        }
                                                    },
                                                    TrashIcon { class: "size-3.5" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // ============================================================
                    // Visual Card Grid View (with Pinned Section)
                    // ============================================================
                    div { class: "flex flex-col gap-6",
                        // Section 1: Pinned Notes
                        if !pinned_notes.is_empty() && !is_pinned_filter {
                            div { class: "flex flex-col gap-3",
                                div { class: "flex items-center gap-2",
                                    PinIcon { class: "size-4 text-amber-500" }
                                    h2 { class: "text-xs font-bold tracking-wider text-muted-foreground uppercase",
                                        "Pinned Notes ({pinned_notes.len()})"
                                    }
                                }
                                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4",
                                    for note in pinned_notes.iter() {
                                        {
                                            render_note_card(
                                                note.clone(),
                                                &all_employees,
                                                toast,
                                                modals,
                                            )
                                        }
                                    }
                                }
                            }
                        }

                        // Section 2: Other Notes / All Notes
                        if !other_notes.is_empty() || is_pinned_filter {
                            div { class: "flex flex-col gap-3",
                                if !pinned_notes.is_empty() && !is_pinned_filter {
                                    div { class: "flex items-center gap-2 pt-2",
                                        StickyNoteIcon { class: "size-4 text-muted-foreground" }
                                        h2 { class: "text-xs font-bold tracking-wider text-muted-foreground uppercase",
                                            "All Notes ({other_notes.len()})"
                                        }
                                    }
                                }
                                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4",
                                    for note in (if is_pinned_filter { pinned_notes.iter() } else { other_notes.iter() }) {
                                        {
                                            render_note_card(
                                                note.clone(),
                                                &all_employees,
                                                toast,
                                                modals,
                                            )
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Dialog: View Note Full Details Modal
            // ============================================================
            if is_view_open() {
                if let Some(n) = view_note.read().clone() {
                    Dialog {
                        open: is_view_open,
                        DialogContent {
                            class: "max-w-xl w-full",
                            DialogHeader {
                                div { class: "flex flex-wrap items-center justify-between gap-2 border-b border-border/60 pb-3",
                                    div { class: "flex items-center gap-2",
                                        span { class: "font-mono text-xs font-bold text-foreground bg-muted px-2.5 py-0.5 rounded border border-border",
                                            "#{n.id}"
                                        }
                                        Badge {
                                            variant: "{category_badge_variant(&n.category)}",
                                            class: "text-[11px] font-semibold px-2 py-0.5",
                                            "{n.category}"
                                        }
                                        if n.is_pinned {
                                            span { class: "inline-flex items-center gap-1 rounded bg-amber-500/15 border border-amber-500/40 px-2 py-0.5 text-[10px] font-bold text-amber-600 dark:text-amber-400",
                                                PinIcon { class: "size-3" }
                                                "Pinned"
                                            }
                                        }
                                    }

                                    div { class: "flex items-center gap-2 text-xs text-muted-foreground",
                                        span { "Created: {crate::state::arrange_date_time(&n.created_at)}" }
                                    }
                                }

                                DialogTitle { class: "text-lg font-bold text-foreground mt-3 leading-snug",
                                    "{n.title}"
                                }
                                if !n.emp_code.is_empty() {
                                    {
                                        let emp_name = emp_name_map
                                            .read()
                                            .get(&n.emp_code)
                                            .cloned()
                                            .unwrap_or_else(|| n.emp_code.clone());
                                        rsx! {
                                            DialogDescription { class: "text-xs text-muted-foreground mt-0.5",
                                                "Author: {emp_name} ({n.emp_code})"
                                            }
                                        }
                                    }
                                }
                            }

                            div { class: "flex flex-col gap-4 py-2",
                                if !n.details.is_empty() {
                                    div { class: "rounded-lg border border-border/60 bg-muted/20 p-4 text-sm text-foreground whitespace-pre-wrap leading-relaxed max-h-[360px] overflow-y-auto select-text font-normal",
                                        "{n.details}"
                                    }
                                } else {
                                    p { class: "text-xs italic text-muted-foreground", "No additional details provided." }
                                }

                                div { class: "flex items-center justify-between text-[11px] text-muted-foreground pt-1 border-t border-border/40",
                                    span { "{n.title.split_whitespace().count() + n.details.split_whitespace().count()} words · {n.title.len() + n.details.len()} characters" }
                                    if !n.updated_at.is_empty() {
                                        span { "Updated: {crate::state::arrange_date_time(&n.updated_at)}" }
                                    }
                                }
                            }

                            DialogFooter {
                                Button {
                                    variant: "outline",
                                    size: "sm",
                                    onclick: {
                                        let c_note = n.clone();
                                        move |_| {
                                            copy_to_clipboard(&c_note.raw_content);
                                            toast.success("Note Copied", format!("\"{}\" copied to clipboard.", c_note.title));
                                        }
                                    },
                                    CopyIcon { class: "size-3.5 mr-1.5" }
                                    "Copy Text"
                                }
                                Button {
                                    variant: "outline",
                                    size: "sm",
                                    onclick: {
                                        let c_note = n.clone();
                                        move |_| {
                                            let _ = crate::db::toggle_note_pin(c_note.id);
                                            modals.trigger_sync();
                                            modals.is_view_open.set(false);
                                            modals.view_note.set(None);
                                        }
                                    },
                                    PinIcon { class: "size-3.5 mr-1.5" }
                                    if n.is_pinned { "Unpin" } else { "Pin to Top" }
                                }
                                Button {
                                    variant: "default",
                                    size: "sm",
                                    onclick: {
                                        let c_note = n.clone();
                                        move |_| {
                                            modals.is_view_open.set(false);
                                            modals.view_note.set(None);
                                            modals.open_edit(&c_note);
                                        }
                                    },
                                    PencilIcon { class: "size-3.5 mr-1.5" }
                                    "Edit"
                                }
                                Button {
                                    variant: "outline",
                                    size: "sm",
                                    onclick: move |_| {
                                        is_view_open.set(false);
                                        view_note.set(None);
                                    },
                                    "Close"
                                }
                            }
                        }
                    }
                }
            }

            // ============================================================
            // Dialog: Create New Note
            // ============================================================
            Dialog {
                open: is_create_open,
                DialogContent {
                    class: "max-w-lg w-full",
                    DialogHeader {
                        DialogTitle { "Create New Note" }
                        DialogDescription { "Capture ideas, tasks, architecture decisions, or quick reminders directly in SQLite." }
                    }

                    div { class: "flex flex-col gap-3.5 py-1 text-xs",
                        // Title
                        div { class: "flex flex-col gap-1.5",
                            label { class: "font-semibold text-foreground", "Title *" }
                            Input {
                                placeholder: "e.g. Upgrade ERP PostgreSQL replication node".to_string(),
                                value: new_title.read().clone(),
                                oninput: move |e: FormEvent| new_title.set(e.value()),
                            }
                        }

                        // Category & Color Selection Row
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                            div { class: "flex flex-col gap-1.5",
                                label { class: "font-semibold text-foreground", "Category" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-3 text-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                    value: new_category.read().clone(),
                                    onchange: move |e: FormEvent| new_category.set(e.value()),
                                    for cat in NOTE_CATEGORIES.iter() {
                                        option { value: "{cat}", "{cat}" }
                                    }
                                }
                            }

                            div { class: "flex flex-col gap-1.5",
                                label { class: "font-semibold text-foreground", "Color Accent" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-3 text-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                    value: new_color.read().clone(),
                                    onchange: move |e: FormEvent| new_color.set(e.value()),
                                    for (val, label, _) in NOTE_COLORS.iter() {
                                        option { value: "{val}", "{label}" }
                                    }
                                }
                            }
                        }

                        // Details
                        div { class: "flex flex-col gap-1.5",
                            div { class: "flex items-center justify-between",
                                label { class: "font-semibold text-foreground", "Details (Markdown / Text)" }
                                span { class: "text-[11px] text-muted-foreground",
                                    "{new_details.read().len()} chars"
                                }
                            }
                            textarea {
                                rows: "6",
                                class: "flex w-full resize-none rounded-md border border-input bg-card px-3 py-2 text-xs outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50 leading-relaxed font-mono",
                                placeholder: "Add supporting context, bullet points, checklists, or commands...".to_string(),
                                value: new_details.read().clone(),
                                oninput: move |e: FormEvent| new_details.set(e.value()),
                            }
                        }

                        // Pin Checkbox
                        label { class: "flex items-center gap-2 cursor-pointer select-none pt-1",
                            input {
                                r#type: "checkbox",
                                class: "size-4 rounded border-border accent-primary cursor-pointer",
                                checked: *new_is_pinned.read(),
                                onchange: move |e: FormEvent| new_is_pinned.set(e.checked()),
                            }
                            span { class: "text-xs font-medium text-foreground", "Pin note to top of workspace" }
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
                                let cat = new_category.read().clone();
                                let col = new_color.read().clone();
                                let pin = *new_is_pinned.read();

                                match crate::db::create_note_full(&crate::db::configured_employee_code(), &content, &cat, &col, pin) {
                                    Ok(new_id) => {
                                        toast.success("Note Saved", format!("Note #{new_id} persisted to database."));
                                        is_create_open.set(false);
                                        modals.trigger_sync();
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
            // Dialog: Edit Note Modal
            // ============================================================
            Dialog {
                open: is_edit_open,
                DialogContent {
                    class: "max-w-lg w-full",
                    DialogHeader {
                        DialogTitle { "Edit Note #{edit_id()}" }
                        DialogDescription { "Update note contents, tags, and display color in SQLite." }
                    }

                    div { class: "flex flex-col gap-3.5 py-1 text-xs",
                        // Title
                        div { class: "flex flex-col gap-1.5",
                            label { class: "font-semibold text-foreground", "Title *" }
                            Input {
                                placeholder: "Note title...".to_string(),
                                value: edit_title.read().clone(),
                                oninput: move |e: FormEvent| edit_title.set(e.value()),
                            }
                        }

                        // Category & Color Selection Row
                        div { class: "grid grid-cols-1 sm:grid-cols-2 gap-3",
                            div { class: "flex flex-col gap-1.5",
                                label { class: "font-semibold text-foreground", "Category" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-3 text-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                    value: edit_category.read().clone(),
                                    onchange: move |e: FormEvent| edit_category.set(e.value()),
                                    for cat in NOTE_CATEGORIES.iter() {
                                        option { value: "{cat}", "{cat}" }
                                    }
                                }
                            }

                            div { class: "flex flex-col gap-1.5",
                                label { class: "font-semibold text-foreground", "Color Accent" }
                                select {
                                    class: "h-9 rounded-md border border-input bg-card px-3 text-xs outline-none transition-colors focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                                    value: edit_color.read().clone(),
                                    onchange: move |e: FormEvent| edit_color.set(e.value()),
                                    for (val, label, _) in NOTE_COLORS.iter() {
                                        option { value: "{val}", "{label}" }
                                    }
                                }
                            }
                        }

                        // Details
                        div { class: "flex flex-col gap-1.5",
                            div { class: "flex items-center justify-between",
                                label { class: "font-semibold text-foreground", "Details" }
                                span { class: "text-[11px] text-muted-foreground",
                                    "{edit_details.read().len()} chars"
                                }
                            }
                            textarea {
                                rows: "6",
                                class: "flex w-full resize-none rounded-md border border-input bg-card px-3 py-2 text-xs outline-none transition-colors placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50 leading-relaxed font-mono",
                                placeholder: "Note details...".to_string(),
                                value: edit_details.read().clone(),
                                oninput: move |e: FormEvent| edit_details.set(e.value()),
                            }
                        }

                        // Pin Checkbox
                        label { class: "flex items-center gap-2 cursor-pointer select-none pt-1",
                            input {
                                r#type: "checkbox",
                                class: "size-4 rounded border-border accent-primary cursor-pointer",
                                checked: *edit_is_pinned.read(),
                                onchange: move |e: FormEvent| edit_is_pinned.set(e.checked()),
                            }
                            span { class: "text-xs font-medium text-foreground", "Pin note to top of workspace" }
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
                                let cat = edit_category.read().clone();
                                let col = edit_color.read().clone();
                                let pin = *edit_is_pinned.read();

                                match crate::db::update_note_full(id, &content, &cat, &col, pin) {
                                    Ok(_) => {
                                        toast.success("Note Updated", format!("Note #{id} saved successfully."));
                                        is_edit_open.set(false);
                                        modals.trigger_sync();
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
            // Dialog: Delete Note Confirmation
            // ============================================================
            Dialog {
                open: is_delete_open,
                DialogContent {
                    class: "max-w-sm w-full",
                    DialogHeader {
                        DialogTitle { "Delete Note #{delete_id()}" }
                        DialogDescription { "Are you sure you want to permanently delete \"{delete_title()}\"? This action cannot be undone." }
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
                                        modals.trigger_sync();
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

// ============================================================
// Helper Component: Note Card (Grid View)
// ============================================================

fn render_note_card(
    note: NoteItem,
    all_employees: &[crate::db::DbEmployee],
    toast: ToastState,
    mut modals: NotesModalState,
) -> Element {
    let n_id = note.id;
    let is_p = note.is_pinned;
    let title_str = note.title.clone();
    let details_str = note.details.clone();
    let cat_str = note.category.clone();
    let color_classes = note_card_color_classes(&note.color);
    let color_stripe = note_stripe_color(&note.color);
    let emp_code_str = note.emp_code.clone();
    let emp_display_name = all_employees
        .iter()
        .find(|e| e.emp_code.eq_ignore_ascii_case(&emp_code_str))
        .map(|e| e.emp_name.clone())
        .unwrap_or_else(|| emp_code_str.clone());

    let note_for_click = note.clone();
    let note_for_copy = note.clone();
    let note_for_edit = note.clone();
    let note_for_del = note.clone();

    rsx! {
        ContextMenu {
            key: "note-ctx-{n_id}",
            ContextMenuTrigger {
                class: "contents",
                div {
                    key: "note-{n_id}",
                    class: "group relative flex flex-col justify-between rounded-xl border p-4 sm:p-5 shadow-xs hover:shadow-md transition-all duration-200 cursor-pointer overflow-hidden {color_classes}",
                    onclick: move |_| modals.open_view(note_for_click.clone()),

                    // Top Left Accent Line
                    div { class: "absolute left-0 top-0 bottom-0 w-1 {color_stripe}" }

                    // Card Header
                    div { class: "flex items-start justify-between gap-2 mb-2.5 pl-1",
                        div { class: "flex flex-wrap items-center gap-1.5",
                            span { class: "font-mono text-[10px] font-bold text-muted-foreground bg-muted/60 px-2 py-0.5 rounded border border-border/70",
                                "#{n_id}"
                            }
                            Badge {
                                variant: "{category_badge_variant(&cat_str)}",
                                class: "text-[10px] font-semibold px-2 py-0.5",
                                "{cat_str}"
                            }
                            if !emp_display_name.is_empty() {
                                span { class: "text-[10px] font-semibold text-muted-foreground bg-muted/40 px-2 py-0.5 rounded border border-border/50 truncate max-w-[120px]",
                                    "{emp_display_name}"
                                }
                            }
                        }

                        // Quick action buttons
                        div { class: "flex items-center gap-0.5 opacity-70 group-hover:opacity-100 transition-opacity",
                            // 1-Click Pin Toggle Button
                            button {
                                r#type: "button",
                                title: if is_p { "Unpin note" } else { "Pin to top" },
                                class: format!(
                                    "flex size-7 items-center justify-center rounded-md hover:bg-muted cursor-pointer transition-colors {}",
                                    if is_p { "text-amber-500 font-bold" } else { "text-muted-foreground hover:text-foreground" }
                                ),
                                onclick: move |e: MouseEvent| {
                                    e.stop_propagation();
                                    let _ = crate::db::toggle_note_pin(n_id);
                                    modals.trigger_sync();
                                },
                                if is_p {
                                    PinIcon { class: "size-3.5 fill-amber-500 text-amber-500" }
                                } else {
                                    PinIcon { class: "size-3.5" }
                                }
                            }

                            // Copy Button
                            button {
                                r#type: "button",
                                title: "Copy note text",
                                class: "flex size-7 items-center justify-center rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                onclick: {
                                    let c_note = note_for_copy.clone();
                                    move |e: MouseEvent| {
                                        e.stop_propagation();
                                        copy_to_clipboard(&c_note.raw_content);
                                        toast.success("Note Copied", format!("\"{}\" copied to clipboard.", c_note.title));
                                    }
                                },
                                CopyIcon { class: "size-3.5" }
                            }

                            // Edit Button
                            button {
                                r#type: "button",
                                title: "Edit note",
                                class: "flex size-7 items-center justify-center rounded-md hover:bg-muted text-muted-foreground hover:text-foreground transition-colors cursor-pointer",
                                onclick: {
                                    let e_note = note_for_edit.clone();
                                    move |e: MouseEvent| {
                                        e.stop_propagation();
                                        modals.open_edit(&e_note);
                                    }
                                },
                                PencilIcon { class: "size-3.5" }
                            }

                            // Delete Button
                            button {
                                r#type: "button",
                                title: "Delete note",
                                class: "flex size-7 items-center justify-center rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors cursor-pointer",
                                onclick: {
                                    let d_note = note_for_del.clone();
                                    move |e: MouseEvent| {
                                        e.stop_propagation();
                                        modals.open_delete(d_note.id, &d_note.title);
                                    }
                                },
                                TrashIcon { class: "size-3.5" }
                            }
                        }
                    }

                    // Card Body
                    div { class: "flex-1 mb-3 pl-1",
                        h3 { class: "text-sm font-bold text-foreground leading-snug line-clamp-2 mb-1.5 group-hover:text-primary transition-colors",
                            "{title_str}"
                        }
                        if !details_str.is_empty() {
                            p { class: "text-xs text-muted-foreground line-clamp-4 whitespace-pre-wrap leading-relaxed",
                                "{details_str}"
                            }
                        }
                    }

                    // Card Footer
                    div { class: "flex items-center justify-between border-t border-border/50 pt-2.5 text-[10px] text-muted-foreground pl-1",
                        span { title: "{note.created_at}",
                            "{crate::state::arrange_date_time(&note.created_at)}"
                        }
                        span { "{title_str.len() + details_str.len()} chars" }
                    }
                }
            }

            // Right-Click Context Menu
            ContextMenuContent {
                class: "w-48 rounded-md shadow-xl border-border/80 bg-popover/95 backdrop-blur-md",
                ContextMenuLabel {
                    div { class: "flex items-center justify-between gap-2",
                        span { class: "font-mono font-bold text-foreground text-xs", "#{n_id}" }
                        Badge {
                            variant: "{category_badge_variant(&cat_str)}",
                            class: "text-[9px] font-semibold px-1.5 py-0.2",
                            "{cat_str}"
                        }
                    }
                    span { class: "text-[11px] text-muted-foreground truncate block font-normal mt-0.5", "{title_str}" }
                }
                ContextMenuSeparator {}

                // View Details
                ContextMenuItem {
                    on_select: {
                        let n_v = note.clone();
                        move |_| modals.open_view(n_v.clone())
                    },
                    FileTextIcon { class: "size-3.5 text-muted-foreground" }
                    "View Note"
                }

                // Toggle Pin
                ContextMenuItem {
                    on_select: move |_| {
                        let _ = crate::db::toggle_note_pin(n_id);
                        modals.trigger_sync();
                    },
                    if is_p {
                        PinOffIcon { class: "size-3.5 text-muted-foreground" }
                    } else {
                        PinIcon { class: "size-3.5 text-muted-foreground" }
                    }
                    if is_p { "Unpin Note" } else { "Pin to Top" }
                }

                // Copy
                ContextMenuItem {
                    on_select: {
                        let c_content = note.raw_content.clone();
                        let c_title = note.title.clone();
                        move |_| {
                            copy_to_clipboard(&c_content);
                            toast.success("Note Copied", format!("\"{}\" copied to clipboard.", c_title));
                        }
                    },
                    CopyIcon { class: "size-3.5 text-muted-foreground" }
                    "Copy Note Text"
                }

                // Edit
                ContextMenuItem {
                    on_select: {
                        let n_e = note.clone();
                        move |_| modals.open_edit(&n_e)
                    },
                    PencilIcon { class: "size-3.5 text-muted-foreground" }
                    "Edit Note"
                }

                ContextMenuSeparator {}

                // Change Category Submenu
                ContextMenuSub {
                    ContextMenuSubTrigger {
                        TagIcon { class: "size-3.5 text-muted-foreground mr-1.5" }
                        "Change Category"
                    }
                    ContextMenuSubContent {
                        for c in NOTE_CATEGORIES.iter() {
                            {
                                let cat_val = c.to_string();
                                let is_active = cat_str == cat_val;
                                rsx! {
                                    ContextMenuItem {
                                        key: "ctx-cat-{cat_val}",
                                        on_select: {
                                            let cat_set = cat_val.clone();
                                            move |_| {
                                                let _ = crate::db::update_note_category(n_id, &cat_set);
                                                modals.trigger_sync();
                                            }
                                        },
                                        div { class: "flex items-center justify-between w-full",
                                            span { "{cat_val}" }
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

                // Change Color Accent Submenu
                ContextMenuSub {
                    ContextMenuSubTrigger {
                        "Change Color"
                    }
                    ContextMenuSubContent {
                        for (col_val, col_label, _) in NOTE_COLORS.iter() {
                            {
                                let color_set = col_val.to_string();
                                let is_active = note.color == color_set;
                                rsx! {
                                    ContextMenuItem {
                                        key: "ctx-col-{color_set}",
                                        on_select: {
                                            let col_save = color_set.clone();
                                            move |_| {
                                                let _ = crate::db::update_note_color(n_id, &col_save);
                                                modals.trigger_sync();
                                            }
                                        },
                                        div { class: "flex items-center justify-between w-full gap-2",
                                            span { "{col_label}" }
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

                // Assign Employee Submenu
                ContextMenuSub {
                    ContextMenuSubTrigger {
                        "Assign Employee"
                    }
                    ContextMenuSubContent {
                        for emp in all_employees.iter().take(20) {
                            {
                                let is_active = emp_code_str == emp.emp_code;
                                let c_str = emp.emp_code.clone();
                                let name_str = emp.emp_name.clone();
                                rsx! {
                                    ContextMenuItem {
                                        key: "emp-{c_str}",
                                        on_select: move |_| {
                                            let _ = crate::db::update_note_emp_code(n_id, &c_str);
                                            toast.success("Assigned", format!("Note #{n_id} assigned to {name_str} ({c_str})"));
                                            modals.trigger_sync();
                                        },
                                        div { class: "flex items-center justify-between w-full gap-2",
                                            span { class: "truncate text-xs", "{emp.emp_name} ({emp.emp_code})" }
                                            if is_active {
                                                CheckIcon { class: "size-3.5 text-primary ml-auto shrink-0" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                ContextMenuSeparator {}

                // Delete
                ContextMenuItem {
                    destructive: true,
                    on_select: {
                        let del_t = note.title.clone();
                        move |_| modals.open_delete(n_id, &del_t)
                    },
                    TrashIcon { class: "size-3.5 text-destructive" }
                    "Delete Note"
                }
            }
        }
    }
}
