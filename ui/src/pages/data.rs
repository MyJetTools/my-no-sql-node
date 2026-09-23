use std::collections::HashMap;

use dioxus::prelude::*;
use serde_json::Value;

use crate::AppContext;
use crate::AppRoute;
use crate::RouteKey;
use crate::components::atoms::{Badge, BadgeTone, Icon, IconKind};
use crate::components::data::{
    PARTITION_KEY, PartitionsPane, ROW_KEY, RowDrawer, RowsTable, TIME_STAMP, TableHeader,
    TablePagination, TableToolbar, TablesPane,
};
use crate::models::{
    DEFAULT_NAMESPACE, StatusModel, TABLE_SYNC_NOT_FOUND, TABLE_SYNC_PENDING, TableModel,
};

const DEFAULT_PAGE_SIZE: usize = 100;

/// Rows of the selected partition. The data section is read-only.
struct DataState {
    /// (table, partition) whose rows are currently loaded or loading.
    loaded_rows_for: Option<(String, String)>,
    /// True once the rows for `loaded_rows_for` have actually arrived.
    rows_ready: bool,
    rows_error: Option<String>,
    headers: Vec<String>,
    rows: Vec<Value>,
    /// The lowercased values of every row, built once when the rows arrive - the filter searches
    /// these instead of walking every row again on each keystroke.
    search: Vec<String>,
    page_size: usize,
    current_page: usize,
}

impl Default for DataState {
    fn default() -> Self {
        Self {
            loaded_rows_for: None,
            rows_ready: false,
            rows_error: None,
            headers: Vec::new(),
            rows: Vec::new(),
            search: Vec::new(),
            page_size: DEFAULT_PAGE_SIZE,
            current_page: 0,
        }
    }
}

impl DataState {
    /// Start loading rows for `(table, partition)`; clears the previous rows.
    fn begin_rows_load(&mut self, table: &str, partition: &str) {
        self.loaded_rows_for = Some((table.to_string(), partition.to_string()));
        self.rows_ready = false;
        self.rows_error = None;
        self.headers = Vec::new();
        self.rows = Vec::new();
        self.search = Vec::new();
        self.current_page = 0;
    }

    fn is_rows_scope(&self, table: &str, partition: &str) -> bool {
        self.loaded_rows_for
            .as_ref()
            .map(|(t, p)| t.as_str() == table && p.as_str() == partition)
            .unwrap_or(false)
    }

    /// Apply fetched rows, unless the (table, partition) changed meanwhile.
    fn set_rows(&mut self, table: &str, partition: &str, rows: Vec<Value>) {
        if self.is_rows_scope(table, partition) {
            self.headers = build_headers(&rows);
            self.search = rows.iter().map(build_search_text).collect();
            self.rows = rows;
            self.rows_ready = true;
        }
    }

    fn set_rows_error(&mut self, table: &str, partition: &str, err: String) {
        if self.is_rows_scope(table, partition) {
            self.rows_error = Some(err);
        }
    }

    fn set_page(&mut self, page: usize) {
        self.current_page = page;
    }

    fn set_page_size(&mut self, size: usize) {
        self.page_size = size.max(1);
        self.current_page = 0;
    }

    fn reset_pagination(&mut self) {
        self.current_page = 0;
    }

    /// Force the next render to refetch rows for the current partition.
    fn clear_rows_scope(&mut self) {
        self.loaded_rows_for = None;
        self.rows_ready = false;
        self.rows_error = None;
    }
}

/// What the data section needs from the status the shell polls, for the namespace of the UI.
/// Memoized: the status changes every second, this only when something in it does.
#[derive(Clone, PartialEq, Default)]
struct NamespaceTables {
    loaded: bool,
    /// By name.
    tables: Vec<TableModel>,
    /// Readers subscribed to the table.
    readers: HashMap<String, usize>,
    /// Readers waiting for the table to arrive from the main node.
    waiting: HashMap<String, usize>,
}

