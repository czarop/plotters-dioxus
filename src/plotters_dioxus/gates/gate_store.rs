use anyhow::anyhow;
use dioxus::prelude::*;
use flow_fcs::TransformType;
use flow_gates::{BooleanOperation, Gate, GateHierarchy};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet, FxHasher};
use std::collections::HashSet;
use std::ops::{Add, Deref, DerefMut};
use std::sync::{Arc, LazyLock};

use crate::plotters_dioxus::gates::gate_single::boolean_gates::BooleanGate;
use crate::plotters_dioxus::gates::gate_types::{GateStats};
use crate::plotters_dioxus::{
    AxisInfo,
    gates::{
        gate_composite::{
            bisector_gate::BisectorGate, quadrant_gate::QuadrantGate,
            skewed_quadrant_gate::SkewedQuadrantGate,
        },
        gate_drag::GateDragData,
        gate_single::{
            ellipse_gate::{EllipseGate, create_default_ellipse},
            line_gate::{LineGate, create_default_line},
            polygon_gate::PolygonGate,
            rectangle_gate::{RectangleGate, create_default_rectangle},
        },
        gate_traits::DrawableGate,
        gate_types::PrimaryGateType,
    },
    plots::parameters::PlotMapper,
};

pub type GateId = std::sync::Arc<str>;
pub type FileId = std::sync::Arc<str>;

pub static ROOTGATE: LazyLock<Arc<str>> = LazyLock::new(|| Arc::from("root"));

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
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


pub struct GateMap(pub im::HashMap<GateId, Arc<dyn DrawableGate>, FxBuildHasher>);

