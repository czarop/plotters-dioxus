#![allow(non_snake_case)]

use clingate::{
    file_load::FcsFiles, gate_store::GateState, plotters_dioxus::{AxisInfo, Plotters}
};
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder},
    prelude::*,
};
use flow_fcs::{Fcs, TransformType, Transformable};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task;

async fn get_flow_data(path: std::path::PathBuf) -> Result<Arc<Fcs>, Arc<anyhow::Error>> {
    println!("Loading FCS file: {:?}", path);
    task::spawn_blocking(move || {
        let fcs_file = Fcs::open(path.to_str().unwrap_or_default())?;
        Ok(Arc::new(fcs_file))
    })
    .await
    .map_err(|e| Arc::new(e.into()))?
}

async fn get_data_to_display(
    fs: Option<Arc<Fcs>>,
    col1_name: &str,
    col2_name: &str,
    col1_cofactor: f32,
    col2_cofactor: f32,
) -> Result<Arc<Vec<(f32, f32)>>, anyhow::Error> {
    let ts_fs =
        fs.ok_or_else(|| anyhow::anyhow!("No FCS data available for processing.".to_string()))?;

    let ts_fs_clone = ts_fs.clone();
    let col1_name = col1_name.to_string();
    let col2_name = col2_name.to_string();
    task::spawn_blocking(move || -> Result<Arc<Vec<(f32, f32)>>, anyhow::Error> {
        let cols = ts_fs_clone.get_xy_pairs(&col1_name, &col2_name).expect("");
        let t1 = TransformType::Arcsinh {
            cofactor: col1_cofactor,
        };
        let t2 = TransformType::Arcsinh {
            cofactor: col2_cofactor,
        };
        let zipped_cols: Vec<(f32, f32)> = cols
            .into_iter()
            .map(|(x, y)| (t1.transform(&x), t2.transform(&y)))
            .collect();
        Ok(Arc::new(zipped_cols))
    })
    .await?
}

fn asinh_transform_f32(value: f32, cofactor: f32) -> f32 {
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
    let mut filehandler: Signal<Option<FcsFiles>> = use_signal(|| None);
    let mut message = use_signal(|| None::<String>);
    let gate_store = use_store(|| GateState::default());
    use_context_provider(|| gate_store);
    

    let _ = use_resource(move || async move {
        // Read the file from the project root
        let result = (|| -> anyhow::Result<FcsFiles> {
            let content = std::fs::read_to_string("file_paths.txt")?;
            let path = content.lines().find(|l| !l.trim().is_empty())
                .ok_or_else(|| anyhow::anyhow!("No path found"))?;
                
            FcsFiles::create(path.trim())
        })();

        match result {
            Ok(files) => {
                message.set(None);
                filehandler.set(Some(files));
            },
            Err(e) => message.set(Some(e.to_string())),
        }
    });

    // Primary States
    let mut sample_index = use_signal(|| 0);
    let current_sample = use_memo(move || {
    let handler = filehandler.read();
    let index = *sample_index.read();
    
    if handler.is_some(){
        message.set(None);
        Some(handler.as_ref().unwrap().file_list()[index].clone())
    } else {
        message.set(Some("Select working directory to load files".to_string()));
        None
    }
    
});

    let mut x_axis_param = use_signal(|| "CD4".to_string());
    let mut y_axis_param = use_signal(|| "CD8".to_string());
    let mut x_cofactor = use_signal(|| 6000.0f32);
    let mut y_cofactor = use_signal(|| 6000.0f32);

    let x_axis_limits = use_memo(move || {
        let x_co = *x_cofactor.read();
        let scaled_x_lower = asinh_transform_f32(-10000_f32, x_co);
        let scaled_x_upper = asinh_transform_f32(4194304_f32, x_co);
        AxisInfo {
            title: x_axis_param(),
            lower: scaled_x_lower,
            upper: scaled_x_upper,
            transform: TransformType::Arcsinh { cofactor: x_co },
        }
    });

    let y_axis_limits = use_memo(move || {
        let y_co = *y_cofactor.read();
        let scaled_y_lower = asinh_transform_f32(-10000_f32, y_co);
        let scaled_y_upper = asinh_transform_f32(4194304_f32, y_co);
        AxisInfo {
            title: y_axis_param(),
            lower: scaled_y_lower,
            upper: scaled_y_upper,
            transform: TransformType::Arcsinh { cofactor: y_co },
        }
    });

    // RESOURCE 1: Load FCS File
    // This resource re-runs when `current_sample` changes
    let fcs_file_resource = use_resource(move || async move {
        if let Some(sample) = current_sample() {
            get_flow_data(sample.full_path).await
        } else {
            Err(Arc::new(anyhow::anyhow!("No file path selected.")))
        }
    });

    let marker_to_fluoro_map = use_memo(move || {
        if let Some(Ok(fcs_file)) = fcs_file_resource.read().clone() {
            let name_param: HashMap<String, String> = fcs_file
                .parameters
                .iter()
                .map(|param| (param.1.label_name.to_string(), param.0.to_string()))
                .collect();
            name_param
        } else {
            HashMap::new()
        }
    });

    // RESOURCE 2: Process Data for Display
    // This resource re-runs when:
    // - fcs_file_resource's value becomes available (or changes if it were mutable)
    // - x_axis_param, y_axis_param, x_cofactor, or y_cofactor changes
    let processed_data_resource = use_resource(move || {
        // println!("processed_data_resource future started");
        let data = fcs_file_resource.read().clone(); // Read the current state of the FCS file resource
        let x_param = x_axis_param.read().clone();
        let y_param = y_axis_param.read().clone();
        // let map = *;
        let x_param = marker_to_fluoro_map
            .read()
            .get(&x_param)
            .unwrap_or(&x_param)
            .clone();
        let y_param = marker_to_fluoro_map
            .read()
            .get(&y_param)
            .unwrap_or(&y_param)
            .clone();
        let x_cf = *x_cofactor.read();
        let y_cf = *y_cofactor.read();

        async move {
            let d = data.and_then(|res| res.ok());
            get_data_to_display(d, &x_param, &y_param, x_cf, y_cf).await
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_STYLE }
        div {
            // h1 { "FCS Plot Viewer" }
            div { class: "controls",
                // File selection
                div { class: "control-group",
                    button {
                        onclick: move |_| {
                            if let Some(fcsfiles) = &*filehandler.read() {
                                let next_index = (*sample_index.read() + 1) % fcsfiles.sample_count();
                                sample_index.set(next_index);
                            }

                        },
                        "Next FCS File"
                    }
                    p {
                        match current_sample() {
                            Some(sample) => format!("Current File: {}", sample.name),
                            None => "No Files".to_string(),
                        }
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
                            if let Ok(val) = evt.value().parse::<f32>() {
                                if val > 0_f32 {
                                    message.set(None);
                                    x_cofactor.set(val);
                                } else {
                                    message.set(Some("Arcsinh cofactor must be > 0".to_string()));
                                }

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
                            if let Ok(val) = evt.value().parse::<f32>() {
                                if val > 0_f32 {
                                    message.set(None);
                                    y_cofactor.set(val);
                                } else {
                                    message.set(Some("Arcsinh cofactor must be > 0".to_string()));
                                }
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
                                x_axis_info: x_axis_limits.read().clone(),
                                y_axis_info: y_axis_limits.read().clone(),
                                // on_click: handle_click,
                            // on_mousemove: handle_mousemove,
                            // on_mousedown: handle_mousedown,
                            // on_mouseup: handle_mouseup,
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
