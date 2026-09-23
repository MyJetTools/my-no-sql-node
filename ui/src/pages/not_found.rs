use dioxus::prelude::*;

use crate::AppRoute;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let path = format!("/{}", segments.join("/"));

    rsx! {
        section { class: "page page--padded",
            div { class: "empty-state",
                div { class: "empty-state__title", "404 · Not found" }
                div { class: "empty-state__sub", "The UI has no page at {path}." }
                div { class: "empty-state__chips",
                    Link { to: AppRoute::Home {}, class: "btn btn--sm", "Back to overview" }
                }
            }
        }
    }
}
