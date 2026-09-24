use dioxus::prelude::*;

use super::{StatePill, StateTone};
use crate::utils::format_ping;

/// Round trip from which a connection reads as slow.
const LATENCY_WARN_MICROS: i64 = 100_000;
/// Round trip from which a connection reads as bad.
const LATENCY_BAD_MICROS: i64 = 300_000;

pub fn classify_latency(micros: i64) -> StateTone {
    if micros >= LATENCY_BAD_MICROS {
        StateTone::Bad
    } else if micros >= LATENCY_WARN_MICROS {
        StateTone::Warn
    } else {
        StateTone::Ok
    }
}

/// Round trip of a reader's connection, as the reader measured it and reported with its
/// `PingWithLatency`. `None` is a reader which has not reported one: an HTTP reader, a reader on
/// an SDK without the packet, or one that has just connected.
#[component]
pub fn LatencyPill(latency: Option<i64>) -> Element {
    match latency {
        Some(micros) => rsx! {
            StatePill { label: format_ping(micros), tone: classify_latency(micros) }
        },
        None => rsx! {
            span { class: "mono muted", title: "Not reported by the reader", "—" }
        },
    }
}
