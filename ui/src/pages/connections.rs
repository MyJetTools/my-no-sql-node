use std::time::Duration;

use dioxus::prelude::*;

use crate::components::atoms::{Badge, BadgeTone, MiniChart, MiniChartSeries, StatePill, StateTone};
use crate::models::{
    ConnectionReaderContract, ConnectionsContract, DEFAULT_NAMESPACE, MainNodeConnectionContract,
};
use crate::utils::{format_bytes, format_bytes_per_sec, format_ping};

const MAX_POINTS: usize = 120;

#[derive(Clone, Copy)]
struct Sample {
    readers_outgoing: u64,
    main_node_incoming: u64,
    main_node_outgoing: u64,
}

impl Sample {
    fn new(snapshot: &ConnectionsContract) -> Self {
        Self {
            readers_outgoing: snapshot.outgoing_per_second,
            main_node_incoming: snapshot
                .main_nodes
                .iter()
                .map(|itm| itm.incoming_per_second)
                .sum(),
            main_node_outgoing: snapshot
                .main_nodes
                .iter()
                .map(|itm| itm.outgoing_per_second)
                .sum(),
        }
    }
}

#[derive(Default)]
struct ConnectionsState {
    started: bool,
    snapshot: Option<ConnectionsContract>,
    error: Option<String>,
    history: Vec<Sample>,
}

impl ConnectionsState {
    fn push(&mut self, snapshot: ConnectionsContract) {
        self.history.push(Sample::new(&snapshot));
        if self.history.len() > MAX_POINTS {
            let overflow = self.history.len() - MAX_POINTS;
            self.history.drain(0..overflow);
        }
        self.snapshot = Some(snapshot);
        self.error = None;
    }

    fn set_error(&mut self, err: String) {
        self.error = Some(err);
    }
}

#[component]
pub fn Connections() -> Element {
    let mut cs = use_signal(ConnectionsState::default);

    let started_val = cs.read().started;
    let on_mount = move |_| {
        if started_val {
            return;
        }
        cs.write().started = true;
        spawn(async move {
            loop {
                match crate::api::get_connections().await {
                    Ok(result) => cs.write().push(result),
                    Err(err) => {
                        dioxus_utils::console_log(format!("Connections error: {}", err));
                        cs.write().set_error(err.to_string());
                    }
                }
                dioxus_utils::js::sleep(Duration::from_secs(1)).await;
            }
        });
    };

    let cs_ra = cs.read();

    let content = match (cs_ra.snapshot.as_ref(), cs_ra.error.as_ref()) {
        (Some(snapshot), _) => render_connections(&cs_ra.history, snapshot),
        (None, Some(err)) => rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Can not load connections" }
                div { class: "empty-state__sub", "{err}" }
            }
        },
        (None, None) => rsx! {
            div { class: "empty-state",
                div { class: "empty-state__title", "Connecting to node…" }
            }
        },
    };

    rsx! {
        section { class: "page page--padded", onmounted: on_mount,
            div { class: "connections", {content} }
        }
    }
}

fn render_connections(history: &[Sample], snapshot: &ConnectionsContract) -> Element {
    // The UI works in one namespace at a time, so the readers list shows that namespace only.
    // The charts and the main node links stay node-wide.
    let namespace =
        crate::storage::load_namespace().unwrap_or_else(|| DEFAULT_NAMESPACE.to_string());

    let readers: Vec<&ConnectionReaderContract> = snapshot
        .readers
        .iter()
        .filter(|itm| itm.namespace == namespace)
        .collect();

    let readers_outgoing = snapshot.outgoing_per_second;

    let readers_series = vec![MiniChartSeries::new(
        history.iter().map(|s| s.readers_outgoing as f64).collect(),
        "mini-chart__line--out",
    )];
    let readers_max = history
        .iter()
        .map(|s| s.readers_outgoing)
        .max()
        .unwrap_or(0)
        .max(1) as f64;

    let current = Sample::new(snapshot);

    let main_node_series = vec![
        MiniChartSeries::new(
            history.iter().map(|s| s.main_node_incoming as f64).collect(),
            "mini-chart__line--in",
        ),
        MiniChartSeries::new(
            history.iter().map(|s| s.main_node_outgoing as f64).collect(),
            "mini-chart__line--out",
        ),
    ];
    let main_node_max = history
        .iter()
        .map(|s| s.main_node_incoming.max(s.main_node_outgoing))
        .max()
        .unwrap_or(0)
        .max(1) as f64;

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Replication from the main node · all namespaces" }
                div { class: "conn-legend",
                    span {
                        class: "conn-legend__item",
                        title: "The table data the main node sent, uncompressed - with Compress on, far fewer bytes cross the network",
                        span { class: "conn-legend__dot conn-legend__dot--in" }
                        "Table data (uncompressed) "
                        b { "{format_bytes_per_sec(current.main_node_incoming as f64)}" }
                    }
                    span { class: "conn-legend__item",
                        span { class: "conn-legend__dot conn-legend__dot--out" }
                        "Outgoing "
                        b { "{format_bytes_per_sec(current.main_node_outgoing as f64)}" }
                    }
                }
            }
            div { class: "card__body",
                MiniChart {
                    series: main_node_series,
                    max: main_node_max,
                    label: format_bytes_per_sec(main_node_max),
                    height: 140.0,
                }
            }
        }

        {render_main_nodes_table(&snapshot.main_nodes)}

        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Sent to readers · all readers" }
                div { class: "conn-legend",
                    span {
                        class: "conn-legend__item",
                        title: "TCP readers - the node does not count what HTTP readers get",
                        span { class: "conn-legend__dot conn-legend__dot--out" }
                        "Outgoing "
                        b { "{format_bytes_per_sec(readers_outgoing as f64)}" }
                    }
                }
            }
            div { class: "card__body",
                MiniChart {
                    series: readers_series,
                    max: readers_max,
                    label: format_bytes_per_sec(readers_max),
                }
            }
        }

        {render_readers_table(&readers, namespace.as_str())}
    }
}

