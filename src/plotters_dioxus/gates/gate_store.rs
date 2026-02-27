use anyhow::anyhow;
use dioxus::prelude::*;
use flow_gates::{Gate, GateHierarchy};
use rustc_hash::FxHashMap;

use std::{
    sync::{Arc},
};

use crate::plotters_dioxus::{
    AxisInfo,
    gates::{
        gate_composite::BisectorGate, gate_drag::GateDragData, gate_draw_helpers, gate_single::{EllipseGate, LineGate, PolygonGate, RectangleGate}, gate_traits::DrawableGate, gate_types::GateType
    }, plot_helpers::PlotMapper,
};

pub type GateId = std::sync::Arc<str>;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct GatesOnPlotKey {
    param_1: GateId,
    param_2: GateId,
    parental_gate_id: Option<GateId>,
}

impl GatesOnPlotKey {
    pub fn new(param_1: Arc<str>, param_2: Arc<str>, parental_gate_id: Option<GateId>) -> Self {
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
    gate_id: GateId,
    file_id: GateId,
}

// #[derive(Hash, PartialEq, Eq, Clone)]
// pub struct GateKey {
//     gate_id: Id,
// }

// impl GateKey {
//     pub fn new(id: Id) -> Self {
//         Self { gate_id: id }
//     }
// }

// impl From<Arc<str>> for GateKey {
//     fn from(id: Arc<str>) -> Self {
//         Self { gate_id: id }
//     }
// }

/// a plot is selected for a file,
/// The currently selected (parental) gate id is stored in a signal and accessed.
/// Create a GatesOnPlotKey with the current params and the parental gate id,
/// to retrieve a list of gate id's shown on the plot.
/// For each gate id, the actual gates can be retrieved from gate_registry.
/// Check for file-specific positioning before drawing
#[derive(Default, Store)]
pub struct GateState {
    // For the Renderer: "What gates do I draw on this Plot?"
    pub gate_ids_by_view: FxHashMap<GatesOnPlotKey, Vec<GateId>>,
    // For the Logic: "What is the actual data for Gate X?"
    pub gate_registry: FxHashMap<GateId, Arc<dyn DrawableGate>>,

    // composite_redirect: FxHashMap<GateId, GateId>,
    // For the Filtering: "How are these gates nested?"
    pub hierarchy: GateHierarchy,
    // are there file-specific overrides for gate positions
    pub position_overrides: FxHashMap<GatePositionKey, flow_gates::GateGeometry>,
}

#[store(pub name = GateStateImplExt)]
impl<Lens> Store<GateState, Lens> {
    fn add_gate(
        &mut self,
        mapper: &PlotMapper,
        click_x: f32,
        click_y: f32,
        x_param: Arc<str>,
        y_param: Arc<str>,
        points: Option<Vec<(f32, f32)>>,
        id: String,
        parental_gate_id: Option<GateId>,
        gate_type: GateType,
    ) -> Result<()> {

        let key = GatesOnPlotKey::new(x_param.clone(), y_param.clone(), parental_gate_id.clone());        
        let parameters = (x_param.clone(), y_param.clone());

        // let mut composite_subgate_ids = vec![];

        let g: Arc<dyn DrawableGate + 'static> = match gate_type{
            GateType::Polygon => {
                let geo = flow_gates::geometry::create_polygon_geometry(
                                            points.ok_or(anyhow!("points not provided for polygon gate"))?,
                                            &x_param,
                                            &y_param,
                                        )
                                        .map_err(|_| anyhow!("failed to create polygon geometry"))?;
                                    let gate = Gate{ 
                                        id: Arc::from(id.as_str()), 
                                        name: id, 
                                        geometry: geo, 
                                        mode: flow_gates::GateMode::Global, 
                                        parameters, 
                                        label_position: None 
                                    };
                                    Arc::new(PolygonGate::try_new(gate)?)
                                    },
            GateType::Ellipse => {
                let geo = gate_draw_helpers::ellipse::create_default_ellipse(
                                    &mapper,
                                    click_x,
                                    click_y,
                                    50f32,
                                    30f32,
                                    &x_param,
                                    &y_param,
                                )?;
                                let gate = Gate{ 
                                        id: Arc::from(id.as_str()), 
                                        name: id, 
                                        geometry: geo, 
                                        mode: flow_gates::GateMode::Global, 
                                        parameters, 
                                        label_position: None 
                                    };
                                Arc::new(EllipseGate::try_new(gate)?)
            },
            GateType::Rectangle => {
                let geo = gate_draw_helpers::rectangle::create_default_rectangle(
                                    &mapper,
                                    click_x,
                                    click_y,
                                    50f32,
                                    50f32,
                                    &x_param,
                                    &y_param,
                                )?;
                                let gate = Gate{ 
                                        id: Arc::from(id.as_str()), 
                                        name: id, 
                                        geometry: geo, 
                                        mode: flow_gates::GateMode::Global, 
                                        parameters, 
                                        label_position: None 
                                    };
                                Arc::new(RectangleGate::try_new(gate)?)
            },
            GateType::Line(y_coord) => {
                let geo = gate_draw_helpers::line::create_default_line(
                    &mapper,
                    click_x,
                    50f32,
                    &x_param,
                    &y_param,
                )?;
                if let Some(y_coord) = y_coord {
                    let gate = Gate{ 
                                        id: Arc::from(id.as_str()), 
                                        name: id, 
                                        geometry: geo, 
                                        mode: flow_gates::GateMode::Global, 
                                        parameters, 
                                        label_position: None 
                                    };
                    Arc::new(LineGate::try_new(gate, y_coord)?)
                } else {
                    Err(anyhow!(
                        "Line gate requires y coordinate for initialization"
                    ))?
                }
            },

        GateType::Bisector => { 

            Arc::new(BisectorGate::try_new(mapper, Arc::from(id.as_str()), (click_x, click_y), x_param, y_param)?)
        },
        GateType::Quadrant => todo!(),
        GateType::FlexiQuadrant => todo!(),
    };





        

