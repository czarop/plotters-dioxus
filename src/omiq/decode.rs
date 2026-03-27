use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::gate_editor::gates::GateId;
use crate::gate_editor::gates::gate_store::{FileId, GateSource, GroupId};
use crate::gate_editor::gates::gate_traits::DrawableGate;
use crate::omiq::metadata::{MetaDataFileMap, MetaDataKey, MetaDataParameter};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentJson {
    pub tree: GatingTree,
}

// Gating Tree and Node will be made into the Gating Hierarchy
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GatingTree {
    pub nodes: HashMap<Arc<str>, GatingNode>,
    pub filter_containers: HashMap<Arc<str>, FilterContainer>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GatingNode {
    pub id: Arc<str>,
    pub parent_id: Arc<str>,
    pub filter_container_id: Arc<str>,
}

//FilterContainer is the actual gate info

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FilterContainer {
    pub id: GateId,
    pub name: Arc<str>,

    // The "Global" fallback
    pub default_filter: GateSerialized,

    // The metadata parameter controlling the grouping (e.g., "$VOL")
    // Use Option because not all gates are grouped by metadata.
    // this will be SOME if there is a metadata param governing this
    // ie group-specific - each file still gets an entry in file-specific
    // even if its group-specific
    pub md: Option<MetaDataParameter>,

    // The "File-Specific" variants
    // JSON: "perFileFilters": { "628415...": { ... } }
    #[serde(default)]
    pub per_file_filters: FxHashMap<FileId, GateSerialized>,
}

impl FilterContainer {
    // pub fn process_gates_to_drawable(
    //     &self,
    //     metadata_file_to_group_map: MetaDataFileMap,
    // ) -> anyhow::Result<Vec<(GateSource, Arc<dyn DrawableGate>)>> {
    //     let mut collections = Vec::new();
    //     let gate_id = self.id.clone();

    //     // 1. Handle Global (Default)
    //     let global_gate = self.default_filter.to_drawable()?;
    //     collections.push((GateSource::Global, global_gate));

    //     if let Some(ref md_key) = self.md { // this is broken - ensure aligns with metadatstore
    //         // --- GROUP SPECIFIC MODE ---
    //         // We use a temporary map to ensure one Arc per GroupId
    //         let mut group_cache: HashMap<MetaDataKey, Arc<dyn DrawableGate>> = HashMap::new();
    //         let file_to_group_map = metadata_file_to_group_map
    //             .get(md_key)
    //             .ok_or_else(|| anyhow::anyhow!("No metadata found for {}", md_key))?;

    //         for (file_id, gate_spec) in &self.per_file_filters {
    //             if let Some(group_id) = file_to_group_map.get(file_id) {
    //                 // Only create the Arc/Gate if we haven't seen this GroupId yet
    //                 if !group_cache.contains_key(group_id) {
    //                     let gate = gate_spec.to_drawable()?;
    //                     group_cache.insert(group_id.clone(), gate);
    //                 }
    //             }
    //         }

    //         // Move the unique group gates into the final collection
    //         for (group_id, gate_instance) in group_cache {
    //             collections.push((
    //                 GateSource::Group((group_id, gate_id.clone())),
    //                 gate_instance,
    //             ));
    //         }
    //     } else {
    //         // --- FILE SPECIFIC MODE ---
    //         // No metadata grouping; every entry is a unique Sample override
    //         for (file_id, gate_spec) in &self.per_file_filters {
    //             let file_gate = gate_spec.to_drawable()?;
    //             collections.push((
    //                 GateSource::Sample((gate_id.clone(), file_id.clone())),
    //                 file_gate,
    //             ));
    //         }
    //     }

    //     Ok(collections)
    // }
    pub fn process_gates_to_drawable(
        &self,
        metadata_file_to_group_map: &MetaDataFileMap, // Changed to ref to avoid move
    ) -> anyhow::Result<Vec<(GateSource, Arc<dyn DrawableGate>)>> {
        let mut collections = Vec::new();
        let gate_id = self.id.clone();

        // 1. Handle Global (Default)
        let global_gate = self.default_filter.to_drawable()?;
        collections.push((GateSource::Global, global_gate));

        if let Some(ref md_parameter) = self.md {
            // --- GROUP SPECIFIC MODE ---
            // We track unique MetaDataKeys (Param + Group) to avoid duplicate gates
            let mut group_cache: HashMap<MetaDataKey, Arc<dyn DrawableGate>> = HashMap::new();

            for (file_id, gate_spec) in &self.per_file_filters {
                // 1. Look up this file in our new File-First map
                if let Some(file_metadata) = metadata_file_to_group_map.get(file_id) {
                    // 2. Check if this file has the parameter Omiq is asking for
                    if let Some(group_id) = file_metadata.get(md_parameter) {
                        
                        let key = MetaDataKey {
                            parameter: md_parameter.clone(),
                            group: group_id.clone(),
                        };

                        // 3. Only create the DrawableGate if we haven't handled this specific Group yet
                        if !group_cache.contains_key(&key) {
                            let gate = gate_spec.to_drawable()?;
                            group_cache.insert(key, gate);
                        }
                    }
                }
            }

            // 4. Move the unique group gates into the final collection
            for (metadata_key, gate_instance) in group_cache {
                collections.push((
                    GateSource::Group((gate_instance.get_id(), metadata_key)), // Matches your new struct
                    gate_instance,
                ));
            }
        } else {
            // --- FILE SPECIFIC MODE ---
            for (file_id, gate_spec) in &self.per_file_filters {
                let file_gate = gate_spec.to_drawable()?;
                collections.push((
                    GateSource::Sample((gate_id.clone(), file_id.clone())),
                    file_gate,
                ));
            }
        }

        Ok(collections)
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum GateSerialized {
    #[serde(rename = "RectangleGate")]
    Rectangle {
        #[serde(rename = "f1")]
        x_param: Arc<str>,
        #[serde(rename = "f2")]
        y_param: Arc<str>,
        min: Point,
        max: Point,
        #[serde(rename = "labelLoc")]
        label_position: Point,
    },
    // Future-proofing for other gate types
    #[serde(other)]
    Unknown,
}

impl GateSerialized {
    pub fn to_drawable(&self) -> anyhow::Result<Arc<dyn DrawableGate>> {
        todo!()
    }
}

#[derive(Deserialize, Debug)]
pub struct Point {
    #[serde(rename = "f1Val")]
    pub x: f64,
    #[serde(rename = "f2Val")]
    pub y: f64,
}
