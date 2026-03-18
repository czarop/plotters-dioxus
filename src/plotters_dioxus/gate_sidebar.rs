use crate::plotters_dioxus::gates::GateState;
use crate::plotters_dioxus::gates::gate_store::{GateStateImplExt, GateStateStoreExt, ROOTGATE};
use crate::plotters_dioxus::plots::parameters::{Param, PlotStore, PlotStoreStoreExt};
use dioxus::prelude::*;
use std::sync::Arc;
use crate::components::context_menu::*;
static SIDEBAR_STYLE: Asset = asset!("assets/gate_sidebar.css");


#[component]
pub fn GateSidebar(selected_id: Signal<Option<Arc<str>>>, x_axis_param: Signal<Param>, y_axis_param: Signal<Param>) -> Element {
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let hierarchy = gate_store.hierarchy();
    let roots = hierarchy.read().get_roots();

    rsx! {
        document::Stylesheet { href: SIDEBAR_STYLE }
        div { class: "custom-sidebar",
            h3 { class: "sidebar-title", "Gate Hierarchy" }

            div { class: "sidebar-tree",

                for root_id in roots {
                    for child_id in hierarchy.read().get_children(&root_id) {
                        GateNode {
                            key: "{child_id}",
                            gate_id: child_id.clone(),
                            selected: selected_id,
                            level: 0,
                            x_axis_param,
                            y_axis_param,
                        }
                    }
                }
            
            }
        }
    }
}

// Keep your exact ChevronIcon, we'll just rotate it with CSS
#[component]
fn ChevronIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            path { d: "m9 18 6-6-6-6" }
        }
    }
}

#[component]
fn GateNode(gate_id: Arc<str>, selected: Signal<Option<Arc<str>>>, level: usize, x_axis_param: Signal<Param>, y_axis_param: Signal<Param>) -> Element {
    let mut gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let param_store: Store<PlotStore> = use_context::<Store<PlotStore>>();
    let mut is_expanded = use_signal(|| true);

    // Fetch children
    let hierarchy = gate_store.hierarchy();
    let children = hierarchy
        .read()
        .get_children(&gate_id)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    let has_children = !children.is_empty();
    let is_root = hierarchy.read().is_root(&gate_id);
    
    let parent = {
        if is_root {
            gate_id.clone()
        } else {
            hierarchy.read().get_parent(&gate_id).unwrap().clone()
        }
        
    };

    // Check if this node is the active one
    let is_selected = selected.read().as_ref() == Some(&gate_id);

    // Calculate dynamic padding based on the level (e.g., 16px per level)
    let padding = format!("{}px", level * 16 + 8);

    let gate_id_clone = gate_id.clone();
    let gate_id_delete_clone = gate_id.clone();
    let gate_id_rename_clone = gate_id.clone();
    let parent_for_delete = parent.clone();
    rsx! {

        // 1. The Row (Clickable)
        ContextMenu {
            ContextMenuTrigger {
                div { class: "gate-node-container",
                    div {
                        class: format!("gate-node-row{}", if is_selected { " selected" } else { "" }),
                        style: "padding-left: {padding};",
                        onclick: move |e: Event<MouseData>| {

                            e.stop_propagation();

                            if let Some(gate) = gate_store.get_gate_by_id(gate_id_clone.clone()) {
                                let (x, y) = gate.get_params();
                                let (new_x, new_y);
                                if let Some(x_axis_settings) = param_store.settings().read().get(&x) {
                                    new_x = Some(x_axis_settings.param.clone());
                                } else {
                                    new_x = None;
                                }
                                if let Some(y_axis_settings) = param_store.settings().read().get(&y) {
                                    new_y = Some(y_axis_settings.param.clone());
                                } else {
                                    new_y = None;
                                }
                                if let (Some(new_x), Some(new_y)) = (new_x, new_y) {
                                    x_axis_param.set(new_x);
                                    y_axis_param.set(new_y);
                                    selected.set(Some(parent.clone()));
                                }

                            }
                        },

                        if has_children {
                            div {
                                class: format!("toggle-icon{}", if is_expanded() { " expanded" } else { "" }),
                                onclick: move |e| {
                                    e.stop_propagation();
                                    is_expanded.toggle();
                                },
                                ChevronIcon {}
                            }
                        } else {
                            // Empty space so leaf nodes align perfectly with parent text
                            div { class: "toggle-icon-placeholder" }
                        }

                        // 3. The Label
                        span { class: "gate-name", "{gate_id}" }
                        button {
                            class: "activate-btn",
                            title: "Activate gate",
                            onclick: move |e| {
                                // IMPORTANT: Stop the row's onclick from firing
                                e.stop_propagation();
                                if let Some(gate) = gate_store.get_gate_by_id(gate_id.clone()) {
                                    let (x, y) = gate.get_params();
                                    let (new_x, new_y);
                                    if let Some(x_axis_settings) = param_store.settings().read().get(&x) {
                                        new_x = Some(x_axis_settings.param.clone());
                                    } else {
                                        new_x = None;
                                    }
                                    if let Some(y_axis_settings) = param_store.settings().read().get(&y) {
                                        new_y = Some(y_axis_settings.param.clone());
                                    } else {
                                        new_y = None;
                                    }
                                    if let (Some(new_x), Some(new_y)) = (new_x, new_y) {
                                        x_axis_param.set(new_x);
                                        y_axis_param.set(new_y);
                                        selected.set(Some(gate_id.clone()));
                                    }

                                }

                            },
                            "🎯"
                        }
                    
                    }

                    // 4. The Children (Recursive call)
                    if has_children && is_expanded() {
                        div { class: "gate-children",
                            for child_id in children {
                                GateNode {
                                    key: "{child_id}",
                                    gate_id: child_id,
                                    selected,
                                    level: level + 1,
                                    x_axis_param,
                                    y_axis_param,
                                }
                            }
                        }
                    }
                }
            }
            ContextMenuContent {

                ContextMenuItem {
                    value: "delete".to_string(),
                    index: 0usize,
                    on_select: move |value| {
                        match gate_store.remove_gate(gate_id_delete_clone.clone()) {
                            Ok(_) => {
                                println!("deleted gate");
                                if is_root {
                                    selected.set(Some(ROOTGATE.clone()));
                                } else {
                                    selected.set(Some(parent_for_delete.clone()));
                                }

                            }
                            Err(e) => println!("failed to delete gate"),
                        }
                    },
                    "Delete"
                }
                ContextMenuItem {
                    value: "rename".to_string(),
                    index: 1usize,
                    on_select: move |value| {},
                    "Rename"
                }
            }
        
        }
    }
}