fn render_main_nodes_table(main_nodes: &[MainNodeConnectionContract]) -> Element {
    let rows = main_nodes.iter().map(|itm| {
        let (label, tone) = if itm.connected {
            ("connected", StateTone::Ok)
        } else {
            ("disconnected", StateTone::Bad)
        };

        let ping = if itm.connected {
            format_ping(itm.ping)
        } else {
            "—".to_string()
        };

        rsx! {
            tr { key: "{itm.namespace}",
                td { "{itm.namespace}" }
                td {
                    StatePill { label: label.to_string(), tone }
                }
                td { class: "conn-table__num", "{ping}" }
                td { class: "conn-table__num", "{format_bytes_per_sec(itm.incoming_per_second as f64)}" }
                td { class: "conn-table__num", "{format_bytes_per_sec(itm.outgoing_per_second as f64)}" }
            }
        }
    });

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Main node links" }
                span { class: "card__subtitle", "one connection per namespace" }
            }
            div { class: "card__body",
                table { class: "conn-table",
                    thead {
                        tr {
                            th { "Namespace" }
                            th { "State" }
                            th { class: "conn-table__num", "Latency" }
                            th {
                                class: "conn-table__num",
                                title: "The table data the main node sent, uncompressed",
                                "Table data"
                            }
                            th { class: "conn-table__num", "Outgoing" }
                        }
                    }
                    tbody { {rows} }
                }
            }
        }
    }
}

fn render_readers_table(readers: &[&ConnectionReaderContract], namespace: &str) -> Element {
    if readers.is_empty() {
        return rsx! {
            div { class: "card",
                div { class: "card__header",
                    span { class: "card__title", "Readers" }
                    span { class: "card__subtitle", "0 connected · ns {namespace}" }
                }
                div { class: "card__body",
                    div { class: "empty-state",
                        div { class: "empty-state__title", "No readers in namespace {namespace}" }
                    }
                }
            }
        };
    }

    let rows = readers.iter().map(|reader| {
        let waiting_badges = reader.awaiting_tables.iter().map(|table| {
            rsx! {
                Badge { key: "waiting:{table}", text: format!("{table} · waiting"), tone: BadgeTone::Warn }
            }
        });
        let table_badges = reader.tables.iter().map(|table| {
            rsx! {
                Badge { key: "{table}", text: table.clone(), tone: BadgeTone::Reader }
            }
        });

        // The node counts what it sends over TCP only - an HTTP reader has no number.
        let outgoing = match reader.outgoing_per_second {
            Some(value) => format_bytes_per_sec(value as f64),
            None => "—".to_string(),
        };

        rsx! {
            tr { key: "{reader.id}",
                td { class: "conn-table__id", "{reader.id}" }
                td { "{reader.name}" }
                td { "{reader.ip}" }
                td { span { class: "badge-list", {waiting_badges} {table_badges} } }
                td { class: "conn-table__num", "{outgoing}" }
                td { class: "conn-table__num", "{format_bytes(reader.pending_to_send as f64)}" }
                td { class: "conn-table__num", "{reader.last_incoming_time}" }
            }
        }
    });

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Readers" }
                span { class: "card__subtitle", "{readers.len()} connected · ns {namespace}" }
            }
            div { class: "card__body",
                table { class: "conn-table",
                    thead {
                        tr {
                            th { "ID" }
                            th { "Name" }
                            th { "IP" }
                            th { "Tables" }
                            th { class: "conn-table__num", "Outgoing" }
                            th { class: "conn-table__num", "Pending" }
                            th { class: "conn-table__num", "Last incoming" }
                        }
                    }
                    tbody { {rows} }
                }
            }
        }
    }
}
