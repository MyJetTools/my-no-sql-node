use dioxus::prelude::*;

use crate::AppContext;
use crate::components::atoms::{StateTone, classify_reader};
use crate::components::overview::{
    HealthBanner, HealthTone, MainNodeLinks, ReaderHealthGrid, ReadersTable, StatsRow,
    TableCoverage,
};
use crate::models::{
    InitializedModel, NamespaceStatusModel, ReaderModel, TABLE_SYNC_NOT_FOUND, TABLE_SYNC_PENDING,
    TableModel,
};
use crate::settings::HealthThresholds;
use crate::utils::format_ping;

/// Node-wide picture: the links to the main node, the readers and the tables they read. The
/// status itself is polled by the shell.
#[component]
pub fn Home() -> Element {
    let app_ctx = use_context::<Signal<AppContext>>();
    let thresholds = *use_context::<Signal<HealthThresholds>>().read();

    let ctx_ra = app_ctx.read();

    let content = match (ctx_ra.status.as_ref(), ctx_ra.status_error.as_ref()) {
        (Some(status), err) => {
            render_overview(&status.initialized, thresholds, err.map(|itm| itm.as_str()))
        }
        (None, Some(err)) => render_message("Can not reach the node", err.as_str()),
        (None, None) => render_message("Connecting to node…", ""),
    };

    rsx! {
        section { class: "page page--padded",
            div { class: "overview", {content} }
        }
    }
}

fn render_message(title: &str, sub: &str) -> Element {
    rsx! {
        div { class: "empty-state",
            div { class: "empty-state__title", "{title}" }
            div { class: "empty-state__sub", "{sub}" }
        }
    }
}

/// `node_error` - the last poll failed: what is shown is the last status the node gave.
fn render_overview(
    init: &InitializedModel,
    thresholds: HealthThresholds,
    node_error: Option<&str>,
) -> Element {
    let (tone, headline, sub) = match node_error {
        Some(err) => (
            HealthTone::Bad,
            "The node does not answer".to_string(),
            format!("{err}. The figures below are the last ones it gave."),
        ),
        None => compute_health(init, thresholds),
    };
    let (aside_label, aside_value) = main_node_summary(&init.namespaces);

    rsx! {
        HealthBanner { tone, headline, sub, aside_label, aside_value }
        StatsRow {
            tables: init.tables.clone(),
            readers: init.readers.clone(),
            namespaces: init.namespaces.clone(),
        }
        div { class: "two-col",
            div { class: "card",
                div { class: "card__header",
                    span { class: "card__title", "Reader health" }
                    span { class: "card__subtitle", "live · {init.readers.len()} clients" }
                }
                div { class: "card__body",
                    ReaderHealthGrid { readers: init.readers.clone() }
                }
            }
            div { class: "card",
                div { class: "card__header",
                    span { class: "card__title", "Table subscriptions" }
                    span { class: "card__subtitle", "tables · readers" }
                }
                div { class: "card__body",
                    TableCoverage {
                        tables: init.tables.clone(),
                        readers: init.readers.clone(),
                    }
                }
            }
        }
        MainNodeLinks {
            namespaces: init.namespaces.clone(),
            tables: init.tables.clone(),
            readers: init.readers.clone(),
        }
        ReadersTable { readers: init.readers.clone(), tables: init.tables.clone() }
    }
}

/// What the right side of the banner says about the main node.
fn main_node_summary(namespaces: &[NamespaceStatusModel]) -> (String, String) {
    let connected: Vec<&NamespaceStatusModel> = namespaces
        .iter()
        .filter(|n| n.connected_to_main_node)
        .collect();

    let value = if connected.len() < namespaces.len() {
        format!("{}/{} linked", connected.len(), namespaces.len())
    } else {
        let slowest = connected
            .iter()
            .map(|n| n.main_node_ping)
            .max()
            .unwrap_or(0);
        format!("linked · {}", format_ping(slowest))
    };

    ("Main node".to_string(), value)
}

/// Worst first: a link to the main node down, a reader stalled, a table readers get no data of,
/// a reader slow.
fn compute_health(
    init: &InitializedModel,
    thresholds: HealthThresholds,
) -> (HealthTone, String, String) {
    let disconnected: Vec<&str> = init
        .namespaces
        .iter()
        .filter(|n| !n.connected_to_main_node)
        .map(|n| n.name.as_str())
        .collect();

    if !disconnected.is_empty() {
        return (
            HealthTone::Bad,
            "No connection to the main node".to_string(),
            format!(
                "Readers of {} get no updates until the link is back: {}.",
                if disconnected.len() == 1 {
                    "this namespace"
                } else {
                    "these namespaces"
                },
                disconnected.join(", ")
            ),
        );
    }

    let mut bad = 0usize;
    let mut warn = 0usize;
    for r in init.readers.iter() {
        match classify_reader(&r.last_incoming_time, thresholds.warn_ms, thresholds.bad_ms) {
            StateTone::Bad => bad += 1,
            StateTone::Warn => warn += 1,
            StateTone::Ok => {}
        }
    }

    if bad > 0 {
        return (
            HealthTone::Bad,
            format!(
                "{} reader{} stalled",
                bad,
                if bad == 1 { " is" } else { "s are" }
            ),
            format!(
                "Nothing has been heard from them for over {} seconds.",
                thresholds.bad_ms / 1000
            ),
        );
    }

    let missing = tables_read_in_state(&init.tables, &init.readers, TABLE_SYNC_NOT_FOUND);
    if !missing.is_empty() {
        return (
            HealthTone::Warn,
            format!(
                "Readers subscribe to {} the main node does not have",
                plural(missing.len(), "a table", "tables")
            ),
            format!(
                "They get an empty table until it is created on the main node: {}.",
                missing.join(", ")
            ),
        );
    }

    let pending = tables_read_in_state(&init.tables, &init.readers, TABLE_SYNC_PENDING);
    if !pending.is_empty() {
        return (
            HealthTone::Warn,
            format!(
                "Readers wait for {} from the main node",
                plural(pending.len(), "a table", "tables")
            ),
            format!(
                "The node asked the main node for it and nothing has arrived yet: {}.",
                pending.join(", ")
            ),
        );
    }

    if warn > 0 {
        return (
            HealthTone::Warn,
            format!(
                "{} reader{} slow",
                warn,
                if warn == 1 { " is" } else { "s are" }
            ),
            format!(
                "Nothing has been heard from them for over {} seconds.",
                thresholds.warn_ms / 1000
            ),
        );
    }

    (
        HealthTone::Ok,
        "All systems nominal".to_string(),
        "The main node is linked, every reader is live and gets the tables it subscribed to."
            .to_string(),
    )
}

/// `namespace/table` of every table in `sync_state` a reader subscribed to or waits for.
fn tables_read_in_state(
    tables: &[TableModel],
    readers: &[ReaderModel],
    sync_state: &str,
) -> Vec<String> {
    tables
        .iter()
        .filter(|t| t.sync_state == sync_state)
        .filter(|t| {
            readers.iter().any(|r| {
                r.namespace == t.namespace
                    && (r.tables.contains(&t.name) || r.awaiting_tables.contains(&t.name))
            })
        })
        .map(|t| format!("{}/{}", t.namespace, t.name))
        .collect()
}

fn plural(amount: usize, one: &str, many: &str) -> String {
    if amount == 1 {
        one.to_string()
    } else {
        format!("{} {}", amount, many)
    }
}
