// #![allow(non_snake_case)]
// use dioxus::html::geometry::ElementPoint;
// use dioxus::prelude::*;
// use plotters::{coord::ReverseCoordTranslate, define_color, doc, prelude::*};
// use plotters_dioxus::{DioxusDrawingArea, Plotters};

// define_color!(BACKGROUND, 11, 20, 31, "background");
// define_color!(ITEM, 57, 90, 131, "item");

// use rand::SeedableRng;
// use rand_distr::{Distribution, Normal};
// use rand_xorshift::XorShiftRng;

// fn main() {
//     dioxus::launch(App);
// }

// fn draw_scatter_plot(
//     drawing_area: DioxusDrawingArea,
//     click_coord: ElementPoint,
//     x_axis_scale: f64,
// ) -> () {
//     let number_sample = 50000;
//     let normal_dist = Normal::new(0.5, 0.1).unwrap();
//     let mut rand = XorShiftRng::from_seed(*b"MyFragileSeed123");
//     let iter_rand = normal_dist.sample_iter(&mut rand);
//     let data = iter_rand
//         .enumerate()
//         .take(number_sample)
//         .map(|(idx, data)| {
//             (
//                 f64::from(i32::try_from(idx).expect("Expect to be not more than 1000")),
//                 data,
//             )
//         })
//         .collect::<Vec<(f64, f64)>>();
//     drawing_area.fill(&BACKGROUND).expect("Expect to work");
//     let mut scatter_ctx = ChartBuilder::on(&drawing_area)
//         .caption("Test graph", ("sans-serif", 14, &WHITE))
//         .margin_top(40)
//         .x_label_area_size(40)
//         .y_label_area_size(40)
//         .build_cartesian_2d(0f64..x_axis_scale * (number_sample as f64), 0f64..1f64)
//         .expect("Expect to work");

//     let original_style = ShapeStyle {
//         color: WHITE.mix(0.5),
//         filled: true,
//         stroke_width: 1,
//     };

//     scatter_ctx
//         .configure_mesh()
//         .disable_x_mesh()
//         .disable_y_mesh()
//         .y_label_style(("sans-serif", 11, &WHITE).into_text_style(&drawing_area))
//         .x_label_style(("sans-serif", 11, &WHITE).into_text_style(&drawing_area))
//         .x_desc("Count")
//         .y_desc("Data")
//         .axis_style(original_style)
//         .axis_desc_style(("sans-serif", 11, &WHITE).into_text_style(&drawing_area))
//         .draw()
//         .expect("Succeed");
//     let t = data
//         .iter()
//         .map(|e| Circle::new(*e, 1i32, ITEM))
//         .collect::<Vec<Circle<(f64, f64), i32>>>();
//     scatter_ctx.draw_series(t).expect("Expect to work");
//     scatter_ctx
//         .as_coord_spec()
//         .reverse_translate((click_coord.x as i32, click_coord.y as i32))
//         .map(|coord| {
//             scatter_ctx
//                 .draw_series(LineSeries::new(
//                     (0..number_sample).map(|x| (x as f64, coord.1)),
//                     WHITE,
//                 ))
//                 .unwrap();
//         });
//     drawing_area
//         .present()
//         .expect(
//             "Unable to write result to file, please make sure 'plotters-doc-data' dir exists under current dir"
//         );
// }

// fn App() -> Element {
//     let click_coord_state = use_signal(ElementPoint::default);
//     let x_axis_scale_state = use_signal(|| 1.0f64);

//     rsx!(
//         Plotters {
//             size: (400, 400),
//             init: move |d| draw_scatter_plot(d, **click_coord_state, **x_axis_scale_state),
//             on_click: |e: Event<MouseData>| click_coord_state.set(e.element_coordinates()),
//             on_wheel: |e: Event<WheelData>| {
//                 x_axis_scale_state
//                     .set(
//                         (**x_axis_scale_state
//                             + (if e.delta().strip_units().y > 0.0 { -0.1 } else { 0.1 }))
//                             .max(0.01),
//                     )
//             },
//         }
//     )
// }

#![allow(non_snake_case)]
use core::error;
use dioxus::prelude::*;
use fcs_rs_2::{FcsError, FcsFile};
use std::rc::Rc; // Needed for event handlers

// Import your Plotters component from the `plotters-dioxus` crate.
// Adjust this path based on your project structure.
// If your Plotters component code is directly in this file, you wouldn't need this.
use plotters_dioxus::Plotters;
use polars::prelude::*;

