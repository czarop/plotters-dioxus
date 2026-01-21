#![allow(non_snake_case)]

// use gui::file_load;





use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use fcs_rs_2::{FcsError, FcsFile, FlowSample};
use std::rc::Rc;
use std::sync::{Arc, RwLock};
use tokio::task;

// Import your Plotters component from the `plotters-dioxus` crate.
use plotters_dioxus::{AxisLimits, Plotters};
use polars::prelude::*;

// --- Helper Functions (No changes needed, they're already good) ---
async fn get_flow_data(path: String) -> Result<Arc<RwLock<FlowSample>>, FcsError> {
    println!("Loading FCS file: {}", path);
    task::spawn_blocking(move || {
        let fcs_file = FcsFile::open(&path)?;
        let fcs_data = fcs_file.read()?; // This is the blocking read
        Ok(Arc::new(RwLock::new(fcs_data)))
    })
    .await?
}

async fn get_data_to_display(
    fs: Option<Arc<RwLock<FlowSample>>>,
    col1_name: &str,
    col2_name: &str,
    col1_cofactor: f64,
    col2_cofactor: f64,
) -> Result<Arc<Vec<(f64, f64)>>, FcsError> {
    let ts_fs = fs.ok_or_else(|| {
        FcsError::InvalidData("No FCS data available for processing.".to_string())
    })?;

    let col_names = vec![col1_name.to_string(), col2_name.to_string()];
    let cofactors = vec![col1_cofactor, col2_cofactor];
    let ts_fs_clone = ts_fs.clone();
    let col1_name = col1_name.to_string();
    let col2_name = col2_name.to_string();
    let result: std::result::Result<Arc<Vec<(f64, f64)>>, FcsError>  = task::spawn_blocking(move || -> Result<Arc<Vec<(f64, f64)>>, FcsError>{
        ts_fs_clone.write().unwrap().arcsinh_transform(&cofactors, &col_names).map_err(|e| FcsError::InvalidData(e.to_string()))?;
        let zipped_cols =get_zipped_column_data(&ts_fs.read().unwrap().data, &col1_name, &col2_name).map_err(|_| FcsError::InvalidData("invalid columns".to_string()))?;
        Ok(Arc::new(zipped_cols))
    })
    .await.map_err(|e| FcsError::InvalidData(e.to_string()))?;

    result
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

fn asinh_transform_f64(value: f64, cofactor: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return value;
    }
    (value / cofactor).asinh()
}



// --- Dioxus App Component ---

static CSS_STYLE: Asset = asset!("assets/styles.css");

