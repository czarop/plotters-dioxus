use crate::plotters_dioxus::gates::GateState;
use crate::plotters_dioxus::gates::gate_store::{GateStateStoreExt, ROOTGATE};
use dioxus::prelude::*;
use std::sync::Arc;

static SIDEBAR_STYLE: Asset = asset!("assets/gate_sidebar.css");
static ROOT_DEFAULT: &'static str = "root_default";

#[component]
pub fn GateSidebar(selected_id: Signal<Option<Arc<str>>>) -> Element {
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let hierarchy = gate_store.hierarchy();
    let roots = hierarchy.read().get_roots();

    rsx! {
        document::Stylesheet { href: SIDEBAR_STYLE }
        div { class: "custom-sidebar",
            h3 { class: "sidebar-title", "Gate Hierarchy" }

            div { class: "sidebar-tree",
                if roots.is_empty() {
                    GateNode {
                        key: "{ROOT_DEFAULT}",
                        gate_id: ROOTGATE.clone(),
                        selected: selected_id,
                        level: 0,
                    }
                } else {
                    for root_id in roots {
                        GateNode {
                            key: "{root_id}",
                            gate_id: root_id,
                            selected: selected_id,
                            level: 0,
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn GateNode(gate_id: Arc<str>, selected: Signal<Option<Arc<str>>>, level: usize) -> Element {
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    // Local state for expanding/collapsing this specific node
    let mut is_expanded = use_signal(|| true);

    // Fetch children
    let hierarchy = gate_store.hierarchy();
    let children = hierarchy.read().get_children(&gate_id).into_iter().cloned().collect::<Vec<_>>();
    let has_children = !children.is_empty();

    // Check if this node is the active one
    let is_selected = selected.read().as_ref() == Some(&gate_id);

    // Calculate dynamic padding based on the level (e.g., 16px per level)
    let padding = format!("{}px", level * 16 + 8); 

    rsx! {
        div { class: "gate-node-container",
            // 1. The Row (Clickable)
            div {
                class: format!("gate-node-row{}", if is_selected { " selected" } else { "" }),
                style: "padding-left: {padding};",
                onclick: move |e| {
                    e.stop_propagation();
                    selected.set(Some(gate_id.clone()));
                },

                // 2. The Toggle Icon (or a spacer if it's a leaf)
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

