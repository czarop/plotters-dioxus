#![allow(non_snake_case)]
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use dioxus::{html::{input_data::MouseButton, u::z_index}, prelude::*};
use flow_gates::{plotmap::PlotMapper, *};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use std::rc::Rc;
use std::sync::Arc;


// use crate::colormap;

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, plots::traits::PlotDrawable, render::RenderConfig
};

use crate::{gate_store::{GateState, GateStateImplExt, GateStateStoreExt, GatesOnPlotKey}, plotters_dioxus::gate_helpers::{GateDraft, GateFinal}};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AxisInfo {
    pub title: Arc<str>,
    pub lower: f32,
    pub upper: f32,
    pub transform: flow_fcs::TransformType,
}

#[component]
pub fn Plotters(
    #[props] data: ReadSignal<Arc<Vec<(f32, f32)>>>,
    #[props] size: ReadSignal<(u32, u32)>,
    #[props] x_axis_info: ReadSignal<AxisInfo>,
    #[props] y_axis_info: ReadSignal<AxisInfo>,
    #[props(optional)] on_click: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_dblclick: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_mousemove: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_mouseout: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_mouseup: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_mousedown: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_mouseover: Option<EventHandler<Rc<MouseData>>>,
    #[props(optional)] on_wheel: Option<EventHandler<Rc<WheelData>>>,
    #[props(default = false)] draggable: bool,
    #[props(optional)] on_drag: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_dragend: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_dragenter: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_dragleave: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_dragover: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_dragstart: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_drop: Option<EventHandler<Rc<DragData>>>,
    #[props(optional)] on_scroll: Option<EventHandler<Rc<ScrollData>>>,
) -> Element {
    let mut gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let mut plot_image_src = use_signal(|| String::new());
    let mut plot_map = use_signal(|| None::<flow_gates::plotmap::PlotMapper>);
    let mut coords = use_signal(|| Vec::<(f32, f32)>::new());
    let mut curr_gate_signal = use_signal(|| None::<GateDraft>);
    let mut curr_gate_id = use_signal(|| 0);

    use_effect(move || {
        let cur_coords = coords();

        if cur_coords.len() > 0 {
            let gate_draft = GateDraft::new_polygon(
                cur_coords,
                &x_axis_info().title,
                &y_axis_info().title,
            );
            curr_gate_signal.set(Some(gate_draft));
        } else {
            curr_gate_signal.set(None);
        }
        
    });

    use_effect(move || {
        let x_axis_info = x_axis_info();
        let y_axis_info = y_axis_info();
        let (width, height) = size();
        let data = data.clone();

        let plot = DensityPlot::new();
        let base_options = BasePlotOptions::new()
            .width(width)
            .height(height)
            .title("My Density Plot")
            .build()
            .expect("shouldn't fail");

        let x_axis_options = flow_plots::AxisOptions::new()
            .range(x_axis_info.lower..=x_axis_info.upper)
            .transform(x_axis_info.transform)
            .label(&x_axis_info.title.to_string())
            .build()
            .expect("axis options failed");
        let y_axis_options = flow_plots::AxisOptions::new()
            .range(y_axis_info.lower..=y_axis_info.upper)
            .transform(y_axis_info.transform)
            .label(y_axis_info.title.to_string())
            .build()
            .expect("axis options failed");

        let options = DensityPlotOptions::new()
            .base(base_options)
            .colormap(ColorMaps::Jet)
            .x_axis(x_axis_options)
            .y_axis(y_axis_options)
            .build()
            .expect("shouldn't fail");

        let mut render_config = RenderConfig::default();

        let plot_data = plot
            .render(
                data(),
                &options,
                &mut render_config,
            )
            .expect("failed to render plot");
        let bytes = plot_data.plot_bytes;
        let helper = plot_data.plot_helper;
        let base64_str = BASE64_STANDARD.encode(&bytes);
        plot_image_src.set(format!("data:image/jpeg;base64,{}", base64_str));
        let mapper = PlotMapper::from_plot_helper(&helper, width as f32, height as f32);
        plot_map.set(Some(mapper));
    });

    
    rsx! {
        div { style: "position: relative; width: {size().0}px; height: {size().1}px;",
            img {
                style: "user-select: none; -webkit-user-select: none; cursor: crosshair;",
                src: "{plot_image_src()}",
                width: "{size().0}",
                height: "{size().1}",
                draggable: "{draggable}",
                oncontextmenu: move |evt| evt.prevent_default(),
                onclick: move |evt| {
                    if let Some(mapper) = plot_map() {
                        let local_coords = &evt.data.coordinates().element();
                        let norm_x = local_coords.x as f32;
                        let norm_y = local_coords.y as f32;
                        println!("Clicked Pixel: {}, {}", norm_x, norm_y);
                        if let Some((data_x, data_y)) = mapper
                            .pixel_to_data(norm_x, norm_y, None, None)
                        {
                            println!("Clicked Data: {}, {}", data_x, data_y);
                            let mut closest_gate = None;
                            if curr_gate_signal.peek().is_none() {
                                let x_axis_title = x_axis_info.peek().title.clone();
                                let y_axis_title = y_axis_info.peek().title.clone();
                                if let Some(gates) = get_gates_for_plot(
                                    x_axis_title,
                                    y_axis_title,
                                    &gate_store,
                                ) {
                                    let tolerance = mapper.get_data_tolerance(5.0);
                                    let mut closest_dist = std::f32::INFINITY;

                                    for gate in gates {
                                        if let Some(dist) = gate
                                            .is_point_on_perimeter((data_x, data_y), tolerance)
                                        {
                                            println!("You clicked on a gate!");
                                            if dist < closest_dist {
                                                closest_dist = dist;
                                                closest_gate = Some(gate.clone());
                                            }

                                        }
                                    }
                                }
                            }
                            if closest_gate.is_none() {
                                println!("You didn't click on a gate");

                                coords.write().push((data_x, data_y));

                            } else {
                                let gate_name = closest_gate.unwrap().name.clone();
                                println!("closest gate was {}", gate_name);
                            }

                        }

                    }

                },
                ondoubleclick: move |evt| {
                    // Finalise the current gate
                    if let Some(curr_gate) = curr_gate_signal.write().take() {
                        // last point is duplicated from the double click
                        let mut points = curr_gate.get_points();
                        points.pop();

                        let finalised_gate = match flow_gates::geometry::create_polygon_geometry(
                            points,
                            &x_axis_info().title,
                            &y_axis_info().title,
                        ) {
                            Ok(gate) => {
                                let id = *curr_gate_id.peek();
                                Some(

                                    Gate::new(
                                        id.to_string(),
                                        id.to_string(),
                                        gate,
                                        x_axis_info().title.clone(),
                                        y_axis_info().title.clone(),
                                    ),
                                )
                            }
                            Err(e) => {
                                coords.write().clear();
                                return;
                            }
                        };
                        gate_store
                            .add_gate(finalised_gate.unwrap(), None)
                            .expect("gate failed");
                        coords.write().clear();
                        *curr_gate_id.write() += 1;
                    }
                },
                onmousemove: move |evt| {
                    if let Some(cb) = &on_mousemove {
                        cb.call(evt.data)
                    }
                },
                onmouseout: move |evt| {
                    if let Some(cb) = &on_mouseout {
                        cb.call(evt.data)
                    }
                },
                onmouseover: move |evt| {
                    if let Some(cb) = &on_mouseover {
                        cb.call(evt.data)
                    }
                },
                onmousedown: move |evt| {
                    match evt.trigger_button() {
                        Some(MouseButton::Secondary) => {
                            coords.set(vec![]);
                        }
                        _ => {}
                    }
                },
                onmouseup: move |evt| {
                    if let Some(cb) = &on_mouseup {
                        cb.call(evt.data)
                    }
                },

                onwheel: move |evt| {
                    if let Some(cb) = &on_wheel {
                        cb.call(evt.data)
                    }
                },

                // ondrag: move |evt| {
                //     if let Some(cb) = &on_drag {
                //         cb.call(evt.data)
                //     }
                // },
                // ondragend: move |evt| {
                //     if let Some(cb) = &on_dragend {
                //         cb.call(evt.data)
                //     }
                // },
                // ondragenter: move |evt| {
                //     if let Some(cb) = &on_dragenter {
                //         cb.call(evt.data)
                //     }
                // },
                // ondragleave: move |evt| {
                //     if let Some(cb) = &on_dragleave {
                //         cb.call(evt.data)
                //     }
                // },
                // ondragover: move |evt| {
                //     if let Some(cb) = &on_dragover {
                //         cb.call(evt.data)
                //     }
                // },
                // ondragstart: move |evt| {
                //     if let Some(cb) = &on_dragstart {
                //         cb.call(evt.data)
                //     }
                // },
                // ondrop: move |evt| {
                //     if let Some(cb) = &on_drop {
                //         cb.call(evt.data)
                //     }
                // },
                onscroll: move |evt| {
                    if let Some(cb) = &on_scroll {
                        cb.call(evt.data)
                    }
                },
            }
            GateLayer {
                plot_map,
                x_channel: x_axis_info().title.clone(),
                y_channel: y_axis_info().title.clone(),
                draft_gate: curr_gate_signal,
            }
        }
    }
}