impl NamespaceTables {
    fn new(status: Option<&StatusModel>, namespace: &str) -> Self {
        let Some(status) = status else {
            return Self::default();
        };

        let mut tables: Vec<TableModel> = status
            .initialized
            .tables
            .iter()
            .filter(|itm| itm.namespace == namespace)
            .cloned()
            .collect();
        tables.sort_by(|a, b| a.name.cmp(&b.name));

        let mut readers: HashMap<String, usize> = HashMap::new();
        let mut waiting: HashMap<String, usize> = HashMap::new();

        for reader in status.initialized.readers.iter() {
            if reader.namespace != namespace {
                continue;
            }
            for table in reader.tables.iter() {
                *readers.entry(table.clone()).or_default() += 1;
            }
            for table in reader.awaiting_tables.iter() {
                *waiting.entry(table.clone()).or_default() += 1;
            }
        }

        Self {
            loaded: true,
            tables,
            readers,
            waiting,
        }
    }

    fn get(&self, table: &str) -> Option<&TableModel> {
        self.tables.iter().find(|itm| itm.name == table)
    }
}

/// Extract `(table, partition, row)` from the current data route.
fn parse_data_route(route: &AppRoute) -> (Option<String>, Option<String>, Option<String>) {
    match route {
        AppRoute::DataTable { table } => (Some(table.0.clone()), None, None),
        AppRoute::DataPartition { table, partition } => {
            (Some(table.0.clone()), Some(partition.0.clone()), None)
        }
        AppRoute::DataRow {
            table,
            partition,
            row,
        } => (
            Some(table.0.clone()),
            Some(partition.0.clone()),
            Some(row.0.clone()),
        ),
        AppRoute::Home {}
        | AppRoute::Data {}
        | AppRoute::Connections {}
        | AppRoute::NotFound { .. } => (None, None, None),
    }
}

// Route placeholders — the URL patterns of the data section. `DataLayout` renders the whole page
// and reads the params via `use_route`, so these render nothing themselves.
#[component]
pub fn Data() -> Element {
    rsx! {}
}

#[component]
pub fn DataTable(table: RouteKey) -> Element {
    let _ = table;
    rsx! {}
}

#[component]
pub fn DataPartition(table: RouteKey, partition: RouteKey) -> Element {
    let _ = (table, partition);
    rsx! {}
}

#[component]
pub fn DataRow(table: RouteKey, partition: RouteKey, row: RouteKey) -> Element {
    let _ = (table, partition, row);
    rsx! {}
}

