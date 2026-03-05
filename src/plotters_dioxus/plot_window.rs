use dioxus::prelude::*;
use flow_gates::Gate;

use crate::{
    file_load::FcsFiles,
    plotters_dioxus::{
        AxisInfo, PseudoColourPlot, gate_sidebar::GateSidebar, gates::{GateState, gate_store::{GateStateImplExt, GateStateStoreExt}, gate_traits::DrawableGate, gate_types::GateType}, plot_helpers::{Param, ParameterStore, ParameterStoreImplExt, ParameterStoreStoreExt as _}
    },
    searchable_select::SearchableSelect,
};
use flow_fcs::{Fcs, TransformType, Transformable};
use crate::components::sidebar::*;
use crate::plotters_dioxus::gates::gate_buttons::NewGateButtons;
use std::sync::Arc;
use tokio::task;

async fn get_flow_data(path: std::path::PathBuf) -> Result<Arc<Fcs>, Arc<anyhow::Error>> {
    task::spawn_blocking(move || {
        let fcs_file = Fcs::open(path.to_str().unwrap_or_default())?;
        Ok(Arc::new(fcs_file))
    })
    .await
    .map_err(|e| Arc::new(e.into()))?
}

// async fn get_scaled_data_to_display(
//     fs: Arc<Fcs>,
//     col1_name: &str,
//     col2_name: &str,
//     transform_1: TransformType,
//     transform_2: TransformType,
//     parental_gate_id: &Option<Arc<str>>
// ) -> Result<Vec<(f32, f32)>, anyhow::Error> {
//     let fs_clone = fs.clone();
//     let col1_name = col1_name.to_string();
//     let col2_name = col2_name.to_string();
//     let gate_store: Store<GateState> = use_context::<Store<GateState>>();
//     let gate_chain: Option<Vec<(Arc<str>, Arc<dyn DrawableGate>)>> = if let Some(parent) = parental_gate_id {
        
//         let arcs: Vec<(Arc<str>, Arc<dyn DrawableGate>)> = gate_store.hierarchy().peek().get_chain_to_root(parent)
//             .iter()
//             .filter_map(|id| {
//                 gate_store.gate_registry().peek().get(id).map(|g| (id.clone(), g.clone()))
//     })
//             .collect();
        
//         if arcs.is_empty() { None } else { Some(arcs) }
//     } else {
//         None
//     };

//     task::spawn_blocking(move || -> Result<Vec<(f32, f32)>, anyhow::Error> {
        
//         let cols = fs_clone.get_xy_pairs(&col1_name, &col2_name)?;
//         let zipped_cols: Vec<(f32, f32)>;
        
//         if let Some(chain) = gate_chain {
            
//             let gate_refs: Vec<&Gate> = chain.iter()
//             .filter_map(|(id, gate)| gate.get_gate_ref(Some(id.clone()))) 
//             .collect();

//             let indices = flow_gates::filter_events_by_hierarchy(&fs_clone, &gate_refs, None, None)?;
//             zipped_cols = indices
//                 .into_iter()
//                 .map(|idx| {
//                     let x = cols[idx].0;
//                     let y = cols[idx].1;
//                     (transform_1.transform(&x), transform_2.transform(&y))
//                 })
//                 .collect();
//         } else {
//             zipped_cols = cols
//                 .into_iter()
//                 .map(|(x, y)| (transform_1.transform(&x), transform_2.transform(&y)))
//                 .collect();
//         }
//         Ok(zipped_cols)
//     })
//     .await?
// }

