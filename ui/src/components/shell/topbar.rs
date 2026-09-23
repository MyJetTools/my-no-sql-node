use dioxus::prelude::*;

use crate::components::atoms::{Icon, IconKind};
use crate::models::DEFAULT_NAMESPACE;
use crate::storage;

#[derive(Clone, PartialEq)]
pub struct Crumb {
    pub label: String,
    pub active: bool,
}

/// A namespace the node replicates, as the namespace select offers it.
#[derive(Clone, PartialEq)]
pub struct NamespaceOption {
    pub name: String,
    pub tables_amount: usize,
    pub connected: bool,
}

#[component]
pub fn Topbar(
    crumbs: Vec<Crumb>,
    namespaces: Vec<NamespaceOption>,
    /// Empty string means the default namespace — the UI then sends no `ns` header at all.
    current_ns: String,
    on_namespace_change: EventHandler<String>,
) -> Element {
    let mut theme = use_signal(|| storage::load_theme().unwrap_or_else(|| "light".to_string()));
    let is_dark = theme.read().as_str() == "dark";

    let toggle_theme = move |_| {
        let next = if theme.read().as_str() == "dark" {
            "light"
        } else {
            "dark"
        };
        storage::save_theme(next);
        storage::apply_theme(next);
        theme.set(next.to_string());
    };

    let crumbs_iter = crumbs.into_iter().enumerate().map(|(i, c)| {
        let cls = if c.active {
            "topbar__crumb active"
        } else {
            "topbar__crumb"
        };
        let sep = if i > 0 {
            rsx! {
                span { class: "topbar__crumb-sep", "/" }
            }
        } else {
            rsx! {}
        };
        rsx! {
            {sep}
            span { class: cls, "{c.label}" }
        }
    });

    let theme_icon = if is_dark {
        IconKind::Sun
    } else {
        IconKind::Moon
    };

    // The default namespace is offered with an empty value: the UI then stores nothing and sends
    // no `ns` header.
    let ns_options = namespaces.into_iter().map(|ns| {
        let value = if ns.name == DEFAULT_NAMESPACE {
            String::new()
        } else {
            ns.name.clone()
        };
        let is_current = value == current_ns;
        let link = if ns.connected { "" } else { " · no main node" };
        let label = format!("{} · {}{}", ns.name, ns.tables_amount, link);

        rsx! {
            option { key: "{ns.name}", value: "{value}", selected: is_current, "{label}" }
        }
    });

    rsx! {
        header { class: "topbar",
            div { class: "topbar__breadcrumbs", {crumbs_iter} }
            div { class: "topbar__ns",
                Icon { kind: IconKind::Layers, class: "topbar__ns-icon".to_string() }
                select {
                    id: "topbar-namespace",
                    title: "Namespace the UI works in",
                    value: "{current_ns}",
                    onchange: move |evt| on_namespace_change.call(evt.value()),
                    {ns_options}
                }
            }
            div { class: "topbar__actions",
                button {
                    class: "topbar__icon-btn",
                    title: "Toggle theme",
                    onclick: toggle_theme,
                    Icon { kind: theme_icon }
                }
            }
        }
    }
}
