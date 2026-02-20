use anyhow::anyhow;
use dioxus::prelude::*;
use flow_gates::{Gate, GateHierarchy};

use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::plotters_dioxus::{AxisInfo, gates::{gate_single::{EllipseGate, PolygonGate, RectangleGate}, gate_traits::DrawableGate, gate_types::GateType}};

pub type Id = std::sync::Arc<str>;

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

impl GateKey {
    pub fn new(id: Id) -> Self {
        Self { gate_id: id }
    }
}

impl From<Arc<str>> for GateKey {
    fn from(id: Arc<str>) -> Self {
        Self { gate_id: id }
    }
}

/// a plot is selected for a file,
/// The currently selected (parental) gate id is stored in a signal and accessed.
/// Create a GatesOnPlotKey with the current params and the parental gate id,
/// to retrieve a list of gate id's shown on the plot.
/// For each gate id, the actual gates can be retrieved from gate_registry.
/// Check for file-specific positioning before drawing
#[derive(Default, Store)]
pub struct GateState {
    // For the Renderer: "What gates do I draw on this Plot?"
    pub gate_ids_by_view: HashMap<GatesOnPlotKey, Vec<GateKey>>,
    // For the Logic: "What is the actual data for Gate X?"
    pub gate_registry: HashMap<GateKey, Arc<Mutex<dyn DrawableGate>>>,
    // For the Filtering: "How are these gates nested?"
    pub hierarchy: GateHierarchy,
    // are there file-specific overrides for gate positions
    pub position_overrides: HashMap<GatePositionKey, flow_gates::GateGeometry>,
}

#[store(pub name = GateStateImplExt)]
impl<Lens> Store<GateState, Lens> {
    fn add_gate(&mut self, gate: Gate, parental_gate_id: Option<Id>, gate_type: GateType) -> Result<()> {
        println!(
            "{}, {}",
            gate.x_parameter_channel_name(),
            gate.y_parameter_channel_name()
        );
        let (x_param, y_param) = &gate.parameters;
        let key = GatesOnPlotKey::new(x_param.clone(), y_param.clone(), parental_gate_id.clone());

        let gate_key = GateKey {
            gate_id: gate.id.clone(),
        };
        self.gate_ids_by_view()
            .write()
            .entry(key)
            .or_insert(vec![])
            .push(gate_key.clone());

        self.hierarchy().write().add_gate_child(
            parental_gate_id.unwrap_or(Arc::from("root")),
            gate.id.clone(),
        )?;

        let g: Arc<Mutex<dyn DrawableGate + 'static>>  = match gate_type {
            GateType::Polygon => Arc::new(Mutex::new(PolygonGate {inner: gate, selected: false, drag_point: None})),
            GateType::Ellipse => Arc::new(Mutex::new(EllipseGate {inner: gate, selected: false, drag_point: None})),
            GateType::Rectangle => Arc::new(Mutex::new(RectangleGate {inner: gate, selected: false, drag_point: None})),
            GateType::Line => todo!(),
            GateType::Bisector => todo!(),
            GateType::Quadrant => todo!(),
            GateType::FlexiQuadrant => todo!(),
        };

        self.gate_registry()
            .write()
            .insert(gate_key, g);

