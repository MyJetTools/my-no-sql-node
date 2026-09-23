use dioxus::prelude::*;

use crate::components::atoms::{StatePill, StateTone};
use crate::models::{NamespaceStatusModel, ReaderModel, TableModel};
use crate::utils::format_ping;

/// The node's links to the main node - one TCP connection per namespace it replicates.
#[component]
pub fn MainNodeLinks(
    namespaces: Vec<NamespaceStatusModel>,
    tables: Vec<TableModel>,
    readers: Vec<ReaderModel>,
) -> Element {
    let count = namespaces.len();

    let rows = namespaces.into_iter().map(|ns| {
        let (label, tone) = if ns.connected_to_main_node {
            ("connected", StateTone::Ok)
        } else {
            ("disconnected", StateTone::Bad)
        };

        let ping = if ns.connected_to_main_node {
            format_ping(ns.main_node_ping)
        } else {
            "—".to_string()
        };

        let tables_amount = tables.iter().filter(|t| t.namespace == ns.name).count();
        let readers_amount = readers.iter().filter(|r| r.namespace == ns.name).count();

        rsx! {
            tr { key: "{ns.name}",
                td { "{ns.name}" }
                td {
                    StatePill { label: label.to_string(), tone }
                }
                td { class: "mono", "{ping}" }
                td { class: "mono", "{tables_amount}" }
                td { class: "mono", "{readers_amount}" }
            }
        }
    });

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Main node" }
                span { class: "card__subtitle", "{count} namespace links" }
            }
            table { class: "dt",
                thead {
                    tr {
                        th { "Namespace" }
                        th { "State" }
                        th { "Latency" }
                        th { "Tables" }
                        th { "Readers" }
                    }
                }
                tbody { {rows} }
            }
        }
    }
}
