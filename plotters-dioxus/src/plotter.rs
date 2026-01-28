#![allow(non_snake_case)]
use dioxus::prelude::*;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use std::rc::Rc;
use std::sync::Arc;
use flow_gates::*;

// use crate::colormap;

use flow_plots::{BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, render::RenderConfig};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;



#[derive(Debug, Clone, PartialEq, Props)]
pub struct AxisInfo{
    pub title: String,
    pub lower: f32,
    pub upper: f32,
    pub transform: flow_fcs::TransformType,
}

#[component]
pub fn Plotters(
    #[props] data: Arc<Vec<(f32, f32)>>,
    #[props] size: (u32, u32),
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
    let mut plot_image_src = use_signal(|| String::new());
    let mut plot_map = use_signal(|| None::<flow_plots::render::plotmap::PlotMapper>);
    let mut coords = use_signal(|| Vec::<(f32, f32)>::new());
    let mut gate_signal = use_signal(|| None::<flow_gates::Gate>);

    use_effect(move || {
        let cur_coords = coords();
        match flow_gates::geometry::create_polygon_geometry(cur_coords, &x_axis_info().title, &y_axis_info().title){
            Ok(gate) => {
                let gate = Gate::new (
                    "my first gate",
                    "My first gate".to_string(),
                    gate,
                    x_axis_info().title.as_str(),
                    y_axis_info().title.as_str()
                
                );

                gate_signal.set(Some(gate));

            },
            Err(e) =>{
                println!("{}", e.to_string());
                gate_signal.set(None);
            }
        
        }
    });

    use_effect(use_reactive!( |size, data| {
        let x_axis_info = x_axis_info();
        let y_axis_info = y_axis_info();
        let (width, height) = size;
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

                // .transform(flow_fcs::TransformType::Arcsinh { cofactor: 6000.0 })

            .label(x_axis_info.title)
            .build().expect("axis options failed");
            let y_axis_options = flow_plots::AxisOptions::new()
                .range(y_axis_info.lower..=y_axis_info.upper)
                .transform(y_axis_info.transform)
                .label(y_axis_info.title)
                .build().expect("axis options failed");


            let options = DensityPlotOptions::new()
            .base(base_options
            )
                .colormap(ColorMaps::Jet)
                .x_axis(x_axis_options)
                .y_axis(y_axis_options)
                .build().expect("shouldn't fail");
 
            let mut render_config = RenderConfig::default();
            let gate_option = gate_signal();
            let refs: [ &dyn flow_plots::plots::traits::PlotDrawable; 1 ];
            let gates_slice: Option<&[&dyn flow_plots::plots::traits::PlotDrawable]> = if let Some(ref g) = gate_option {
                refs = [g as &dyn flow_plots::plots::traits::PlotDrawable];
                Some(&refs[..])
            } else {
                None
            };
            
            let plot_data = plot.render(data, &options, &mut render_config, gates_slice).expect("failed to render plot");
            let bytes = plot_data.plot_bytes;
            let mapper = plot_data.plot_map;
            let base64_str = BASE64_STANDARD.encode(&bytes);
            plot_image_src.set(format!("data:image/jpeg;base64,{}", base64_str));
            plot_map.set(Some(mapper));
            
        }
    // }
    ));

    

     

    rsx! {

        img {
            src: "{plot_image_src()}",
            width: "{size.0}",
            height: "{size.1}",
            draggable: "{draggable}",
            onclick: move |evt| {
                if let Some(cb) = &on_click {
                    let local_coords = &evt.data.coordinates().element();
                    if let Some(mapper) = plot_map() {

                        let norm_x = local_coords.x as f32;
                        let norm_y = local_coords.y as f32;

                        if let Some((data_x, data_y)) = mapper.pixel_to_data(norm_x, norm_y) {
                            println!("Clicked Data: {}, {}", data_x, data_y);
                            coords.write().push((data_x, data_y));
                        } else {
                            println!("Clicked outside plot area");
                        }
                    }
                    cb.call(evt.data)
                }
            },
            ondoubleclick: move |evt| {
                if let Some(cb) = &on_dblclick {
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














// #![allow(non_snake_case)]
// use dioxus::prelude::*;
// use plotters::coord::Shift;
// use plotters::prelude::*;
// use plotters_bitmap::BitMapBackend;

// use base64::Engine as _;
// use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

// use image::ImageEncoder;
// use image::codecs::png::PngEncoder;
// use polars::prelude::*;

// use std::io::Cursor;
// use std::rc::Rc;
// use std::sync::Arc;
// use fcs_rs_2::FlowSample;

// pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

// fn get_zipped_column_data(
//     df: DataFrame,
//     col1_name: &str,
//     col2_name: &str,
// ) -> Result<Vec<(f64, f64)>, PolarsError> {
//     let float_series1 = df.column(col1_name)?.f64()?;
//     let float_series2 = df.column(col2_name)?.f64()?;
//     let zipped_data = float_series1
//         .into_no_null_iter()
//         .zip(float_series2.into_no_null_iter())
//         .collect();
//     Ok(zipped_data)
// }

// #[component]
// pub fn Plotters(
//     // data: Signal<Option<Arc<Vec<(f64, f64)>>>>,
//     fcs: Signal<Option<FlowSample>>,
//     size: (u32, u32),
//     // init: F,
//     #[props(optional)] on_click: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_dblclick: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_mousemove: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_mouseout: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_mouseup: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_mousedown: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_mouseover: Option<EventHandler<Rc<MouseData>>>,
//     #[props(optional)] on_wheel: Option<EventHandler<Rc<WheelData>>>,
//     #[props(default = false)] draggable: bool,
//     #[props(optional)] on_drag: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_dragend: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_dragenter: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_dragleave: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_dragover: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_dragstart: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_drop: Option<EventHandler<Rc<DragData>>>,
//     #[props(optional)] on_scroll: Option<EventHandler<Rc<ScrollData>>>,
// ) -> Element {
//     let mut plot_image_src = use_signal(|| String::new());

//     use_effect(move || {
//         let (width, height) = size;
        
//         if fcs.read().is_none() {
//             plot_image_src.set(String::new());
//         } else {


//             let df = fcs.clone();
//             spawn(async move {
//                 let data = get_zipped_column_data(df.read().unwrap().data, "CD4", "CD8").expect("failed to get data");
//                 let buffer_size = (width * height * 3) as usize;
//                 let mut buffer = vec![0u8; buffer_size];

//                 {
//                     let drawing_area =
//                         BitMapBackend::with_buffer(buffer.as_mut_slice(), (width, height))
//                             .into_drawing_area();

//                     drawing_area
//                         .fill(&WHITE)
//                         .expect("Failed to fill drawing area");

//                     let (x_min, x_max, y_min, y_max) = if data.is_empty() {
//                         // Default ranges if data is empty
//                         (0.0, 1.0, 0.0, 1.0)
//                     } else {
//                         let mut min_x = f64::INFINITY;
//                         let mut max_x = f64::NEG_INFINITY;
//                         let mut min_y = f64::INFINITY;
//                         let mut max_y = f64::NEG_INFINITY;

//                         for &(x, y) in data.iter() {
//                             min_x = min_x.min(x);
//                             max_x = max_x.max(x);
//                             min_y = min_y.min(y);
//                             max_y = max_y.max(y);
//                         }
//                         (min_x, max_x, min_y, max_y)
//                     };

//                     let x_range_margin = (x_max - x_min) * 0.1;
//                     let y_range_margin = (y_max - y_min) * 0.1;

//                     // Ensure ranges are std::ops::Range (exclusive end)
//                     let x_range = (x_min - x_range_margin)..(x_max + x_range_margin);
//                     let y_range = (y_min - y_range_margin)..(y_max + y_range_margin);

//                     // Let plotters infer the coordinate system from the `Range<f64>` inputs.
//                     let mut chart = ChartBuilder::on(&drawing_area)
//                         .caption("Dynamic Dot Plot", ("sans-serif", 20).into_font())
//                         .margin(5)
//                         .x_label_area_size(30)
//                         .y_label_area_size(30)
//                         .build_cartesian_2d(x_range, y_range)
//                         .expect("Failed to build chart");

//                     chart.configure_mesh().draw().expect("Failed to draw mesh");

//                     // chart
//                     //     .draw_series(
//                     //         data.iter()
//                     //             .map(|&(x, y)| Circle::new((x, y), 2, RED.filled())),
//                     //     )
//                     //     .expect("Failed to draw scatter points");

//                     let coord_spec = chart.plotting_area().as_coord_spec();

//                     for &(x, y) in data.iter() {
//                         let (px, py) = coord_spec.translate(&(x, y));
//                         drawing_area.draw_pixel((px, py), &RED).expect("Failed to draw pixel");
//                     }

//                     drawing_area
//                         .present()
//                         .expect("Failed to present drawing area");
//                 }

//                 let mut png_data = Vec::new();
//                 let cursor = Cursor::new(&mut png_data);
//                 let encoder = PngEncoder::new(cursor);
//                 let color = image::ColorType::Rgb8;

//                 encoder
//                     .write_image(buffer.as_slice(), width, height, color.into())
//                     .expect("Failed to write the image");

//                 let buffer_base64 = BASE64_STANDARD.encode(png_data);

//                 plot_image_src.set(format!("data:image/png;base64,{}", buffer_base64));
//             });
//         };
//     });

//     rsx! {

//         img {
//             src: "{plot_image_src()}",
//             width: "{size.0}",
//             height: "{size.1}",
//             draggable: "{draggable}",
//             onclick: move |evt| {
//                 if let Some(cb) = &on_click {
//                     cb.call(evt.data)
//                 }
//             },
//             ondoubleclick: move |evt| {
//                 if let Some(cb) = &on_dblclick {
//                     cb.call(evt.data)
//                 }
//             },
//             onmousemove: move |evt| {
//                 if let Some(cb) = &on_mousemove {
//                     cb.call(evt.data)
//                 }
//             },
//             onmouseout: move |evt| {
//                 if let Some(cb) = &on_mouseout {
//                     cb.call(evt.data)
//                 }
//             },
//             onmouseover: move |evt| {
//                 if let Some(cb) = &on_mouseover {
//                     cb.call(evt.data)
//                 }
//             },
//             onmousedown: move |evt| {
//                 if let Some(cb) = &on_mousedown {
//                     cb.call(evt.data)
//                 }
//             },
//             onmouseup: move |evt| {
//                 if let Some(cb) = &on_mouseup {
//                     cb.call(evt.data)
//                 }
//             },

//             onwheel: move |evt| {
//                 if let Some(cb) = &on_wheel {
//                     cb.call(evt.data)
//                 }
//             },

//             ondrag: move |evt| {
//                 if let Some(cb) = &on_drag {
//                     cb.call(evt.data)
//                 }
//             },
//             ondragend: move |evt| {
//                 if let Some(cb) = &on_dragend {
//                     cb.call(evt.data)
//                 }
//             },
//             ondragenter: move |evt| {
//                 if let Some(cb) = &on_dragenter {
//                     cb.call(evt.data)
//                 }
//             },
//             ondragleave: move |evt| {
//                 if let Some(cb) = &on_dragleave {
//                     cb.call(evt.data)
//                 }
//             },
//             ondragover: move |evt| {
//                 if let Some(cb) = &on_dragover {
//                     cb.call(evt.data)
//                 }
//             },
//             ondragstart: move |evt| {
//                 if let Some(cb) = &on_dragstart {
//                     cb.call(evt.data)
//                 }
//             },
//             ondrop: move |evt| {
//                 if let Some(cb) = &on_drop {
//                     cb.call(evt.data)
//                 }
//             },

//             onscroll: move |evt| {
//                 if let Some(cb) = &on_scroll {
//                     cb.call(evt.data)
//                 }
//             },
//         }
//     }
// }