fn get_gates_for_plot(x_axis_title: Arc<str>, y_axis_title: Arc<str>, gate_store: &Store<GateState>) -> Option<Vec<Arc<GateFinal>>> {
    let key = GatesOnPlotKey::new(
            x_axis_title,
            y_axis_title,
            None
        );
       let key_options = gate_store.gate_ids_by_view().get(key);
       let mut gate_list = vec![];
       if let Some(key_store) = key_options {
            
            let ids = key_store.read().clone(); 
            let registry = gate_store.gate_registry();
            let registry_guard = registry.read();
            for k in ids {
                if let Some(gate_store_entry) = registry_guard.get(&k) {
                    gate_list.push(gate_store_entry.clone());
                }
            }
        } else {
            println!("No gates for plot");
            return None;
        }
        return Some(gate_list);
}

#[component]
fn GateLayer(plot_map: ReadSignal<Option<PlotMapper>>, x_channel: ReadSignal<Arc<str>>, y_channel: ReadSignal<Arc<str>>, draft_gate: ReadSignal<Option<GateDraft>>) -> Element {
    
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();

    let gates = use_memo(move || {
        match get_gates_for_plot(x_channel(), y_channel(), &gate_store) {
            Some(g) => g,
            None => vec![],
        }
    });

    rsx! {
        match plot_map() {
            Some(mapper) => rsx! {
                svg {
                    width: "100%",
                    height: "100%",
                    view_box: "0 0 {mapper.view_width} {mapper.view_height}",
                    style: "position: absolute; top: 0; left: 0; pointer-events: none; z-index: 2;",

                    for gate in &*gates.read() {
                        match &gate.geometry {
                            GateGeometry::Polygon { nodes: _, closed: _ } => {
                                let points_attr = gate
                                    .get_points()
                                    .iter()
                                    .map(|v| {
                                        let (px, py) = mapper.map_to_svg(v.0, v.1);
                                        format!("{px},{py}")
                                    })
                                    .collect::<Vec<_>>()
                                    .join(" ");

                                rsx! {
                                    polygon {
                                        points: "{points_attr}",
                                        fill: "rgba(0, 255, 255, 0.2)",
                                        stroke: "cyan",
                                        stroke_width: "2",
                                        pointer_events: "none",
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


