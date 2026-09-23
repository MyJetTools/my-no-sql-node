use dioxus::prelude::*;

use crate::components::atoms::{Badge, BadgeTone};

#[component]
pub fn TableToolbar(filter_value: Signal<String>, reader_count: usize, waiting_count: usize) -> Element {
    let waiting = if waiting_count > 0 {
        rsx! {
            Badge { text: format!("{waiting_count} waiting"), tone: BadgeTone::Warn }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "table-toolbar-new",
            input {
                class: "filter-input",
                placeholder: "filter rows… any text of the row",
                value: "{filter_value.read()}",
                oninput: move |evt| filter_value.set(evt.value()),
            }
            div { class: "table-toolbar-new__spacer" }
            div { class: "table-toolbar-new__group",
                Badge { text: format!("{reader_count} readers"), tone: BadgeTone::Reader }
                {waiting}
            }
        }
    }
}