async fn get_scaled_data_to_display(
    fs: Arc<Fcs>,
    col1_name: &str,
    col2_name: &str,
    transform_1: TransformType,
    transform_2: TransformType,
    parental_gate_id: &Option<Arc<str>>
) -> Result<Vec<(f32, f32)>, anyhow::Error> {
    let fs_clone = fs.clone();
    let col1_name = col1_name.to_string();
    let col2_name = col2_name.to_string();
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let gate_chain: Option<Vec<(Arc<str>, Arc<dyn DrawableGate>)>> = if let Some(parent) = parental_gate_id {
        println!("parental gate id is {}", parent);
        let arcs: Vec<(Arc<str>, Arc<dyn DrawableGate>)> = gate_store.hierarchy().peek().get_chain_to_root(parent)
            .iter()
            .filter_map(|id| {
                gate_store.gate_registry().peek().get(id).map(|g| (id.clone(), g.clone()))
    })
            .collect();
        
        if arcs.is_empty() { None } else { Some(arcs) }
    } else {
        None
    };

    task::spawn_blocking(move || -> Result<Vec<(f32, f32)>, anyhow::Error> {
    let mut df = fs_clone.data_frame.as_ref().clone();

    if let Some(chain) = gate_chain {
        let gate_refs: Vec<&Gate> = chain.iter()
            .filter_map(|(id, gate)| gate.get_gate_ref(Some(id.clone()))) 
            .collect();

        // 1. Get the final narrowed mask for the whole hierarchy
        let mask = super::gates::gate_filtering::filter_events_by_hierarchy_to_mask(&fs_clone, &gate_refs)?;

        // 2. Filter the dataframe once at the end
        df = df.filter(&mask)?;
    }

    // 3. Extract and Transform (The heavy math)
    let x_series = df.column(&col1_name)?.f32()?;
    let y_series = df.column(&col2_name)?.f32()?;

    let zipped_cols: Vec<(f32, f32)> = x_series.into_iter()
        .zip(y_series.into_iter())
        .filter_map(|(x, y)| {
            match (x, y) {
                (Some(vx), Some(vy)) => Some((transform_1.transform(&vx), transform_2.transform(&vy))),
                _ => None
            }
        })
        .collect();

    Ok(zipped_cols)
})
    .await?
}

static CSS_STYLE: Asset = asset!("assets/plot_window.css");

