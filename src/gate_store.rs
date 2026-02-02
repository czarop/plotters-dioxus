use dioxus::prelude::*;
use flow_gates::{Gate, GateHierarchy};
use std::{collections::HashMap, sync::Arc};

use crate::plotters_dioxus::gate_helpers::GateFinal;

type Id = std::sync::Arc<str>;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GatesOnPlotKey {
    param_1: Id,
    param_2: Id,
    parental_gate_id: Option<Id>,
}

impl GatesOnPlotKey {
    pub fn new(param_1: Arc<str>, param_2: Arc<str>, parental_gate_id: Option<Id>) -> Self {
        if param_1 <= param_2 {
            Self {
                param_1: param_1,
                param_2: param_2,
                parental_gate_id: parental_gate_id,
            }
        } else {
            Self {
                param_1: param_2,
                param_2: param_1,
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
    pub gate_registry: HashMap<GateKey, Arc<crate::plotters_dioxus::gate_helpers::GateFinal>>,
    // For the Filtering: "How are these gates nested?"
    pub hierarchy: GateHierarchy,
    // are there file-specific overrides for gate positions
    pub position_overrides: HashMap<GatePositionKey, flow_gates::GateGeometry>
}

#[store(pub name = GateStateImplExt)]
impl<Lens> Store<GateState, Lens> {
    fn add_gate(&mut self, gate: Gate, parental_gate_id: Option<Id>) -> Result<()> {
        println!("{}, {}", gate.x_parameter_channel_name(), gate.y_parameter_channel_name());
        let (x_param, y_param) = &gate.parameters;
        let key = GatesOnPlotKey::new(
            x_param.clone(),
            y_param.clone(),
            parental_gate_id.clone(),
        );

        let gate_key = GateKey { gate_id: gate.id.clone() };
        self.gate_ids_by_view().write()
            .entry(key)
            .or_insert(vec![])
            .push(gate_key.clone());

        self.hierarchy().write()
            .add_gate_child(parental_gate_id.unwrap_or(Arc::from("root")), gate.id.clone())?;

        self.gate_registry().write().insert(gate_key, Arc::new(GateFinal::new(gate, false)));

        

        Ok(())
    }

    fn remove_gate(&mut self, gate: Arc<Gate>, parental_gate_id: Option<Id>) -> Result<()> {
        let (x_param, y_param) = &gate.parameters;
        let key = GatesOnPlotKey::new(
            x_param.clone(),
            y_param.clone(),
            parental_gate_id.clone(),
        );
        self.gate_ids_by_view()
            .write()
            .entry(key)
            .and_modify(|l| l.retain(|name| &name.gate_id != &gate.id));

        let gate_key = GateKey { gate_id: gate.id.clone() };
        self.gate_registry().write().remove_entry(&gate_key);

        self.hierarchy()
            .write()
            .add_gate_child(parental_gate_id.unwrap_or(Arc::from("root")), gate.id.clone())?;

        Ok(())
    }

    // fn get_gates_for_plot(&self, key: GatesOnPlotKey) -> Option<Store<Vec<Arc<Gate>>, impl Readable<Target = Vec<GateKey>>>> {
    // fn get_gates_for_plot(&self, key: GatesOnPlotKey) -> Vec<Arc<Gate>> {
    //    let key_options = self.gate_ids_by_view().get(key);
    //    let mut gate_list = vec![];
    //    if let Some(key_store) = key_options {
    //     let key_list = key_store();
    //     key_list.into_iter().map(|k| {
    //         if let Some(gate) = self.gate_registry().get(k) {
    //             gate_list.push(gate().clone());
    //         }
    //    });
        
    //    }

    //     gate_list
    // }
}