#[component]
pub fn DataLayout() -> Element {
    let mut cs = use_signal(DataState::default);
    let row_filter = use_signal(String::new);
    let app_ctx = use_context::<Signal<AppContext>>();
    let nav = navigator();

    let route = use_route::<AppRoute>();
    let (url_table, url_partition, url_row) = parse_data_route(&route);

    let namespace =
        crate::storage::load_namespace().unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    // Only whether the last status poll failed - reading the whole context would re-render the
    // page every second.
    let status_error = use_memo(move || app_ctx.read().status_error.clone());

    let ns_tables = use_memo(move || {
        let namespace =
            crate::storage::load_namespace().unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());
        NamespaceTables::new(app_ctx.read().status.as_ref(), namespace.as_str())
    });

    // Indexes of the rows matching the filter. Recomputed when the rows or the filter change -
    // not on every render.
    let filtered = use_memo(move || {
        let filter = row_filter.read().trim().to_lowercase();
        let cs_ra = cs.read();
        if filter.is_empty() {
            return (0..cs_ra.rows.len()).collect::<Vec<usize>>();
        }
        cs_ra
            .search
            .iter()
            .enumerate()
            .filter(|(_, text)| text.contains(filter.as_str()))
            .map(|(index, _)| index)
            .collect()
    });

    // ---- reset to page 1 whenever the filter changes ----
    use_effect(move || {
        let _ = row_filter.read();
        if cs.peek().current_page != 0 {
            cs.write().reset_pagination();
        }
    });

    let ns_tables_ra = ns_tables.read();
    let selected_table = url_table.clone().unwrap_or_default();
    let table_stats = ns_tables_ra.get(selected_table.as_str()).cloned();
    let table_is_replicated = matches!(
        table_stats.as_ref().map(|itm| itm.sync_state.as_str()),
        Some(state) if state != TABLE_SYNC_PENDING
    );

    // ---- load rows whenever the URL (table, partition) changes ----
    if let (Some(table), Some(partition), true) = (
        url_table.clone(),
        url_partition.clone(),
        table_is_replicated,
    ) {
        let pair = (table, partition);
        let already = { cs.read().loaded_rows_for.as_ref() == Some(&pair) };
        if !already {
            spawn(async move {
                if cs.peek().loaded_rows_for.as_ref() == Some(&pair) {
                    return;
                }
                cs.write().begin_rows_load(&pair.0, &pair.1);
                match crate::api::get_rows(&pair.0, &pair.1).await {
                    Ok(rows) => cs.write().set_rows(&pair.0, &pair.1, rows),
                    Err(err) => {
                        dioxus_utils::console_log(format!("Rows error: {}", err.details));
                        cs.write().set_rows_error(&pair.0, &pair.1, err.message);
                    }
                }
            });
        }
    }

    // ---- read state for rendering ----
    let cs_ra = cs.read();

    let rows_scope_matches = match (&url_table, &url_partition, &cs_ra.loaded_rows_for) {
        (Some(t), Some(p), Some((lt, lp))) => t == lt && p == lp,
        _ => false,
    };
    let rows_ready = rows_scope_matches && cs_ra.rows_ready;
    let rows_error = if rows_scope_matches {
        cs_ra.rows_error.clone()
    } else {
        None
    };
    let all_rows: &[Value] = if rows_scope_matches {
        cs_ra.rows.as_slice()
    } else {
        &[]
    };

    let filtered_ra = filtered.read();
    let filtered_indexes: &[usize] = if rows_scope_matches {
        filtered_ra.as_slice()
    } else {
        &[]
    };

    // Paginate the filtered set — clamp during render (no signal write here).
    let page_size = cs_ra.page_size;
    let filtered_total = filtered_indexes.len();
    let total_pages = filtered_total.div_ceil(page_size).max(1);
    let current_page = cs_ra.current_page.min(total_pages - 1);
    let page_start = current_page * page_size;
    let page_end = (page_start + page_size).min(filtered_total);
    let visible_rows: Vec<Value> = filtered_indexes[page_start..page_end]
        .iter()
        .filter_map(|index| all_rows.get(*index).cloned())
        .collect();

    // The drawer carries only the row key in the URL — resolve the full row.
    let resolved_row: Option<Value> = url_row.as_ref().and_then(|rk| {
        all_rows
            .iter()
            .find(|r| r.get(ROW_KEY).and_then(|v| v.as_str()) == Some(rk.as_str()))
            .cloned()
    });

    let headers = if rows_scope_matches {
        cs_ra.headers.clone()
    } else {
        Vec::new()
    };
    let partition_rows_amount = all_rows.len();
    drop(filtered_ra);
    drop(cs_ra);

    // ---- navigation handlers ----
    let select_table = move |name: String| {
        nav.push(AppRoute::DataTable { table: name.into() });
    };

    let select_partition = {
        let table = url_table.clone();
        move |pk: String| {
            if let Some(table) = table.clone() {
                nav.push(AppRoute::DataPartition {
                    table: table.into(),
                    partition: pk.into(),
                });
            }
        }
    };

    // A table with a single partition opens it right away. `replace`, so Back skips this step.
    // Called by the partitions pane of this very table - a pane of a table left behind is gone,
    // its loop with it.
    let on_first_partitions_load = {
        let table = url_table.clone();
        let partition = url_partition.clone();
        move |(total, first): (u64, Option<String>)| {
            let (Some(table), None, 1, Some(only)) =
                (table.clone(), partition.as_ref(), total, first)
            else {
                return;
            };
            nav.replace(AppRoute::DataPartition {
                table: table.into(),
                partition: only.into(),
            });
        }
    };

    let on_row_click = {
        let table = url_table.clone();
        let partition = url_partition.clone();
        move |row: Value| {
            let (Some(table), Some(partition)) = (table.clone(), partition.clone()) else {
                return;
            };
            let rk = row
                .get(ROW_KEY)
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            nav.push(AppRoute::DataRow {
                table: table.into(),
                partition: partition.into(),
                row: rk.into(),
            });
        }
    };

    let close_drawer = {
        let table = url_table.clone();
        let partition = url_partition.clone();
        move |_| {
            if let (Some(table), Some(partition)) = (table.clone(), partition.clone()) {
                nav.push(AppRoute::DataPartition {
                    table: table.into(),
                    partition: partition.into(),
                });
            }
        }
    };

    let readers_of_selected = ns_tables_ra
        .readers
        .get(selected_table.as_str())
        .copied()
        .unwrap_or(0);
    let waiting_for_selected = ns_tables_ra
        .waiting
        .get(selected_table.as_str())
        .copied()
        .unwrap_or(0);

    let center_content = match (&url_table, ns_tables_ra.loaded, table_stats.as_ref()) {
        (None, _, _) => {
            render_empty_state(
                &ns_tables_ra.tables,
                ns_tables_ra.loaded,
                status_error.read().as_deref(),
                select_table,
            )
        }
        (Some(_), false, _) => match status_error.read().as_ref() {
            Some(err) => render_message("Can not reach the node", err.as_str()),
            None => render_message("Connecting to node…", ""),
        },
        (Some(table), true, None) => render_message(
            "The node does not replicate this table",
            format!(
                "No reader subscribed to '{}' in namespace '{}', so the node does not hold it.",
                table, namespace
            )
            .as_str(),
        ),
        (Some(_), true, Some(stats)) => {
            let rows_content = if !table_is_replicated {
                render_sync_state(stats, waiting_for_selected)
            } else {
                match (&url_partition, &rows_error) {
                    (None, _) => render_sync_state(stats, waiting_for_selected),
                    (Some(_), Some(err)) => render_rows_message("Can not load rows", err.as_str()),
                    (Some(_), None) if !rows_ready => render_rows_message("Loading rows…", ""),
                    (Some(_), None) if filtered_total == 0 && partition_rows_amount > 0 => {
                        render_rows_message(
                            "No rows match the filter",
                            format!("{} rows in the partition", partition_rows_amount).as_str(),
                        )
                    }
                    (Some(_), None) if filtered_total == 0 => {
                        render_rows_message("No rows", "This partition is empty.")
                    }
                    (Some(_), None) => rsx! {
                        RowsTable {
                            headers,
                            rows: visible_rows,
                            selected_row_key: url_row.clone(),
                            on_row_click,
                        }
                        TablePagination {
                            total: filtered_total,
                            page_size,
                            current_page,
                            on_page_change: move |p: usize| { cs.write().set_page(p); },
                            on_page_size_change: move |sz: usize| { cs.write().set_page_size(sz); },
                        }
                    },
                }
            };

            // Row controls only make sense with a partition open.
            let toolbar = if url_partition.is_some() && table_is_replicated {
                rsx! {
                    TableToolbar {
                        filter_value: row_filter,
                        reader_count: readers_of_selected,
                        waiting_count: waiting_for_selected,
                    }
                }
            } else {
                rsx! {}
            };

            rsx! {
                div { class: "rows-col",
                    TableHeader {
                        name: selected_table.clone(),
                        stats: Some(stats.clone()),
                        can_reload: url_partition.is_some() && table_is_replicated,
                        on_reload: move |_| {
                            cs.write().clear_rows_scope();
                        },
                    }
                    {toolbar}
                    {rows_content}
                }
            }
        }
    };

    let partitions_content = match (&url_table, table_is_replicated) {
        (Some(table), true) => rsx! {
            PartitionsPane {
                key: "{table}",
                table: table.clone(),
                selected: url_partition.clone(),
                on_select: select_partition,
                on_first_load: on_first_partitions_load,
            }
        },
        (Some(_), false) | (None, _) => rsx! {
            aside { class: "partitions-pane" }
        },
    };

    // A row of a table the node does not hold (or does not hold yet) is never loaded - the centre
    // says why, and a drawer waiting for it would wait forever.
    let drawer_content = match (url_row.as_ref(), table_is_replicated) {
        (None, _) | (Some(_), false) => rsx! {},
        (Some(rk), true) => {
            if let Some(row) = resolved_row {
                rsx! {
                    RowDrawer { row, on_close: close_drawer }
                }
            } else if let Some(err) = rows_error.as_ref() {
                rsx! {
                    DrawerMessage {
                        title: "Can not load row".to_string(),
                        message: err.clone(),
                        on_close: close_drawer,
                    }
                }
            } else if !rows_ready {
                rsx! {
                    DrawerMessage {
                        title: "Loading row…".to_string(),
                        message: "Fetching partition rows…".to_string(),
                        on_close: close_drawer,
                    }
                }
            } else {
                rsx! {
                    DrawerMessage {
                        title: "Row not found".to_string(),
                        message: format!("No row with key \"{}\" in this partition.", rk),
                        on_close: close_drawer,
                    }
                }
            }
        }
    };

    let data_cls = if url_row.is_some() && table_is_replicated {
        "data"
    } else {
        "data data--no-drawer"
    };

    rsx! {
        section { class: "page page--flush",
            div { class: data_cls,
                TablesPane {
                    tables: ns_tables_ra.tables.clone(),
                    selected: selected_table.clone(),
                    readers: ns_tables_ra.readers.clone(),
                    waiting: ns_tables_ra.waiting.clone(),
                    on_select: select_table,
                }
                {partitions_content}
                {center_content}
                {drawer_content}
            }
            Outlet::<AppRoute> {}
        }
    }
}