#[component]
pub fn PlotWindow() -> Element {
    let mut filehandler: Signal<Option<FcsFiles>> = use_signal(|| None);
    let mut message = use_signal(|| None::<String>);
    let mut gate_store = use_store(|| GateState::default());
    use_context_provider(|| gate_store);

    let mut current_gate_type = use_signal(|| GateType::Polygon);
    use_context_provider(|| current_gate_type);

    let _ = use_resource(move || async move {
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

    let mut sample_index = use_signal(|| 0);
    let mut parameter_settings = use_store(|| ParameterStore::default());
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
    let fcs_file_resource = use_resource(move || async move {
        if let Some(sample) = current_sample() {
            get_flow_data(std::path::PathBuf::from(sample.get_filepath())).await
        } else {
            Err(Arc::new(anyhow::anyhow!("No file path selected.")))
        }
    });

    let sorted_params = use_memo(move || {
        if let Some(Ok(fcs_file)) = &*fcs_file_resource.read() {
            // pull the parameters from the file
            let mut sorted_params: Vec<Param> = fcs_file
                .parameters
                .iter()
                .map(|(_, param)| {
                    let p = Param {
                        marker: param.label_name.clone(),
                        fluoro: param.channel_name.clone(),
                    };
                    // add the parameter to the store if required
                    parameter_settings.add_new_axis_settings(&p, &fcs_file);
                    p
                })
                .collect();

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

    let mut x_axis_marker: Signal<Param> = use_signal(|| {
        let p: Arc<str> = Arc::from("FSC-A");
        Param {
            marker: p.clone(),
            fluoro: p,
        }
    });
    let mut y_axis_marker = use_signal(|| {
        let p: Arc<str> = Arc::from("SSC-A");
        Param {
            marker: p.clone(),
            fluoro: p,
        }
    });

    // fetch the axis limits from the settings dict when axis changed
    let x_axis_limits = use_memo(move || {
        let param = x_axis_marker.read();
        match parameter_settings.settings().get(param.fluoro.clone()) {
            Some(d) => d().clone(),
            None => AxisInfo::default(),
        }
    });

    let y_axis_limits = use_memo(move || {
        let param = y_axis_marker();
        match parameter_settings.settings().get(param.fluoro.clone()) {
            Some(d) => d().clone(),
            None => AxisInfo::default(),
        }
    });

    // this should be set when smth is selected in the sidebar
    let mut parental_gate: Signal<Option<Arc<str>>> = use_signal(|| None);

    let mut plot_data_signal = use_signal(|| vec![]);
    let processed_data_resource = use_resource(move || {
        let x_fluoro = x_axis_marker.read().fluoro.clone();
        let y_fluoro = y_axis_marker.read().fluoro.clone();
        

        async move {
            let parental_gate = &*parental_gate.read();
            let x_transform = parameter_settings
                .settings()
                .get(x_fluoro.clone())
                .ok_or_else(|| anyhow::anyhow!("No data yet"))?()
            .transform
            .clone();
            let y_transform = parameter_settings
                .settings()
                .get(y_fluoro.clone())
                .ok_or_else(|| anyhow::anyhow!("No data yet"))?()
            .transform
            .clone();

            if let Some(Ok(fcs_file)) = &*fcs_file_resource.read() {
                match get_scaled_data_to_display(
                    fcs_file.clone(),
                    &x_fluoro,
                    &y_fluoro,
                    x_transform,
                    y_transform,
                    parental_gate
                )
                .await
                {
                    Ok(d) => {
                        plot_data_signal.set(d);
                        Ok(())
                    }
                    Err(e) => {
                        plot_data_signal.set(vec![]);
                        Err(anyhow::anyhow!(e.to_string()))
                    }
                }
            } else {
                plot_data_signal.set(vec![]);
                Err(anyhow::anyhow!("No data yet"))
            }
        }
    });

    rsx! {
        document::Stylesheet { href: CSS_STYLE }
        div { class: "sidebar-local",
            // SidebarProvider {
            //     Sidebar { variant: SidebarVariant::Sidebar,
            //         SidebarContent {
            GateSidebar { selected_id: parental_gate }
            //     }
            // }
            main { class: "main-content",

                div { class: "gate-window",

                    div { class: "axis-controls-grid", style: "width: 600px;",
                        div { class: "grid-label", "X-Axis" }
                        SearchableSelect {
                            items: sorted_params(),
                            on_select: move |(_, v)| { x_axis_marker.set(v) },
                            placeholder: x_axis_marker.peek().to_string(),
                        }

                        div { class: "input-unit",
                            label { "Cofactor" }
                            input {
                                r#type: "number",
                                value: "{x_axis_limits.read().get_cofactor().unwrap_or_default().round()}",
                                disabled: if x_axis_limits.read().is_linear() { true } else { false },
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<i32>() {
                                        if val >= 1 {
                                            let param = x_axis_marker.peek();
                                            let res = parameter_settings.update_cofactor(&param.fluoro, val as f32);
                                            match res {
                                                Some((old, new)) => {
                                                    match gate_store.rescale_gates(&param.fluoro, &old, &new) {
                                                        Ok(_) => message.set(None),
                                                        Err(e) => {
                                                            message.set(Some(e.join("\n")));
                                                        }
                                                    };
                                                }
                                                None => {}
                                            }

                                        } else {
                                            message
                                                .set(
                                                    Some("Arcsinh cofactor should be a positive integer".to_string()),
                                                );
                                        }
                                    }
                                },
                                step: "any",
                            }
                        }
                        div { class: "input-unit",
                            label { "Lower" }
                            input {
                                r#type: "number",
                                value: "{x_axis_limits.read().get_untransformed_lower().round()}",
                                disabled: if x_axis_limits.read().is_linear() { true } else { false },
                                oninput: move |e| {
                                    if let Ok(lower) = e.value().parse::<i32>() {
                                        let param = x_axis_marker.peek();
                                        parameter_settings.update_lower(&param.fluoro, lower as f32);
                                    }
                                },
                            }
                        }
                        div { class: "input-unit",
                            label { "Upper" }
                            input {
                                r#type: "number",
                                value: "{x_axis_limits.read().get_untransformed_upper().round()}",
                                oninput: move |e| {
                                    if let Ok(upper) = e.value().parse::<i32>() {
                                        let param = x_axis_marker.peek();
                                        parameter_settings.update_upper(&param.fluoro, upper as f32);
                                    }
                                },
                            }
                        }

                        div { class: "grid-label", "Y-Axis" }
                        SearchableSelect {
                            items: sorted_params(),
                            on_select: move |(_, v)| { y_axis_marker.set(v) },
                            placeholder: y_axis_marker.peek().to_string(),
                        }

                        div { class: "input-unit",
                            label { "Cofactor" }
                            input {
                                r#type: "number",
                                value: "{y_axis_limits.read().get_cofactor().unwrap_or_default().round()}",
                                disabled: if y_axis_limits.read().is_linear() { true } else { false },
                                oninput: move |evt| {
                                    if let Ok(val) = evt.value().parse::<i32>() {
                                        if val >= 1 {
                                            message.set(None);
                                            let param = y_axis_marker.peek();
                                            let res = parameter_settings.update_cofactor(&param.fluoro, val as f32);
                                            match res {
                                                Some((old, new)) => {
                                                    match gate_store.rescale_gates(&param.fluoro, &old, &new) {
                                                        Ok(_) => message.set(None),
                                                        Err(e) => {
                                                            message.set(Some(e.join("\n")));
                                                        }
                                                    };
                                                }
                                                None => {}
                                            }
                                        } else {
                                            message
                                                .set(
                                                    Some("Arcsinh cofactor should be a positive integer".to_string()),
                                                );
                                        }
                                    }
                                },
                                step: "any",
                            }
                        }
                        div { class: "input-unit",
                            label { "Lower" }
                            input {
                                r#type: "number",
                                value: "{y_axis_limits.read().get_untransformed_lower().round()}",
                                disabled: if y_axis_limits.read().is_linear() { true } else { false },
                                oninput: move |e| {
                                    if let Ok(lower) = e.value().parse::<i32>() {
                                        let param = y_axis_marker.peek();
                                        parameter_settings.update_lower(&param.fluoro, lower as f32);
                                    }
                                },
                            }
                        }
                        div { class: "input-unit",
                            label { "Upper" }
                            input {
                                r#type: "number",
                                value: "{y_axis_limits.read().get_untransformed_upper().round()}",
                                oninput: move |e| {
                                    if let Ok(upper) = e.value().parse::<i32>() {
                                        let param = y_axis_marker.peek();
                                        parameter_settings.update_upper(&param.fluoro, upper as f32);
                                    }
                                },
                            }
                        }
                    }
                    div { class: "file-info",
                        div { class: "file-info_button-panel",
                            button {
                                onclick: move |_| {
                                    if let Some(fcsfiles) = &*filehandler.read() {
                                        let count = fcsfiles.sample_count();
                                        let prev_index = (*sample_index.read() + count - 1) % count;
                                        sample_index.set(prev_index);
                                    }

                                },
                                "Prev"
                            }
                            button {
                                onclick: move |_| {
                                    if let Some(fcsfiles) = &*filehandler.read() {
                                        let next_index = (*sample_index.read() + 1) % fcsfiles.sample_count();
                                        sample_index.set(next_index);
                                    }

                                },
                                "Next"
                            }
                        }
                        match &*filehandler.read() {
                            Some(fh) => {
                                let list = fh.get_file_names();
                                rsx! {
                                    SearchableSelect {
                                        items: list,
                                        on_select: move |(i, _)| { sample_index.set(i) },
                                        placeholder: "Select a file".to_string(),
                                        selected_index: Some(sample_index.into()),
                                    }
                                }
                            }
                            None => rsx! {},
                        }
                    
                    }
                }
                div { class: "status-message",
                    {
                        match &*processed_data_resource.read() {
                            Some(Ok(_)) => {
                                rsx! {}
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

                    if !plot_data_signal.read().is_empty() {

                        rsx! {
                            div {
                                NewGateButtons { callback: move |gate_type| current_gate_type.set(gate_type) }
                                PseudoColourPlot {
                                    size: (600, 600),
                                    data: plot_data_signal,
                                    x_axis_info: x_axis_limits.read().clone(),
                                    y_axis_info: y_axis_limits.read().clone(),
                                    parental_gate_id: parental_gate,
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
    }

