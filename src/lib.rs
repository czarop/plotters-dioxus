use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

pub mod components;
pub mod file_load;
pub mod gate_editor;
pub mod searchable_select;
pub mod omiq;
pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
