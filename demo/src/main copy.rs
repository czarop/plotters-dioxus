#![allow(non_snake_case)]

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use fcs_rs_2::{FcsError, FcsFile};
use std::rc::Rc; // Needed for event handlers

// Import your Plotters component from the `plotters-dioxus` crate.
// Adjust this path based on your project structure.
// If your Plotters component code is directly in this file, you wouldn't need this.
use plotters_dioxus::Plotters;
use polars::prelude::*;

async fn get_flow_data(path: String) -> Result<Arc<Vec<(f64, f64)>>, FcsError> {
    // let path = "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G7 FMX_1_Plate_001.fcs";
    // let path = path.to_string();
    // IMPORTANT: The closure passed to spawn_blocking must be `Send + 'static`.
    // It should contain *all* the blocking operations.
    let result = tokio::task::spawn_blocking(move || {
        println!("loading file");
        let fcs_file = FcsFile::open(&path)?;
        println!("file opened");
        let fcs_data = fcs_file.read()?; // This is the blocking read
        println!("data loaded");
        // Print head is also blocking I/O/computation, so it should be in spawn_blocking too.
        // println!("{}", fcs_data.data.head(Some(50)));

        let new_data = get_zipped_column_data(&fcs_data.data, "CD4", "CD8")
            .map_err(|e| FcsError::InvalidData(format!("Polars extraction error: {}", e)))?;
        println!("data zipped");
        Ok(Arc::new(new_data))
    })
    .await; // Await the result of the blocking task

    let inner_result = result.map_err(|e| FcsError::IoError(e.into()))?;
    inner_result
}

fn get_zipped_column_data(
    df: &DataFrame,
    col1_name: &str,
    col2_name: &str,
) -> Result<Vec<(f64, f64)>, PolarsError> {
    let float_series1 = df.column(col1_name)?.f64()?;
    let float_series2 = df.column(col2_name)?.f64()?;
    let zipped_data = float_series1
        .into_no_null_iter()
        .zip(float_series2.into_no_null_iter())
        .collect();
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
                            size: (400, 400), // Define the size of the plot image
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

            h1 { "FCS View" }

            div {
                margin_bottom: "1rem",
                // align_content: "center",
                // justify_content: "center",
                // flex_direction: "column",
                text_align: "center",

                button {
                    // style: "padding: 10px 20px; font-size: 16px; border-radius: 8px; border: 1px solid #ccc; background-color: #f0f0f0; cursor: pointer;",
                    onclick: move |_| {
                        data_version
                            .set(
                                "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G8 FMX_2_Plate_001.fcs"
                                    .to_string(),
                            );
                        scatter_data.restart();
                    },
                    "Update Data"
                }
                div { margin_top: "1rem", {data_version()} }
            }

            div { {element} }
        }
    }
}

fn main() {
    // dioxus::launch(App);
    // let conf = dioxus::desktop::Config::new().with_window(
    //     WindowBuilder::new()
    //         .with_title("FCS")
    //         .with_always_on_top(false),
    // );
    // dioxus::desktop::launch::launch(App, vec![Box::new(Dyn)], vec![Box::new(conf)]);

    LaunchBuilder::new()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("FCS")
                    .with_always_on_top(false) // Set the window to not be always on top
                    .with_inner_size(LogicalSize::new(1000.0, 800.0)),
            ),
        )
        .launch(App); // Launch the application
}
