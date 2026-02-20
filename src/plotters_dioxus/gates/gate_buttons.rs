
use dioxus::prelude::*;

use crate::plotters_dioxus::gates::gate_types::GateType;


#[component]
pub fn NewGateButtons(callback: EventHandler<GateType>) -> Element {
    rsx! {
        button { onclick: move |_| callback.call(GateType::Polygon), "P" }
        button { onclick: move |_| callback.call(GateType::Ellipse), "E" }
        button { onclick: move |_| callback.call(GateType::Rectangle), "R" }
    }
}
