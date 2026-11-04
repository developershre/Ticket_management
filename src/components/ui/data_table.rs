#![allow(dead_code)]

use dioxus::prelude::*;

use crate::components::ui::{
    DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuTrigger, Input,
    Select, SelectContent, SelectItem, SelectTrigger, SelectValue,
};
use crate::icons::{ChevronLeftIcon, ChevronRightIcon, SearchIcon};

// ============================================================
// Sort value — a comparable value a column can sort by
// ============================================================

/// A value a column can be sorted by. `Number` sorts numerically, `Text`
/// sorts case-insensitively. Numbers always sort before text so mixed columns
/// stay stable.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SortValue {
    Text(String),
    Number(i64),
}

impl SortValue {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn num(n: impl Into<i64>) -> Self {
        Self::Number(n.into())
    }
}

impl Ord for SortValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a.cmp(b),
            (Self::Text(a), Self::Text(b)) => a.to_lowercase().cmp(&b.to_lowercase()),
            (Self::Number(_), Self::Text(_)) => std::cmp::Ordering::Less,
            (Self::Text(_), Self::Number(_)) => std::cmp::Ordering::Greater,
        }
    }
}

impl PartialOrd for SortValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ============================================================
// Column + filter definitions
// ============================================================

/// A single column definition. `cell` and `sort_value` are plain function
/// pointers (non-capturing), which keeps the struct cheap to clone/compare so
/// it can live in reactive props. Cell renderers may still return components
/// that use context, because those components are instantiated in the tree.
#[derive(Clone)]
pub struct DataTableColumn<T: Clone + PartialEq + 'static> {
    pub id: String,
    pub header: String,
    pub sortable: bool,
    pub cell: fn(&T) -> Element,
    pub sort_value: fn(&T) -> SortValue,
}

// Compare by identity/metadata only — function pointers are `PartialEq` but
// their addresses aren't guaranteed unique, so we skip them for change detection.
impl<T: Clone + PartialEq + 'static> PartialEq for DataTableColumn<T> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.header == other.header && self.sortable == other.sortable
    }
}

impl<T: Clone + PartialEq + 'static> DataTableColumn<T> {
    /// Convenience constructor for a sortable column.
    pub fn new(
        id: impl Into<String>,
        header: impl Into<String>,
        cell: fn(&T) -> Element,
        sort_value: fn(&T) -> SortValue,
    ) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            sortable: true,
            cell,
            sort_value,
        }
    }

    /// A non-sortable column (e.g. an actions column).
    pub fn actions(
        id: impl Into<String>,
        header: impl Into<String>,
        cell: fn(&T) -> Element,
    ) -> Self {
        Self {
            id: id.into(),
            header: header.into(),
            sortable: false,
            cell,
            sort_value: |_| SortValue::Text(String::new()),
        }
    }
}

/// One option in the faceted filter dropdown.
#[derive(Clone, PartialEq, Debug)]
pub struct DataTableFilter {
    /// Value stored in the filter signal when this option is chosen.
    pub value: String,
    /// Human-readable label shown in the dropdown.
    pub label: String,
}

impl DataTableFilter {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

// ============================================================
// Props
// ============================================================

#[derive(Props, Clone)]
pub struct DataTableProps<T: Clone + PartialEq + 'static> {
    /// The full dataset. Search / filter / sort / paginate are applied on top.
    pub data: Vec<T>,

    /// Column definitions, rendered left to right.
    pub columns: Vec<DataTableColumn<T>>,

    /// Returns a stable unique id for a row (used for selection + keys).
    pub row_id: fn(&T) -> String,

    /// Returns the searchable text for a row (matched against the search box).
    pub search_text: fn(&T) -> String,

    /// Optional faceted filter dropdown options. When empty, no filter is shown.
    #[props(default)]
    pub filter: Vec<DataTableFilter>,

    /// Optional placeholder/label for the first filter (default "Filter").
    #[props(default)]
    pub filter_label: Option<String>,