/// A minimal row drawer used while rows are still loading or when the URL points at a row key
/// that no longer exists.
#[component]
fn DrawerMessage(title: String, message: String, on_close: EventHandler<()>) -> Element {
    rsx! {
        aside { class: "row-drawer",
            div { class: "row-drawer__header",
                span { class: "row-drawer__title", "Row Detail" }
                button {
                    class: "topbar__icon-btn",
                    onclick: move |_| on_close.call(()),
                    Icon { kind: IconKind::X }
                }
            }
            div { class: "row-drawer__body",
                div { class: "empty-state",
                    div { class: "empty-state__title", "{title}" }
                    div { class: "empty-state__sub", "{message}" }
                }
            }
        }
    }
}

/// The values of the row as they are shown - no field names, no JSON escaping - lowercased, so
/// the filter matches what is on the screen.
fn build_search_text(row: &Value) -> String {
    let mut result = String::new();
    push_search_values(row, &mut result);
    result.to_lowercase()
}

fn push_search_values(value: &Value, dest: &mut String) {
    match value {
        Value::Null => {}
        Value::Bool(value) => push_search_value(dest, value.to_string().as_str()),
        Value::Number(value) => push_search_value(dest, value.to_string().as_str()),
        Value::String(value) => push_search_value(dest, value.as_str()),
        Value::Array(items) => {
            for item in items {
                push_search_values(item, dest);
            }
        }
        Value::Object(fields) => {
            for field in fields.values() {
                push_search_values(field, dest);
            }
        }
    }
}

