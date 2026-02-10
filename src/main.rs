#![allow(non_snake_case)]

use clingate::{
    Select, SelectGroup, SelectGroupLabel, SelectItemIndicator, SelectList, SelectOption, SelectTrigger, SelectValue, file_load::FcsFiles, gate_store::{GateState, Id}, plotters_dioxus::{AxisInfo, PseudoColourPlot}
};
use dioxus::{
    desktop::{Config, LogicalSize, WindowBuilder}, html::param, prelude::*
};

use flow_fcs::{Fcs, TransformType, Transformable, transform};
use flow_gates::transforms::Axis;
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

async fn get_scaled_data_to_display(
    fs: Option<Arc<Fcs>>,
    col1_name: &str,
    col2_name: &str,
    transform_1: TransformType,
    transform_2: TransformType,
) -> Result<Arc<Vec<(f32, f32)>>, anyhow::Error> {
    let ts_fs =
        fs.ok_or_else(|| anyhow::anyhow!("No FCS data available for processing.".to_string()))?;

    let ts_fs_clone = ts_fs.clone();
    let col1_name = col1_name.to_string();
    let col2_name = col2_name.to_string();
    task::spawn_blocking(move || -> Result<Arc<Vec<(f32, f32)>>, anyhow::Error> {
        let cols = ts_fs_clone.get_xy_pairs(&col1_name, &col2_name).expect("");
        
        let zipped_cols: Vec<(f32, f32)> = cols
            .into_iter()
            .map(|(x, y)| (transform_1.transform(&x), transform_2.transform(&y)))
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
static COMPONENTS_STYLE: Asset = asset!("assets/dx-components-theme.css");

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
            let path = content
                .lines()
                .find(|l| !l.trim().is_empty())
                .ok_or_else(|| anyhow::anyhow!("No path found"))?;

            FcsFiles::create(path.trim())
        })();

        match result {
            Ok(files) => {
                message.set(None);
                filehandler.set(Some(files));
            }
            Err(e) => message.set(Some(e.to_string())),
        }
    });

    // Primary States
    let mut sample_index = use_signal(|| 0);
    let mut parameter_settings: Signal<HashMap<Id, AxisInfo>> = use_signal(|| HashMap::default());
    use_context_provider(|| parameter_settings);
    
    let current_sample = use_memo(move || {
        let handler = filehandler.read();
        let index = *sample_index.read();

        if handler.is_some() {
            message.set(None);
            Some(handler.as_ref().unwrap().file_list()[index].clone())
        } else {
            message.set(Some("Select working directory to load files".to_string()));
            None
        }
    });

    let mut x_axis_marker = use_signal(|| Arc::from("FSC-A"));
    let mut y_axis_marker = use_signal(|| Arc::from("SSC-A"));
    let mut x_cofactor = use_signal(|| 6000.0f32);
    let mut y_cofactor = use_signal(|| 6000.0f32);

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
            let name_param: HashMap<Arc<str>, Arc<str>> = fcs_file
                .parameters
                .iter()
                .map(|(key, val)| {
                    println!("{}, {}", val.label_name.clone(), key.clone());
                    (val.label_name.clone(), key.clone())
                })
                .collect();
            name_param
        } else {
            HashMap::new()
        }
    });

    let sorted_params = use_memo(move || {
        let hashmap = marker_to_fluoro_map();
        if let Some(Ok(fcs_file)) = fcs_file_resource.peek().clone() {
            
            let mut sorted_params: Vec<(Arc<str>, Arc<str>)> = hashmap
            .into_iter()
            .collect();
        
        // Sort by parameter number
        sorted_params.sort_by_key(|(_, param_key)| {
            fcs_file.parameters.get(param_key.as_ref())
                .map(|p| p.parameter_number)
                .unwrap_or(usize::MAX)
        });
        
        sorted_params
    } else {
        Vec::new()
    }
    });

    use_effect(move || {
        let file_params = &*marker_to_fluoro_map.read();
        if let Some(Ok(fcs_file)) = (*fcs_file_resource.read()).clone() {
            for (_, fluoro) in file_params.iter() {
                parameter_settings.write().entry(fluoro.clone()).or_insert_with(|| {
                    let is_fluoresence_channel;
                    if let Some(param) = fcs_file.parameters.get(fluoro){
                        is_fluoresence_channel = {
                            match param.transform {
                                TransformType::Linear => false,
                                _ => true,
                                }
                        };
                    } else {
                        is_fluoresence_channel = true;
                    };
                    if is_fluoresence_channel {
                        let cofactor = 6000_f32;
                        let lower = asinh_transform_f32(-10000_f32, cofactor);
                        let upper = asinh_transform_f32(4194304_f32, cofactor);
                        AxisInfo { title: fluoro.clone(), lower, upper, transform: TransformType::Arcsinh { cofactor: cofactor } }
                    } else {
                        AxisInfo { title: fluoro.clone(), lower: 0_f32, upper: 4194304_f32, transform: TransformType::Linear }
                    }
                });
            }
        }
    });

    // fetch the axis limits from the settings dict when axis changed
    let x_axis_limits = use_memo(move || {
        let marker = x_axis_marker();
        if let Some(fluoro) = marker_to_fluoro_map.get(&marker){
        if let Some(axis_info) = parameter_settings.get(&fluoro){
            return axis_info.clone();
        } else {
            let x_co = 6000_f32;
            let scaled_x_lower = asinh_transform_f32(-10000_f32, x_co);
            let scaled_x_upper = asinh_transform_f32(4194304_f32, x_co);
            return AxisInfo {
                title: fluoro.clone(),
                lower: scaled_x_lower,
                upper: scaled_x_upper,
                transform: TransformType::Arcsinh { cofactor: x_co },
            };
    }
        }
        AxisInfo::default()
    });
    



    use_effect(move || {
        let marker = x_axis_marker.peek().clone();
        let x_co = *x_cofactor.read();
        if let Some(fluoro) = marker_to_fluoro_map().get(&marker) {
        parameter_settings.write()
            .entry(fluoro.clone())
            .and_modify(|axis| {
                // Update the cofactor in the existing transform
                axis.transform = match &axis.transform {
                    TransformType::Arcsinh{..}=>TransformType::Arcsinh{cofactor:x_co},
                    _ => axis.transform.clone()
                };
            })
            .or_insert_with(|| {
                // Entry doesn't exist - create new
                AxisInfo {
                    title: fluoro.clone(),
                    lower: asinh_transform_f32(-10000_f32, x_co),
                    upper: asinh_transform_f32(4194304_f32, x_co),
                    transform: TransformType::Arcsinh { cofactor: x_co },
                }
            });
        }
    });

    let y_axis_limits = use_memo(move || {
        let marker = y_axis_marker();
        if let Some(fluoro) = marker_to_fluoro_map.get(&marker){
        if let Some(axis_info) = parameter_settings.get(&fluoro){
            return axis_info.clone();
        } else {
            let y_co = 6000_f32;
            let scaled_y_lower = asinh_transform_f32(-10000_f32, y_co);
            let scaled_y_upper = asinh_transform_f32(4194304_f32, y_co);
            return AxisInfo {
                title: fluoro.clone(),
                lower: scaled_y_lower,
                upper: scaled_y_upper,
                transform: TransformType::Arcsinh { cofactor: y_co },
            };
    }
        }
        AxisInfo::default()
    });

    use_effect(move || {
        let marker = y_axis_marker.peek().clone();
        let y_co = *y_cofactor.read();
        if let Some(fluoro) = marker_to_fluoro_map().get(&marker) {
            parameter_settings.write()
                .entry(fluoro.clone())
                .and_modify(|axis| {
                    // Update the cofactor in the existing transform
                    axis.transform = match &axis.transform {
                        TransformType::Arcsinh{..}=>TransformType::Arcsinh{cofactor:y_co},
                        _ => axis.transform.clone()
                    };
                })
                .or_insert_with(|| {
                    // Entry doesn't exist - create new
                    AxisInfo {
                        title: fluoro.clone(),
                        lower: asinh_transform_f32(-10000_f32, y_co),
                        upper: asinh_transform_f32(4194304_f32, y_co),
                        transform: TransformType::Arcsinh { cofactor: y_co },
                    }
                });
        }
    });

    // RESOURCE 2: Process Data for Display
    // This resource re-runs when:
    // - fcs_file_resource's value becomes available (or changes if it were mutable)
    // - x_axis_param, y_axis_param, x_cofactor, or y_cofactor changes
    let processed_data_resource = use_resource(move || {
        let data = fcs_file_resource.read().clone(); // Read the current state of the FCS file resource
        
        
        
        let x_marker = x_axis_marker.read().clone();
        let y_marker = y_axis_marker.read().clone();

        let x_fluoro = marker_to_fluoro_map
            .read()
            .get(&x_marker)
            .unwrap_or(&x_marker)
            .clone();
        let y_fluoro = marker_to_fluoro_map
            .read()
            .get(&y_marker)
            .unwrap_or(&y_marker)
            .clone();
        

        async move {
            let d = data.and_then(|res| res.ok());

            match d {
            Some(_) => {},
            None => return Err(anyhow::anyhow!("No data yet")),
        };

            let x_transform = {
            if let Some(axis) = parameter_settings.get(&x_fluoro) {
                axis.transform.clone()
            } else {
                return Err(anyhow::anyhow!("No data yet"))
            }
            
        };
        let y_transform = {
            if let Some(axis) = parameter_settings.get(&y_fluoro) {
                axis.transform.clone()
            } else {
                return Err(anyhow::anyhow!("No data yet"))
            }
            
        };
            get_scaled_data_to_display(d, &x_fluoro, &y_fluoro, x_transform, y_transform).await
        }
    });



    rsx! {
        document::Stylesheet { href: CSS_STYLE }
        document::Stylesheet { href: COMPONENTS_STYLE }
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

                Select::<Arc<str>> {
                    default_value: Some(Arc::from("FSC-A")),
                    placeholder: "Select X Parameter",
                    on_value_change: move |val: Option<Arc<str>>| {
                        if val.is_some() {
                            let marker = val.unwrap();
                            x_axis_marker.set(marker.clone());
                            if let Some(fluoro) = marker_to_fluoro_map().get(&marker) {
                                if let Some(options) = parameter_settings.peek().get(fluoro) {
                                    if let TransformType::Arcsinh { cofactor } = options.transform {
                                        x_cofactor.set(cofactor);
                                    }
                                }
                            }
                        }
                    },
                    SelectTrigger { width: "12rem", SelectValue {} }
                    SelectList { aria_label: "Select x parameter",
                        SelectGroup {
                            SelectGroupLabel { "channels" }
                            for (i , (a , b)) in sorted_params().iter().enumerate() {
                                {
                                    let s;
                                    if a == b {
                                        s = format!("{}", a);
                                    } else {
                                        let trimmed = &b[..b.len().saturating_sub(2)];
                                        s = format!("{}-{}", a, trimmed);
                                    }

                                    rsx! {
                                        SelectOption::<Arc<str>> { index: i, value: a.clone(), text_value: "{s}",
                                            {s}
                                            SelectItemIndicator {}
                                        }
                                    }
                                }
                            }
                        }
                    
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
                Select::<Arc<str>> {
                    default_value: Some(Arc::from("SSC-A")),
                    placeholder: "Select Y Parameter",
                    on_value_change: move |val: Option<Arc<str>>| {
                        if val.is_some() {
                            let marker = val.unwrap();
                            y_axis_marker.set(marker.clone());
                            if let Some(fluoro) = marker_to_fluoro_map().get(&marker) {
                                if let Some(options) = parameter_settings.peek().get(fluoro) {
                                    if let TransformType::Arcsinh { cofactor } = options.transform {
                                        y_cofactor.set(cofactor);
                                    }
                                }
                            }
                        }
                    },
                    SelectTrigger { width: "12rem", SelectValue {} }
                    SelectList { aria_label: "Select y parameter",
                        SelectGroup {
                            SelectGroupLabel { "channels" }
                            for (i , (a , b)) in sorted_params().iter().enumerate() {
                                {
                                    let s;
                                    if a == b {
                                        s = format!("{}", a);
                                    } else {
                                        let trimmed = &b[..b.len().saturating_sub(2)];
                                        s = format!("{}-{}", a, trimmed);
                                    }
                                    rsx! {
                                        SelectOption::<Arc<str>> { index: i, value: a.clone(), text_value: "{s}",
                                            {s}
                                            SelectItemIndicator {}
                                        }
                                    }
                                }
                            }
                        }
                    
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
                            PseudoColourPlot {
                                size: (600, 600),
                                data: plot_data.clone(),
                                x_axis_info: x_axis_limits.read().clone(),
                                y_axis_info: y_axis_limits.read().clone(),
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
                    .with_inner_size(LogicalSize::new(1200.0, 900.0)),
            ),
        )
        .launch(App);
}