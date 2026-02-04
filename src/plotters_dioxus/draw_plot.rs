#![allow(non_snake_case)]
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use dioxus::prelude::*;
use flow_gates::{plotmap::PlotMapper, *};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use std::rc::Rc;
use std::sync::Arc;

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, plots::traits::PlotDrawable,
    render::RenderConfig,
};

use crate::plotters_dioxus::draw_gates::GateLayer;
use crate::{
    gate_store::{GateState, GateStateImplExt},
    plotters_dioxus::gate_helpers::GateDraft,
};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AxisInfo {
    pub title: Arc<str>,
    pub lower: f32,
    pub upper: f32,
    pub transform: flow_fcs::TransformType,
}

#[component]
pub fn PseudoColourPlot(
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
    let mut draft_gate_coords = use_signal(|| Vec::<(f32, f32)>::new());
    let mut draft_gate = use_signal(|| None::<GateDraft>);
    let mut next_gate_id = use_signal(|| 0);
    let mut selected_gate_id = use_signal(|| None::<Arc<str>>);

    use_effect(move || {
        let cur_coords = draft_gate_coords();

        if cur_coords.len() > 0 {
            let gate_draft =
                GateDraft::new_polygon(cur_coords, &x_axis_info().title, &y_axis_info().title);
            draft_gate.set(Some(gate_draft));
        } else {
            draft_gate.set(None);
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
            .render(data(), &options, &mut render_config)
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
                            if draft_gate.peek().is_none() {
                                let x_axis_title = x_axis_info.peek().title.clone();
                                let y_axis_title = y_axis_info.peek().title.clone();
                                if let Some(gates) = gate_store
                                    .get_gates_for_plot(x_axis_title, y_axis_title)
                                {
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
                            &x_axis_info().title,
                            &y_axis_info().title,
                        ) {
                            Ok(gate) => {
                                let id = *next_gate_id.peek();
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
                            Err(_) => {
                                draft_gate_coords.write().clear();
                                return;
                            }
                        };
                        gate_store
                            .add_gate(finalised_gate.unwrap(), None)
                            .expect("gate failed");
                        draft_gate_coords.write().clear();
                        *next_gate_id.write() += 1;
                    }
                },
            }
            GateLayer {
                plot_map,
                x_channel: x_axis_info().title.clone(),
                y_channel: y_axis_info().title.clone(),
                draft_gate,
                selected_gate_id,
            }
        }
    }
}
