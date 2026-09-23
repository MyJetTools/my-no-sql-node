use dioxus::prelude::*;

use crate::utils::format_bytes_per_sec;

#[component]
pub fn Sparkline(values: Vec<u64>, #[props(default)] bytes_label: bool) -> Element {
    let width: f64 = 200.0;
    let height: f64 = 36.0;
    let count = values.len().max(1) as f64;
    let bar_w = (width / count).max(0.5);

    let max = values.iter().copied().max().unwrap_or(0) as f64;
    let coef = if max == 0.0 {
        0.0
    } else {
        (height - 2.0) / max
    };

    let bars = values.iter().enumerate().map(|(i, &v)| {
        let h = (v as f64 * coef).max(0.0);
        let x = i as f64 * bar_w;
        let y = height - h;
        let w = (bar_w - 0.5).max(0.5);
        rsx! {
            rect {
                class: "sparkline__bar",
                x: "{x:.2}",
                y: "{y:.2}",
                width: "{w:.2}",
                height: "{h:.2}",
                rx: "0.5",
            }
        }
    });

    // The label is html over the svg, which stretches with `preserveAspectRatio="none"` - text
    // inside it would stretch along.
    let label_el = if bytes_label && max > 0.0 {
        let txt = format_bytes_per_sec(max);
        rsx! {
            span { class: "sparkline__label", "{txt}" }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "sparkline-wrap",
            svg { class: "sparkline", view_box: "0 0 {width} {height}", preserve_aspect_ratio: "none",
                {bars}
            }
            {label_el}
        }
    }
}