    /// Returns the value a row is grouped by for the faceted filter.
    /// Only used when `filter` is non-empty.
    #[props(default)]
    pub filter_value: Option<fn(&T) -> String>,

    /// Optional second faceted filter dropdown options.
    #[props(default)]
    pub filter2: Vec<DataTableFilter>,

    /// Optional placeholder/label for the second filter (default "Priority").
    #[props(default)]
    pub filter2_label: Option<String>,

    /// Placeholder text for the toolbar search box.
    /// Defaults to the tickets wording for backwards compatibility.
    #[props(default)]
    pub search_placeholder: Option<String>,

    /// Returns the value a row is grouped by for the second filter.
    #[props(default)]
    pub filter2_value: Option<fn(&T) -> String>,

    /// Rows per page (the starting value of the "Rows per page" selector).
    #[props(default = 10)]
    pub page_size: usize,

    /// Initial sort as `(column_id, ascending)`. The header sort state starts here.
    #[props(default)]
    pub initial_sort: Option<(String, bool)>,

    /// Called with the row when it is clicked (e.g. to open a detail drawer).
    #[props(default)]
    pub on_row_click: Option<EventHandler<T>>,

    /// Called with `(row, (x, y))` when a row is right-clicked.
    #[props(default)]
    pub on_row_context_menu: Option<EventHandler<(T, (f64, f64))>>,

    /// Called with the selected row ids whenever selection changes.
    #[props(default)]
    pub on_selection_change: Option<EventHandler<Vec<String>>>,

    /// Called with the current sort `(column_id, ascending)` (or `None` when cleared).
    #[props(default)]
    pub on_sort_change: Option<EventHandler<Option<(String, bool)>>>,

    /// Extra classes for the outer container.
    #[props(default)]
    pub class: String,
}

// Skip the fn-pointer fields (row_id, search_text, filter_value, filter2_value) — their
// addresses aren't unique, so comparing them for change detection is meaningless.
impl<T: Clone + PartialEq + 'static> PartialEq for DataTableProps<T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
            && self.columns == other.columns
            && self.filter == other.filter
            && self.filter_label == other.filter_label
            && self.filter2 == other.filter2
            && self.filter2_label == other.filter2_label
            && self.search_placeholder == other.search_placeholder
            && self.page_size == other.page_size
            && self.initial_sort == other.initial_sort
            && self.on_row_click == other.on_row_click
            && self.on_row_context_menu == other.on_row_context_menu
            && self.on_selection_change == other.on_selection_change
            && self.on_sort_change == other.on_sort_change
            && self.class == other.class
    }
}

// ============================================================
// Selection checkbox (shadcn-style square checkbox)
// ============================================================

#[component]
fn SelectionCheckbox(checked: bool, indeterminate: bool, on_toggle: EventHandler<()>) -> Element {
    let state_class = if checked || indeterminate {
        "border-primary bg-primary text-primary-foreground"
    } else {
        "border-border bg-transparent text-transparent hover:border-primary/60"
    };

    rsx! {
        button {
            r#type: "button",
            role: "checkbox",
            "aria-checked": if indeterminate { "mixed" } else if checked { "true" } else { "false" },
            class: "flex size-4 shrink-0 cursor-pointer items-center justify-center rounded-[4px] border transition-colors {state_class}",
            onclick: move |e| {
                e.stop_propagation();
                on_toggle.call(());
            },
            if indeterminate {
                svg {
                    class: "size-2.5",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    path { d: "M5 12h14" }
                }
            } else if checked {
                svg {
                    class: "size-2.5",
                    view_box: "0 0 24 24",
                    fill: "none",
                    stroke: "currentColor",
                    stroke_width: "3",
                    stroke_linecap: "round",
                    stroke_linejoin: "round",
                    path { d: "M20 6 9 17l-5-5" }
                }
            }
        }
    }
}

// ============================================================
// Sort indicator (neutral chevron shows on hover when unsorted)
// ============================================================