impl Deref for GateMap {
    type Target = im::HashMap<GateId, Arc<dyn DrawableGate>, FxBuildHasher>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for GateMap {
    
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for GateMap {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl flow_gates::GateResolver for GateMap {
    fn resolve(&self, id: &str) -> Option<&Gate> {
        self.0
            .get(id)
            .map(|drawable| drawable.get_gate_ref(Some(id)))?
    }
}

// #[derive(Clone)]
// pub struct TrackedGate(Arc<dyn DrawableGate>);

// impl PartialEq for TrackedGate {
//     fn eq(&self, other: &Self) -> bool {
//         Arc::ptr_eq(&self.0, &other.0)
//     }
// }

// impl Deref for TrackedGate {
//     type Target = Arc<dyn DrawableGate>;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for TrackedGate {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl From::<Arc<dyn DrawableGate>> for TrackedGate {
//     fn from(value: Arc<dyn DrawableGate>) -> Self {
//         Self(value)
//     }
// }





#[derive(Clone)]
pub struct GateOverrideResolver {
    pub curr_file_id: FileId,
    // a map of id to all gate types incl boolean and subgate
    pub gates_subgates_and_boolean_gates: im::HashMap<GateId, Arc<dyn DrawableGate>, rustc_hash::FxBuildHasher>,

    // a map of overrides by gate id and then by file id - this contains 
    // primary_gate_id -> primary_gate override
    // sub_gate_id -> primary_gate override
    pub position_overrides: im::HashMap<GateId, FxHashMap<FileId, Arc<dyn DrawableGate>>, rustc_hash::FxBuildHasher>,
}

impl flow_gates::GateResolver for GateOverrideResolver {
    fn resolve(&self, id: &str) -> Option<&Gate> {

        if let Some(file_map) = self.position_overrides.get(id) {
            if let Some(gate) = file_map.get(&self.curr_file_id) {
                return gate.get_gate_ref(Some(id))
            } 
        }
        if let Some(gate) = self.gates_subgates_and_boolean_gates.get(id) {
            return gate.get_gate_ref(Some(id));
        } 

        None
        
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
    gate_ids_by_view: FxHashMap<GatesOnPlotKey, Vec<GateId>>,
    // For the Logic: "What is the actual data for Gate X?"
    pub primary_gate_registry: GateMap,


    pub primary_and_subgate_registry: GateMap,

    // For the Filtering: "How are these gates nested?"
    pub hierarchy: GateHierarchy,
    // are there file-specific overrides for gate positions
    pub position_overrides: im::HashMap<GateId, FxHashMap<FileId, Arc<dyn DrawableGate>>, rustc_hash::FxBuildHasher>,

    // when deleting a gate, do you need to delete any boolean gates that depend on it?
    boolean_gate_links: FxHashMap<GateId, Vec<GateId>>,

    pub gate_stats: FxHashMap<Arc<str>, GateStats>,


}

#[store(pub name = GateStateImplExt)]
impl<Lens> Store<GateState, Lens> {

    fn get_gate_by_id(&self, id: GateId) -> Option<Arc<dyn DrawableGate>> {
        self.primary_and_subgate_registry().peek().get(&id).cloned()
    }

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
        gate_type: PrimaryGateType,
    ) -> Result<()> {
        let key = GatesOnPlotKey::new(x_param.clone(), y_param.clone(), parental_gate_id.clone());
        println!("{:?}", key);
        let parameters = (x_param.clone(), y_param.clone());

        // let mut composite_subgate_ids = vec![];

        let g: Arc<dyn DrawableGate + 'static> = match gate_type {
            PrimaryGateType::Polygon => {
                let geo = flow_gates::geometry::create_polygon_geometry(
                    points.ok_or(anyhow!("points not provided for polygon gate"))?,
                    &x_param,
                    &y_param,
                )
                .map_err(|_| anyhow!("failed to create polygon geometry"))?;
                let gate = Gate {
                    id: Arc::from(id.as_str()),
                    name: id,
                    geometry: geo,
                    mode: flow_gates::GateMode::Global,
                    parameters,
                    label_position: None,
                };
                Arc::new(PolygonGate::try_new(gate)?)
            }
            PrimaryGateType::Ellipse => {
                let geo = create_default_ellipse(
                    &mapper, click_x, click_y, 50f32, 30f32, &x_param, &y_param,
                )?;
                let gate = Gate {
                    id: Arc::from(id.as_str()),
                    name: id,
                    geometry: geo,
                    mode: flow_gates::GateMode::Global,
                    parameters,
                    label_position: None,
                };
                Arc::new(EllipseGate::try_new(gate)?)
            }
            PrimaryGateType::Rectangle => {
                let geo = create_default_rectangle(
                    &mapper, click_x, click_y, 50f32, 50f32, &x_param, &y_param,
                )?;
                let gate = Gate {
                    id: Arc::from(id.as_str()),
                    name: id,
                    geometry: geo,
                    mode: flow_gates::GateMode::Global,
                    parameters,
                    label_position: None,
                };
                Arc::new(RectangleGate::try_new(gate)?)
            }
            PrimaryGateType::Line(y_coord) => {
                let geo = create_default_line(&mapper, click_x, 50f32, &x_param, &y_param)?;
                if let Some(y_coord) = y_coord {
                    let gate = Gate {
                        id: Arc::from(id.as_str()),
                        name: id,
                        geometry: geo,
                        mode: flow_gates::GateMode::Global,
                        parameters,
                        label_position: None,
                    };
                    Arc::new(LineGate::try_new(gate, y_coord)?)
                } else {
                    Err(anyhow!(
                        "Line gate requires y coordinate for initialization"
                    ))?
                }
            }

            PrimaryGateType::Bisector => Arc::new(BisectorGate::try_new(
                mapper,
                Arc::from(id.as_str()),
                (click_x, click_y),
                x_param,
                y_param,
            )?),
            PrimaryGateType::Quadrant => Arc::new(QuadrantGate::try_new_from_raw_coord(
                mapper,
                Arc::from(id.as_str()),
                (click_x, click_y),
                x_param,
                y_param,
            )?),
            PrimaryGateType::SkewedQuadrant => Arc::new(SkewedQuadrantGate::try_new_from_raw_coord(
                mapper,
                Arc::from(id.as_str()),
                (click_x, click_y),
                x_param,
                y_param,
            )?),
            _ => panic!("add boolean gate with add_boolean_gate")
        };

        let mut w = self.write();

  
        let gate_key = g.get_id();

        w.gate_ids_by_view
            .entry(key)
            .or_insert(vec![])
            .push(gate_key.clone());

        if g.is_composite() {
            let gates = g.get_inner_gate_ids();
            for sg in gates {
                println!(
                    "Adding composite subgate gate {} with parent {}",
                    sg,
                    parental_gate_id.as_ref().unwrap_or(&ROOTGATE)
                );
                w.hierarchy.add_gate_child(
                    parental_gate_id.clone().unwrap_or(ROOTGATE.clone()),
                    sg.clone(),
                )?;
                w.primary_and_subgate_registry
                    .insert(sg, g.clone());
            }
        } else {
            println!(
                "Adding gate {} with parent {}",
                g.get_id(),
                parental_gate_id.as_ref().unwrap_or(&ROOTGATE)
            );
            w.hierarchy
                .add_gate_child(parental_gate_id.unwrap_or(ROOTGATE.clone()), g.get_id())?;
        }

        w.primary_gate_registry
            .insert(gate_key.clone(), g.clone());
        w.primary_and_subgate_registry
            .insert(gate_key.clone(), g.clone());
            

            // w.primary_subgate_and_bool_registry.insert(g.get_id(), g);
        

        Ok(())
    }

    fn add_boolean_gate(&mut self, id: &str, operation: BooleanOperation, linked_gate_ids: Vec<GateId>, parental_gate_id: Option<GateId>, x_param: Arc<str>, y_param: Arc<str>) -> anyhow::Result<()> {

        let gate_id: Arc<str> = Arc::from(id);
        for link in linked_gate_ids.iter(){
            self.boolean_gate_links().write()
                .entry(link.clone()) 
                .or_insert_with(Vec::new) 
                .push(gate_id.clone());
        } 
        let g = Arc::new(BooleanGate::new(gate_id.clone(), linked_gate_ids, operation, x_param, y_param)?);
        let mut w = self.write();
        w.hierarchy.add_gate_child(parental_gate_id.unwrap_or(ROOTGATE.clone()), gate_id.clone())?;
        
        w.primary_gate_registry.insert(g.get_id(), g.clone());
        w.primary_and_subgate_registry.insert(g.get_id(), g);
        
        Ok(())
    }

    fn remove_gate(&mut self, gate_id: GateId) -> anyhow::Result<()> {

        let mut brothers = vec![];
        if let Some((_id, temp_g)) = self.primary_and_subgate_registry().peek().get_key_value(&gate_id){
            if temp_g.is_composite(){
                brothers.extend_from_slice(&temp_g.get_inner_gate_ids());
            } else {
                brothers.push(gate_id.clone());
            }
        }

        let mut state = self.write();

        let mut roots: HashSet<Arc<str>> = HashSet::default();

        while let Some(id) = brothers.pop() {
            if roots.insert(id.clone()) {
                if let Some(deps) = state.boolean_gate_links.remove(&id) {
                    brothers.extend(deps);
                }
            }
        }

        let mut gates_to_delete: HashSet<Arc<str>> = HashSet::default();

        for brother in roots {
            gates_to_delete.extend(state.hierarchy.delete_subtree(&brother));
        }
        

        for child_gate_id in gates_to_delete{

            if let Some((id, gate)) = state.primary_and_subgate_registry.remove_with_key(&child_gate_id){
                let drawable_gate_id = gate.get_id();
                let params = gate.get_params();
                let parent = state.hierarchy.get_parent(&id).unwrap_or_else(|| &ROOTGATE).clone();
                state.primary_gate_registry.remove(&drawable_gate_id);
                let key = GatesOnPlotKey::new(params.0, params.1, Some(parent));
                if let Some(gate_list) = state.gate_ids_by_view.get_mut(&key) {
                    gate_list.retain(|id| id != &drawable_gate_id);
                }
                
                state.position_overrides.remove(&drawable_gate_id);
            } else {

            }
        }
        

        Ok(())
    }

    fn move_gate_point(
        &mut self,
        gate_id: GateId,
        point_idx: usize,
        new_point: (f32, f32),
        plot_map: &PlotMapper,
    ) -> anyhow::Result<()> {
        let new_gate = self
            .primary_gate_registry()
            .read()
            .get(&gate_id.clone())
            .ok_or_else(|| anyhow!("Gate {} does not exist", gate_id.clone()))?
            .replace_point(new_point, point_idx, plot_map)?;
        
        let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);

        if new_gate_arc.is_composite() {
            let subgate_ids = new_gate_arc.get_inner_gate_ids();
            for subgate_id in subgate_ids {
                if let Some(gate_ptr) = self
                    .primary_and_subgate_registry()
                    .write()
                    .get_mut(&subgate_id)
                {
                    *gate_ptr = new_gate_arc.clone();
                }
            }
        }

        if let Some(gate_ptr) = self
            .primary_and_subgate_registry()
            .write()
            .get_mut(&gate_id)
        {
            *gate_ptr = new_gate_arc.clone();
        }

        if let Some(gate_ptr) = self.primary_gate_registry().write().get_mut(&gate_id) {
            *gate_ptr = new_gate_arc.clone();
        }

        Ok(())
    }

    fn move_gate(&mut self, gate_drag_data: GateDragData) -> Result<()> {
        let gate_id = &gate_drag_data.gate_id();
        let new_gate = self
            .primary_gate_registry()
            .read()
            .get(&gate_id.clone())
            .ok_or_else(|| anyhow!("Gate {} does not exist", gate_id.clone()))?
            .replace_points(gate_drag_data)?;
        if let Some(new_gate) = new_gate {
            let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);

            if new_gate_arc.is_composite() {
                let subgate_ids = new_gate_arc.get_inner_gate_ids();
                for subgate_id in subgate_ids {
                    if let Some(gate_ptr) = self
                        .primary_and_subgate_registry()
                        .write()
                        .get_mut(&subgate_id)
                    {
                        *gate_ptr = new_gate_arc.clone();
                    }
                }
            }

            if let Some(gate_ptr) = self
                .primary_and_subgate_registry()
                .write()
                .get_mut(&gate_id.clone())
            {
                *gate_ptr = new_gate_arc.clone();
            }

            if let Some(gate_ptr) = self.primary_gate_registry().write().get_mut(gate_id) {
                *gate_ptr = new_gate_arc.clone();
            }
        }

        Ok(())
    }

