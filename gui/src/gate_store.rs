use dioxus::prelude::*;
use flow_gates::{Gate, GateHierarchy};
use std::collections::HashMap;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GateKey {
    param_1: String,
    param_2: String,
    parental_gate_id: Option<String>,
}

impl GateKey {
    pub fn new(param_1: &str, param_2: &str, parental_gate_id: Option<&str>) -> Self {
        if param_1 <= param_2 {
            Self {
                param_1: param_1.to_string(),
                param_2: param_2.to_string(),
                parental_gate_id: parental_gate_id.map(String::from),
            }
        } else {
            Self {
                param_1: param_2.to_string(),
                param_2: param_1.to_string(),
                parental_gate_id: parental_gate_id.map(String::from),
            }
        }
    }
}

#[derive(Default, Store)]
pub struct GateState {
    // For the Renderer: "What gates do I draw on this Plot?"
    pub gates_by_view: HashMap<GateKey, Vec<flow_gates::Gate>>,
    // For the Logic: "What is the actual data for Gate X?"
    pub gate_registry: HashMap<std::sync::Arc<str>, flow_gates::Gate>,
    // For the Filtering: "How are these gates nested?"
    pub hierarchy: GateHierarchy,
}

impl GateState {
    pub fn add_gate(&mut self, gate: &Gate, parental_gate_id: Option<&str>) -> Result<()> {
        let key = GateKey::new(
            gate.x_parameter_channel_name(),
            gate.y_parameter_channel_name(),
            parental_gate_id,
        );
        self.gates_by_view
            .entry(key)
            .or_insert(vec![gate.clone()])
            .push(gate.clone());

        self.hierarchy
            .add_gate_child(parental_gate_id.unwrap_or("root"), gate.id.clone())?;

        Ok(())
    }
}
