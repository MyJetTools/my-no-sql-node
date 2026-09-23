use dioxus::prelude::*;
use std::collections::HashMap;

use crate::models::{TABLE_SYNC_NOT_FOUND, TABLE_SYNC_PENDING, TableModel};
use crate::utils::format_bytes;

/// The tables of the namespace - each one is on the node because a reader subscribed to it.
#[component]
pub fn TablesPane(
    tables: Vec<TableModel>,
    selected: String,
    /// Readers of the namespace subscribed to each table right now.
    readers: HashMap<String, usize>,
    /// Readers of the namespace waiting for each table to arrive from the main node.
    waiting: HashMap<String, usize>,
    on_select: EventHandler<String>,
) -> Element {
    let mut filter = use_signal(String::new);
    let filter_ra = filter.read();
    let needle = filter_ra.to_lowercase();
    let needle_empty = needle.is_empty();
    drop(filter_ra);

    let visible: Vec<TableModel> = tables
        .into_iter()
        .filter(|t| needle_empty || t.name.to_lowercase().contains(&needle))
        .collect();

    let total = visible.len();

    // Summed over what is on screen, so filtering the list narrows the total too.
    let total_size: u64 = visible.iter().map(|t| t.data_size).sum();
    let header_count = format!("{} · {}", total, format_bytes(total_size as f64));

    let rows = visible.into_iter().map(|t| {
        let active = t.name == selected;
        let cls = if active {
            "tables-pane__item active"
        } else {
            "tables-pane__item"
        };

        let (dot_cls, dot_title) = dot_of(
            &t,
            readers.get(&t.name).copied().unwrap_or(0),
            waiting.get(&t.name).copied().unwrap_or(0),
        );

        // A table which is not replicated has no data to count: say why instead.
        let (part_str, size_str) = match t.sync_state.as_str() {
            TABLE_SYNC_PENDING => ("waiting".to_string(), String::new()),
            TABLE_SYNC_NOT_FOUND => ("not on main".to_string(), String::new()),
            _ => (
                super::format_compact_count(t.partitions_count),
                format_bytes(t.data_size as f64),
            ),
        };

        let name = t.name.clone();
        rsx! {
            div { key: "{t.name}", class: cls, onclick: move |_| on_select.call(name.clone()),
                span { class: dot_cls, title: dot_title }
                span { class: "tables-pane__name", "{t.name}" }
                span { class: "tables-pane__meta",
                    span {
                        class: "tables-pane__count",
                        title: "Partitions",
                        "{part_str}"
                    }
                    span {
                        class: "tables-pane__size",
                        title: "Data size",
                        "{size_str}"
                    }
                }
            }
        }
    });

    rsx! {
        aside { class: "tables-pane",
            div { class: "pane-header",
                span { class: "pane-header__title", "Tables" }
                span { class: "pane-header__count", "{header_count}" }
            }
            div { class: "pane-filter",
                input {
                    class: "filter-input",
                    placeholder: "filter tables…",
                    value: "{filter.read()}",
                    oninput: move |evt| filter.set(evt.value()),
                }
            }
            div { class: "pane-list", {rows} }
        }
    }
}

/// The dot next to a table and what it says: who reads the table, and whether they get it.
fn dot_of(table: &TableModel, readers: usize, waiting: usize) -> (&'static str, String) {
    let readers_text = match readers {
        1 => "1 reader".to_string(),
        n => format!("{} readers", n),
    };

    match (table.sync_state.as_str(), readers + waiting) {
        (TABLE_SYNC_PENDING, _) => (
            "tables-pane__dot tables-pane__dot--warn",
            format!("{} waiting - not replicated yet", waiting),
        ),
        (TABLE_SYNC_NOT_FOUND, 0) => (
            "tables-pane__dot",
            "No readers right now - not on the main node".to_string(),
        ),
        (TABLE_SYNC_NOT_FOUND, _) => (
            "tables-pane__dot tables-pane__dot--warn",
            format!("{} - the main node does not have it", readers_text),
        ),
        (_, 0) => (
            "tables-pane__dot",
            "No readers right now - still replicated".to_string(),
        ),
        (_, _) => ("tables-pane__dot has-writer", readers_text),
    }
}