        Ok(())
    }

    fn remove_gate(&mut self, gate: Arc<Gate>, parental_gate_id: Option<Id>) -> Result<()> {
        let (x_param, y_param) = &gate.parameters;
        let key = GatesOnPlotKey::new(x_param.clone(), y_param.clone(), parental_gate_id.clone());
        self.gate_ids_by_view()
            .write()
            .entry(key)
            .and_modify(|l| l.retain(|name| &name.gate_id != &gate.id));

        let gate_key = GateKey {
            gate_id: gate.id.clone(),
        };
        self.gate_registry().write().remove_entry(&gate_key);

        self.hierarchy().write().add_gate_child(
            parental_gate_id.unwrap_or(Arc::from("root")),
            gate.id.clone(),
        )?;

        Ok(())
    }

    fn move_gate_point(
        &mut self,
        gate_id: GateKey,
        point_idx: usize,
        new_point: (f32, f32),
    ) -> anyhow::Result<()> {

        self.gate_registry()
        .write()
        .get(&gate_id)
        .and_then(|g|{
            let mut g = g.lock().unwrap();
            g.set_drag_point(None);
            
            Some(g.replace_point(new_point, point_idx))
        });

        

        Ok(())
    }

    fn move_gate(&mut self, gate_id: GateKey, data_space_offset: (f32, f32)) -> Result<()> {

        self.gate_registry()
        .write()
        .get(&gate_id)
        .and_then(|g|{
            let mut g = g.lock().unwrap();
            let points = g
                .get_points()
                .into_iter()
                .map(|(x, y)| (x - data_space_offset.0, y - data_space_offset.1))
                .collect();
        
            Some(g.replace_points(points))
        }).ok_or(anyhow!("No Gate Found"))??;

        

        Ok(())
    }

    fn rotate_gate(
        &mut self,
        gate_id: GateKey,
        current_position: (f32, f32),
    ) -> anyhow::Result<()> {
        self.gate_registry()
        .write()
        .get(&gate_id)
        .and_then(|g|{
            let mut g = g.lock().unwrap();
            Some(g.rotate_gate(current_position))
        }).ok_or(anyhow!("No Gate Found"))??;

            Ok(())
        
        
    }

    fn get_gates_for_plot(
        &self,
        x_axis_title: Arc<str>,
        y_axis_title: Arc<str>,
    ) -> Option<Vec<Arc<Mutex<dyn DrawableGate>>>> {
        let key = GatesOnPlotKey::new(x_axis_title, y_axis_title, None);
        let key_options = self.gate_ids_by_view().get(key);
        let mut gate_list = vec![];
        if let Some(key_store) = key_options {
            let ids = key_store.read().clone();
            let registry = self.gate_registry();
            let registry_guard = registry.read();
            for k in ids {
                if let Some(gate_store_entry) = registry_guard.get(&k) {
                    gate_list.push(gate_store_entry.clone());
                }
            }
        } else {
            return None;
        }
        return Some(gate_list);
    }

    fn match_gates_to_plot(
        &mut self,
        x_axis_title: Arc<str>,
        y_axis_title: Arc<str>,
    ) -> anyhow::Result<()> {
        let x: &str = &x_axis_title;
        let y: &str = &y_axis_title;
        let key = GatesOnPlotKey::new(x_axis_title.clone(), y_axis_title.clone(), None);
        let key_options = self.gate_ids_by_view().get(key);
        
        if let Some(key_store) = key_options {
            let ids = key_store.read().clone();
            
            for k in ids {
                if let Some(gate) = self.gate_registry().write().get_mut(&k) {
                    gate.lock().unwrap().match_to_plot_axis(x, y)?;
                }
            }
        } 
        return Ok(());
    }

    // fn get_boxed_gates_for_plot(
    //     &self,
    //     x_axis_title: Arc<str>,
    //     y_axis_title: Arc<str>,
    // ) -> Option<Vec<Box<dyn PlotDrawable>>> {
    //     let key = GatesOnPlotKey::new(x_axis_title, y_axis_title, None);
    //     let key_options = self.gate_ids_by_view().get(key);
    //     let mut gate_list = vec![];
    //     if let Some(key_store) = key_options {
    //         let ids = key_store.read().clone();
    //         let registry = self.gate_registry();
    //         let registry_guard = registry.read();
    //         for k in ids {
    //             if let Some(gate_store_entry) = registry_guard.get(&k) {
    //                 let gate_clone = gate_store_entry.clone();
    //                 let gate: Box<dyn PlotDrawable> = Box::new(gate_clone);
    //                 gate_list.push(gate);
    //             }
    //         }
    //     } else {
    //         println!("No gates for plot");
    //         return None;
    //     }
    //     return Some(gate_list);
    // }

    fn rescale_gates(
        &mut self,
        marker: &Arc<str>,
        old_axis_options: &AxisInfo,
        new_axis_options: &AxisInfo,
    ) -> Result<(), Vec<String>> {
        let mut errors = vec![];
        for (_, gate) in self.gate_registry().write().iter_mut() {
            let (x_marker, y_marker) = &gate.lock().unwrap().get_params();
            if marker == x_marker || marker == y_marker {
                let res = gate.lock().unwrap().recalculate_gate_for_rescaled_axis(
                    marker.clone(),
                    &old_axis_options.transform,
                    &new_axis_options.transform,
                );

                let _ = res.inspect_err(|e| {
                    errors.push(e.to_string());
                });
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
