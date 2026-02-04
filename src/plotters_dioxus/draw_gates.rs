use std::sync::Arc;

use dioxus::prelude::*;
use flow_gates::{GateGeometry, plotmap::PlotMapper};
use flow_plots::plots::traits::PlotDrawable;

use crate::{
    gate_store::{GateState, GateStateImplExt},
    plotters_dioxus::gate_helpers::GateDraft,
};

#[component]
pub fn GateLayer(
    plot_map: ReadSignal<Option<PlotMapper>>,
    x_channel: ReadSignal<Arc<str>>,
    y_channel: ReadSignal<Arc<str>>,
    draft_gate: ReadSignal<Option<GateDraft>>,
    selected_gate_id: ReadSignal<Option<Arc<str>>>,
) -> Element {
    let mut gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let mut drag_data = use_signal(|| Option::<(usize, (f32, f32))>::None);

    let gates = use_memo(
        move || match gate_store.get_gates_for_plot(x_channel(), y_channel()) {
            Some(g) => g,
            None => vec![],
        },
    );

    rsx! {
        match plot_map() {
            Some(mapper) => rsx! {
                svg {
                    width: "100%",
                    height: "100%",
                    view_box: "0 0 {&mapper.view_width} {&mapper.view_height}",
                    style: "position: absolute; top: 0; left: 0; pointer-events: none; z-index: 2;",
                    onmousemove: 
            },
            None => rsx! {},
        }

    }
}
