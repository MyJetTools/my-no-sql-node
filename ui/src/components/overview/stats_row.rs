use dioxus::prelude::*;

use crate::components::atoms::{DeltaTone, Stat, StatTone, StateTone, classify_reader};
use crate::models::{NamespaceStatusModel, ReaderModel, TABLE_SYNC_REPLICATED, TableModel};
use crate::settings::HealthThresholds;
use crate::utils::{format_bytes, format_ping};

#[component]
pub fn StatsRow(
    tables: Vec<TableModel>,
    readers: Vec<ReaderModel>,
    namespaces: Vec<NamespaceStatusModel>,
) -> Element {
    let thresholds = *use_context::<Signal<HealthThresholds>>().read();

    let table_count = tables.len();
    let replicated = tables
        .iter()
        .filter(|t| t.sync_state == TABLE_SYNC_REPLICATED)
        .count();
    let total_rows: u64 = tables.iter().map(|t| t.records_amount).sum();
    let total_size: u64 = tables.iter().map(|t| t.data_size).sum();

    let mut ok = 0;
    let mut warn = 0;
    let mut bad = 0;
    for r in readers.iter() {
        match classify_reader(&r.last_incoming_time, thresholds.warn_ms, thresholds.bad_ms) {
            StateTone::Ok => ok += 1,
            StateTone::Warn => warn += 1,
            StateTone::Bad => bad += 1,
        }
    }

    let reader_count = readers.len();
    let reader_tone = if bad > 0 {
        StatTone::Bad
    } else if warn > 0 {
        StatTone::Warn
    } else {
        StatTone::Ok
    };

    let pending_to_send: u64 = readers.iter().map(|r| r.pending_to_send).sum();

    let connected = namespaces
        .iter()
        .filter(|n| n.connected_to_main_node)
        .count();
    let link_tone = if connected < namespaces.len() {
        StatTone::Bad
    } else {
        StatTone::Ok
    };
    let slowest_ping = namespaces
        .iter()
        .filter(|n| n.connected_to_main_node)
        .map(|n| n.main_node_ping)
        .max();
    let ping_value = match slowest_ping {
        Some(ping) => format_ping(ping),
        None => "—".to_string(),
    };

    rsx! {
        div { class: "stats-row",
            Stat {
                label: "Main node".to_string(),
                value: format!("{connected}/{}", namespaces.len()),
                unit: "linked".to_string(),
                delta: "namespaces connected".to_string(),
                delta_tone: tone_to_delta(link_tone),
                tone: link_tone,
            }
            Stat {
                label: "Latency".to_string(),
                value: ping_value,
                delta: "slowest ping to the main node".to_string(),
                tone: StatTone::Info,
            }
            Stat {
                label: "Readers".to_string(),
                value: format!("{reader_count}"),
                unit: "connected".to_string(),
                delta: format!("{ok} ok · {warn} slow · {bad} stalled"),
                delta_tone: tone_to_delta(reader_tone),
                tone: reader_tone,
            }
            Stat {
                label: "Pending to send".to_string(),
                value: format_bytes(pending_to_send as f64),
                delta: "buffered for readers".to_string(),
                tone: StatTone::Info,
            }
            Stat {
                label: "Tables".to_string(),
                value: format!("{table_count}"),
                unit: "subscribed".to_string(),
                delta: format!("{replicated} replicated"),
                tone: StatTone::Info,
            }
            Stat {
                label: "Rows in memory".to_string(),
                value: format_compact(total_rows),
                unit: "rows".to_string(),
                delta: format_bytes(total_size as f64),
                tone: StatTone::Ok,
            }
        }
    }
}

fn tone_to_delta(tone: StatTone) -> DeltaTone {
    match tone {
        StatTone::Ok => DeltaTone::Ok,
        StatTone::Warn => DeltaTone::Warn,
        StatTone::Bad => DeltaTone::Bad,
        StatTone::Info => DeltaTone::Neutral,
    }
}

pub fn format_compact(n: u64) -> String {
    let v = n as f64;
    if v >= 1_000_000_000.0 {
        format!("{:.1}B", v / 1_000_000_000.0)
    } else if v >= 1_000_000.0 {
        format!("{:.1}M", v / 1_000_000.0)
    } else if v >= 1_000.0 {
        format!("{:.1}K", v / 1_000.0)
    } else {
        format!("{}", n)
    }
}