#[component]
fn App() -> Element {
    // Hardcoded paths (will be selectable later)
    let samples = use_signal(|| {
        vec![
        "/Users/czarop/Downloads/unscaled_t/[PMA_IONO_STIM] H8 FMX_Plate_001.fcs".to_string(),
        "/Users/czarop/Downloads/unscaled_t/[PMA_IONO_STIM] H9 FS_Plate_001.fcs".to_string()
    ]
    });

    // Primary States
    let mut sample_index = use_signal(|| 0);
    let current_sample_path = use_memo(move || samples.read()[*sample_index.read()].clone());

    let mut x_axis_param = use_signal(|| "CD4".to_string());
    let mut y_axis_param = use_signal(|| "CD8".to_string());
    let mut x_cofactor = use_signal(|| 6000.0f64);
    let mut y_cofactor = use_signal(|| 6000.0f64);

    let x_axis_limits = use_memo(move || {
        let x_co = *x_cofactor.read();
        let scaled_x_lower = asinh_transform_f64(-10000_f64, x_co);
        let scaled_x_upper = asinh_transform_f64(4194304_f64, x_co);
        AxisLimits {
            lower: scaled_x_lower,
            upper: scaled_x_upper,
        }
    });

    let y_axis_limits = use_memo(move || {
        let y_co = *y_cofactor.read();
        let scaled_y_lower = asinh_transform_f64(-10000_f64, y_co);
        let scaled_y_upper = asinh_transform_f64(4194304_f64, y_co);
        AxisLimits {
            lower: scaled_y_lower,
            upper: scaled_y_upper,
        }
    });

    // RESOURCE 1: Load FCS File
    // This resource re-runs when `current_sample_path` changes
    let fcs_file_resource: Resource<Result<Arc<RwLock<FlowSample>>, FcsError>> = use_resource(move || {
        let path = current_sample_path.read().clone();
        async move { get_flow_data(path).await }
    });

    // RESOURCE 2: Process Data for Display
    // This resource re-runs when:
    // - fcs_file_resource's value becomes available (or changes if it were mutable)
    // - x_axis_param, y_axis_param, x_cofactor, or y_cofactor changes
    let processed_data_resource = use_resource(move || {
        println!("processed_data_resource future started");
        let data = fcs_file_resource.read().clone(); // Read the current state of the FCS file resource
        let x_param = x_axis_param.read().clone();
        let y_param = y_axis_param.read().clone();
        let x_cf = *x_cofactor.read();
        let y_cf = *y_cofactor.read();

        async move {
            // Pass the inner `Ok` value if available, or `None` if still loading/errored
            get_data_to_display(
                data.and_then(|res| res.ok()),
                &x_param,
                &y_param,
                x_cf,
                y_cf,
            )
            .await
        }
    });

    // --- Event Handlers for Plotters Component (Optional, as before) ---
    let handle_click = move |event: Rc<MouseData>| {
        println!(
            "Click event on plot: x={}, y={}",
            event.client_coordinates().x,
            event.client_coordinates().y
        );
    };

    let mut is_dragging = use_signal(|| false);
    let mut last_mouse_pos = use_signal(|| (0.0, 0.0));

    let handle_mousedown = move |evt: Rc<MouseData>| {
        is_dragging.set(true);
        last_mouse_pos.set((evt.client_coordinates().x, evt.client_coordinates().y));
    };

    let handle_mouseup = move |_evt: Rc<MouseData>| {
        is_dragging.set(false);
    };

    let handle_mousemove = move |evt: Rc<MouseData>| {
        if *is_dragging.read() {
            let (last_x, last_y) = *last_mouse_pos.read();
            let (current_x, current_y) = (evt.client_coordinates().x, evt.client_coordinates().y);

            let dx = current_x - last_x;
            let dy = current_y - last_y;

            last_mouse_pos.set((current_x, current_y));
        }
    };

    rsx! {
        document::Stylesheet { href: CSS_STYLE }
        div {
            h1 { "FCS Plot Viewer" }

            div { class: "controls",
                // File selection
                div { class: "control-group",
                    button {
                        onclick: move |_| {
                            let next_index = (*sample_index.read() + 1) % samples.read().len();
                            sample_index.set(next_index);
                        },
                        "Next FCS File"
                    }
                    p {
                        "Current File: {current_sample_path.read().split('/').last().unwrap_or_default()}"
                    }
                }

                // X-axis parameter selection
                div { class: "control-group",
                    label { "X-Axis Parameter:" }
                    input {
                        r#type: "text",
                        value: "{x_axis_param.read()}",
                        oninput: move |evt| x_axis_param.set(evt.value()),
                        placeholder: "e.g., CD4",
                    }
                }
                div { class: "control-group",
                    label { "X-Axis Cofactor:" }
                    input {
                        r#type: "number",
                        value: "{x_cofactor.read()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                x_cofactor.set(val);
                            }
                        },
                        step: "any",
                    }
                }

                // Y-axis parameter selection
                div { class: "control-group",
                    label { "Y-Axis Parameter:" }
                    input {
                        r#type: "text",
                        value: "{y_axis_param.read()}",
                        oninput: move |evt| y_axis_param.set(evt.value()),
                        placeholder: "e.g., CD8",
                    }
                }
                div { class: "control-group",
                    label { "Y-Axis Cofactor:" }
                    input {
                        r#type: "number",
                        value: "{y_cofactor.read()}",
                        oninput: move |evt| {
                            if let Ok(val) = evt.value().parse::<f64>() {
                                y_cofactor.set(val);
                            }
                        },
                        step: "any",
                    }
                }
            }

            div { class: "status-message",
                {
                    match &*processed_data_resource.read() {
                        Some(Ok(_)) => {
                            rsx! {
                                p { class: "loading-message", "Data Ready." }
                            }
                        }
                        Some(Err(e)) => {
                            rsx! {
                                p { class: "error-message", "Error: {e}" }
                            }
                        }
                        None => {
                            rsx! {
                                p { class: "loading-message", "Loading and processing data..." }
                            }
                        }
                    }
                }
            }

            // Conditional rendering of the plot
            {
                if let Some(Ok(plot_data)) = &*processed_data_resource.read() {
                    rsx! {
                        div {
                            Plotters {
                                size: (600, 600),
                                data: plot_data.clone(),
                                x_axis_limits: x_axis_limits.read().clone(),
                                y_axis_limits: y_axis_limits.read().clone(),
                                on_click: handle_click,
                                on_mousemove: handle_mousemove,
                                on_mousedown: handle_mousedown,
                                on_mouseup: handle_mouseup,
                            }
                        }
                    }
                } else {
                    rsx! {
                        div {
                            border: "1px solid #ddd",
                            width: "600px",
                            height: "600px",
                            display: "flex",
                            align_items: "center",
                            justify_content: "center",
                            background_color: "#f9f9f9",
                            color: "#888",
                            "Plot area (waiting for data)"
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    LaunchBuilder::new()
        .with_cfg(
            Config::new().with_window(
                WindowBuilder::new()
                    .with_title("FCS Plot Viewer")
                    .with_always_on_top(false)
                    .with_inner_size(LogicalSize::new(1200.0, 900.0)), // Larger window for controls
            ),
        )
        .launch(App);
}
