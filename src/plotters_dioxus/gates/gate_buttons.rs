use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub enum GateShapeStub {
    Polygon,
    Ellipse,
}

#[component]
pub fn NewGateButtons(callback: EventHandler<GateShapeStub>) -> Element {
    rsx! {
        button { onclick: move |_| callback.call(GateShapeStub::Polygon), "P" }
        button { onclick: move |_| callback.call(GateShapeStub::Ellipse), "E" }
    }
}
