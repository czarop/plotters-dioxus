use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;

pub mod components;
pub mod file_load;
pub mod plotters_dioxus;
pub mod searchable_select;


pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;