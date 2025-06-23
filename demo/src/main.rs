#![allow(non_snake_case)]

use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder}, prelude::*
};
use fcs_rs_2::{FcsError, FcsFile, FlowSample};
use std::rc::Rc; // Needed for event handlers

// Import your Plotters component from the `plotters-dioxus` crate.
// Adjust this path based on your project structure.
// If your Plotters component code is directly in this file, you wouldn't need this.
use plotters_dioxus::Plotters;
use polars::prelude::*;



async fn get_flow_data(path: String) -> Result<Arc<FlowSample>, FcsError> {
    println!("{path}");
    let result = tokio::task::spawn_blocking(move || {
        let fcs_file = FcsFile::open(&path)?;
        let fcs_data = fcs_file.read()?; // This is the blocking read
        Ok(Arc::new(fcs_data))
    })
    .await; // Await the result of the blocking task
    let inner_result = result.map_err(|e| FcsError::IoError(e.into()))?;
    inner_result
}

async fn get_data_to_display(
    fs: Option<Arc<FlowSample>>,
    col1_name: &str,
    col2_name: &str,
    col1_cofactor: f64,
    col2_cofactor: f64,) -> Result<Arc<Vec<(f64, f64)>>, FcsError> {
    
    let ts_fs = fs.ok_or(FcsError::InvalidData("No Data".to_string()))?;
    let binding = ts_fs.clone();
    // let ts_df = (binding.data);

    // let ts_df = Arc::new(df);
    let c1 = Arc::new(col1_name.to_string());
    let c2 = Arc::new(col2_name.to_string());
    let result = tokio::task::spawn_blocking(move || {
        let scaled_cols = apply_arcsinh_scaling(&binding.data, &c1, &c2, col1_cofactor, col2_cofactor)?;
        let zipped_data = get_zipped_column_data(&scaled_cols, &c1, &c2).map_err(|_| FcsError::InvalidData("Columns not valid".to_string()))?;
        Ok(Arc::new(zipped_data))
    }).await;
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

fn arcsinh_transform_series(col_data: Column, cofactor: f64) -> PolarsResult<Option<Column>> {
    // Convert the Column into a Series to use Series-specific methods like .cast() and .f64()
    let s = col_data.as_series().ok_or(PolarsError::ColumnNotFound("error transforming column".into()))?;

    let cast_s = s.cast(&DataType::Float64)?; // Ensure it's f64

    let transformed_chunked = cast_s
        .f64()
        .expect("Series was not f64 after casting; this should not happen.")
        .apply(|value| Some(value?.asinh()/cofactor)); // Apply arcsinh scaling

    // Ok(transformed_chunked.into_series())
    Ok(Some(transformed_chunked.into_column()))
}

pub fn apply_arcsinh_scaling(
    df: &DataFrame,
    col1_name: &str,
    col2_name: &str,
    col1_cofactor: f64,
    col2_cofactor: f64,
) -> Result<DataFrame, FcsError> {
    // Validate cofactors early to prevent division by zero
    if col1_cofactor == 0.0 || col2_cofactor == 0.0 {
        return Err(FcsError::InvalidData("Cofactors for arcsinh scaling cannot be zero.".to_string()));
    }

    // --- OPTIMIZATION START ---
    // 1. Select only the columns of interest from the original DataFrame.
    // This creates a new, smaller DataFrame containing only these two columns.
    let selected_df = df
        .select([col1_name, col2_name])
        .map_err(|e| FcsError::InvalidData(format!("Failed to select columns: {}", e)))?;

    // 2. Convert this *smaller* DataFrame to a LazyFrame.
    // The clone happens on this smaller DataFrame's data, not the original full DataFrame.
    let lazy_df = selected_df.lazy();
    // --- OPTIMIZATION END ---



    let transformed_lazy_df = lazy_df
        .with_columns([
            // Expression for the first column
            col(col1_name)
                .map(

                    move |s| arcsinh_transform_series(s, col1_cofactor),
                    // Specify the output type of the transformation
                    GetOutput::from_type(DataType::Float64),
                )
                .alias(col1_name), // Re-alias to maintain original column name

            // Expression for the second column
            col(col2_name)
                .map(
                    // CRITICAL FIX: Convert Series to Column and wrap in Some
                    move |s| arcsinh_transform_series(s, col2_cofactor),
                    // Specify the output type of the transformation
                    GetOutput::from_type(DataType::Float64),
                )
                .alias(col2_name), // Re-alias to maintain original column name
        ]);

    // Execute the lazy plan and collect the result into an eager DataFrame
    let df_transformed = transformed_lazy_df
        .collect()
        .map_err(|e| FcsError::InvalidData(format!("Failed to collect transformed DataFrame: {}", e)))?;

    Ok(df_transformed)
}



#[component]
fn App() -> Element {
    let samples = use_signal(||vec![
        "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G7 FMX_1_Plate_001.fcs".to_string(),
    "C:/Users/ts286220/Documents/FACS Temp/ELX21208_Th17MAITHUPDBMXA/Unmixed/Plate_001/DN 382/G8 FMX_2_Plate_001.fcs"
                                    .to_string()]);

    let mut sample_index = use_signal(||0);


    let mut sample = use_signal(|| {
        samples.read()[0].clone()
    });
    // let mut fcs_data: Signal<Option<FlowSample>> = use_signal(|| None);
    let mut processed_data = use_signal(||None);
    let mut message = use_signal(|| "No data loaded".to_string());

    let x_axis_parameter = use_signal(|| "CD4".to_string());
    let y_axis_parameter = use_signal(|| "CD8".to_string());

    use_effect(move || {
        sample.set(samples.read()[*sample_index.read()].clone());

    });
    let fcs_file = use_resource(move || {
        let name = sample.read().clone();
        async move {
            println!("resouce running");
            get_flow_data(name).await
    }});

    let fcs_file_data = use_memo(move || {
        println!("data read memo called");
        match &*fcs_file.read() {
            Some(Ok(d)) => {

                Some(d.clone())
                
            }
            Some(Err(e)) => {
                let error_s = format!("Error loading data: {}", e.to_string());
                message.set(error_s);
                None
            }
            None => {
                message.set("Loading data.".to_string());
                None
            }
        }
    });

    let data_to_display = use_resource(move || {
        let x = x_axis_parameter.read().clone();
        let y = y_axis_parameter.read().clone();
        let data =  fcs_file_data.read().clone();
        async move {
            get_data_to_display(data, &x, &y, 4000_f64, 4000_f64).await
    }
    
        

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

    
    // let _fcs_file_data = use_memo(move || {
    //     println!("data read memo called");
    //     match &*fcs_file.read() {
    //         Some(Ok(d)) => {

    //             Some(d.clone())
                
    //         }
    //         Some(Err(e)) => {
    //             let error_s = format!("Error loading data: {}", e.to_string());
    //             message.set(error_s);
    //             None
    //         }
    //         None => {
    //             message.set("Loading data.".to_string());
    //             None
    //         }
    //     }
    // });

    use_effect(move || {

        match &*data_to_display.read() {
            Some(Ok(d)) => processed_data.set(Some(d.clone())),
            Some(Err(e)) => {
                processed_data.set(None);
                message.set(format!("Error processing data: {}", e.to_string()));
            }
            None => {
                processed_data.set(None);
                message.set(format!("No data"));
            },
        }
    });

    let element = use_memo(move || {
        println!("plot memo called");
        match processed_data() {
            Some(_) => {
                rsx! {
                    div {
                        Plotters {
                            size: (400, 400), // Define the size of the plot image
                            data: processed_data, // Pass a clone of the generated data

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
                        let mut curr = sample_index();
                        let length = samples.len() - 1;
                        println!("curr: {}, samples.len: {}", curr, length);
                        if curr == length {
                            curr = 0;
                        } else {
                            curr += 1;
                        };
                        println!("{}", curr);
                        sample_index.set(curr);
                    },
                    "Update Data"
                }
                div { margin_top: "1rem", {sample()} }
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
