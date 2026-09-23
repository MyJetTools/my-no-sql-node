use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum IconKind {
    Activity,
    Database,
    Plug,
    Moon,
    Sun,
    RefreshCw,
    ShieldCheck,
    AlertTriangle,
    X,
    Copy,
    Layers,
}

impl IconKind {
    fn paths(&self) -> &'static str {
        match self {
            IconKind::Activity => r#"<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>"#,
            IconKind::Database => {
                r#"<ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/><path d="M3 12c0 1.66 4 3 9 3s9-1.34 9-3"/>"#
            }
            IconKind::Plug => {
                r#"<path d="M9 2v6"/><path d="M15 2v6"/><path d="M6 8h12v4a6 6 0 0 1-12 0z"/><path d="M12 18v4"/>"#
            }
            IconKind::Moon => r#"<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>"#,
            IconKind::Sun => {
                r#"<circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>"#
            }
            IconKind::RefreshCw => {
                r#"<polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/><path d="M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>"#
            }
            IconKind::ShieldCheck => {
                r#"<path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/><polyline points="9 12 11 14 15 10"/>"#
            }
            IconKind::AlertTriangle => {
                r#"<path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>"#
            }
            IconKind::X => {
                r#"<line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/>"#
            }
            IconKind::Copy => {
                r#"<rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>"#
            }
            IconKind::Layers => {
                r#"<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>"#
            }
        }
    }
}

#[component]
pub fn Icon(kind: IconKind, #[props(default = String::new())] class: String) -> Element {
    let paths = kind.paths();
    let full_class = if class.is_empty() {
        "icon".to_string()
    } else {
        format!("icon {}", class)
    };
    rsx! {
        svg {
            class: full_class,
            view_box: "0 0 24 24",
            dangerous_inner_html: "{paths}",
        }
    }
}
