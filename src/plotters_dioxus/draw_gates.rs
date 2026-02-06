use std::sync::Arc;
use dioxus::prelude::*;
use flow_gates::{Gate, GateGeometry};
use crate::{
    gate_store::{GateState, GateStateImplExt},
    plotters_dioxus::{PlotDrawable, gates::{gate_draft::GateDraft, gate_drag::{GateDragData, PointDragData}}, plot_helpers::PlotMapper}
};


#[derive(Clone, PartialEq, Copy)]
enum GateDragType {
    Point(PointDragData),
    Gate{gate_index: usize, gate_drag_data: GateDragData},
}

impl GateDragType {
    fn clone_with_point(self, point: (f32, f32)) -> Self {
        match self {
            GateDragType::Point(point_drag_data) => {
                GateDragType::Point(
                    PointDragData::clone_from_data(point, point_drag_data)
                )
            },
            GateDragType::Gate{gate_index, gate_drag_data} => {
                GateDragType::Gate{
                    gate_index,
                    gate_drag_data: GateDragData::clone_from_data(
                        point,
                    gate_drag_data
                )}
            },
        }
    }
}






#[component]
pub fn GateLayer(
    plot_map: ReadSignal<Option<PlotMapper>>,
    x_channel: ReadSignal<Arc<str>>,
    y_channel: ReadSignal<Arc<str>>,
) -> Element {
    let mut gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let mut draft_gate_coords = use_signal(|| Vec::<(f32, f32)>::new());
    let mut draft_gate = use_signal(|| None::<GateDraft>);
    let mut next_gate_id = use_signal(|| 0);
    let mut selected_gate_id = use_signal(|| None::<Arc<str>>);

    // convert clicked coords into a draft gate
    use_effect(move || {
        let cur_coords = draft_gate_coords();
        if cur_coords.len() > 0 {
            let gate_draft = GateDraft::new_polygon(cur_coords, x_channel(), y_channel());
            draft_gate.set(Some(gate_draft));
        } else {
            draft_gate.set(None);
        }
    });

    // for editing a gate's points
    let mut drag_data = use_signal(|| Option::<GateDragType>::None);

    // the list of finalised gates
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
                    view_box: "0 0 {&mapper.width()} {&mapper.height()}",
                    style: "position: absolute; top: 0; left: 0; z-index: 2; user-select: none; -webkit-user-select: none; cursor: crosshair;",
                    oncontextmenu: move |evt| evt.prevent_default(),
                    onclick: move |evt| {
                        if let Some(mapper) = plot_map() {
                            let local_coords = &evt.data.coordinates().element();
                            let norm_x = local_coords.x as f32;
                            let norm_y = local_coords.y as f32;
                            let (data_x, data_y) = mapper
                                .pixel_to_data(norm_x, norm_y, None, None);

                            let mut closest_gate = None;
                            if draft_gate.peek().is_none() {
                                let x_axis_title = x_channel();
                                let y_axis_title = y_channel();
                                if let Some(gates) = gate_store
                                    .get_gates_for_plot(x_axis_title, y_axis_title)
                                {
                                    let tolerance = mapper.get_data_tolerance(5.0);
                                    let mut closest_dist = std::f32::INFINITY;

                                    for gate in gates {
                                        if let Some(dist) = gate
                                            .is_point_on_perimeter((data_x, data_y), tolerance)
                                        {
                                            // println!("You clicked on a gate!");
                                            if dist < closest_dist {
                                                closest_dist = dist;
                                                closest_gate = Some(gate.clone());
                                            }

                                        }
                                    }
                                }
                            }
                            if closest_gate.is_none() {
                                // println!("You didn't click on a gate");
                                if selected_gate_id.peek().is_none() {
                                    draft_gate_coords.write().push((data_x, data_y));
                                } else {
                                    selected_gate_id.set(None);
                                }

                            } else {
                                let closest_gate = closest_gate.unwrap();
                                let gate_name = closest_gate.name.clone();
                                let gate_id = closest_gate.id.clone();
                                selected_gate_id.set(Some(gate_id));
                                println!("closest gate was {}", gate_name);
                            }

                        }
                    },
                    ondoubleclick: move |_| {
                        // Finalise the current gate
                        if let Some(curr_gate) = draft_gate.write().take() {
                            // last point is duplicated from the double click
                            let mut points = curr_gate.get_points();
                            points.pop();

                            let finalised_gate = match flow_gates::geometry::create_polygon_geometry(
                                points,
                                &*x_channel(),
                                &*y_channel(),
                            ) {
                                Ok(gate) => {
                                    let id = *next_gate_id.peek();
                                    Some(

                                        Gate::new(
                                            id.to_string(),
                                            id.to_string(),
                                            gate,
                                            x_channel(),
                                            y_channel(),
                                        ),
                                    )
                                }
                                Err(_) => {
                                    draft_gate_coords.write().clear();
                                    return;
                                }
                            };
                            gate_store
                                .add_gate(finalised_gate.unwrap(), None)
                                .expect("Failed to add gate to gate store");
                            draft_gate_coords.write().clear();
                            *next_gate_id.write() += 1;
                        }
                    },
                    onmousemove: move |evt| {
                        if let Some(data) = drag_data() {
                            let local_coords = &evt.data.coordinates().element();
                            let px = local_coords.x as f32;
                            let py = local_coords.y as f32;
                            let data_coords = plot_map
                                .as_ref()
                                .unwrap()
                                .pixel_to_data(px, py, None, None);
                            let new_data = data.clone_with_point(data_coords);
                            drag_data.set(Some(new_data));

                        }
                    },
                    onmouseup: move |evt| {
                        if let Some(data) = drag_data() {
                            let local_coords = &evt.data.coordinates().element();
                            let px = local_coords.x as f32;
                            let py = local_coords.y as f32;
                            let data_coords = plot_map
                                .as_ref()
                                .unwrap()
                                .pixel_to_data(px, py, None, None);

                            let new_data = data.clone_with_point(data_coords);
                            match new_data {
                                GateDragType::Point(point_drag_data) => {
                                    if let Some(selected_gate_id) = selected_gate_id() {
                                        gate_store
                                            .move_gate_point(
                                                selected_gate_id.into(),
                                                point_drag_data.point_index(),
                                                data_coords,
                                            )
                                            .expect("Gate Move Failed");
                                        drag_data.set(None);
                                    }
                                }
                                GateDragType::Gate { gate_index, gate_drag_data } => {
                                    todo!("move whole gate");
                                    drag_data.set(None);
                                }
                            }

                        }
                    },
                    onmousedown: move |evt| {
                        match evt.trigger_button() {
                            Some(dioxus_elements::input_data::MouseButton::Secondary) => {
                                selected_gate_id.set(None);
                                draft_gate_coords.set(vec![]);
                                draft_gate.set(None);

                            }
                            _ => {}
                        }
                    },
                    for (gate_index , gate) in gates.read().iter().enumerate() {
                        match &gate.geometry {
                            GateGeometry::Polygon { nodes: _, closed: _ } => {
                                let is_selected = match selected_gate_id() {
                                    Some(id) => if gate.id == id { true } else { false }
                                    None => false,
                                };
                                let is_being_edited = is_selected && drag_data.read().is_some();
                                let stroke = if is_selected { "red" } else { "cyan" };
                                let points: Vec<(f32, f32)> = gate
                                    .get_points()
                                    .iter()
                                    .map(|v| plot_map.as_ref().unwrap()
                                        .data_to_pixel(v.0, v.1, None, None))
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
                                        pointer_events: if is_selected { "all" } else { "none" },
                                        onmousedown: move |evt| {
                                            match evt.trigger_button() {
                                                Some(dioxus_elements::input_data::MouseButton::Primary) => {
                                                    let local_coords = &evt.data.coordinates().element();
                                                    let px = local_coords.x as f32;
                                                    let py = local_coords.y as f32;
                                                    let data_coords = plot_map()
                                                        .unwrap()
                                                        .pixel_to_data(px, py, None, None);
                                                    let gate_drag_data = GateDragData::new(data_coords, data_coords);
                                                    drag_data
                                                        .set(
                                                            Some(GateDragType::Gate {
                                                                gate_index,
                                                                gate_drag_data,
                                                            }),
                                                        );
                                                }
                                                Some(dioxus_elements::input_data::MouseButton::Secondary) => {
                                                    todo!("make context menu to delete points");
                                                }
                                                _ => return,
                                            }
                                        },
                                    }
                                    if is_selected {
                                        for (index , point) in points.iter().enumerate() {
                                            circle {
                                                key: "{gate.id}-{gate_index}",
                                                cx: "{point.0}",
                                                cy: "{point.1}",
                                                r: "4",
                                                fill: "red",
                                                cursor: "move",
                                                onmousedown: move |evt| {
                                                    match evt.trigger_button() {
                                                        Some(dioxus_elements::input_data::MouseButton::Primary) => {
                                                            let local_coords = &evt.data.coordinates().element();
                                                            let px = local_coords.x as f32;
                                                            let py = local_coords.y as f32;
                                                            let data_coords = plot_map()
                                                                .unwrap()
                                                                .pixel_to_data(px, py, None, None);
                                                            let point_drag_data = PointDragData::new(index, data_coords);
                                                            drag_data.set(Some(GateDragType::Point(point_drag_data)));
                                                        }
                                                        Some(dioxus_elements::input_data::MouseButton::Secondary) => {
                                                            println!("make context menu to add or delete points");

                                                        }
                                                        _ => {}
                                                    }
                                                },

                                            }
                                        }
                                        if is_being_edited {
                                            if let Some(drag_type) = drag_data() {
                                                match drag_type {
                                                    GateDragType::Point(data) => {
                                                        let n = points.len();
                                                        let point_index = data.point_index();
                                                        let idx_before = (point_index + n - 1) % n;
                                                        let idx_after = (point_index + 1) % n;
                                                        let p_prev = points[idx_before];
                                                        let p_next = points[idx_after];
                                                        let (prev_x, prev_y) = (p_prev.0, p_prev.1);
                                                        let (current_x, current_y) = data.loc();
                                                        let (mouse_x, mouse_y) = mapper
                                                            .data_to_pixel(current_x, current_y, None, None);
                                                        let (next_x, next_y) = (p_next.0, p_next.1);
                                                        rsx! {
                                                            polyline {
                                                                points: "{prev_x},{prev_y} {mouse_x},{mouse_y} {next_x},{next_y}",
                                                                stroke: "yellow",
                                                                stroke_width: "2",
                                                                stroke_dasharray: "4",
                                                                fill: "none",
                                                            }
                                                            circle {
                                                                cx: "{mouse_x}",
                                                                cy: "{mouse_y}",
                                                                r: "5",
                                                                fill: "rgba(255, 255, 0, 0.5)",
                                                                pointer_events: "none",
                                                            }
                                                        }
                                                    }
                                                    GateDragType::Gate { gate_index, gate_drag_data } => {
                                                        println!("Draw a ghost gate at the offset.");
                                                        rsx! {}
                                                    }
                                                }
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
                                    let (px, py) = mapper
                                        .data_to_pixel(v.0, v.1, None, None);
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