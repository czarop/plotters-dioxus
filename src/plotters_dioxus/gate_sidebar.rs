use crate::components::collapsible::*;
use crate::components::sidebar::*;
use crate::plotters_dioxus::gates::GateState;
use crate::plotters_dioxus::gates::gate_store::GateStateStoreExt;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn GateSidebar(selected_id: Signal<Option<Arc<str>>>) -> Element {
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let hierarchy = gate_store.hierarchy();
    let roots = hierarchy.read().get_roots();

    rsx! {
        SidebarGroup {
            SidebarGroupLabel { "Gate Hierarchy" }
            SidebarMenu {
                for root_id in roots {
                    GateNode { gate_id: root_id, selected_gate: selected_id }
                }
            }
        }
    }
}

#[component]
fn GateNode(gate_id: Arc<str>, selected_gate: Signal<Option<Arc<str>>>) -> Element {
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let children: Vec<Arc<str>> = gate_store
        .hierarchy()
        .read()
        .get_children(&gate_id)
        .into_iter()
        .cloned()
        .collect();
    let has_children = !children.is_empty();
    let is_active = selected_gate().as_ref() == Some(&gate_id);

    // We clone the ID for the closure
    let current_id = gate_id.clone();

    rsx! {
        if has_children {
            // Nested folder-style gate
            Collapsible {
                default_open: true,
                r#as: move |attributes: Vec<Attribute>| {
                    let children = children.clone();
                    let current_id = current_id.clone();
                    rsx! {
                        SidebarMenuItem { attributes,
                            CollapsibleTrigger {
                                r#as: move |trigger_attrs: Vec<Attribute>| {
                                    let click_id = current_id.clone();
                                    // Inject the onclick handler into the attributes list

                                    rsx! {
                                        SidebarMenuButton {
                                            is_active,
                                            attributes: trigger_attrs,
                                            r#as: move |_| {
                                                let click_id = click_id.clone();
                                                rsx! {
                                                    div {
                                                        onclick: move |_| selected_gate.set(Some(click_id.clone())),
                                                        style: "display: flex; width: 100%; align-items: center;",
                                                        Icon {}
                                                        span { "{click_id}" }
                                                        ChevronIcon {}
                                                    }
                                                }
                                            },
                                        }

                                    }
                                },
                            }
                            CollapsibleContent {
                                SidebarMenuSub {
                                    for child_id in children {
                                        SidebarMenuSubItem { key: "{child_id}",
                                            GateNode { gate_id: child_id.clone(), selected_gate }
                                        }
                                    }
                                }
                            }

                        }
                    }
                },
            }
        } else {
            // Leaf gate
            SidebarMenuButton { is_active,
                Icon {}
                span { "{gate_id}" }
            }
        }
    }
}

#[component]
fn Icon(#[props(default = "sidebar-icon")] class: &'static str) -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class,
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            circle { cx: "12", cy: "12", r: "10" }
        }
    }
}

#[component]
fn ChevronIcon() -> Element {
    rsx! {
        svg {
            xmlns: "http://www.w3.org/2000/svg",
            class: "sidebar-icon sidebar-chevron",
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
