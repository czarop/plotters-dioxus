use dioxus::prelude::*;

use crate::{
    file_load::FcsFiles,
    gate_store::{GateState, Id},
    plotters_dioxus::{AxisInfo, PseudoColourPlot, plot_helpers::Param},
    searchable_select::SearchableSelect,
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

static CSS_STYLE: Asset = asset!("assets/plot_window.css");

#[component]
pub fn PlotWindow() -> Element {
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

    // RESOURCE 1: Load FCS File
    // This resource re-runs when `current_sample` changes
    let fcs_file_resource = use_resource(move || async move {
        if let Some(sample) = current_sample() {
            get_flow_data(sample.full_path).await
        } else {
            Err(Arc::new(anyhow::anyhow!("No file path selected.")))
        }
    });

    let sorted_params = use_memo(move || {
        // let hashmap = marker_to_fluoro_map();
        if let Some(Ok(fcs_file)) = fcs_file_resource.read().clone() {
            let mut sorted_params: Vec<Param> = fcs_file
                .parameters
                .iter()
                .map(|(_, param)| Param {
                    marker: param.label_name.clone(),
                    fluoro: param.channel_name.clone(),
                })
                .collect();

            // Sort by parameter number
            sorted_params.sort_by_key(|param| {
                fcs_file
                    .parameters
                    .get(param.fluoro.as_ref())
                    .map(|p| p.parameter_number)
                    .unwrap_or(usize::MAX)
            });

            sorted_params
        } else {
            Vec::new()
        }
    });

    use_effect(move || {
        if let Some(Ok(fcs_file)) = fcs_file_resource.read().clone() {
            for param in sorted_params.iter() {
                parameter_settings
                    .write()
                    .entry(param.fluoro.clone())
                    .or_insert_with(|| {
                        let is_fluoresence_channel;
                        if let Some(param) = fcs_file.parameters.get(&param.fluoro) {
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
                            AxisInfo {
                                title: param.fluoro.clone(),
                                lower,
                                upper,
                                transform: TransformType::Arcsinh { cofactor: cofactor },
                            }
                        } else {
                            println!("{}, {} linear", param.marker, param.fluoro);
                            AxisInfo {
                                title: param.fluoro.clone(),
                                lower: 0_f32,
                                upper: 4194304_f32,
                                transform: TransformType::Linear,
                            }
                        }
                    });
            }
        }
    });

    let x_axis_marker: Signal<Param> = use_signal(|| {
        let p: Arc<str> = Arc::from("FSC-A");
        Param {
            marker: p.clone(),
            fluoro: p,
        }
    });
    let y_axis_marker = use_signal(|| {
        let p: Arc<str> = Arc::from("SSC-A");
        Param {
            marker: p.clone(),
            fluoro: p,
        }
    });
    let mut x_cofactor = use_signal(|| 6000.0f32);
    let mut y_cofactor = use_signal(|| 6000.0f32);

    // fetch the axis limits from the settings dict when axis changed
    let x_axis_limits = use_memo(move || {
        let param = x_axis_marker();

        match parameter_settings.read().get(&param.fluoro) {
            Some(d) => Some(d.clone()),
            None => None,
        }
    });

    use_effect(move || {
        let param = x_axis_marker.peek().clone();
        let x_co = *x_cofactor.read();

        let mut settings = parameter_settings.write();
        settings.entry(param.fluoro.clone()).and_modify(|axis| {
            let old_axis = std::mem::take(axis);
            let new_axis = match old_axis.transform {
                TransformType::Linear => old_axis,
                TransformType::Arcsinh { .. } => match old_axis.into_archsinh(x_co) {
                    Ok(a) => a,
                    Err(_) => old_axis,
                },
                _ => old_axis,
            };

            *axis = new_axis;
        });
    });

    let y_axis_limits = use_memo(move || {
        let param = y_axis_marker();

        match parameter_settings.read().get(&param.fluoro) {
            Some(d) => Some(d.clone()),
            None => None,
        }
    });

    use_effect(move || {
        let param = y_axis_marker.peek().clone();
        let y_co = *y_cofactor.read();

        let mut settings = parameter_settings.write();
        settings.entry(param.fluoro.clone()).and_modify(|axis| {
            let old_axis = std::mem::take(axis);
            let new_axis = match old_axis.transform {
                TransformType::Linear => old_axis,
                TransformType::Arcsinh { .. } => match old_axis.into_archsinh(y_co) {
                    Ok(a) => a,
                    Err(_) => old_axis,
                },
                _ => old_axis,
            };

            *axis = new_axis;
        });
    });

    let processed_data_resource = use_resource(move || {
        let data = fcs_file_resource.read().clone();
        let x_fluoro = x_axis_marker.read().fluoro.clone();
        let y_fluoro = y_axis_marker.read().fluoro.clone();
        async move {
            let d = data.and_then(|res| res.ok());

            match d {
                Some(_) => {}
                None => return Err(anyhow::anyhow!("No data yet")),
            };

            let x_transform = {
                if let Some(axis) = parameter_settings.get(&x_fluoro) {
                    axis.transform.clone()
                } else {
                    return Err(anyhow::anyhow!("No data yet"));
                }
            };
            let y_transform = {
                if let Some(axis) = parameter_settings.get(&y_fluoro) {
                    axis.transform.clone()
                } else {
                    return Err(anyhow::anyhow!("No data yet"));
                }
            };
            get_scaled_data_to_display(d, &x_fluoro, &y_fluoro, x_transform, y_transform).await
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_STYLE }

        div {
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

                SearchableSelect {
                    items: sorted_params(),
                    selected_value: x_axis_marker,
                    placeholder: x_axis_marker.peek().to_string(),
                }

                div { class: "control-group",
                    label { "X-Axis Cofactor:" }
                    input {
                        r#type: "number",
                        value: "{x_cofactor.read()}",
                        disabled: if x_axis_limits.read().is_none() || x_axis_limits.read().as_ref().unwrap().is_linear() { true } else { false },
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

                SearchableSelect {
                    items: sorted_params(),
                    selected_value: y_axis_marker,
                    placeholder: y_axis_marker.peek().to_string(),
                }

                div { class: "control-group",
                    label { "Y-Axis Cofactor:" }
                    input {
                        r#type: "number",
                        value: "{y_cofactor.read()}",
                        disabled: if y_axis_limits.read().is_none() || y_axis_limits.read().as_ref().unwrap().is_linear() { true } else { false },
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

            {

                if let Some(Ok(plot_data)) = &*processed_data_resource.read() {

                    rsx! {
                        div {
                            PseudoColourPlot {
                                size: (600, 600),
                                data: plot_data.clone(),
                                x_axis_info: x_axis_limits.read().as_ref().unwrap().clone(),
                                y_axis_info: y_axis_limits.read().as_ref().unwrap().clone(),
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
