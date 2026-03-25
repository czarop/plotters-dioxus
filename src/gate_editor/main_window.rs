use crate::gate_editor::gates::gate_buttons::NewGateButtons;
use crate::gate_editor::plots::parameters::AxisStore;
use crate::gate_editor::plots::parameters::AxisStoreImplExt;
use crate::gate_editor::plots::parameters::AxisStoreStoreExt;
use crate::gate_editor::plots::plot_window::PlotWindow;
use crate::searchable_select::SearchableSelectSet;
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
        plots::parameters::Param,
    },
    searchable_select::SearchableSelectList,
};
use dioxus::prelude::*;
use dioxus::stores::use_store_sync;

use std::sync::Arc;

static CSS_STYLE: Asset = asset!("assets/plot_window.css");

#[component]
pub fn MainWindow() -> Element {
    let mut filehandler: Signal<Option<FcsFiles>> = use_signal(|| None);
    let mut message = use_signal(|| None::<String>);
    let mut gate_store: Store<GateState, CopyValue<GateState, SyncStorage>> =
        use_store_sync(|| GateState::default());
    use_context_provider(|| gate_store);

    let mut current_gate_type = use_signal(|| PrimaryGateType::Polygon);
    use_context_provider(|| current_gate_type);

    let mut axis_store = use_store(|| AxisStore::default());
    use_context_provider(|| axis_store);

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

    // fetch the axis limits from the settings dict when axis changed
    let x_axis_limits = use_memo(move || {
        let param = x_axis_marker.read();
        match axis_store.settings().read().get(&param.fluoro) {
            Some(d) => d.clone(),
            None => AxisInfo::default(),
        }
    });

    let y_axis_limits = use_memo(move || {
        let param = y_axis_marker();
        match axis_store.settings().read().get(&param.fluoro) {
            Some(d) => d.clone(),
            None => AxisInfo::default(),
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

    let x_axis_selected_index = use_memo(move || {
        let curr: Arc<str> = (&*x_axis_marker.read()).fluoro.clone();
        axis_store
            .sorted_settings()
            .peek()
            .get_index_of(&curr)
            .unwrap_or(0)
    });
    let y_axis_selected_index = use_memo(move || {
        let curr: Arc<str> = (&*y_axis_marker.read()).fluoro.clone();
        axis_store
            .sorted_settings()
            .peek()
            .get_index_of(&curr)
            .unwrap_or(0)
    });

    let parental_gate: Signal<Option<Arc<str>>> = use_signal(|| Some(ROOTGATE.clone()));

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
                        SearchableSelectSet {
                            items: axis_store.sorted_settings(),
                            on_select: move |(_, k): (_, Arc<str>)| {
                                if let Some(axis) = axis_store.settings().peek().get(&k.clone()) {
                                    x_axis_marker.set(axis.param.clone());
                                }

                            },
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
                                            let res = axis_store.update_cofactor(&param.fluoro, val as f32);
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
                                        match axis_store.update_lower(&param.fluoro, lower as f32) {
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
                                        match axis_store.update_upper(&param.fluoro, upper as f32) {
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
                        SearchableSelectSet {
                            items: axis_store.sorted_settings(),
                            on_select: move |(_, k): (_, Arc<str>)| {
                                if let Some(axis) = axis_store.settings().peek().get(&k.clone()) {
                                    y_axis_marker.set(axis.param.clone());
                                }

                            },
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
                                            let res = axis_store.update_cofactor(&param.fluoro, val as f32);
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
                                        match axis_store.update_lower(&param.fluoro, lower as f32) {
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
                                        match axis_store.update_upper(&param.fluoro, upper as f32) {
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

                div {
                    NewGateButtons { callback: move |gate_type| current_gate_type.set(gate_type) }
                    {
                        if let Some(files) = &*filehandler.read() {
                            let sample_stub = files.file_list()[sample_index()].clone();
                            let sample_stub2 = files.file_list()[sample_index() + 1].clone();
                            rsx! {
                                PlotWindow { sample_stub, x_axis_marker, y_axis_marker }
                                PlotWindow { sample_stub: sample_stub2, x_axis_marker, y_axis_marker }
                            }
                        } else {
                            rsx! { "No directory selected" }
                        }
                    }
                }
            
            }
        }
    }
}
