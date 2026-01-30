use dioxus::prelude::*;
use flow_gates::{Gate, GateHierarchy};
use std::{collections::HashMap, sync::Arc};

type Id = std::sync::Arc<str>;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GatesOnPlotKey {
    param_1: Id,
    param_2: Id,
    parental_gate_id: Option<Id>,
}

impl GatesOnPlotKey {
    pub fn new(param_1: &str, param_2: &str, parental_gate_id: Option<Id>) -> Self {
        if param_1 <= param_2 {
            Self {
                param_1: Arc::from(param_1),
                param_2: Arc::from(param_2),
                parental_gate_id: parental_gate_id,
            }
        } else {
            Self {
                param_1: Arc::from(param_2),
                param_2: Arc::from(param_1),
                parental_gate_id: parental_gate_id,
            }
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GatePositionKey {
    gate_id: Id,
    file_id: Id,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GateKey {
    gate_id: Id,
}

/// a plot is selected for a file, 
/// The currently selected (parental) gate id is stored in a signal and accessed. 
/// Create a GatesOnPlotKey with the current params and the parental gate id, 
/// to retrieve a list of gate id's shown on the plot. 
/// For each gate id, the actual gates can be retrieved from gate_registry. 
/// Check for file-specific positioning before drawing
#[derive(Default, Store, Clone)]
pub struct GateState {
    // For the Renderer: "What gates do I draw on this Plot?"
    pub gate_ids_by_view: HashMap<GatesOnPlotKey, Vec<GateKey>>,
    // For the Logic: "What is the actual data for Gate X?"
    pub gate_registry: HashMap<GateKey, Arc<flow_gates::Gate>>,
    // For the Filtering: "How are these gates nested?"
    pub hierarchy: GateHierarchy,
    // are there file-specific overrides for gate positions
    pub position_overrides: HashMap<GatePositionKey, flow_gates::GateGeometry>
}

impl GateState {
    pub fn add_gate(&mut self, gate: &Gate, parental_gate_id: Option<Id>) -> Result<()> {
        let key = GatesOnPlotKey::new(
            gate.x_parameter_channel_name(),
            gate.y_parameter_channel_name(),
            parental_gate_id.clone(),
        );
        self.gate_ids_by_view
            .entry(key)
            .or_insert(vec![])
            .push(GateKey { gate_id: gate.id.clone() });

        self.hierarchy
            .add_gate_child(parental_gate_id.unwrap_or(Arc::from("root")), gate.id.clone())?;

        Ok(())
    }

    pub fn remove_gate(&mut self, gate: &Gate, parental_gate_id: Option<Id>) -> Result<()> {
        let key = GatesOnPlotKey::new(
            gate.x_parameter_channel_name(),
            gate.y_parameter_channel_name(),
            parental_gate_id.clone(),
        );
        self.gate_ids_by_view
            .entry(key)
            .and_modify(|l| l.retain(|name| &name.gate_id != &gate.id));

        self.hierarchy
            .add_gate_child(parental_gate_id.unwrap_or(Arc::from("root")), gate.id.clone())?;

        Ok(())
    }

    pub fn get_gates_for_plot(&self, key: &GatesOnPlotKey) -> impl Iterator<Item = Arc<Gate>> {

        self.gate_ids_by_view
            .get(key)
            .into_iter()
            .flatten()
            .filter_map(|id| self.gate_registry.get(id).cloned())
    }
    }
