use dioxus::prelude::*;
use serde_json::Value;

use super::cell::{cell_class, cell_string, pretty_json_html};
use crate::components::atoms::{Icon, IconKind};

#[derive(Clone, Copy, PartialEq, Default)]
pub enum DrawerView {
    #[default]
    Form,
    Json,
}

#[component]
pub fn RowDrawer(row: Value, on_close: EventHandler<()>) -> Element {
    let mut view = use_signal(|| DrawerView::Form);
    let view_val = *view.read();

    let body = match view_val {
        DrawerView::Form => render_form(&row),
        DrawerView::Json => render_json(&row),
    };

    let json_string = serde_json::to_string_pretty(&row).unwrap_or_default();
    let copy_handler = move |_| {
        copy_to_clipboard(&json_string);
    };

    let form_cls = if matches!(view_val, DrawerView::Form) {
        "btn-group__item active"
    } else {
        "btn-group__item"
    };
    let json_cls = if matches!(view_val, DrawerView::Json) {
        "btn-group__item active"
    } else {
        "btn-group__item"
    };

    rsx! {
        aside { class: "row-drawer",
            div { class: "row-drawer__header",
                span { class: "row-drawer__title", "Row Detail" }
                div { class: "btn-group",
                    button {
                        class: form_cls,
                        onclick: move |_| view.set(DrawerView::Form),
                        "Form"
                    }
                    button {
                        class: json_cls,
                        onclick: move |_| view.set(DrawerView::Json),
                        "JSON"
                    }
                }
                button {
                    class: "topbar__icon-btn",
                    onclick: move |_| on_close.call(()),
                    Icon { kind: IconKind::X }
                }
            }
            div { class: "row-drawer__body", {body} }
            div { class: "row-drawer__footer",
                button { class: "btn btn--ghost btn--sm", onclick: copy_handler,
                    Icon { kind: IconKind::Copy }
                    "Copy JSON"
                }
            }
        }
    }
}

fn render_form(row: &Value) -> Element {
    let Value::Object(map) = row else {
        return rsx! {
            pre { class: "row-drawer__json", "{row}" }
        };
    };

    let fields = map.iter().map(|(k, v)| {
        let s = cell_string(v);
        let mut cls = format!("row-drawer__value {}", cell_class(k, v));
        if k == "PartitionKey" {
            cls = "row-drawer__value pk".to_string();
        } else if k == "RowKey" {
            cls = "row-drawer__value rk".to_string();
        }
        rsx! {
            div { class: "row-drawer__field",
                div { class: "row-drawer__label", "{k}" }
                div { class: cls, "{s}" }
            }
        }
    });

    rsx! { div { {fields} } }
}

fn render_json(row: &Value) -> Element {
    let html = pretty_json_html(row);
    rsx! {
        pre { class: "row-drawer__json", dangerous_inner_html: "{html}" }
    }
}

/// The Clipboard API exists only in a secure context (https, localhost), and the node's UI is
/// usually opened over plain http - there the text goes through a hidden textarea instead. All in
/// JS and inside try/catch: an exception thrown from a web-sys import in an event handler leaves
/// the wasm runtime broken.
const COPY_JS: &str = r#"(function (text) {
    try {
        if (window.isSecureContext && navigator.clipboard) {
            navigator.clipboard.writeText(text).catch(function () {});
            return;
        }
        const area = document.createElement('textarea');
        area.value = text;
        area.setAttribute('readonly', '');
        area.style.position = 'fixed';
        area.style.opacity = '0';
        document.body.appendChild(area);
        area.select();
        document.execCommand('copy');
        document.body.removeChild(area);
    } catch (e) {
        console.error('Copy failed', e);
    }
})"#;

fn copy_to_clipboard(text: &str) {
    let text = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
    let _ = dioxus::document::eval(format!("{}({});", COPY_JS, text).as_str());
}
