use crate::gate_editor::gates::{gate_store::FileId, gate_types::GateStats};
use dioxus::prelude::*;
use flow_gates::EventIndex;
use rustc_hash::FxHashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct EventIndexMapped {
    pub event_index: Arc<EventIndex>,
    pub index_map: Arc<Vec<usize>>,
}

impl PartialEq for EventIndexMapped {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.event_index, &other.event_index)
            && Arc::ptr_eq(&self.index_map, &other.index_map)
    }
}

#[derive(Default, Store, Clone)]
pub struct PlotStore {
    pub current_file_id: FileId,
    pub event_index_map: Option<EventIndexMapped>,
    pub gate_stats: FxHashMap<Arc<str>, GateStats>,
    // current settings ordered by the current sample
}
