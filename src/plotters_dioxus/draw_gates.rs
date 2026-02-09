use std::sync::Arc;
use dioxus::{prelude::*};
use flow_gates::{Gate};
use crate::{
    gate_store::{GateState, GateStateImplExt, GateStateStoreExt as _},
    plotters_dioxus::{PlotDrawable, gates::{gate_draft::GateDraft, gate_drag::{GateDragData, GateDragType, PointDragData}, gate_final::GateFinal, gate_styles::{GateShape, ShapeType}}, plot_helpers::PlotMapper}
};









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
    let gates = use_memo(move ||  {
            let gates = match gate_store.get_gates_for_plot(x_channel(), y_channel()) {
            Some(g) => g,
            None => vec![],
        };
        next_gate_id.set(gates.len());
        gates
    });

    use_effect(move || {
        let dd = &*drag_data.read();
        if let Some(selected_gate) = selected_gate_id.peek().clone(){
            let gate_key = selected_gate.into();
            if let Some(mut gate) =
                gate_store.gate_registry().get_mut(&gate_key)
                {
                    match dd {
                        Some(data) => match data{
                            GateDragType::Point(point_drag_data) => gate.set_drag_point(Some(*point_drag_data)),
                            GateDragType::Gate(gate_drag_data) => {
                                // gate.set_drag_self(Some(*gate_drag_data));
                            },
                        },
                        None => {
                            gate.set_drag_point(None);
                            gate.set_drag_self(None);
                        },
                    }
                }
                
        };
        
        
    });
    
    
   
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

                            let mut clicked_gate = None;
                            if draft_gate.peek().is_none() && drag_data.peek().is_none() {
                                let x_axis_title = x_channel();
                                let y_axis_title = y_channel();
                                if let Some(gates) = gate_store
                                    .get_gates_for_plot(x_axis_title, y_axis_title)
                                {
                                    clicked_gate = was_gate_clicked((norm_x, norm_y), &mapper, &gates);
                                }
                            }
                            if clicked_gate.is_none() {
                                if selected_gate_id.peek().is_none() {
                                    draft_gate_coords.write().push((data_x, data_y));
                                } else {
                                    let curr_selected = selected_gate_id.take().unwrap();
                                    let gate_key = curr_selected.into();
                                    if let Some(mut gate) =
                                    gate_store.gate_registry().get_mut(&gate_key)
                                    {
                                        gate.set_selected(false);
                                    }
                                }
                            } else {
                                if let Some(curr_selected) = selected_gate_id.take() {
                                    let gate_key = curr_selected.into();
                                    if let Some(mut gate) =
                                    gate_store.gate_registry().get_mut(&gate_key)
                                    {
                                        gate.set_selected(false);
                                    }
                                }

                                let closest_gate = clicked_gate.unwrap();
                                let gate_name = closest_gate.name.clone();
                                let gate_id = closest_gate.id.clone();
                                selected_gate_id.set(Some(gate_id.clone()));
                                let gate_key = gate_id.into();
                                if let Some(mut gate) = gate_store.gate_registry().get_mut(&gate_key) {
                                    gate.set_selected(true);
                                }

                            }

                        }
                    },
                    ondoubleclick: move |_| {
                        if let Some(curr_gate) = draft_gate.write().take() {
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
                                GateDragType::Gate(gate_drag_data) => {
                                    if let Some(selected_gate_id) = selected_gate_id() {
                                        let offset = gate_drag_data.offset();
                                        gate_store
                                            .move_gate(selected_gate_id.into(), offset)
                                            .expect("Gate Move Failed");
                                        drag_data.set(None);
                                    }
                                    drag_data.set(None);
                                }
                            }
                        }
                    },
                    onmousedown: move |evt| {
                        match evt.trigger_button() {
                            Some(dioxus_elements::input_data::MouseButton::Primary) => {
                                if let Some(mapper) = plot_map() {
                                    let local_coords = &evt.data.coordinates().element();
                                    let norm_x = local_coords.x as f32;
                                    let norm_y = local_coords.y as f32;
                                    let pixel_coords = (norm_x, norm_y);
                                    let data_coords = mapper.pixel_to_data(norm_x, norm_y, None, None);
                                    if selected_gate_id.peek().is_some() && draft_gate.peek().is_none()
                                        && drag_data.peek().is_none()
                                    {
                                        let x_axis_title = x_channel();
                                        let y_axis_title = y_channel();
                                        if let Some(gates) = gate_store
                                            .get_gates_for_plot(x_axis_title, y_axis_title)
                                        {
                                            let clicked_gate = was_gate_clicked(
                                                pixel_coords,
                                                &mapper,
                                                &gates,
                                            );
                                            if clicked_gate.is_some() {
                                                let data = GateDragData::new(data_coords, data_coords);
                                                drag_data.set(Some(GateDragType::Gate(data)));
                                            }
                                        }
                                    }
                                }
                            }
                            Some(dioxus_elements::input_data::MouseButton::Secondary) => {
                                if let Some(curr_selected) = selected_gate_id.take() {
                                    let gate_key = curr_selected.into();
                                    if let Some(gate) = gate_store().gate_registry.get_mut(&gate_key) {
                                        gate.set_selected(false);
                                    }
                                }
                                draft_gate_coords.set(vec![]);
                                draft_gate.set(None);
                            }
                            _ => {}
                        }
                    },

                    for (gate_index , gate) in gates.iter().enumerate() {
                        for (point_index , element) in gate.draw_self().into_iter().enumerate() {
                            RenderShape {
                                shape: element,
                                gate_id: &gate.id.clone(),
                                gate_index,
                                shape_index: point_index,
                                drag_data,
                                plot_map,

                            }
                        }
                    }
                    match draft_gate() {
                        Some(draft) => {
                            let id = "draft".to_string();
                            rsx! {
                                for (i , element) in draft.draw_self().into_iter().enumerate() {
                                    RenderShape {
                                        shape: element,
                                        gate_id: &id,
                                        gate_index: 0,
                                        shape_index: i,
                                        drag_data,
                                        plot_map,
                                    }
                                }
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

fn was_gate_clicked(click_coords: (f32, f32), mapper: &PlotMapper, gates: &[GateFinal]) -> Option<GateFinal> {

    let (data_x, data_y) = mapper
        .pixel_to_data(click_coords.0, click_coords.1, None, None);

    let mut closest_gate = None;

        let tolerance = mapper.get_data_tolerance(5.0);
        let mut closest_dist = std::f32::INFINITY;
        for gate in gates {
            if let Some(dist) = gate
                .is_point_on_perimeter((data_x, data_y), tolerance)
            {
                if dist < closest_dist {
                    closest_dist = dist;
                    closest_gate = Some(gate.clone());
                }
            }
        }
    
    closest_gate

}

#[component]
fn RenderShape(
    shape: GateShape, 
    gate_id: String, 
    gate_index: usize, 
    shape_index: usize,
    drag_data: Signal<Option<GateDragType>>,
    plot_map: ReadSignal<Option<PlotMapper>>,
) -> Element {

    
    
    if let Some(mapper) = &*plot_map.read(){
    let transform = {
        match &*drag_data.read(){
            Some(GateDragType::Gate(data)) => {
                let offset = data.offset();
                let p_start = mapper.data_to_pixel(0.0, 0.0, None, None);
                let p_current = mapper.data_to_pixel(offset.0, offset.1, None, None);
                let dx = p_current.0 - p_start.0;
                let dy = p_current.1 - p_start.1;
                format!("translate({} {})", -dx, -dy)
            },
            _ => format!("none"),
        }
    };
    match shape {
        GateShape::PolyLine { points, style, shape_type } => {
            let transform = {
                if let ShapeType::GhostGate(offset) = shape_type {
                    let p_start = mapper.data_to_pixel(0.0, 0.0, None, None);
                    let p_current = mapper.data_to_pixel(offset.0, offset.1, None, None);
                    let dx = p_current.0 - p_start.0;
                    let dy = p_current.1 - p_start.1;
                    format!("translate({} {})", -dx, -dy)

                } else {
                    String::new()
                }
            };
            let mapped = points
                .iter()
                .map(|(x, y)| {
                    let p = mapper.data_to_pixel(*x, *y, None, None);
                    format!("{},{}", p.0, p.1)
                })
                .collect::<Vec<_>>()
                .join(" ");
            rsx! {
                g { transform,
                    polyline {
                        key: "{gate_id}-{gate_index}-{shape_index}",
                        points: "{mapped}",
                        stroke: style.stroke,
                        stroke_width: style.stroke_width,
                        stroke_dasharray: if style.dashed { "4" } else { "none" },
                        fill: style.fill,
                    }
                }
            }
        }
        GateShape::Circle { center, radius, fill, shape_type } => {
            let p = mapper.data_to_pixel(center.0, center.1, None, None);
            // let transform = {
            //     if let ShapeType::GhostGate(offset) = shape_type {
            //         let p_start = mapper.data_to_pixel(0.0, 0.0, None, None);
            //         let p_current = mapper.data_to_pixel(offset.0, offset.1, None, None);
            //         let dx = p_current.0 - p_start.0;
            //         let dy = p_current.1 - p_start.1;
            //         format!("translate({} {})", -dx, -dy)

            //     } else {
            //         String::new()
            //     }
            // };
            rsx! {
                g { transform,

                    circle {
                        key: "{gate_id}-{gate_index}-{shape_index}",
                        cx: "{p.0}",
                        cy: "{p.1}",
                        r: radius,
                        fill,
                        onmousedown: move |evt| {
                            match shape_type {
                                ShapeType::Point(index) => {
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
                                }
                                _ => {}
                            }
                        },
                    }
                }
            }
        },
        GateShape::Polygon { points, style, shape_type } => {
            let mapped_points = points
                .iter()
                .map(|(x, y)| {
                    let p = mapper.data_to_pixel(*x, *y, None, None);
                    format!("{},{}", p.0, p.1)
                })
                .collect::<Vec<_>>()
                .join(" ");
            // let transform = {
            //     if let ShapeType::GhostGate(offset) = shape_type {
            //         let p_start = mapper.data_to_pixel(0.0, 0.0, None, None);
            //         let p_current = mapper.data_to_pixel(offset.0, offset.1, None, None);
            //         let dx = p_current.0 - p_start.0;
            //         let dy = p_current.1 - p_start.1;
            //         format!("translate({} {})", -dx, -dy)

            //     } else {
            //         String::new()
            //     }
            // };
            rsx! {
                g { transform,
                    polygon {
                        key: "{gate_id}-{gate_index}-{shape_index}",
                        points: mapped_points,
                        stroke: style.stroke,
                        stroke_width: style.stroke_width,
                        stroke_dasharray: if style.dashed { "4" } else { "none" },
                        fill: style.fill,
                    }
                }
            }
        }

    }
} else {
    rsx!{}
}
}