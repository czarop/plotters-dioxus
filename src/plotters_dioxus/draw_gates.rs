use crate::plotters_dioxus::{
        PlotDrawable, gates::{
            GateState, gate_draft::GateDraft, gate_drag::{GateDragData, GateDragType, PointDragData, RotationData}, gate_draw_helpers::rectangle::map_rect_to_pixels, gate_store::{GateStateImplExt, GateStateStoreExt as _}, gate_traits::DrawableGate, gate_types::{GateRenderShape, GateType, ShapeType}
        }, plot_helpers::PlotMapper
    };

use dioxus::prelude::*;
use flow_gates::Gate;
use std::sync::{Arc, Mutex};

#[component]
pub fn GateLayer(
    plot_map: ReadSignal<Option<PlotMapper>>,
    x_channel: ReadSignal<Arc<str>>,
    y_channel: ReadSignal<Arc<str>>,
) -> Element {
    let mut gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let mut draft_gate_coords = use_signal(|| Vec::<(f32, f32)>::new());
    let mut next_gate_id = use_signal(|| 0);
    let mut selected_gate_id = use_signal(|| None::<Arc<str>>);
    let current_gate_type = use_context::<Signal<GateType>>();

    let mut gates = use_signal(|| Vec::<Arc<Mutex<dyn DrawableGate>>>::new());

    // convert clicked coords into a draft gate
    let draft_gate = use_memo(move || {
        
        if let GateType::Polygon = &*current_gate_type.read(){
            let cur_coords = draft_gate_coords();
            if cur_coords.len() > 0 {
                let gate_draft = GateDraft::new_polygon(cur_coords, x_channel(), y_channel());
                Some(gate_draft)
            } else {
                None
            }
        } else {
            None
        }
    });

    // for editing a gate's points
    let mut drag_data = use_signal(|| Option::<GateDragType>::None);

    use_effect(move || {
        let x_param = x_channel();
        let y_param= y_channel();
        let _ = gate_store.match_gates_to_plot(x_param, y_param).inspect_err(|e| println!("{}", e.to_string()));
    });


    // the list of finalised gates
    use_effect(move || {
        let g = match gate_store.get_gates_for_plot(x_channel(), y_channel()) {
            Some(g) => g,
            None => vec![],
        };
        next_gate_id.set(g.len());
        gates.set(g);
    });

    use_effect(move || {
        if let Some(GateDragType::Point(point_drag_data)) = &*drag_data.read(){
            if let Some(selected_gate) = selected_gate_id.peek().clone() {
                let gate_key = selected_gate.into();
                if let Some(gate) = gate_store.gate_registry().get_mut(&gate_key) {
                    gate.lock().unwrap().set_drag_point(Some(point_drag_data.clone()));
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
                                clicked_gate = was_gate_clicked(
                                    (norm_x, norm_y),
                                    &mapper,
                                    &*gates.read(),
                                );
                            }
                            if clicked_gate.is_none() {
                                if selected_gate_id.peek().is_none() {
                                    if &GateType::Polygon == &*current_gate_type.peek() {
                                        draft_gate_coords.write().push((data_x, data_y));
                                    }
                                } else if drag_data.peek().is_none() {
                                    let curr_selected = selected_gate_id.take().unwrap();
                                    let gate_key = curr_selected.into();
                                    if let Some(gate) = gate_store.gate_registry().get_mut(&gate_key)
                                    {
                                        gate.lock().unwrap().set_selected(false);
                                    }
                                }
                            } else {
                                if let Some(curr_selected) = selected_gate_id.take() {
                                    let gate_key = curr_selected.into();
                                    if let Some(gate) = gate_store.gate_registry().get_mut(&gate_key)
                                    {
                                        gate.lock().unwrap().set_selected(false);
                                    }
                                }
                                let closest_gate = clicked_gate.unwrap();
                                let gate_id = closest_gate.lock().unwrap().get_id().clone();
                                selected_gate_id.set(Some(gate_id.clone()));
                                let gate_key = gate_id.into();
                                if let Some(gate) = gate_store.gate_registry().get_mut(&gate_key) {
                                    gate.lock().unwrap().set_selected(true);
                                }
                            }
                        }
                        drag_data.set(None);
                    },
                    ondoubleclick: move |evt| {
                        let local_coords = evt.data.coordinates().element();
                        let px = local_coords.x as f32;
                        let py = local_coords.y as f32;
                        let x_param = &*x_channel.peek();
                        let y_param = &*y_channel.peek();
                        let (dx, dy) = mapper.pixel_to_data(px, py, None, None);

                        let points = {
                            if let Some(curr_gate) = &*draft_gate.peek() {
                                let mut points = curr_gate.get_points();
                                draft_gate_coords.write().clear();
                                points.pop();
                                Some(points)
                            } else {
                                None
                            }
                        };

                        let geometry_res = current_gate_type
                            .peek()
                            .to_gate_geometry(&mapper, px, py, x_param, y_param, points);

                        let geo = if let GateType::Line(_) = &*current_gate_type.peek() {
                            GateType::Line(Some(dy))
                        } else {
                            current_gate_type.peek().cloned()
                        };
                        match geometry_res {
                            Ok(gate) => {
                                let id = *next_gate_id.peek();
                                let g = Gate::new(
                                    id.to_string(),
                                    id.to_string(),
                                    gate,
                                    x_channel(),
                                    y_channel(),
                                );
                                gate_store
                                    .add_gate(g, None, geo)
                                    .expect("Failed to add gate to gate store");
                                *next_gate_id.write() += 1;
                            }
                            Err(_) => {}
                        };
                    },
                    onmousemove: move |evt| {
                        evt.stop_propagation();
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
                            if let Some(selected_gate_id) = &*selected_gate_id.peek() {
                                match new_data {
                                    GateDragType::Point(point_drag_data) => {
                                        gate_store
                                            .move_gate_point(
                                                selected_gate_id.clone().into(),
                                                point_drag_data.point_index(),
                                                data_coords,
                                            )
                                            .expect("Gate Move Failed");

                                    }
                                    GateDragType::Gate(gate_drag_data) => {
                                        let offset = gate_drag_data.offset();
                                        gate_store
                                            .move_gate(selected_gate_id.clone().into(), offset)
                                            .expect("Gate Move Failed");
                                    }
                                    GateDragType::Rotation(rotation_data) => {
                                        gate_store
                                            .rotate_gate(
                                                selected_gate_id.clone().into(),
                                                rotation_data.current_loc(),
                                            )
                                            .expect("Gate Move Failed");
                                    }
                                }
                            }
                        }
                    },
                    onmousedown: move |evt| {
                        evt.stop_propagation();
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
                                        let clicked_gate = was_gate_clicked(
                                            pixel_coords,
                                            &mapper,
                                            &*gates.read(),
                                        );
                                        if clicked_gate.is_some()
                                            && clicked_gate
                                                .as_ref()
                                                .unwrap()
                                                .lock()
                                                .unwrap()
                                                .is_selected()
                                        {
                                            let data = GateDragData::new(
                                                clicked_gate.unwrap().lock().unwrap().get_id().clone(),
                                                data_coords,
                                                data_coords,
                                            );
                                            drag_data.set(Some(GateDragType::Gate(data)));
                                        }
                                    }
                                }
                            }
                            Some(dioxus_elements::input_data::MouseButton::Secondary) => {
                                if let Some(curr_selected) = selected_gate_id.take() {
                                    let gate_key = curr_selected.into();
                                    if let Some(gate) = gate_store.gate_registry().get_mut(&gate_key)
                                    {
                                        let mut g = gate.lock().unwrap();
                                        g.set_selected(false);
                                        g.set_drag_point(None);
                                        draft_gate_coords.write().clear();
                                    }
                                } else {
                                    draft_gate_coords.write().clear();
                                }
                            }
                            _ => {}
                        }
                    },

                    for (gate_index , gate) in (&*gates.read()).iter().enumerate() {
                        {
                            let g = gate.lock().unwrap();
                            rsx! {
                                for (shape_index , shape) in g.draw_self().into_iter().enumerate() {
                                    RenderShape {
                                        shape,
                                        gate_id: g.get_id().clone(),
                                        gate_index,
                                        shape_index,
                                        drag_data,
                                        plot_map,
                                    }
                                }
                            }
                        }
                    }
                    match draft_gate() {
                        Some(draft) => {
                            let id = "draft".to_string();
                            rsx! {

                                RenderDraftGate {
                                    key: "{id}",
                                    gate: draft.clone(),
                                    drag_data,
                                    plot_map,
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

fn was_gate_clicked(
    click_coords: (f32, f32),
    mapper: &PlotMapper,
    gates: &[Arc<Mutex<dyn DrawableGate>>],
) -> Option<Arc<Mutex<dyn DrawableGate>>> {
    let (data_x, data_y) = mapper.pixel_to_data(click_coords.0, click_coords.1, None, None);

    let mut closest_gate = None;

    let tolerance = mapper.get_data_tolerance(5.0);
    let mut closest_dist = std::f32::INFINITY;
    for gate in gates {
        if let Some(dist) = gate.lock().unwrap().is_point_on_perimeter((data_x, data_y), tolerance) {
            if dist < closest_dist {
                closest_dist = dist;
                closest_gate = Some(gate.clone());
            }
        }
    }

    closest_gate
}

#[component]
fn RenderDraftGate(
    gate: GateDraft,
    drag_data: Signal<Option<GateDragType>>,
    plot_map: ReadSignal<Option<PlotMapper>>,
) -> Element {
    rsx! {
        for (shape_index , shape) in gate.draw_self().into_iter().enumerate() {
            RenderShape {
                shape,
                gate_id: Arc::from("draft"),
                gate_index: 0,
                shape_index,
                drag_data,
                plot_map,
            }
        }
    }
}

#[component]
fn RenderShape(
    shape: GateRenderShape,
    gate_id: Arc<str>,
    gate_index: usize,
    shape_index: usize,
    drag_data: Signal<Option<GateDragType>>,
    plot_map: ReadSignal<Option<PlotMapper>>,
) -> Element {
    if let Some(mapper) = &*plot_map.read() {
        let transform = {
            match &*drag_data.read() {
                Some(GateDragType::Gate(data)) => {
                    if *gate_id == *data.gate_id() {
                        let offset = data.offset();
                        let p_start = mapper.data_to_pixel(0.0, 0.0, None, None);
                        let p_current = mapper.data_to_pixel(offset.0, offset.1, None, None);
                        let dx = p_current.0 - p_start.0;
                        let dy = p_current.1 - p_start.1;
                        format!("translate({} {})", -dx, -dy)
                    } else {
                        format!("none")
                    }
                }
                Some(GateDragType::Rotation(data)) => {
                    if *gate_id == *data.gate_id() {
                        let rotation_degs = data.rotation_deg();
                        let c = data.pivot_point();
                        let (cx, cy) = mapper.data_to_pixel(c.0, c.1, None, None);
                        format!("rotate({rotation_degs} {cx} {cy})")
                    } else {
                        format!("none")
                    }
                },
                _ => format!("none"),
            }
        };
        match shape {
            GateRenderShape::PolyLine {
                points,
                style,
                shape_type:_,
            } => {
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
            GateRenderShape::Circle {
                center,
                radius,
                fill,
                shape_type,
            } => {
                let p = mapper.data_to_pixel(center.0, center.1, None, None);
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
            }
            GateRenderShape::Polygon {
                points,
                style,
                shape_type:_,
            } => {
                let mapped_points = points
                    .iter()
                    .map(|(x, y)| {
                        let p = mapper.data_to_pixel(*x, *y, None, None);
                        format!("{},{}", p.0, p.1)
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
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
            GateRenderShape::Ellipse {
                center,
                radius_x,
                radius_y,
                degrees_rotation,
                style,
                shape_type:_,
            } => {
                let cp = mapper.data_to_pixel(center.0, center.1, None, None);
                let x_edge = mapper.data_to_pixel(center.0 + radius_x, center.1, None, None);
                let rx_px = (x_edge.0 - cp.0).abs();
                let y_edge = mapper.data_to_pixel(center.0, center.1 + radius_y, None, None);
                let ry_px = (y_edge.1 - cp.1).abs();

                rsx! {
                    g { transform,
                        ellipse {
                            key: "{gate_id}-{gate_index}-{shape_index}",
                            cx: cp.0,
                            cy: cp.1,
                            rx: rx_px,
                            ry: ry_px,
                            stroke: style.stroke,
                            stroke_width: style.stroke_width,
                            stroke_dasharray: if style.dashed { "4" } else { "none" },
                            fill: style.fill,
                            transform: "rotate({degrees_rotation} {cp.0} {cp.1})",
                        }
                    }

                }
            },
            GateRenderShape::Handle {center, size, shape_center, shape_type} => {
                let c = mapper.data_to_pixel(center.0, center.1, None, None);
                let cp = mapper.data_to_pixel(shape_center.0, shape_center.1, None, None);
                
                let pixel_offset = 15.0;
                let handle_x = c.0;
                let handle_y = c.1 - (size + pixel_offset); 

                let translate = {
                    if let Some(GateDragType::Gate(data)) = &&*drag_data.read() {
                        if *gate_id == *data.gate_id() {
                            let offset = data.offset();
                            let p_start = mapper.data_to_pixel(0.0, 0.0, None, None);
                            let p_current = mapper.data_to_pixel(offset.0, offset.1, None, None);
                            let dx = p_current.0 - p_start.0;
                            let dy = p_current.1 - p_start.1;
                            Some(format!("translate({} {})", -dx, -dy))
                        } else {
                            None
                        }
                } else {
                    None
                }};
                let rotate = { 
                    if let Some(GateDragType::Rotation(data)) = &*drag_data.read() {
                        if let ShapeType::Rotation(handle_angle_rad) = shape_type {
                            let angle = -(handle_angle_rad.to_degrees()) + data.rotation_deg();
                            Some(format!("rotate({} {} {})", angle, cp.0, cp.1))
                        } else {
                            None
                        }
                    
                    } else if let ShapeType::Rotation(handle_angle_rad) = shape_type {
                        let r = -(handle_angle_rad.to_degrees());
                        Some(format!("rotate({} {} {})", r, cp.0, cp.1))
                    } else {
                        None
                    }
                };

                let transform = [translate, rotate]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .join(" ");

                rsx!{
                    g { transform,
                        line {
                            x1: "{c.0}",
                            y1: "{c.1}",
                            x2: "{handle_x}",
                            y2: "{handle_y}",
                            stroke: "orange",
                            stroke_width: "1.5",
                            stroke_dasharray: "4",
                            pointer_events: "none",
                        }
                        circle {
                            key: "{gate_id}-{gate_index}-{shape_index}",
                            cx: "{handle_x}",
                            cy: "{handle_y}",
                            stroke: "orange",
                            r: size,
                            onmousedown: move |evt| {
                                let local_coords = &evt.data.coordinates().element();
                                let px = local_coords.x as f32;
                                let py = local_coords.y as f32;
                                let data_coords = plot_map()
                                    .unwrap()
                                    .pixel_to_data(px, py, None, None);
                                drag_data
                                    .set(
                                        Some(
                                            GateDragType::Rotation(
                                                RotationData::new(
                                                    gate_id.clone(),
                                                    shape_center,
                                                    data_coords,
                                                    data_coords,
                                                ),
                                            ),
                                        ),
                                    )
                            },
                        }
                    }
                }
            }
            GateRenderShape::Rectangle { x, y, width, height, style, shape_type:_ } => {
                let (mx, my, m_width, m_height) = map_rect_to_pixels(x, y, width, height, mapper);
                    
                rsx! {
                    g { transform,
                        rect {
                            key: "{gate_id}-{gate_index}-{shape_index}",
                            x: mx,
                            y: my,
                            width: m_width,
                            height: m_height,
                            stroke: style.stroke,
                            stroke_width: style.stroke_width,
                            stroke_dasharray: if style.dashed { "4" } else { "none" },
                            fill: style.fill,
                        }
                    }

                }
            },
            GateRenderShape::Line { x1, y1, x2, y2, style, shape_type:_ } => {
                let p1 = mapper.data_to_pixel(x1, y1, None, None);
                let p2 = mapper.data_to_pixel(x2, y2, None, None);
                rsx! {
                    g { transform,
                        line {
                            key: "{gate_id}-{gate_index}-{shape_index}",
                            x1: "{p1.0}",
                            y1: "{p1.1}",
                            x2: "{p2.0}",
                            y2: "{p2.1}",
                            stroke: style.stroke,
                            stroke_width: style.stroke_width,
                            stroke_dasharray: if style.dashed { "4" } else { "none" },
                        }
                    }

                }
            }
        }
    } else {
        rsx! {}
    }
}
