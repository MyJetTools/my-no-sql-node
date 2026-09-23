use dioxus::prelude::*;

use crate::components::atoms::{
    Badge, BadgeTone, Sparkline, StateTone, StatusDot, classify_reader,
};
use crate::models::{ReaderModel, TABLE_SYNC_NOT_FOUND, TableModel};
use crate::settings::HealthThresholds;

#[derive(Clone, Copy, PartialEq)]
enum ReaderFilter {
    All,
    Healthy,
    Issues,
}

#[derive(Default)]
struct ReadersState {
    filter: Option<ReaderFilter>,
    show_all: bool,
}

impl ReadersState {
    fn current_filter(&self) -> ReaderFilter {
        self.filter.unwrap_or(ReaderFilter::All)
    }
}

/// A table as a reader has it: `None` - replicated, the reader gets its data.
fn table_problem(tables: &[TableModel], namespace: &str, table: &str) -> Option<&'static str> {
    let state = tables
        .iter()
        .find(|itm| itm.namespace == namespace && itm.name == table)
        .map(|itm| itm.sync_state.as_str());

    match state {
        Some(TABLE_SYNC_NOT_FOUND) => Some("not on main"),
        Some(_) | None => None,
    }
}

/// The reader waits for a table, or reads one the main node does not have.
fn has_table_problem(reader: &ReaderModel, tables: &[TableModel]) -> bool {
    !reader.awaiting_tables.is_empty()
        || reader
            .tables
            .iter()
            .any(|t| table_problem(tables, &reader.namespace, t).is_some())
}

#[component]
pub fn ReadersTable(readers: Vec<ReaderModel>, tables: Vec<TableModel>) -> Element {
    let thresholds = *use_context::<Signal<HealthThresholds>>().read();
    let mut cs = use_signal(ReadersState::default);
    let cs_ra = cs.read();
    let active_filter = cs_ra.current_filter();
    let show_all = cs_ra.show_all;
    drop(cs_ra);

    let total = readers.len();

    let filtered: Vec<ReaderModel> = readers
        .into_iter()
        .filter(|r| {
            let tone =
                classify_reader(&r.last_incoming_time, thresholds.warn_ms, thresholds.bad_ms);
            let healthy = matches!(tone, StateTone::Ok) && !has_table_problem(r, &tables);
            match active_filter {
                ReaderFilter::All => true,
                ReaderFilter::Healthy => healthy,
                ReaderFilter::Issues => !healthy,
            }
        })
        .collect();

    let filtered_total = filtered.len();
    let display_limit = if show_all { filtered_total } else { 14 };
    let display: Vec<ReaderModel> = filtered.into_iter().take(display_limit).collect();

    let rows = display.into_iter().map(|r| {
        let tone = classify_reader(&r.last_incoming_time, thresholds.warn_ms, thresholds.bad_ms);
        let tone_for_time = tone;
        // The tables it waits for first - they are the ones worth seeing.
        let mut all_badges: Vec<(String, BadgeTone)> = r
            .awaiting_tables
            .iter()
            .map(|t| (format!("{t} · waiting"), BadgeTone::Warn))
            .collect();
        for t in r.tables.iter() {
            match table_problem(&tables, &r.namespace, t) {
                Some(problem) => all_badges.insert(0, (format!("{t} · {problem}"), BadgeTone::Warn)),
                None => all_badges.push((t.clone(), BadgeTone::Reader)),
            }
        }

        let overflow = if all_badges.len() > 3 {
            Some(all_badges.len() - 3)
        } else {
            None
        };

        let table_badges = all_badges.into_iter().take(3).map(|(text, tone)| {
            rsx! {
                Badge { text, tone }
            }
        });

        let overflow_badge = if let Some(n) = overflow {
            rsx! {
                Badge { text: format!("+{n}"), tone: BadgeTone::Neutral }
            }
        } else {
            rsx! {}
        };

        let (last_class, last_style) = match tone_for_time {
            StateTone::Bad => ("mono", "color: var(--danger);"),
            StateTone::Warn => ("mono", "color: var(--warn);"),
            StateTone::Ok => ("mono muted", ""),
        };

        let sent = r.sent_per_second.clone();

        rsx! {
            tr { key: "{r.id}",
                td { class: "mono muted", "{r.id}" }
                td {
                    div { style: "display:flex; align-items:center; gap:8px;",
                        StatusDot { tone }
                        span { "{r.name}" }
                    }
                }
                td { class: "mono muted", "{r.namespace}" }
                td { class: "mono muted",
                    span { class: "dt-ellipsis", "{r.ip}" }
                }
                td { style: "max-width:220px;",
                    Sparkline { values: sent, bytes_label: true }
                }
                td {
                    span { class: "badge-list",
                        {table_badges}
                        {overflow_badge}
                    }
                }
                td { class: "{last_class}", style: "{last_style}", "{r.last_incoming_time}" }
            }
        }
    });

    let mut on_filter = move |f: ReaderFilter| {
        let mut w = cs.write();
        w.filter = Some(f);
        w.show_all = false;
    };

    let footer = if filtered_total > display_limit {
        let remaining = filtered_total - display_limit;
        rsx! {
            div { class: "card__footer",
                a { onclick: move |_| { cs.write().show_all = true; },
                    "Show all ({remaining} more)"
                }
            }
        }
    } else if show_all && filtered_total > 14 {
        rsx! {
            div { class: "card__footer",
                a { onclick: move |_| { cs.write().show_all = false; },
                    "Collapse"
                }
            }
        }
    } else {
        rsx! {}
    };

    let chip_cls = move |f: ReaderFilter| -> &'static str {
        if f == active_filter {
            "chip active"
        } else {
            "chip"
        }
    };

    rsx! {
        div { class: "card",
            div { class: "card__header",
                span { class: "card__title", "Readers" }
                div { style: "display:flex; align-items:center; gap:10px;",
                    span { class: "card__subtitle", "{total} connected · all namespaces" }
                    div { class: "chip-group",
                        button {
                            class: chip_cls(ReaderFilter::All),
                            onclick: move |_| on_filter(ReaderFilter::All),
                            "All"
                        }
                        button {
                            class: chip_cls(ReaderFilter::Healthy),
                            onclick: move |_| on_filter(ReaderFilter::Healthy),
                            "Healthy"
                        }
                        button {
                            class: chip_cls(ReaderFilter::Issues),
                            onclick: move |_| on_filter(ReaderFilter::Issues),
                            "Issues"
                        }
                    }
                }
            }
            table { class: "dt",
                thead {
                    tr {
                        th { "Id" }
                        th { "Client" }
                        th { "Namespace" }
                        th { "Address" }
                        th { "Traffic" }
                        th { "Tables" }
                        th { "Last incoming" }
                    }
                }
                tbody { {rows} }
            }
            {footer}
        }
    }
}