        let gate_key = g.get_id();

        self.gate_ids_by_view()
            .write()
            .entry(key)
            .or_insert(vec![])
            .push(gate_key.clone());

        self.hierarchy().write().add_gate_child(
            parental_gate_id.unwrap_or(Arc::from("root")),
            g.get_id(),
        )?;

        // for subgate in composite_subgate_ids{
        //     self.composite_redirect().insert(subgate.into(), g.get_id().into());
        // }

        self.gate_registry().write().insert(gate_key, g);


        Ok(())
    }

    fn remove_gate(&mut self, gate_id: GateId, parental_gate_id: Option<GateId>) -> Result<()> {

        // redirect to parent
        // let id = if let Some(id) = self.composite_redirect().peek().get(&gate_id) {
        //     id.clone()
        // } else {
        //     gate_id.clone()
        // };

        if let Some((id, gate)) = self.gate_registry().write().remove_entry(&gate_id){
            let (x_param, y_param) = gate.get_params();
            let key = GatesOnPlotKey::new(x_param.clone(), y_param.clone(), parental_gate_id.clone());
        
            self.gate_ids_by_view()
                .write()
                .entry(key)
                .and_modify(|l| l.retain(|name| name != &id));

        };

        // iteratively do this for all child gates in the hierarchy!
        todo!();
        // let child_gates = self.hierarchy().write().delete_subtree(
        //     id.clone(),
        // );

        Ok(())
    }

    fn move_gate_point(
        &mut self,
        gate_id: GateId,
        point_idx: usize,
        new_point: (f32, f32),
    ) -> anyhow::Result<()> {

        // redirect to parent
        // let id = if let Some(id) = self.composite_redirect().peek().get(&gate_id) {
        //     id.clone()
        // } else {
        //     gate_id.clone()
        // };

        if let Some(mut gate_ptr) = self
            .gate_registry()
            .get_mut(&gate_id)
        {
            if let Ok(new_gate_box) = gate_ptr.replace_point(new_point, point_idx) {
                *gate_ptr = Arc::from(new_gate_box);
            }
        }

        Ok(())
    }

    fn move_gate(&mut self, gate_drag_data: GateDragData) -> Result<()> {

        // redirect to parent
        // let id = if let Some(id) = self.composite_redirect().peek().get(&gate_drag_data.gate_id()) {
        //     id.clone()
        // } else {
        //     gate_drag_data.gate_id()
        // };

        if let Some(mut gate_ptr) = self
            .gate_registry()
            .get_mut(&gate_drag_data.gate_id())
        {
            if let Ok(new_gate_box) = gate_ptr.replace_points(gate_drag_data) {
                *gate_ptr = Arc::from(new_gate_box);
            }
        }
        Ok(())
    }

    fn rotate_gate(
        &mut self,
        gate_id: GateId,
        current_position: (f32, f32),
    ) -> anyhow::Result<()> {

        // redirect to parent
        // let id = if let Some(id) = self.composite_redirect().peek().get(&gate_id) {
        //     id.clone()
        // } else {
        //     gate_id.clone()
        // };

        if let Some(mut gate_ptr) = self
            .gate_registry()
            .get_mut(&gate_id)
        {
            if let Ok(Some(new_gate_box)) = gate_ptr.rotate_gate(current_position) {
                *gate_ptr = Arc::from(new_gate_box);
            }
        }

        Ok(())
    }

    fn get_gates_for_plot(
        &self,
        x_axis_title: Arc<str>,
        y_axis_title: Arc<str>,
    ) -> Option<Vec<Arc<dyn DrawableGate>>> {
        let key = GatesOnPlotKey::new(x_axis_title.clone(), y_axis_title.clone(), None);
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
                if let Some(mut gate) = self.gate_registry().get_mut(&k) {
                    if let Ok(Some(g)) = gate.match_to_plot_axis(x, y){
                        *gate = Arc::from(g);
                    };
                }
            }
        }

        
        return Ok(());
    }

    fn rescale_gates(
        &mut self,
        marker: &Arc<str>,
        old_axis_options: &AxisInfo,
        new_axis_options: &AxisInfo,
    ) -> Result<(), Vec<String>> {
        let mut errors = vec![];
        for (_, gate) in self.gate_registry().write().iter_mut() {
            let (x_marker, y_marker) = &gate.get_params();
            if marker == x_marker || marker == y_marker {
                match gate.recalculate_gate_for_rescaled_axis(
                    marker.clone(),
                    &old_axis_options.transform,
                    &new_axis_options.transform,
                ){
                    Ok(g) => *gate = Arc::from(g),
                    Err(e) => errors.push(e.to_string()),
                } 


            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
