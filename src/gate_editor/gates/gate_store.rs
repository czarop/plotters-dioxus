use anyhow::anyhow;
use dioxus::prelude::*;
use flow_fcs::TransformType;
use flow_gates::{BooleanOperation, Gate, GateHierarchy};
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, LazyLock};
use uuid::Uuid;

use crate::gate_editor::gates::gate_single::boolean_gates::BooleanGate;
use crate::gate_editor::gates::gate_types::GateStats;
use crate::gate_editor::{
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
pub type GroupId = std::sync::Arc<str>;

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

pub struct GateMap(pub FxHashMap<GateId, Arc<dyn DrawableGate + 'static>>);

impl Deref for GateMap {
    type Target = FxHashMap<GateId, Arc<dyn DrawableGate + 'static>>;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GateSource {
    Global,
    Group((GateId, GroupId)),
    Sample((GateId, FileId)),
}

pub type GroupGateMap =
    im::HashMap<(GateId, GroupId), Arc<dyn DrawableGate>, rustc_hash::FxBuildHasher>;
pub type SampleGateMap =
    im::HashMap<(GateId, FileId), Arc<dyn DrawableGate>, rustc_hash::FxBuildHasher>;

#[derive(Default, Store)]
pub struct GateSubStore {
    pub primary_and_subgate_registry: GateMap,
    pub sample_position_overrides: SampleGateMap,
    pub group_position_overrides: GroupGateMap,
}

#[derive(Clone, Default, PartialEq)]
pub struct GateOverrideResolver {
    pub active_gates: im::HashMap<GateId, ComparableGate, FxBuildHasher>,
    pub gate_origins: im::HashMap<GateId, GateSource, FxBuildHasher>,
}

#[derive(Clone)]
pub struct ComparableGate(pub Arc<dyn DrawableGate>);

impl PartialEq for ComparableGate {
    fn eq(&self, other: &Self) -> bool {
        // Fast pointer comparison: Are these the same allocation?
        Arc::ptr_eq(&self.0, &other.0)
    }
}

// This allows: my_comparable_gate.draw()
impl Deref for ComparableGate {
    type Target = Arc<dyn DrawableGate>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Arc<dyn DrawableGate>> for ComparableGate {
    fn from(arc: Arc<dyn DrawableGate>) -> Self {
        Self(arc)
    }
}

// for overides generate a clone of the drawable in new position with the same Uuid
impl GateOverrideResolver {
    fn resolve(&self, id: &GateId) -> anyhow::Result<Gate> {
        self.active_gates
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Gate {} not found in active set", id))?
            .get_gate_ref(Some(id))
            .map(|g| g.clone())
            .ok_or_else(|| anyhow::anyhow!("Gate {} has no internal data", id))
    }

    fn resolve_drawable(&self, id: &str) -> anyhow::Result<Arc<dyn DrawableGate + 'static>> {
        let drawable = self
            .active_gates
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Gate {} not found in active set", id))?;
        Ok(drawable.deref().clone())
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
    file_id: FileId,
    selected_gate: Option<Arc<str>>,
    // For the Renderer: "What gates do I draw on this Plot?"
    gate_ids_by_view: FxHashMap<GatesOnPlotKey, Vec<GateId>>,

    // For the Filtering: "How are these gates nested?" - this is the master hierarchy
    hierarchy: GateHierarchy,

    // when deleting a gate, do you need to delete any boolean gates that depend on it?
    boolean_gate_links: FxHashMap<GateId, Vec<GateId>>,

    gate_stats: FxHashMap<Arc<str>, GateStats>,

    gate_store: GateSubStore,
}

#[store(pub name = GateStateImplExt)]
impl<Lens> Store<GateState, Lens> {
    fn get_current_sample(
        &mut self,
        file_id: FileId,
        group_ids: &[GroupId],
    ) -> Result<GateOverrideResolver> {
        // construct the GateResolver for this file
        let mut active_gates: im::HashMap<Arc<str>, ComparableGate, FxBuildHasher> =
            im::HashMap::with_hasher(FxBuildHasher);
        let mut gate_origins = im::HashMap::with_hasher(FxBuildHasher);

        {
            let registry_binding = self.gate_store().primary_and_subgate_registry();
            let registry = registry_binding.read();
            let sample_ovr_binding = self.gate_store().sample_position_overrides();
            let sample_overrides = sample_ovr_binding.read();
            let group_ovr_binding = self.gate_store().group_position_overrides();
            let group_overrides = group_ovr_binding.read();

            for (default_id, base_arc) in &registry.0 {
                if let Some((key, s_ovr)) =
                    sample_overrides.get_key_value(&(default_id.clone(), file_id.clone()))
                {
                    active_gates.insert(default_id.clone(), s_ovr.clone().into());
                    gate_origins.insert(default_id.clone(), GateSource::Sample(key.clone()));
                } else if let Some((key, g_ovr)) = group_ids.iter().find_map(|gid| {
                    group_overrides.get_key_value(&(default_id.clone(), gid.clone()))
                }) {
                    active_gates.insert(default_id.clone(), g_ovr.clone().into());
                    gate_origins.insert(default_id.clone(), GateSource::Group(key.clone()));
                } else {
                    active_gates.insert(default_id.clone(), base_arc.clone().into());
                    gate_origins.insert(default_id.clone(), GateSource::Global);
                }
            }
        }

        Ok(GateOverrideResolver {
            active_gates,
            gate_origins,
        })
    }

    fn get_gate_by_id(
        &self,
        id: GateId,
        resolver: &GateOverrideResolver,
    ) -> Option<Arc<dyn DrawableGate>> {
        resolver.resolve_drawable(&id).ok()
    }

    fn add_gate(
        &mut self,
        mapper: &PlotMapper,
        click_x: f32,
        click_y: f32,
        x_param: Arc<str>,
        y_param: Arc<str>,
        points: Option<Vec<(f32, f32)>>,
        parental_gate_id: Option<GateId>,
        gate_type: PrimaryGateType,
        name: Option<String>,
    ) -> Result<()> {
        let key = GatesOnPlotKey::new(x_param.clone(), y_param.clone(), parental_gate_id.clone());
        println!("{:?}", key);
        let parameters = (x_param.clone(), y_param.clone());

        let id = Uuid::new_v4().to_string();
        let id_arc: Arc<str> = Arc::from(id.as_ref() as &str);

        let g: Arc<dyn DrawableGate + 'static> = match gate_type {
            PrimaryGateType::Polygon => {
                let geo = flow_gates::geometry::create_polygon_geometry(
                    points.ok_or(anyhow!("points not provided for polygon gate"))?,
                    &x_param,
                    &y_param,
                )
                .map_err(|_| anyhow!("failed to create polygon geometry"))?;
                let gate = Gate {
                    id: id_arc,
                    name: name.unwrap_or(id.to_string()),
                    geometry: geo,
                    mode: flow_gates::GateMode::Global,
                    parameters,
                    label_position: None,
                };
                Arc::new(PolygonGate::try_new(gate, true)?)
            }
            PrimaryGateType::Ellipse => {
                let geo = create_default_ellipse(
                    &mapper, click_x, click_y, 50f32, 30f32, &x_param, &y_param,
                )?;
                let gate = Gate {
                    id: id_arc,
                    name: name.unwrap_or(id.to_string()),
                    geometry: geo,
                    mode: flow_gates::GateMode::Global,
                    parameters,
                    label_position: None,
                };
                Arc::new(EllipseGate::try_new(gate, true)?)
            }
            PrimaryGateType::Rectangle => {
                let geo = create_default_rectangle(
                    &mapper, click_x, click_y, 50f32, 50f32, &x_param, &y_param,
                )?;
                let gate = Gate {
                    id: id_arc,
                    name: name.unwrap_or(id.to_string()),
                    geometry: geo,
                    mode: flow_gates::GateMode::Global,
                    parameters,
                    label_position: None,
                };
                Arc::new(RectangleGate::try_new(gate, true)?)
            }
            PrimaryGateType::Line(y_coord) => {
                let geo = create_default_line(&mapper, click_x, 50f32, &x_param, &y_param)?;
                if let Some(y_coord) = y_coord {
                    let gate = Gate {
                        id: id_arc,
                        name: name.unwrap_or(id.to_string()),
                        geometry: geo,
                        mode: flow_gates::GateMode::Global,
                        parameters,
                        label_position: None,
                    };
                    Arc::new(LineGate::try_new(gate, y_coord, true)?)
                } else {
                    Err(anyhow!(
                        "Line gate requires y coordinate for initialization"
                    ))?
                }
            }

            PrimaryGateType::Bisector => Arc::new(BisectorGate::try_new(
                mapper,
                id_arc,
                name.unwrap_or(id.to_string()),
                (click_x, click_y),
                x_param,
                y_param,
            )?),
            PrimaryGateType::Quadrant => Arc::new(QuadrantGate::try_new_from_raw_coord(
                mapper,
                id_arc,
                name.unwrap_or(id.to_string()),
                (click_x, click_y),
                x_param,
                y_param,
            )?),
            PrimaryGateType::SkewedQuadrant => {
                Arc::new(SkewedQuadrantGate::try_new_from_raw_coord(
                    mapper,
                    id_arc,
                    name.unwrap_or(id.to_string()),
                    (click_x, click_y),
                    x_param,
                    y_param,
                )?)
            }
            _ => panic!("add boolean gate with add_boolean_gate"),
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
                w.gate_store
                    .primary_and_subgate_registry
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

        w.gate_store
            .primary_and_subgate_registry
            .insert(gate_key.clone(), g.clone());

        Ok(())
    }

    fn add_boolean_gate(
        &mut self,
        name: Option<String>,
        operation: BooleanOperation,
        linked_gate_ids: Vec<GateId>,
        parental_gate_id: Option<GateId>,
        x_param: Arc<str>,
        y_param: Arc<str>,
    ) -> anyhow::Result<()> {
        let id = Uuid::new_v4().to_string();
        let gate_id: Arc<str> = Arc::from(id.as_ref() as &str);

        self.boolean_gate_links().with_mut(|w| {
            for link in linked_gate_ids.iter() {
                w.entry(link.clone())
                    .or_insert_with(Vec::new)
                    .push(gate_id.clone());
            }
        });

        let g = Arc::new(BooleanGate::new(
            gate_id.clone(),
            name.unwrap_or(id),
            linked_gate_ids,
            operation,
            x_param,
            y_param,
        )?);

        self.hierarchy().write().add_gate_child(
            parental_gate_id.unwrap_or(ROOTGATE.clone()),
            gate_id.clone(),
        )?;

        self.gate_store()
            .primary_and_subgate_registry()
            .write()
            .insert(g.get_id(), g.clone());

        Ok(())
    }

    fn remove_gate(&mut self, gate_id: GateId) -> anyhow::Result<()> {
        // build the collection of gates at the same level that need deleting
        // that's any composite 'brothers'
        let mut brothers = vec![];
        if let Some((_id, temp_g)) = self
            .gate_store()
            .primary_and_subgate_registry()
            .peek()
            .get_key_value(&gate_id)
        {
            if temp_g.is_composite() {
                brothers.extend_from_slice(&temp_g.get_inner_gate_ids());
            } else {
                brothers.push(gate_id.clone());
            }
        }
        let mut state = self.write();

        let mut roots: HashSet<Arc<str>> = HashSet::default();
        // and any boolean gates that depend on these gates - and any that depend on them etc
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

        for child_gate_id in gates_to_delete {
            if let Some((id, gate)) = state
                .gate_store
                .primary_and_subgate_registry
                .remove_entry(&child_gate_id)
            {
                let drawable_gate_id = gate.get_id();
                let params = gate.get_params();
                let parent = state
                    .hierarchy
                    .get_parent(&id)
                    .unwrap_or_else(|| &ROOTGATE)
                    .clone();

                let key = GatesOnPlotKey::new(params.0, params.1, Some(parent));
                if let Some(gate_list) = state.gate_ids_by_view.get_mut(&key) {
                    gate_list.retain(|id| id != &drawable_gate_id);
                }

                state
                    .gate_store
                    .sample_position_overrides
                    .retain(|(gid, _file_id), _| gid != &gate_id);
                state
                    .gate_store
                    .group_position_overrides
                    .retain(|(gid, _group_id), _| gid != &gate_id);
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
        resolver: &GateOverrideResolver,
    ) -> anyhow::Result<()> {
        let new_gate = resolver
            .resolve_drawable(&gate_id)?
            .replace_point(new_point, point_idx, plot_map)?;
        let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);
        let gate_origin = resolver
            .gate_origins
            .get(&gate_id)
            .ok_or_else(|| anyhow!("error finding gate source for {}", &gate_id))?
            .clone();

        let ids_to_update = if new_gate_arc.is_composite() {
            let mut ids = new_gate_arc.get_inner_gate_ids();
            ids.push(gate_id.clone());
            ids
        } else {
            vec![gate_id.clone()]
        };

        self.gate_store().with_mut(|state| {
            for id in ids_to_update {
                match &gate_origin {
                    GateSource::Global => {
                        state
                            .primary_and_subgate_registry
                            .insert(id.clone(), new_gate_arc.clone());
                    }
                    GateSource::Group(k) => {
                        state
                            .group_position_overrides
                            .insert(k.clone(), new_gate_arc.clone());
                    }
                    GateSource::Sample(k) => {
                        state
                            .sample_position_overrides
                            .insert(k.clone(), new_gate_arc.clone());
                    }
                }
            }
        });
        Ok(())
    }

    fn move_gate(
        &mut self,
        gate_drag_data: GateDragData,
        resolver: &GateOverrideResolver,
    ) -> Result<()> {
        let gate_id = gate_drag_data.gate_id();

        let new_gate = resolver
            .resolve_drawable(&gate_id)?
            .replace_points(gate_drag_data)?;

        let gate_origin = resolver
            .gate_origins
            .get(&gate_id)
            .ok_or_else(|| anyhow!("error finding gate source for {}", &gate_id))?
            .clone();

        if let Some(new_gate) = new_gate {
            let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);
            let ids_to_update = if new_gate_arc.is_composite() {
                let mut ids = new_gate_arc.get_inner_gate_ids();
                ids.push(gate_id.clone());
                ids
            } else {
                vec![gate_id.clone()]
            };

            self.gate_store().with_mut(|state| {
                for id in ids_to_update {
                    match &gate_origin {
                        GateSource::Global => {
                            state
                                .primary_and_subgate_registry
                                .insert(id.clone(), new_gate_arc.clone());
                        }
                        GateSource::Group(k) => {
                            state
                                .group_position_overrides
                                .insert(k.clone(), new_gate_arc.clone());
                        }
                        GateSource::Sample(k) => {
                            state
                                .sample_position_overrides
                                .insert(k.clone(), new_gate_arc.clone());
                        }
                    }
                }
            });
        }
        Ok(())
    }

    fn rotate_gate(
        &mut self,
        gate_id: GateId,
        current_position: (f32, f32),
        resolver: &GateOverrideResolver,
    ) -> anyhow::Result<()> {
        let new_gate = resolver
            .resolve_drawable(&gate_id)?
            .rotate_gate(current_position)?;

        let gate_origin = resolver
            .gate_origins
            .get(&gate_id)
            .ok_or_else(|| anyhow!("error finding gate source for {}", &gate_id))?
            .clone();
        drop(resolver);
        if let Some(new_gate) = new_gate {
            let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);
            let ids_to_update = if new_gate_arc.is_composite() {
                let mut ids = new_gate_arc.get_inner_gate_ids();
                ids.push(gate_id.clone());
                ids
            } else {
                vec![gate_id.clone()]
            };

            self.gate_store().with_mut(|state| {
                for id in ids_to_update {
                    match &gate_origin {
                        GateSource::Global => {
                            state
                                .primary_and_subgate_registry
                                .insert(id.clone(), new_gate_arc.clone());
                        }
                        GateSource::Group(k) => {
                            state
                                .group_position_overrides
                                .insert(k.clone(), new_gate_arc.clone());
                        }
                        GateSource::Sample(k) => {
                            state
                                .sample_position_overrides
                                .insert(k.clone(), new_gate_arc.clone());
                        }
                    }
                }
            });
        }
        Ok(())
    }

    fn get_gates_for_plot<T>(
        &mut self,
        x_axis_title: T,
        y_axis_title: T,
        parental_gate_id: Option<T>,
        resolver: &GateOverrideResolver,
    ) -> Result<Vec<Arc<dyn DrawableGate>>>
    where
        T: Into<GateId> + Clone,
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

            for k in ids {
                if let Ok(gate_store_entry) = resolver.resolve_drawable(&k) {
                    if gate_store_entry.is_primary() {
                        gate_list.push(gate_store_entry.clone());
                    }
                }
            }
        } else {
            return Err(anyhow::anyhow!("No keys found").into());
        }

        return Ok(gate_list);
    }

    fn match_gates_to_plot<T>(
        &mut self,
        x_axis_title: T,
        y_axis_title: T,
        parental_gate_id: Option<T>,
        resolver: &GateOverrideResolver,
    ) -> anyhow::Result<()>
    where
        T: Into<GateId> + Clone,
    {
        let (x, y) = (x_axis_title.clone().into(), y_axis_title.clone().into());
        let key = GatesOnPlotKey::new(
            x_axis_title.clone().into(),
            y_axis_title.into(),
            parental_gate_id.map(|id| id.into()),
        );
        let mut updates = Vec::new();
        {
            let key_bind = self.gate_ids_by_view();
            let kbp = &*key_bind.peek();
            let Some(ids) = kbp.get(&key) else {
                return Err(anyhow::anyhow!("No keys found"));
            };

            for k in ids {
                let Some(new_gate) = resolver.resolve_drawable(k)?.match_to_plot_axis(&x, &y)?
                else {
                    continue;
                };
                let new_gate_arc: Arc<dyn DrawableGate> = Arc::from(new_gate);
                let gate_origin = resolver
                    .gate_origins
                    .get(k)
                    .ok_or_else(|| anyhow!("error finding gate source for {}", k))?
                    .clone();

                updates.push((
                    new_gate_arc.get_id(),
                    new_gate_arc.clone(),
                    gate_origin.clone(),
                ));

                if new_gate_arc.is_composite() {
                    for sub_id in new_gate_arc.get_inner_gate_ids() {
                        updates.push((sub_id, new_gate_arc.clone(), gate_origin.clone()));
                    }
                }
            }
        }
        self.gate_store().with_mut(|s| {
            for (k, v, o) in updates {
                match &o {
                    GateSource::Global => {
                        s.primary_and_subgate_registry.insert(k.clone(), v.clone());
                    }
                    GateSource::Group(k) => {
                        s.group_position_overrides.insert(k.clone(), v.clone());
                    }
                    GateSource::Sample(k) => {
                        s.sample_position_overrides.insert(k.clone(), v.clone());
                    }
                }
            }
        });

        return Ok(());
    }

    // to do

    fn rescale_gates(
        &mut self,
        marker: &Arc<str>,
        old_axis_options: &AxisInfo,
        new_axis_options: &AxisInfo,
    ) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        self.gate_store().with_mut(|s| {
            let mut memo: FxHashMap<usize, Arc<dyn DrawableGate>> = FxHashMap::default();
            let mut scale_gate = |gate: &Arc<dyn DrawableGate>| -> Arc<dyn DrawableGate> {
                // to avoid rescaling composite gates over and over (as they are also stored under subgate id's)
                // we load any scaled gates into a hashmap
                // we compare by heap memory address - anything pointing to the same address
                // will use the cached rescaled value
                let ptr = Arc::as_ptr(gate) as *const () as usize;
                if let Some(scaled) = memo.get(&ptr) {
                    return scaled.clone();
                }
                let (x_marker, y_marker) = gate.get_params();
                if marker == &x_marker || marker == &y_marker {
                    let new_gate = match gate.recalculate_gate_for_rescaled_axis(
                        marker.clone(),
                        &old_axis_options.transform,
                        &new_axis_options.transform,
                        (new_axis_options.data_lower, new_axis_options.data_upper),
                        (new_axis_options.axis_lower, new_axis_options.axis_upper),
                    ) {
                        Ok(new_gate) => Arc::from(new_gate),
                        Err(e) => {
                            errors.push(e.to_string());
                            gate.clone()
                        }
                    };
                    memo.insert(ptr, new_gate.clone());
                    new_gate
                } else {
                    gate.clone()
                }
            };

            s.primary_and_subgate_registry = GateMap(
                s.primary_and_subgate_registry
                    .iter()
                    .map(|(id, gate)| (id.clone(), scale_gate(gate)))
                    .collect(),
            );

            s.sample_position_overrides = s
                .sample_position_overrides
                .iter()
                .map(|(key, gate)| (key.clone(), scale_gate(gate)))
                .collect();

            s.group_position_overrides = s
                .group_position_overrides
                .iter()
                .map(|(key, gate)| (key.clone(), scale_gate(gate)))
                .collect();
        });
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
    ) -> Result<(), Vec<String>> {
        let mut errors = vec![];

        self.gate_store().with_mut(|s| {
            let mut memo: FxHashMap<usize, Arc<dyn DrawableGate>> = FxHashMap::default();
            let mut scale_gate = |gate: &Arc<dyn DrawableGate>| -> Arc<dyn DrawableGate> {
                // to avoid rescaling composite gates over and over (as they are also stored under subgate id's)
                // we load any scaled gates into a hashmap
                // we compare by heap memory address - anything pointing to the same address
                // will use the cached rescaled value
                let ptr = Arc::as_ptr(gate) as *const () as usize;
                if let Some(scaled) = memo.get(&ptr) {
                    return scaled.clone();
                }
                let (x_marker, y_marker) = gate.get_params();
                if axis_name == x_marker || axis_name == y_marker {
                    let new_gate = match gate.recalculate_gate_for_new_axis_limits(
                        axis_name.clone(),
                        lower,
                        upper,
                        &transform,
                    ) {
                        Ok(Some(new_gate)) => Arc::from(new_gate),
                        Ok(None) => gate.clone(),
                        Err(e) => {
                            errors.push(e.to_string());
                            gate.clone()
                        }
                    };
                    memo.insert(ptr, new_gate.clone());
                    new_gate
                } else {
                    gate.clone()
                }
            };

            s.primary_and_subgate_registry = GateMap(
                s.primary_and_subgate_registry
                    .iter()
                    .map(|(id, gate)| (id.clone(), scale_gate(gate)))
                    .collect(),
            );

            s.sample_position_overrides = s
                .sample_position_overrides
                .iter()
                .map(|(key, gate)| (key.clone(), scale_gate(gate)))
                .collect();

            s.group_position_overrides = s
                .group_position_overrides
                .iter()
                .map(|(key, gate)| (key.clone(), scale_gate(gate)))
                .collect();
        });

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn get_gate_name(&self, id: GateId) -> Option<String> {
        if let Some(g) = self
            .gate_store()
            .peek()
            .primary_and_subgate_registry
            .get(&id)
        {
            return Some(g.get_name().to_string());
        }

        None
    }
}