    fn rotate_gate(&mut self, gate_id: GateId, current_position: (f32, f32)) -> anyhow::Result<()> {
        let new_gate = self
            .primary_gate_registry()
            .read()
            .get(&gate_id)
            .ok_or_else(|| anyhow!("Gate {} does not exist", gate_id.clone()))?
            .rotate_gate(current_position)?;
        if let Some(new_gate) = new_gate {
            let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);

            if new_gate_arc.is_composite() {
                let subgate_ids = new_gate_arc.get_inner_gate_ids();
                for subgate_id in subgate_ids {
                    if let Some(gate_ptr) = self
                        .primary_and_subgate_registry()
                        .write()
                        .get_mut(&subgate_id)
                    {
                        *gate_ptr = new_gate_arc.clone();
                    }
                }
            }

            if let Some(gate_ptr) = self
                .primary_and_subgate_registry()
                .write()
                .get_mut(&gate_id)
            {
                *gate_ptr = new_gate_arc.clone();
            }

            if let Some(gate_ptr) = self.primary_gate_registry().write().get_mut(&gate_id) {
                *gate_ptr = new_gate_arc.clone();
            }
        }

        Ok(())
    }

    fn get_gates_for_plot<T>(
        &self,
        x_axis_title: T,
        y_axis_title: T,
        parental_gate_id: Option<T>,
    ) -> Option<Vec<Arc<dyn DrawableGate>>>
    where
        T: Into<GateId>,
    {
        let key = GatesOnPlotKey::new(
            x_axis_title.into(),
            y_axis_title.into(),
            parental_gate_id.map(|id| id.into()),
        );
        let key_options = self.gate_ids_by_view().get(key);
        let mut gate_list = vec![];
        if let Some(key_store) = key_options {
            let ids = key_store.read().clone();
            let registry = self.primary_gate_registry();
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
        parental_gate_id: Option<Arc<str>>,
    ) -> anyhow::Result<()> {
        let x: &str = &x_axis_title;
        let y: &str = &y_axis_title;
        let key = GatesOnPlotKey::new(x_axis_title.clone(), y_axis_title.clone(), parental_gate_id);
        let key_options = self.gate_ids_by_view().get(key);

        if let Some(key_store) = key_options {
            let ids = key_store.read().clone();

            for k in ids {
                let new_gate = self
                    .primary_gate_registry()
                    .read()
                    .get(&k)
                    .ok_or_else(|| anyhow!("Gate {} does not exist", k.clone()))?
                    .match_to_plot_axis(x, y)?;

                if let Some(new_gate) = new_gate {
                    let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);

                    if new_gate_arc.is_composite() {
                        let subgate_ids = new_gate_arc.get_inner_gate_ids();
                        for subgate_id in subgate_ids {
                            if let Some(gate_ptr) = self
                                .primary_and_subgate_registry()
                                .write()
                                .get_mut(&subgate_id)
                            {
                                *gate_ptr = new_gate_arc.clone();
                            }
                        }
                    }

                    if let Some(gate_ptr) = self.primary_and_subgate_registry().write().get_mut(&k)
                    {
                        *gate_ptr = new_gate_arc.clone();
                    }

                    if let Some(gate_ptr) = self.primary_gate_registry().write().get_mut(&k) {
                        *gate_ptr = new_gate_arc.clone();
                    }
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

        // 1. Create a temporary storage for the new gates to avoid nested locking
        let mut updates: Vec<(Arc<str>, Arc<dyn DrawableGate>)> = Vec::new();

        // Scope for the first read/write lock
        {
            for (_, gate) in self.primary_gate_registry().read().iter() {
                let (x_marker, y_marker) = gate.get_params();
                if marker == &x_marker || marker == &y_marker {
                    match gate.recalculate_gate_for_rescaled_axis(
                        marker.clone(),
                        &old_axis_options.transform,
                        &new_axis_options.transform,
                        (new_axis_options.data_lower, new_axis_options.data_upper),
                        (new_axis_options.axis_lower, new_axis_options.axis_upper),
                    ) {
                        Ok(new_g) => updates.push((gate.get_id(), Arc::from(new_g))),
                        Err(e) => errors.push(e.to_string()),
                    }
                }
            }
        } // First lock drops here!

        // 2. Now apply updates to both registries
        if !updates.is_empty() {
            for (gate_id, new_gate_arc) in updates {
                // Update subgates if composite
                if new_gate_arc.is_composite() {
                    for subgate_id in new_gate_arc.get_inner_gate_ids() {
                        if let Some(gate_ptr) = self
                            .primary_and_subgate_registry()
                            .write()
                            .get_mut(&subgate_id)
                        {
                            *gate_ptr = new_gate_arc.clone();
                        }
                    }
                }

                // Update primary and sub registry for the main gate
                if let Some(gate_ptr) = self
                    .primary_and_subgate_registry()
                    .write()
                    .get_mut(&gate_id)
                {
                    *gate_ptr = new_gate_arc.clone();
                }

                // Update primary registry
                if let Some(gate_ptr) = self.primary_gate_registry().write().get_mut(&gate_id) {
                    *gate_ptr = new_gate_arc.clone();
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn set_current_axis_limits(
        &mut self,
        axis_name: Arc<str>,
        lower: f32,
        upper: f32,
        transform: TransformType,
    ) -> anyhow::Result<()> {
        let mut updates: Vec<(Arc<str>, Arc<dyn DrawableGate>)> = Vec::new();
        let mut errors = vec![];
        // Scope for the first read/write lock
        {
            for (_, gate) in self.primary_gate_registry().read().iter() {
                let (x_marker, y_marker) = gate.get_params();
                if axis_name == x_marker || axis_name == y_marker {
                    match gate.recalculate_gate_for_new_axis_limits(
                        axis_name.clone(),
                        lower,
                        upper,
                        &transform,
                    ) {
                        Ok(Some(new_g)) => updates.push((gate.get_id(), Arc::from(new_g))),
                        Ok(None) => continue,
                        Err(e) => errors.push(e.to_string()),
                    }
                }
            }
        }

        // 2. Now apply updates to both registries

        for (gate_id, new_gate_arc) in updates {
            if new_gate_arc.is_composite() {
                let subgate_ids = new_gate_arc.get_inner_gate_ids();
                for subgate_id in subgate_ids {
                    if let Some(gate_ptr) = self
                        .primary_and_subgate_registry()
                        .write()
                        .get_mut(&subgate_id)
                    {
                        *gate_ptr = new_gate_arc.clone();
                    }
                }
            }

            if let Some(gate_ptr) = self
                .primary_and_subgate_registry()
                .write()
                .get_mut(&gate_id)
            {
                *gate_ptr = new_gate_arc.clone();
            }

            if let Some(gate_ptr) = self.primary_gate_registry().write().get_mut(&gate_id) {
                *gate_ptr = new_gate_arc.clone();
            }
        }

        Ok(())
    }
}