/// Values are kept apart, so a filter never matches across two of them.
fn push_search_value(dest: &mut String, value: &str) {
    if !dest.is_empty() {
        dest.push('\u{1f}');
    }
    dest.push_str(value);
}

fn build_headers(rows: &[Value]) -> Vec<String> {
    let mut headers: Vec<String> = vec![
        PARTITION_KEY.to_string(),
        ROW_KEY.to_string(),
        TIME_STAMP.to_string(),
    ];

    for row in rows.iter() {
        if let Value::Object(map) = row {
            for key in map.keys() {
                if key == PARTITION_KEY || key == ROW_KEY || key == TIME_STAMP {
                    continue;
                }
                if !headers.iter().any(|h| h == key) {
                    headers.push(key.clone());
                }
            }
        }
    }

    headers
}

fn render_message(title: &str, sub: &str) -> Element {
    rsx! {
        div { class: "rows-col",
            div { class: "empty-state",
                div { class: "empty-state__title", "{title}" }
                div { class: "empty-state__sub", "{sub}" }
            }
        }
    }
}

fn render_rows_message(title: &str, sub: &str) -> Element {
    rsx! {
        div { class: "rows-wrap",
            div { class: "empty-state",
                div { class: "empty-state__title", "{title}" }
                div { class: "empty-state__sub", "{sub}" }
            }
        }
    }
}

