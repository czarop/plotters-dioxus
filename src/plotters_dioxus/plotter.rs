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

// use crate::colormap;

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, plots::traits::PlotDrawable,
    render::RenderConfig,
};

use crate::gate_store::{GateState, GatesOnPlotKey};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AxisInfo {
    pub title: String,
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
    let mut curr_gate_signal = use_signal(|| None::<super::gate_draft::GateDraft>);
    let mut gates: Memo<Vec<Arc<flow_gates::Gate>>> = use_memo(move || {
        let key = GatesOnPlotKey::new(
            &x_axis_info().title,
            &y_axis_info().title,
            None
        );
        let gates = gate_store().get_gates_for_plot(&key).collect();
        gates
    }); 
    let mut curr_gate_id = use_signal(|| 0);

    use_effect(move || {
        let cur_coords = coords();

        let gate_draft = super::gate_draft::GateDraft::new_polygon(
            cur_coords,
            &x_axis_info().title,
            &y_axis_info().title,
        );
        curr_gate_signal.set(Some(gate_draft));
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
            .label(x_axis_info.title)
            .build()
            .expect("axis options failed");
        let y_axis_options = flow_plots::AxisOptions::new()
            .range(y_axis_info.lower..=y_axis_info.upper)
            .transform(y_axis_info.transform)
            .label(y_axis_info.title)
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

        let gate_vec= gates();
        let mut gate_refs: Vec<&dyn PlotDrawable> = gate_vec
            .iter()
            .map(|g| g.as_ref() as &dyn PlotDrawable)
            .collect();
        let curr_gate_binding = curr_gate_signal.read();
        if let Some(gate) = curr_gate_binding.as_ref() {
            gate_refs.push(gate as &dyn PlotDrawable);
        }

        let plot_data = plot
            .render(
                data(),
                &options,
                &mut render_config,
                Some(gate_refs.as_slice()),
                None,
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

        img {
            style: "user-select: none; -webkit-user-select: none; cursor: crosshair;",
            src: "{plot_image_src()}",
            width: "{size().0}",
            height: "{size().1}",
            draggable: "{draggable}",
            onclick: move |evt| {
                // if let Some(cb) = &on_click {
                let local_coords = &evt.data.coordinates().element();

                if let Some(mapper) = plot_map() {

                    let norm_x = local_coords.x as f32;
                    let norm_y = local_coords.y as f32;

                    if let Some((data_x, data_y)) = mapper
                        .pixel_to_data(norm_x, norm_y, None, None)
                    {
                        println!("Clicked Data: {}, {}", data_x, data_y);
                        coords.write().push((data_x, data_y));
                    } else {
                        println!("Clicked outside plot area");
                    }

                }

                // cb.call(evt.data)
                // }
            },
            ondoubleclick: move |evt| {
                // Finalise the current gate
                if let Some(curr_gate) = curr_gate_signal.write().take() {

                    let finalised_gate = match flow_gates::geometry::create_polygon_geometry(
                        curr_gate.get_points(),
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
                                    x_axis_info().title.as_str(),
                                    y_axis_info().title.as_str(),
                                ),
                            )
                        }
                        Err(e) => {
                            println!("{}", e.to_string());
                            return;
                        }
                    };
                    gates.write().push(Arc::new(finalised_gate.unwrap()));
                    coords.write().clear();
                    *curr_gate_id.write() += 1;
                }

                if let Some(cb) = &on_mousemove {
                    cb.call(evt.data)
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
                if let Some(cb) = &on_mousedown {
                    cb.call(evt.data)
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
    }
}