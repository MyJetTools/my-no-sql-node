use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeTone {
    Reader,
    Warn,
    Neutral,
}

impl BadgeTone {
    fn class(&self) -> &'static str {
        match self {
            BadgeTone::Reader => "badge--reader",
            BadgeTone::Warn => "badge--warn",
            BadgeTone::Neutral => "badge--neutral",
        }
    }
}

#[component]
pub fn Badge(text: String, tone: BadgeTone) -> Element {
    rsx! {
        span { class: "badge {tone.class()}", "{text}" }
    }
}
