use dioxus::prelude::*;

use crate::plotters_dioxus::gates::gate_types::GateType;

const GATE_CONFIG: &[(GateType, &str)] = &[
    (GateType::Polygon, "P"),
    (GateType::Ellipse, "E"),
    (GateType::Rectangle, "R"),
    (GateType::Line(None), "L"),
];

#[component]
pub fn NewGateButtons(callback: EventHandler<GateType>) -> Element {
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