async fn get_flow_data(path: String) -> Result<Arc<Vec<(f32, f32)>>, FcsError> {
    // let path = "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G7 FMX_1_Plate_001.fcs";
    // let path = path.to_string();
    // IMPORTANT: The closure passed to spawn_blocking must be `Send + 'static`.
    // It should contain *all* the blocking operations.
    let result = tokio::task::spawn_blocking(move || {
        println!("loading file");
        let fcs_file = FcsFile::open(&path)?;
        let fcs_data = fcs_file.read()?; // This is the blocking read

        // Print head is also blocking I/O/computation, so it should be in spawn_blocking too.
        // println!("{}", fcs_data.data.head(Some(50)));

        let new_data = get_zipped_column_data(&fcs_data.data, "CD4", "CD8")
            .map_err(|e| FcsError::InvalidData(format!("Polars extraction error: {}", e)))?;

        Ok(Arc::new(new_data))
    })
    .await; // Await the result of the blocking task

    // Handle the result from the spawn_blocking future.
    // The `?` operator here propagates errors from both the inner blocking task
    // and the `JoinError` from `spawn_blocking`.
    // result.map_err(|e| FcsError::IoError(e))
    let inner_result = result.map_err(|e| FcsError::IoError(e.into()))?;
    inner_result
}

fn get_zipped_column_data(
    df: &DataFrame,
    col1_name: &str,
    col2_name: &str,
) -> Result<Vec<(f32, f32)>, PolarsError> {
    let series1 = df.column(col1_name)?;
    let series2 = df.column(col2_name)?;

    // Cast both series to f64
    let float_series1 = series1.cast(&DataType::Float32)?;
    let float_series2 = series2.cast(&DataType::Float32)?;

    let ca1 = float_series1.f32().map_err(|e| {
        PolarsError::ComputeError(
            format!("Failed to get f64 data from series {}: {}", col1_name, e).into(),
        )
    })?;
    let ca2 = float_series2.f32().map_err(|e| {
        PolarsError::ComputeError(
            format!("Failed to get f64 data from series {}: {}", col2_name, e).into(),
        )
    })?;

    // Zip the iterators. `zip` stops when the shorter iterator ends.
    // `.flatten()` handles potential nulls in each column independently.
    let zipped_data: Vec<(f32, f32)> = ca1
        .into_iter()
        .zip(ca2.into_iter())
        .filter_map(|(opt_x, opt_y)| {
            match (opt_x, opt_y) {
                (Some(x), Some(y)) => Some((x, y)), // Only include pairs where both are non-null
                _ => None,                          // Discard if either is null
            }
        })
        .collect();
    println!("data loaded");
    Ok(zipped_data)
}

#[component]
fn App() -> Element {
    let mut data_version = use_signal_sync(|| {
        "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G7 FMX_1_Plate_001.fcs".to_string()
    });
    let mut data = use_signal(|| None);
    let mut message = use_signal(|| "No data loaded".to_string());
    // let mut data_version = use_signal_sync(|| 0);

    let mut scatter_data = use_resource(move || async move {
        println!("resouce running");
        get_flow_data(data_version()).await
    });

    // --- Optional Event Handlers for the Plotters Component ---
    // These functions demonstrate how you can receive events from the plot image.
    let handle_click = move |event: Rc<MouseData>| {
        println!(
            "Click event on plot: x={}, y={}",
            event.client_coordinates().x,
            event.client_coordinates().y
        );
    };

    let handle_mousemove = move |_event: Rc<MouseData>| {
        // This can be very chatty, so it's commented out by default.
        // Uncomment if you need to debug mouse movement.
        // println!("Mouse move on plot: x={}, y={}", event.client_x, event.client_y);
    };

    let handle_drag = move |event: Rc<DragData>| {
        println!(
            "Drag event on plot: x={}, y={}",
            event.client_coordinates().x,
            event.client_coordinates().y
        );
    };

    use_effect(move || {
        println!("data read memo called");
        match &*scatter_data.read() {
            Some(Ok(d)) => {
                let x = d.clone();
                data.set(Some(x));
            }
            Some(Err(e)) => {
                let error_s = format!("Error loading data: {}", e.to_string());
                message.set(error_s);
                data.set(None)
            }
            None => {
                message.set("Loading data.".to_string());
                data.set(None)
            }
        }
    });

    let element = use_memo(move || {
        println!("plot memo called");
        match data() {
            Some(_) => {
                rsx! {
                    div {
                        Plotters {
                            size: (800, 600), // Define the size of the plot image
                            data, // Pass a clone of the generated data

                            // Pass optional event handlers
                            on_click: handle_click,
                            on_mousemove: handle_mousemove,
                            on_drag: handle_drag,
                            draggable: true, // Allow the image to be draggable
                        }
                    }
                }
            }

            None => rsx! {
                div { {message()} }
            },
        }
    });

    rsx! {
        div {
            // Basic styling to center content
            style: "display: flex; flex-direction: column; align-items: center; padding: 20px; font-family: sans-serif;",

            h1 { "Dioxus Plotters Demo App" }

            div { margin_bottom: "20px",
                button {
                    style: "padding: 10px 20px; font-size: 16px; border-radius: 8px; border: 1px solid #ccc; background-color: #f0f0f0; cursor: pointer;",
                    onclick: move |_| {
                        data_version
                            .set(
                                "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G8 FMX_2_Plate_001.fcs"
                                    .to_string(),
                            );
                        scatter_data.restart();
                    },
                    "Update Data ({data_version()})"
                }
            }

            div { {element} }
        }
    }
}

fn main() {
    dioxus::launch(App);
}
