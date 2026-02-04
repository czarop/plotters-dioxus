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
                    onmousemove: move |evt| {},
                    for gate in &*gates.read() {
                        match &gate.geometry {
                            GateGeometry::Polygon { nodes: _, closed: _ } => {
                                let is_selected = match selected_gate_id() {
                                    Some(id) => if gate.id == id { true } else { false }
                                    None => false,
                                };
                                let stroke = if is_selected { "red" } else { "cyan" };
                                let points: Vec<(f32, f32)> = gate
                                    .get_points()
                                    .iter()
                                    .map(|v| plot_map.as_ref().unwrap().map_to_svg(v.0, v.1))
                                    .collect();
                                let points_attr = points
                                    .iter()
                                    .map(|(px, py)| { format!("{px},{py}") })
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                rsx! {
                                    polygon {
                                        points: "{points_attr}",
                                        fill: "rgba(0, 255, 255, 0.2)",
                                        stroke,
                                        stroke_width: "2",
                                        pointer_events: "none",
                                    }
                                    if is_selected {
                                        for (idx , point) in points.iter().enumerate() {
                                            circle {
                                                key: "{gate.id}-{idx}",
                                                cx: "{point.0}",
                                                cy: "{point.1}",
                                                r: "4",
                                                fill: "red",
                                                cursor: "move",
                                                onmousedown: move |evt| {
                                                    let local_coords = &evt.data.coordinates().element();
                                                    let px = local_coords.x as f32;
                                                    let py = local_coords.y as f32;
                                                    let data_coords = plot_map.as_ref().unwrap().map_to_svg(px, py);
                                                    drag_data.set(Some((idx, data_coords)));
                                                },

                                            }
                                        }
                                    }
                                }
                            }

                            _ => rsx! {},
                        }
                    }

                    match draft_gate() {
                        Some(draft) => {
                            let mut points_attr = draft
                                .get_points()
                                .iter()
                                .map(|v| {
                                    let (px, py) = mapper.map_to_svg(v.0, v.1);
                                    format!("{px},{py}")
                                })
                                .collect::<Vec<_>>();
                            if let Some(first) = points_attr.first() && points_attr.len() > 2 {
                                points_attr.push(first.clone());
                            }
                            let draft_string = points_attr.join(" ");

                            match points_attr.len() {
                                0 => rsx! {},
                                1 => {
                                    let points: Vec<&str> = draft_string.split(",").collect();
                                    rsx! {
                                        circle {
                                            cx: "{points[0]}",
                                            cy: "{points[1]}",
                                            r: "3",
                                            fill: "red",

                                        }
                                    }
                                }
                                2 => rsx! {
                                    polyline {
                                        points: "{draft_string}",
                                        stroke: "red",
                                        stroke_width: "2",
                                        fill: "none",

                                    }
                                },
                                _ => rsx! {
                                    polygon {
                                        points: "{draft_string}",
                                        fill: "rgba(0, 255, 255, 0.2)",
                                        stroke: "red",
                                        stroke_width: "2",
                                        pointer_events: "none",
                                    }
                                },

                            }
                        }
                        None => rsx! {},
                    }

                }
            },
            None => rsx! {},
        }

    }
}