#[component]
fn SortIcon(active: bool, asc: bool) -> Element {
    if !active {
        return rsx! {
            svg {
                class: "size-3 shrink-0 text-muted-foreground/40 transition-colors group-hover:text-foreground",
                view_box: "0 0 24 24",
                fill: "none",
                stroke: "currentColor",
                stroke_width: "2",
                stroke_linecap: "round",
                stroke_linejoin: "round",
                path { d: "m7 15 5 5 5-5" }
                path { d: "m7 9 5-5 5 5" }
            }
        };
    }
    let dir_path = if asc {
        "m18 15-6-6-6 6"
    } else {
        "m6 9 6 6 6-6"
    };

    rsx! {
        svg {
            class: "size-3 shrink-0 text-foreground transition-transform",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2.5",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "{dir_path}" }
        }
    }
}

// ============================================================
// Tri-state sort toggle: None -> Asc -> Desc -> None
// ============================================================

fn toggle_sort(
    mut sort: Signal<Option<(String, bool)>>,
    mut page: Signal<usize>,
    col_id: &str,
    sortable: bool,
) {
    if !sortable {
        return;
    }
    {
        let mut s = sort.write();
        let current = s.as_ref().map(|(id, asc)| (id.clone(), *asc));
        match current {
            Some((id, asc)) if id == col_id => {
                if asc {
                    *s = Some((col_id.to_string(), false));
                } else {
                    *s = None;
                }
            }
            _ => *s = Some((col_id.to_string(), true)),
        }
    }
    *page.write() = 0;
}

// ============================================================
// DataTable
// ============================================================

