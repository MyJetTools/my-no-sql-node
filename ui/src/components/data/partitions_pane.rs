use std::time::Duration;

use dioxus::prelude::*;

use crate::models::PartitionsDetailsContract;

/// How often the list is re-read, so the counters stay live.
const REFRESH: Duration = Duration::from_secs(3);

/// How many partitions the list shows at most - a table can have hundreds of thousands; the
/// filter narrows them down on the node.
const LIMIT: usize = 500;

#[derive(Default)]
struct PartitionsPaneState {
    filter: String,
    /// The answer to `filter` - an answer to an older filter is dropped.
    details: Option<PartitionsDetailsContract>,
    error: Option<String>,
}

impl PartitionsPaneState {
    /// Whether `result` changes anything - an answer to an older filter, or the same answer
    /// again, does not.
    fn is_changed_by(&self, filter: &str, result: &Result<PartitionsDetailsContract, String>) -> bool {
        if self.filter != filter {
            return false;
        }

        match result {
            Ok(details) => self.details.as_ref() != Some(details) || self.error.is_some(),
            Err(err) => self.error.as_ref() != Some(err),
        }
    }

    fn set_details(&mut self, result: Result<PartitionsDetailsContract, String>) {
        match result {
            Ok(details) => {
                self.details = Some(details);
                self.error = None;
            }
            Err(err) => self.error = Some(err),
        }
    }
}

/// Stores the answer - only if it changes something: every write re-renders the pane, and the
/// list is re-read every few seconds.
fn apply_details(
    mut cs: Signal<PartitionsPaneState>,
    filter: &str,
    result: Result<PartitionsDetailsContract, String>,
) {
    if cs.peek().is_changed_by(filter, &result) {
        cs.write().set_details(result);
    }
}

/// The partitions of one table. It owns its polling: keyed by the table in the parent, it lives
/// exactly as long as the table is selected, and its loop dies with it.
#[component]
pub fn PartitionsPane(
    table: String,
    selected: Option<String>,
    on_select: EventHandler<String>,
    /// Called once, after the first answer: the total amount of partitions and the first one.
    on_first_load: EventHandler<(u64, Option<String>)>,
) -> Element {
    let mut cs = use_signal(PartitionsPaneState::default);

    let table_for_loop = table.clone();
    use_hook(move || {
        spawn(async move {
            let mut first = true;
            loop {
                let filter = cs.peek().filter.clone();
                let result = crate::api::get_partition_details(&table_for_loop, &filter, LIMIT)
                    .await
                    .map_err(|err| err.message);

                if first {
                    first = false;
                    if let Ok(details) = result.as_ref() {
                        on_first_load.call((
                            details.total,
                            details.partitions.first().map(|itm| itm.partition_key.clone()),
                        ));
                    }
                }

                apply_details(cs, &filter, result);
                dioxus_utils::js::sleep(REFRESH).await;
            }
        });
    });

    let on_filter = {
        let table = table.clone();
        move |evt: Event<FormData>| {
            let filter = evt.value();
            cs.write().filter = filter.clone();

            // Answered right away rather than on the next tick of the loop.
            let table = table.clone();
            spawn(async move {
                let result = crate::api::get_partition_details(&table, &filter, LIMIT)
                    .await
                    .map_err(|err| err.message);
                apply_details(cs, &filter, result);
            });
        }
    };

    let cs_ra = cs.read();

    let header = match cs_ra.details.as_ref() {
        Some(details) if details.matched == details.total => {
            format!("Partitions · {}", details.total)
        }
        Some(details) => format!("Partitions · {} of {}", details.matched, details.total),
        None => "Partitions".to_string(),
    };

    let list = match (cs_ra.details.as_ref(), cs_ra.error.as_ref()) {
        (Some(details), _) => {
            let rows = details.partitions.iter().map(|itm| {
                let active = selected.as_deref() == Some(itm.partition_key.as_str());
                let cls = if active {
                    "partitions-pane__item active"
                } else {
                    "partitions-pane__item"
                };
                let records_str = super::format_compact_count(itm.records_count);
                let size_str = crate::utils::format_bytes(itm.data_size as f64);
                let pk = itm.partition_key.clone();
                rsx! {
                    div {
                        key: "{itm.partition_key}",
                        class: cls,
                        onclick: move |_| on_select.call(pk.clone()),
                        span { class: "partitions-pane__name", "{itm.partition_key}" }
                        span { class: "partitions-pane__meta",
                            span {
                                class: "partitions-pane__count",
                                title: "Records",
                                "{records_str}"
                            }
                            span {
                                class: "partitions-pane__size",
                                title: "Size",
                                "{size_str}"
                            }
                        }
                    }
                }
            });

            // The open partition is always in the list - on a deep link it can be far beyond the
            // first page, or filtered out.
            let pinned = match selected.as_ref() {
                Some(pk) if !details.partitions.iter().any(|itm| &itm.partition_key == pk) => {
                    let pk = pk.clone();
                    rsx! {
                        div {
                            class: "partitions-pane__item active",
                            title: "The open partition",
                            span { class: "partitions-pane__name", "{pk}" }
                            span { class: "partitions-pane__meta",
                                span { class: "partitions-pane__count", "open" }
                            }
                        }
                    }
                }
                Some(_) | None => rsx! {},
            };

            let shown = details.partitions.len() as u64;
            let more = if details.matched > shown {
                let rest = details.matched - shown;
                rsx! {
                    div { class: "pane-list__more",
                        "+ {rest} more - narrow the filter to find them"
                    }
                }
            } else {
                rsx! {}
            };

            rsx! {
                {pinned}
                {rows}
                {more}
            }
        }
        (None, Some(err)) => rsx! {
            div { class: "pane-list__more", "{err}" }
        },
        (None, None) => rsx! {
            div { class: "pane-list__more", "loading…" }
        },
    };

    rsx! {
        aside { class: "partitions-pane",
            div { class: "pane-header",
                span { class: "pane-header__title", "{header}" }
            }
            div { class: "pane-filter",
                input {
                    class: "filter-input",
                    placeholder: "filter partitions…",
                    value: "{cs_ra.filter}",
                    oninput: on_filter,
                }
            }
            div { class: "pane-list", {list} }
        }
    }
}
