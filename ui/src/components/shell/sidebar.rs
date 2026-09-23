use dioxus::prelude::*;

use crate::AppRoute;
use crate::components::atoms::{Icon, IconKind};

#[component]
pub fn Sidebar(
    active: SidebarSection,
    version: String,
    location: String,
    /// Tables of the namespace the UI is pointed at - what the Tables page lists.
    tables_count: usize,
    readers_count: usize,
    /// Readers of the namespace the UI is pointed at - what the Connections page lists.
    readers_in_current_ns: usize,
    /// Readers per namespace, biggest first. Empty while the status has not arrived yet.
    readers_by_namespace: Vec<(String, usize)>,
    online: bool,
) -> Element {
    let dot_class = if online {
        "sidebar__live-dot"
    } else {
        "sidebar__live-dot offline"
    };
    let live_text = if online {
        format!("Live · {} readers", readers_count)
    } else {
        "Offline".to_string()
    };

    let brand_sub = match (version.is_empty(), location.is_empty()) {
        (true, _) => "read-only node".to_string(),
        (false, true) => format!("v{version} · node"),
        (false, false) => format!("v{version} · {location}"),
    };

    // Only worth the line when there is something to disambiguate: a node running a single
    // namespace says nothing new by naming it.
    let ns_breakdown = if online && readers_by_namespace.len() > 1 {
        let items = readers_by_namespace.into_iter().map(|(namespace, amount)| {
            rsx! {
                span { class: "sidebar__live-ns", key: "{namespace}",
                    span { class: "sidebar__live-ns-name", "{namespace}" }
                    span { class: "sidebar__live-ns-count", "{amount}" }
                }
            }
        });

        rsx! {
            div { class: "sidebar__live-by-ns", {items} }
        }
    } else {
        rsx! {}
    };

    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar__brand",
                div { class: "sidebar__logo",
                    img {
                        class: "sidebar__logo-img",
                        src: asset!("/public/favicon.svg"),
                        alt: "MyNoSql",
                    }
                }
                div {
                    div { class: "sidebar__brand-name", "MyNoSql Node" }
                    div { class: "sidebar__brand-sub", "{brand_sub}" }
                }
            }
            nav { class: "sidebar__nav",
                Link {
                    to: AppRoute::Home {},
                    class: nav_class(active == SidebarSection::Overview),
                    Icon { kind: IconKind::Activity, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Overview" }
                }
                Link {
                    to: AppRoute::Data {},
                    class: nav_class(active == SidebarSection::Tables),
                    Icon { kind: IconKind::Database, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Tables" }
                    span { class: "sidebar__nav-count", "{tables_count}" }
                }
                Link {
                    to: AppRoute::Connections {},
                    class: nav_class(active == SidebarSection::Connections),
                    Icon { kind: IconKind::Plug, class: "sidebar__nav-icon".to_string() }
                    span { class: "sidebar__nav-label", "Connections" }
                    span { class: "sidebar__nav-count", "{readers_in_current_ns}" }
                }
            }
            div { class: "sidebar__foot",
                div { class: "sidebar__live",
                    div { class: "sidebar__live-line",
                        span { class: dot_class }
                        span { "{live_text}" }
                    }
                    {ns_breakdown}
                }
            }
        }
    }
}

fn nav_class(active: bool) -> &'static str {
    if active {
        "sidebar__nav-item active"
    } else {
        "sidebar__nav-item"
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SidebarSection {
    Overview,
    Tables,
    Connections,
}