#[component]
pub fn DataTable<T: Clone + PartialEq + 'static>(props: DataTableProps<T>) -> Element {
    let mut query = use_signal(String::new);
    let mut filter = use_signal(String::new);
    let mut filter2 = use_signal(String::new);
    let initial_sort = props.initial_sort.clone();
    let sort = use_signal(move || initial_sort);
    let mut page = use_signal(|| 0usize);
    let mut selected = use_signal(Vec::<String>::new);
    let mut hidden = use_signal(Vec::<String>::new);

    let per_page_default = props.page_size.to_string();
    let per_page_str = use_signal(move || per_page_default);

    let columns = props.columns.clone();
    let row_id = props.row_id;
    let search_text = props.search_text;
    let show_filter = !props.filter.is_empty() && props.filter_value.is_some();
    let show_filter2 = !props.filter2.is_empty() && props.filter2_value.is_some();
    let show_column_toggle = columns.len() > 1;

    // ---- Column visibility ----
    let hidden_ids = hidden.read().clone();
    let visible_columns: Vec<DataTableColumn<T>> = columns
        .iter()
        .filter(|c| !hidden_ids.contains(&c.id))
        .cloned()
        .collect();

    // ---- Filter + search ----
    let q = query.read().to_lowercase();
    let filter_val = filter.read().clone();
    let filter2_val = filter2.read().clone();
    let is_filtered = !q.is_empty() || !filter_val.is_empty() || !filter2_val.is_empty();

    let mut rows: Vec<T> = props
        .data
        .iter()
        .filter(|t| {
            let matches_search = q.is_empty() || search_text(t).to_lowercase().contains(&q);
            let matches_filter = filter_val.is_empty()
                || props
                    .filter_value
                    .map(|fv| fv(t) == filter_val)
                    .unwrap_or(true);
            let matches_filter2 = filter2_val.is_empty()
                || props
                    .filter2_value
                    .map(|fv| fv(t) == filter2_val)
                    .unwrap_or(true);
            matches_search && matches_filter && matches_filter2
        })
        .cloned()
        .collect();

    // ---- Sort ----
    let sort_state = sort();
    if let Some((col_id, asc)) = &sort_state {
        if let Some(col) = columns.iter().find(|c| &c.id == col_id) {
            let sv = col.sort_value;
            rows.sort_by(|a, b| {
                let ord = sv(a).cmp(&sv(b));
                if *asc {
                    ord
                } else {
                    ord.reverse()
                }
            });
        }
    }

    // ---- Paginate ----
    let per_page: usize = per_page_str
        .read()
        .parse()
        .unwrap_or(props.page_size)
        .max(1);
    let total = rows.len();
    let total_pages = total.div_ceil(per_page).max(1);
    let current_page = (*page.read()).min(total_pages - 1);
    let start = current_page * per_page;
    let end = (start + per_page).min(total);
    let page_rows: Vec<T> = rows[start.min(end)..end].to_vec();

    // ---- Selection snapshot ----
    let selected_ids = selected.read().clone();
    let selected_count = selected_ids.len();
    let page_ids: Vec<String> = page_rows.iter().map(row_id).collect();
    let page_selected_count = page_ids
        .iter()
        .filter(|id| selected_ids.contains(id))
        .count();
    let all_page_selected = !page_ids.is_empty() && page_selected_count == page_ids.len();
    let some_page_selected = page_selected_count > 0 && !all_page_selected;

    // ---- Notify parent of selection / sort changes ----
    let on_selection_change = props.on_selection_change;
    use_effect(move || {
        if let Some(h) = &on_selection_change {
            h.call(selected.read().clone());
        }
    });
    let on_sort_change = props.on_sort_change;
    use_effect(move || {
        if let Some(h) = &on_sort_change {
            h.call(sort.read().clone());
        }
    });

    let col_count = visible_columns.len() + 1;
    let showing_from = if total == 0 { 0 } else { start + 1 };
    let showing_to = end;
    let filter_options = props.filter.clone();
    let filter2_options = props.filter2.clone();
    let filter_placeholder = props
        .filter_label
        .clone()
        .unwrap_or_else(|| "Status".to_string());
    let filter2_placeholder = props
        .filter2_label
        .clone()
        .unwrap_or_else(|| "Priority".to_string());
    let search_placeholder = props
        .search_placeholder
        .clone()
        .unwrap_or_else(|| "Search tickets, requester, ID...".to_string());

    // Page-size choices
    let mut page_sizes = vec![5usize, 10, 20, 30, 50];
    if !page_sizes.contains(&props.page_size) {
        page_sizes.push(props.page_size);
        page_sizes.sort_unstable();
    }

    rsx! {
        div { class: "flex flex-col flex-1 w-full min-h-0 rounded-md border border-border/80 bg-card shadow-xs {props.class}",

            // ================= Toolbar =================
            div { class: "flex shrink-0 flex-wrap items-center justify-between gap-2.5 border-b border-border/60 p-2.5 bg-card",

                div { class: "flex flex-wrap items-center gap-2 flex-1 min-w-0",
                    // Search
                    div { class: "relative w-full min-w-[180px] max-w-xs flex-1",
                        SearchIcon { class: "pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" }
                        Input {
                            placeholder: search_placeholder,
                            value: query.read().clone(),
                            class: "h-8 pl-8 pr-7 text-xs rounded-md bg-background border-border/80".to_string(),
                            oninput: move |e: FormEvent| {
                                query.set(e.value());
                                *page.write() = 0;
                            },
                        }
                        if !query.read().is_empty() {
                            button {
                                r#type: "button",
                                title: "Clear search",
                                class: "absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground cursor-pointer p-0.5",
                                onclick: move |_| {
                                    query.set(String::new());
                                    *page.write() = 0;
                                },
                                svg { class: "size-3", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                    path { d: "M18 6L6 18M6 6l12 12" }
                                }
                            }
                        }
                    }

                    // Faceted filter 1
                    if show_filter {
                        div { class: "w-[130px] shrink-0",
                            Select {
                                value: Some(filter),
                                SelectTrigger { class: "h-8 text-xs rounded-md bg-background border-border/80".to_string(),
                                    SelectValue { placeholder: "{filter_placeholder}".to_string() }
                                }
                                SelectContent {
                                    SelectItem { value: "".to_string(), text: format!("All {}", filter_placeholder), "All" }
                                    for opt in filter_options {
                                        SelectItem {
                                            key: "{opt.value}",
                                            value: opt.value.clone(),
                                            text: opt.label.clone(),
                                            "{opt.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Faceted filter 2
                    if show_filter2 {
                        div { class: "w-[130px] shrink-0",
                            Select {
                                value: Some(filter2),
                                SelectTrigger { class: "h-8 text-xs rounded-md bg-background border-border/80".to_string(),
                                    SelectValue { placeholder: "{filter2_placeholder}".to_string() }
                                }
                                SelectContent {
                                    SelectItem { value: "".to_string(), text: format!("All {}", filter2_placeholder), "All" }
                                    for opt in filter2_options {
                                        SelectItem {
                                            key: "{opt.value}",
                                            value: opt.value.clone(),
                                            text: opt.label.clone(),
                                            "{opt.label}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Reset Filters button
                    if is_filtered {
                        button {
                            r#type: "button",
                            class: "flex h-8 items-center gap-1.5 rounded-md px-2.5 text-xs text-muted-foreground hover:text-foreground hover:bg-muted border border-dashed border-border/80 transition-colors cursor-pointer",
                            onclick: move |_| {
                                query.set(String::new());
                                filter.set(String::new());
                                filter2.set(String::new());
                                *page.write() = 0;
                            },
                            svg { class: "size-3", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
                                path { d: "M18 6L6 18M6 6l12 12" }
                            }
                            "Reset"
                        }
                    }
                }

                div { class: "flex items-center gap-2",
                    // Selection bulk-actions
                    if selected_count > 0 {
                        div { class: "flex items-center gap-1.5 rounded-md bg-muted/60 px-2 py-1",
                            span { class: "text-xs font-medium text-foreground",
                                "{selected_count} selected"
                            }
                            button {
                                r#type: "button",
                                class: "h-6 cursor-pointer rounded px-1.5 text-[11px] font-medium text-muted-foreground hover:text-foreground hover:bg-background transition-colors",
                                onclick: move |_| selected.write().clear(),
                                "Clear"
                            }
                        }
                    }

                    // Column visibility toggle
                    if show_column_toggle {
                        DropdownMenu {
                            DropdownMenuTrigger { class: "h-8 gap-1.5 rounded-md border border-border/80 bg-background px-2.5 text-xs text-foreground hover:bg-muted transition-colors".to_string(),
                                svg {
                                    class: "size-3.5 text-muted-foreground",
                                    view_box: "0 0 24 24",
                                    fill: "none",
                                    stroke: "currentColor",
                                    stroke_width: "2",
                                    stroke_linecap: "round",
                                    stroke_linejoin: "round",
                                    path { d: "M21 4H14" }
                                    path { d: "M10 4H3" }
                                    path { d: "M21 12H12" }
                                    path { d: "M8 12H3" }
                                    path { d: "M21 20H16" }
                                    path { d: "M12 20H3" }
                                    path { d: "M14 2V6" }
                                    path { d: "M8 10V14" }
                                    path { d: "M16 18V22" }
                                }
                                "Columns"
                            }
                            DropdownMenuContent { align: "end".to_string(), class: "w-44".to_string(),
                                for col in &columns {
                                    {
                                        let cid = col.id.clone();
                                        let cheader = col.header.clone();
                                        let is_hidden = hidden_ids.contains(&cid);
                                        rsx! {
                                            DropdownMenuCheckboxItem {
                                                key: "{cid}",
                                                checked: !is_hidden,
                                                onchange: move |visible: bool| {
                                                    let mut h = hidden.write();
                                                    if visible {
                                                        h.retain(|x| x != &cid);
                                                    } else if !h.contains(&cid) {
                                                        h.push(cid.clone());
                                                    }
                                                },
                                                "{cheader}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ================= Table (Middle flex scroll area) =================
            div { class: "flex-1 overflow-auto min-h-0 w-full",
                table { class: "w-full caption-bottom border-collapse text-left text-sm",
                    thead { class: "sticky top-0 z-10 border-b border-border/70 bg-muted/80 backdrop-blur-xs",
                        tr {
                            // Select-all header
                            th { class: "w-10 px-3.5 py-2.5 align-middle",
                                SelectionCheckbox {
                                    checked: all_page_selected,
                                    indeterminate: some_page_selected,
                                    on_toggle: move |_| {
                                        let mut sel = selected.write();
                                        if all_page_selected {
                                            sel.retain(|id| !page_ids.contains(id));
                                        } else {
                                            for id in &page_ids {
                                                if !sel.contains(id) {
                                                    sel.push(id.clone());
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Column headers
                            for col in &visible_columns {
                                {
                                    let sort_id = col.id.clone();
                                    let sort_id2 = col.id.clone();
                                    let key_id = col.id.clone();
                                    let header = col.header.clone();
                                    let sortable = col.sortable;
                                    let (is_sorted, asc) = match &sort_state {
                                        Some((id, a)) if *id == sort_id => (true, *a),
                                        _ => (false, true),
                                    };
                                    let aria_sort = if is_sorted {
                                        if asc { "ascending" } else { "descending" }
                                    } else {
                                        "none"
                                    };
                                    let header_color = if is_sorted { "text-foreground font-semibold" } else { "text-muted-foreground" };
                                    rsx! {
                                        th {
                                            key: "{key_id}",
                                            "aria-sort": aria_sort,
                                            tabindex: if sortable { "0" } else { "-1" },
                                            class: if sortable {
                                                "cursor-pointer select-none px-3.5 py-2.5 align-middle text-[11px] uppercase tracking-wider transition-colors hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring {header_color}"
                                            } else {
                                                "px-3.5 py-2.5 align-middle text-[11px] font-semibold uppercase tracking-wider text-muted-foreground"
                                            },
                                            onclick: move |_| {
                                                toggle_sort(sort, page, &sort_id, sortable);
                                            },
                                            onkeydown: move |e: KeyboardEvent| {
                                                if e.key() == Key::Enter {
                                                    toggle_sort(sort, page, &sort_id2, sortable);
                                                }
                                            },
                                            div { class: "flex items-center gap-1.5",
                                                span { "{header}" }
                                                if sortable {
                                                    SortIcon { active: is_sorted, asc }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    tbody { class: "divide-y divide-border/60 bg-card",
                        if total == 0 {
                            tr {
                                td { class: "px-4 py-16 text-center text-xs text-muted-foreground",
                                    colspan: "{col_count}",
                                    div { class: "flex flex-col items-center justify-center gap-2",
                                        svg {
                                            class: "size-8 text-muted-foreground/40",
                                            view_box: "0 0 24 24",
                                            fill: "none",
                                            stroke: "currentColor",
                                            stroke_width: "1.5",
                                            circle { cx: "11", cy: "11", r: "8" }
                                            path { d: "m21 21-4.3-4.3" }
                                        }
                                        span { class: "font-medium text-foreground", "No tickets found" }
                                        span { class: "text-muted-foreground", "Try adjusting your search or active filters." }
                                    }
                                }
                            }
                        } else {
                            for task in page_rows {
                                {
                                    let rid = row_id(&task);
                                    let is_selected = selected_ids.contains(&rid);
                                    let has_click = props.on_row_click.is_some();
                                    let cursor = if has_click { "cursor-pointer" } else { "" };
                                    let click_task = task.clone();
                                    let click_task_ctx = task.clone();
                                    let toggle_id = rid.clone();
                                    let row_on_click = props.on_row_click;
                                    let row_on_context = props.on_row_context_menu;
                                    rsx! {
                                        tr {
                                            key: "{rid}",
                                            "data-state": if is_selected { "selected" } else { "" },
                                            class: "group border-b border-border/50 transition-colors duration-100 last:border-b-0 hover:bg-muted/40 data-[state=selected]:bg-muted/60 {cursor}",
                                            onclick: move |_| {
                                                if let Some(h) = &row_on_click {
                                                    h.call(click_task.clone());
                                                }
                                            },
                                            oncontextmenu: move |e: MouseEvent| {
                                                if let Some(h) = &row_on_context {
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    let coords = e.client_coordinates();
                                                    h.call((click_task_ctx.clone(), (coords.x, coords.y)));
                                                }
                                            },
                                            td { class: "px-3.5 py-2.5 align-middle",
                                                SelectionCheckbox {
                                                    checked: is_selected,
                                                    indeterminate: false,
                                                    on_toggle: move |_| {
                                                        let mut sel = selected.write();
                                                        if let Some(pos) = sel.iter().position(|id| id == &toggle_id) {
                                                            sel.remove(pos);
                                                        } else {
                                                            sel.push(toggle_id.clone());
                                                        }
                                                    }
                                                }
                                            }
                                            for col in &visible_columns {
                                                td {
                                                    key: "{col.id}",
                                                    class: "px-3.5 py-2.5 align-middle text-xs [&:has([role=checkbox])]:pr-0",
                                                    {(col.cell)(&task)}
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

            // ================= Pagination (Pinned Bottom Bar) =================
            div { class: "flex shrink-0 flex-wrap items-center justify-between gap-3 border-t border-border/60 px-4 py-2.5 bg-card",

                // Left: summary / selection
                div { class: "flex items-center gap-3 text-xs text-muted-foreground",
                    span {
                        if selected_count > 0 {
                            "{selected_count} of {total} row(s) selected."
                        } else {
                            "Showing {showing_from}–{showing_to} of {total} entries"
                        }
                    }
                }

                // Right: rows-per-page + navigation
                div { class: "flex items-center gap-4",
                    div { class: "flex items-center gap-2",
                        span { class: "text-xs text-muted-foreground",
                            "Rows per page"
                        }
                        div { class: "w-16",
                            Select {
                                value: Some(per_page_str),
                                SelectTrigger { class: "h-8 text-xs rounded-md bg-background border-border/80".to_string(),
                                    SelectValue {}
                                }
                                SelectContent {
                                    for n in page_sizes {
                                        {
                                            let s = n.to_string();
                                            rsx! {
                                                SelectItem {
                                                    key: "{s}",
                                                    value: s.clone(),
                                                    text: s.clone(),
                                                    "{s}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "flex items-center gap-1",
                        PaginationButton {
                            disabled: current_page == 0,
                            title: "First page",
                            onclick: move |_| *page.write() = 0,
                            svg {
                                class: "size-3.5",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "m11 17-5-5 5-5" }
                                path { d: "m18 17-5-5 5-5" }
                            }
                        }
                        PaginationButton {
                            disabled: current_page == 0,
                            title: "Previous page",
                            onclick: move |_| *page.write() = current_page.saturating_sub(1),
                            ChevronLeftIcon { class: "size-3.5" }
                        }

                        span { class: "px-2 text-xs font-medium text-foreground tabular-nums",
                            "Page {current_page + 1} of {total_pages}"
                        }

                        PaginationButton {
                            disabled: current_page + 1 >= total_pages,
                            title: "Next page",
                            onclick: move |_| *page.write() = current_page + 1,
                            ChevronRightIcon { class: "size-3.5" }
                        }
                        PaginationButton {
                            disabled: current_page + 1 >= total_pages,
                            title: "Last page",
                            onclick: move |_| *page.write() = total_pages - 1,
                            svg {
                                class: "size-3.5",
                                view_box: "0 0 24 24",
                                fill: "none",
                                stroke: "currentColor",
                                stroke_width: "2",
                                stroke_linecap: "round",
                                stroke_linejoin: "round",
                                path { d: "m6 17 5-5-5-5" }
                                path { d: "m13 17 5-5-5-5" }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ============================================================
// Pagination button
// ============================================================

#[component]
fn PaginationButton(
    disabled: bool,
    title: String,
    onclick: EventHandler<()>,
    children: Element,
) -> Element {
    let state = if disabled {
        "pointer-events-none cursor-not-allowed opacity-40"
    } else {
        "cursor-pointer hover:bg-muted"
    };

    rsx! {
        button {
            r#type: "button",
            title: title,
            disabled: disabled,
            class: "inline-flex size-8 items-center justify-center rounded-md border border-border/80 bg-background text-foreground transition-colors {state}",
            onclick: move |_| {
                if !disabled {
                    onclick.call(());
                }
            },
            {children}
        }
    }
}