/// The centre of a table with no partition open: what the node knows about the table.
fn render_sync_state(table: &TableModel, waiting: usize) -> Element {
    let waiting_text = match waiting {
        0 => String::new(),
        1 => " A reader is waiting for it.".to_string(),
        n => format!(" {} readers are waiting for it.", n),
    };

    let (title, sub) = match table.sync_state.as_str() {
        TABLE_SYNC_PENDING => (
            "Waiting for the main node",
            format!(
                "A reader subscribed to this table and the node asked the main node for it. Nothing has arrived yet.{}",
                waiting_text
            ),
        ),
        TABLE_SYNC_NOT_FOUND => (
            "The main node does not have this table",
            "The node serves it to its readers empty and asks the main node for it again from time to time.".to_string(),
        ),
        _ => (
            "Select a partition",
            "Choose a partition from the list to see its rows.".to_string(),
        ),
    };

    rsx! {
        div { class: "rows-wrap",
            div { class: "empty-state",
                div { class: "empty-state__icon",
                    Icon { kind: IconKind::Layers }
                }
                div { class: "empty-state__title", "{title}" }
                div { class: "empty-state__sub", "{sub}" }
            }
        }
    }
}

fn render_empty_state(
    tables: &[TableModel],
    status_loaded: bool,
    status_error: Option<&str>,
    on_pick: impl FnMut(String) + Clone + 'static,
) -> Element {
    if !status_loaded {
        return match status_error {
            Some(err) => render_message("Can not reach the node", err),
            None => render_message("Connecting to node…", ""),
        };
    }

    let chips = tables.iter().take(8).map(|t| {
        let name = t.name.clone();
        let mut on_pick = on_pick.clone();
        rsx! {
            button {
                key: "{t.name}",
                class: "btn btn--sm",
                onclick: move |_| on_pick(name.clone()),
                Badge { text: t.name.clone(), tone: BadgeTone::Neutral }
            }
        }
    });

    let sub = if tables.is_empty() {
        "The node has no tables in this namespace: a table is replicated once a reader subscribes to it."
    } else {
        "Every table here is replicated because a reader subscribed to it. Choose one on the left, or below."
    };

    rsx! {
        div { class: "rows-col",
            div { class: "empty-state",
                div { class: "empty-state__icon",
                    Icon { kind: IconKind::Layers }
                }
                div { class: "empty-state__title", "Select a table to begin" }
                div { class: "empty-state__sub", "{sub}" }
                div { class: "empty-state__chips", {chips} }
            }
        }
    }
}
