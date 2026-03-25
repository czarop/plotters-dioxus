use crate::FxIndexMap;
use crate::gate_editor::gates::gate_buttons::NewGateButtons;
use crate::gate_editor::plots::data_helpers::{
    get_event_mask_from_scaled_df, get_filtered_dataframe, get_flow_data, zip_cols_from_filtered_df,
};
use crate::gate_editor::plots::draw_plot::PseudoColourPlot;
use crate::gate_editor::plots::parameters::EventIndexMapped;
use crate::searchable_select::SearchableSelectMap;
use crate::{
    file_load::FcsFiles,
    gate_editor::{
        AxisInfo,
        gate_sidebar::GateSidebar,
        gates::{
            GateState,
            gate_store::{GateStateImplExt, ROOTGATE},
            gate_types::PrimaryGateType,
        },
        plots::parameters::{Param, PlotStore, PlotStoreImplExt, PlotStoreStoreExt as _},
    },
    searchable_select::SearchableSelectList,
};
use dioxus::prelude::*;
use dioxus::stores::use_store_sync;
use polars::frame::DataFrame;
use std::sync::Arc;

static CSS_STYLE: Asset = asset!("assets/plot_window.css");

#[component]
pub fn PlotWindow() -> Element {
    let mut filehandler: Signal<Option<FcsFiles>> = use_signal(|| None);
    let mut message = use_signal(|| None::<String>);
    let mut gate_store: Store<GateState, CopyValue<GateState, SyncStorage>> =
        use_store_sync(|| GateState::default());
    use_context_provider(|| gate_store);

    let mut current_gate_type = use_signal(|| PrimaryGateType::Polygon);
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
    let mut plot_store = use_store(|| PlotStore::default());
    use_context_provider(|| plot_store);

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
    let mut fcs_file_resource: Signal<Option<flow_fcs::Fcs>> = use_signal(|| None);
    let _ = use_resource(move || async move {
        if let Some(sample) = current_sample() {
            match get_flow_data(std::path::PathBuf::from(sample.get_filepath())).await {
                Ok(mut f) => {
                    f.metadata.insert_string_keyword(
                        String::from("$GUID"),
                        uuid::Uuid::new_v4().to_string(),
                    );
                    let file_id: crate::gate_editor::gates::gate_store::FileId = f
                        .metadata
                        .get_string_keyword("$GUID")
                        .expect("no guid store in the fcs")
                        .into();
                    *plot_store.current_file_id().write() = file_id.clone();
                    // gate_store
                    //     .set_current_sample(file_id, &[])
                    //     .expect("failed to set current sample on gate store");
                    fcs_file_resource.set(Some(f))
                }
                Err(e) => {
                    fcs_file_resource.set(None);
                    println!("error generating fcs file {}", e);
                }
            }
        } else {
            fcs_file_resource.set(None);
            println!("No file path selected.");
        }
    });

    let sorted_params = use_memo(move || {
        if let Some(fcs_file) = &*fcs_file_resource.read() {
            // pull the parameters from the file
            let mut sorted_params: FxIndexMap<Arc<str>, Param> = fcs_file
                .parameters
                .iter()
                .map(|(_, param)| {
                    let p = Param {
                        marker: param.label_name.clone(),
                        fluoro: param.channel_name.clone(),
                    };
                    // add the parameter to the store if required
                    plot_store.add_new_axis_settings(&p, &fcs_file);
                    (param.channel_name.clone(), p)
                })
                .collect();

            sorted_params.sort_by_key(|_, param| {
                fcs_file
                    .parameters
                    .get(param.fluoro.as_ref())
                    .map(|p| p.parameter_number)
                    .unwrap_or(usize::MAX)
            });

            sorted_params
        } else {
            FxIndexMap::default()
        }
    });

    // this is currently scaling the data but filtering is done elsewhere!
    let scaled_data = use_resource(move || async move {
        let mut params: Vec<(Arc<str>, f32)> = Vec::new();
        for (k, v) in plot_store.settings().read().iter() {
            if v.is_arcsinh() {
                params.push((k.clone(), v.get_cofactor().unwrap()))
            }
        }

        if let Some(fcs_file) = &*fcs_file_resource.read() {
            let fcs_clone = fcs_file.clone();
            let result =
                tokio::task::spawn_blocking(move || -> Result<Arc<DataFrame>, anyhow::Error> {
                    let param_refs: Vec<(&str, f32)> =
                        params.iter().map(|(k, v)| (k.as_ref(), *v)).collect();
                    let scaled_df = fcs_clone.apply_arcsinh_transforms(param_refs.as_slice())?;
                    let df_with_index = scaled_df.with_row_index("original_index".into(), None)?;

                    Ok(Arc::new(df_with_index))
                })
                .await;

            match result {
                Ok(d) => d,
                Err(_) => Err(anyhow::anyhow!("error scaling data")),
            }
        } else {
            Err(anyhow::anyhow!("No data to scale"))
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
        match plot_store.settings().read().get(&param.fluoro) {
            Some(d) => d.clone(),
            None => AxisInfo::default(),
        }
    });

    let y_axis_limits = use_memo(move || {
        let param = y_axis_marker();
        match plot_store.settings().read().get(&param.fluoro) {
            Some(d) => d.clone(),
            None => AxisInfo::default(),
        }
    });

    let x_axis_selected_index = use_memo(move || {
        let curr: Arc<str> = (&*x_axis_marker.read()).fluoro.clone();
        sorted_params.peek().get_index_of(&curr).unwrap_or(0)
    });
    let y_axis_selected_index = use_memo(move || {
        let curr: Arc<str> = (&*y_axis_marker.read()).fluoro.clone();
        sorted_params.peek().get_index_of(&curr).unwrap_or(0)
    });

    let resolver = use_memo(move || {
        let id: Arc<str> = plot_store.current_file_id()();
        gate_store.get_current_sample(id, &[])
    });

    let parental_gate: Signal<Option<Arc<str>>> = use_signal(|| Some(ROOTGATE.clone()));

    let mut plot_data_signal = use_signal(|| vec![]);

    let filtered_dataframe: Resource<std::result::Result<Arc<DataFrame>, anyhow::Error>> =
        use_resource(move || {
            let x_fluoro = x_axis_marker.read().fluoro.clone();
            let y_fluoro = y_axis_marker.read().fluoro.clone();
            let parental = parental_gate();

            async move {
                let Ok(resolver) = resolver() else {
                    return Err(anyhow::anyhow!("No resolver"));
                };
                if let Some(Ok(d)) = &*scaled_data.read() {
                    let filtered_data = match get_filtered_dataframe(d.clone(), parental, resolver)
                        .await
                    {
                        Ok(d) => d.clone(),
                        Err(e) => {
                            plot_data_signal.set(vec![]);
                            return Err(anyhow::anyhow!("No data to display {}", e.to_string()));
                        }
                    };

                    match zip_cols_from_filtered_df(filtered_data.clone(), x_fluoro, y_fluoro).await
                    {
                        Ok(d) => plot_data_signal.set(d),
                        Err(_) => plot_data_signal.set(vec![]),
                    };

                    Ok(filtered_data)
                } else {
                    plot_data_signal.set(vec![]);
                    Err(anyhow::anyhow!("No data yet"))
                }
            }
        });

    let event_index = use_resource(move || {
        let df_arc = match &*filtered_dataframe.read() {
            Some(Ok(df)) => Some(df.clone()),
            _ => None,
        };
        let x_name = x_axis_marker.read().fluoro.clone();
        let y_name = y_axis_marker.read().fluoro.clone();
        async move {
            let df = match df_arc {
                Some(d) => d,
                None => return Ok(None),
            };

            let join_result =
                tokio::task::spawn_blocking(move || -> anyhow::Result<EventIndexMapped> {
                    // Build the R-Tree
                    let ei = get_event_mask_from_scaled_df(df.clone(), x_name, y_name)
                        .map_err(|e| anyhow::anyhow!("R-Tree build failed: {e}"))?;
                    // Extract the mapping
                    let map: Vec<usize> = df
                        .column("original_index")?
                        .u32()?
                        .into_iter()
                        .flatten()
                        .map(|v| v as usize)
                        .collect();
                    Ok(EventIndexMapped {
                        event_index: ei,
                        index_map: Arc::new(map),
                    })
                })
                .await;

            match join_result {
                Ok(Ok(index)) => Ok(Some(index)),
                Ok(Err(e)) => {
                    println!("{e}");
                    Err(e)
                }
                Err(join_err) => {
                    println!("{join_err}");
                    Err(anyhow::anyhow!("Task panicked: {join_err}"))
                }
            }
        }
    });

    // for the actual gating, we just use df filtering. however for the currently displayed events we use an EventIndex
    // to get real time % for child gates
    use_effect(move || {
        let data = event_index
            .read()
            .as_ref()
            .and_then(|res| res.as_ref().ok())
            .and_then(|opt| opt.clone());

        *plot_store.event_index_map().write() = data;
    });

    rsx! {
        document::Stylesheet { href: CSS_STYLE }
        div { class: "sidebar-local",

            GateSidebar {
                selected_id: parental_gate,
                x_axis_param: x_axis_marker,
                y_axis_param: y_axis_marker,
            }

            main { class: "main-content",

                div { class: "gate-window",

                    div { class: "axis-controls-grid", style: "width: 600px;",
                        div { class: "grid-label", "X-Axis" }
                        SearchableSelectMap {
                            items: sorted_params(),
                            on_select: move |(_, v)| { x_axis_marker.set(v) },
                            placeholder: x_axis_marker.peek().to_string(),
                            selected_index: Some(x_axis_selected_index.into()),
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
                                            let res = plot_store.update_cofactor(&param.fluoro, val as f32);
                                            match res {
                                                Ok((old, new)) => {
                                                    match gate_store.rescale_gates(&param.fluoro, &old, &new) {
                                                        Ok(_) => message.set(None),
                                                        Err(e) => {
                                                            message.set(Some(e.join("\n")));
                                                        }
                                                    };
                                                }
                                                Err(e) => println!("{e}"),

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
                                        match plot_store.update_lower(&param.fluoro, lower as f32) {
                                            Ok(l_u_t) => {
                                                match gate_store
                                                    .set_current_axis_limits(
                                                        param.fluoro.clone(),
                                                        l_u_t.0,
                                                        l_u_t.1,
                                                        l_u_t.2,
                                                    )
                                                {
                                                    Ok(_) => {}
                                                    Err(e) => println!("{:#?}", e),
                                                };
                                            }
                                            Err(e) => println!("{e}"),
                                        };
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
                                        match plot_store.update_upper(&param.fluoro, upper as f32) {
                                            Ok(l_u_t) => {
                                                match gate_store
                                                    .set_current_axis_limits(
                                                        param.fluoro.clone(),
                                                        l_u_t.0,
                                                        l_u_t.1,
                                                        l_u_t.2,
                                                    )
                                                {
                                                    Ok(_) => {}
                                                    Err(e) => println!("{:#?}", e),
                                                };
                                            }
                                            Err(e) => println!("{e}"),
                                        };
                                    }
                                },
                            }
                        }

                        div { class: "grid-label", "Y-Axis" }
                        SearchableSelectMap {
                            items: sorted_params(),
                            on_select: move |(_, v)| { y_axis_marker.set(v) },
                            placeholder: y_axis_marker.peek().to_string(),
                            selected_index: Some(y_axis_selected_index.into()),
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
                                            let res = plot_store.update_cofactor(&param.fluoro, val as f32);
                                            match res {
                                                Ok((old, new)) => {
                                                    match gate_store.rescale_gates(&param.fluoro, &old, &new) {
                                                        Ok(_) => message.set(None),
                                                        Err(e) => {
                                                            message.set(Some(e.join("\n")));
                                                        }
                                                    };
                                                }
                                                Err(e) => println!("{e}"),
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
                                        match plot_store.update_lower(&param.fluoro, lower as f32) {
                                            Ok(l_u_t) => {
                                                match gate_store
                                                    .set_current_axis_limits(
                                                        param.fluoro.clone(),
                                                        l_u_t.0,
                                                        l_u_t.1,
                                                        l_u_t.2,
                                                    )
                                                {
                                                    Ok(_) => {}
                                                    Err(e) => println!("{:#?}", e),
                                                };
                                            }
                                            Err(e) => println!("{e}"),
                                        };
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
                                        match plot_store.update_upper(&param.fluoro, upper as f32) {
                                            Ok(l_u_t) => {
                                                match gate_store
                                                    .set_current_axis_limits(
                                                        param.fluoro.clone(),
                                                        l_u_t.0,
                                                        l_u_t.1,
                                                        l_u_t.2,
                                                    )
                                                {
                                                    Ok(_) => {}
                                                    Err(e) => println!("{:#?}", e),
                                                };
                                            }
                                            Err(e) => println!("{e}"),
                                        };
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
                                    SearchableSelectList {
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
                        match &*filtered_dataframe.read() {
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

                    rsx! {
                        div {
                            NewGateButtons { callback: move |gate_type| current_gate_type.set(gate_type) }
                            if let Ok(resolver) = resolver() {
                                PseudoColourPlot {
                                    size: (600, 600),
                                    data: plot_data_signal,
                                    x_axis_info: x_axis_limits.read().clone(),
                                    y_axis_info: y_axis_limits.read().clone(),
                                    parental_gate_id: parental_gate,
                                    resolver,
                                }
                            }
                        }
                    }

                }

            }
        }
    }
}
