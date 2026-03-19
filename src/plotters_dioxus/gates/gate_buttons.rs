use dioxus::prelude::*;

use crate::plotters_dioxus::gates::gate_types::DrawableGateType;

const GATE_CONFIG: &[(DrawableGateType, &str)] = &[
    (DrawableGateType::Polygon, "P"),
    (DrawableGateType::Ellipse, "E"),
    (DrawableGateType::Rectangle, "R"),
    (DrawableGateType::Line(None), "L"),
    (DrawableGateType::Bisector, "B"),
    (DrawableGateType::Quadrant, "Q"),
    (DrawableGateType::SkewedQuadrant, "S"),
];

#[component]
pub fn NewGateButtons(callback: EventHandler<DrawableGateType>) -> Element {
    let mut selected_index = use_signal(|| 0);
    let selected_style = "background-color: orange";

    rsx! {
        for (i , (t , d_text)) in GATE_CONFIG.iter().enumerate() {
            button {
                style: if selected_index() == i { Some(selected_style) } else { None },
                onclick: move |_| {
                    selected_index.set(i);
                    callback.call(*t);

                },
                "{d_text}"
            }
        }

    }
}
