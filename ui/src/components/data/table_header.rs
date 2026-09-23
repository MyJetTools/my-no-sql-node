use dioxus::prelude::*;

use crate::components::atoms::{Icon, IconKind};
use crate::models::TableModel;
use crate::utils::format_bytes;

#[component]
pub fn TableHeader(
    name: String,
    stats: Option<TableModel>,
    /// A partition is open - there are rows to reload.
    can_reload: bool,
    on_reload: EventHandler<()>,
) -> Element {
    let meta = if let Some(t) = stats {
        let size = format_bytes(t.data_size as f64);
        rsx! {
            div { class: "table-header__meta-item",
                "rows: " b { "{t.records_amount}" }
            }
            div { class: "table-header__meta-item",
                "partitions: " b { "{t.partitions_count}" }
            }
            div { class: "table-header__meta-item",
                "size: " b { "{size}" }
            }
        }
    } else {
        rsx! {
            div { class: "table-header__meta-item muted", "loading…" }
        }
    };

    let reload = if can_reload {
        rsx! {
            button {
                class: "topbar__icon-btn",
                title: "Reload rows",
                onclick: move |_| on_reload.call(()),
                Icon { kind: IconKind::RefreshCw }
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "table-header",
            div { class: "table-header__name",
                span { class: "table-header__title", "{name}" }
            }
            div { class: "table-header__meta", {meta} }
            div { class: "table-header__actions", {reload} }
        }
    }
}
