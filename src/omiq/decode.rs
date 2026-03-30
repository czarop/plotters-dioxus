use flow_gates::{GateError, GateGeometry, GateNode, create_polygon_geometry, create_rectangle_geometry};
use flow_gates::types::LabelPosition;
use rustc_hash::{FxHashMap};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::gate_editor::gates::GateId;
use crate::gate_editor::gates::gate_single::ellipse_gate::EllipseGate;
use crate::gate_editor::gates::gate_single::line_gate::LineGate;
use crate::gate_editor::gates::gate_single::polygon_gate::PolygonGate;
use crate::gate_editor::gates::gate_single::rectangle_gate::RectangleGate;
use crate::gate_editor::gates::gate_store::{FileId, GateSource};
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

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterContainer {
    pub id: GateId,
    pub name: Arc<str>,

    // The "Global" fallback
    #[serde(rename = "defaultFilter")]
    pub default_filter: GateSerialized,

    // for composite gates - this ties the individual gates together.
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,

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

    pub fn is_composite(&self) -> bool {
        self.group_id.is_some()
    }

    pub fn process_gates_to_drawable(
        &self,
        metadata_file_to_group_map: &MetaDataFileMap,
    ) -> anyhow::Result<Vec<(GateSource, Arc<dyn DrawableGate>)>> {
        let mut collections = Vec::new();
        let gate_id = self.id.clone();

        let is_composite = self.group_id.is_some();

        let rect_to_polygon = {
            if is_composite {
                self.group_id.as_ref().unwrap().contains("QUAD")
            } else {
                false
            }
        };

        // 1. Handle Global (Default)
        println!("attempting to make gate");
        let global_gate = self.default_filter.to_drawable(gate_id.clone(), self.name.clone(), flow_gates::GateMode::Global, rect_to_polygon)?;
        println!("CREATED GLOBAL GATE! ID: {}, Name: {}", global_gate.get_id(), global_gate.get_name());
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
                            let gate = gate_spec.to_drawable(gate_id.clone(), self.name.clone(), flow_gates::GateMode::Global, rect_to_polygon)?;
                            println!("CREATED GROUP-SPECIFIC GATE! ID: {}, Name: {}, Group: {}", gate.get_id(), gate.get_name(), key.group);
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
                let file_gate = gate_spec.to_drawable(gate_id.clone(), self.name.clone(), flow_gates::GateMode::Global, rect_to_polygon)?;
                println!("CREATED FILE-SPECIFIC GATE! ID: {}, Name: {}, File: {}", file_gate.get_id(), file_gate.get_name(), file_id);

                collections.push((
                    GateSource::Sample((gate_id.clone(), file_id.clone())),
                    file_gate,
                ));
            }
        }

        Ok(collections)
    }
}

#[derive(Deserialize, Debug, Clone)]
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
        label_position: Option<Point>,
    },
    #[serde(rename = "PolygonGate")]
    Polygon {
        #[serde(rename = "f1")]
        x_param: Arc<str>,
        #[serde(rename = "f2")]
        y_param: Arc<str>,
        #[serde(rename = "vertices")]
        points: Vec<Point>,
        #[serde(rename = "labelLoc")]
        label_position: Option<Point>,
    },
    #[serde(rename = "EllipseGate")]
    Ellipse {
        #[serde(rename = "f1")]
        x_param: Arc<str>,
        #[serde(rename = "f2")]
        y_param: Arc<str>,
        left: Point,
        top: Point,
        right: Point,
        bottom: Point,
        #[serde(rename = "labelLoc")]
        label_position: Option<Point>,
    },
    #[serde(rename = "RangeGate")]
    Line {
        #[serde(rename = "f1")]
        x_param: Arc<str>,
        #[serde(rename = "f2")]
        y_param: Arc<str>,
        #[serde(rename = "f1Min")]
        f1min: f64,
        #[serde(rename = "f1Max")]
        f1max: f64,
        #[serde(rename = "labelLoc")]
        label_position: Option<Point>,
    },
    // Future-proofing for other gate types
    #[serde(other)]
    Unknown,
}

impl GateSerialized {
    pub fn to_drawable(&self, id: Arc<str>, name: Arc<str>, gate_mode: flow_gates::GateMode, rect_to_polygon: bool) -> anyhow::Result<Arc<dyn DrawableGate>> {
        match self {
            GateSerialized::Rectangle { x_param, y_param, min, max, label_position } => {
                let raw_coords = vec![
                    (min.x as f32, min.y as f32),
                    (max.x as f32, min.y as f32),
                    (max.x as f32, max.y as f32),
                    (min.x as f32, max.y as f32),
                ];
                let parameters = (x_param.clone(), y_param.clone());
                let geom = {
                    if !rect_to_polygon {
                        create_rectangle_geometry(raw_coords, x_param, y_param)?
                    } else {
                        create_polygon_geometry(raw_coords, x_param, y_param)?
                    }
                };
                let label_position = label_position.map(|p| LabelPosition {
                    offset_x: p.x as f32,
                    offset_y: p.y as f32,
                });
                let gate = flow_gates::Gate {
                    id: id.clone(),
                    name: name.to_string(),
                    geometry: geom,
                    mode: gate_mode,
                    parameters,
                    label_position,
                };
                
                Ok(Arc::new(RectangleGate::try_new(gate, true)?))
            },

            GateSerialized::Polygon { x_param, y_param, points, label_position } => {
                let parameters = (x_param.clone(), y_param.clone());
                let points: Vec<(f32, f32)> = points.iter().map(|&p| p.into()).collect();
                let geom = create_polygon_geometry(points, x_param, y_param)?;
                let label_position = label_position.map(|p| LabelPosition {
                    offset_x: p.x as f32,
                    offset_y: p.y as f32,
                });
                let gate = flow_gates::Gate {
                    id: id.clone(),
                    name: name.to_string(),
                    geometry: geom,
                    mode: gate_mode,
                    parameters,
                    label_position,
                };
                
                Ok(Arc::new(PolygonGate::try_new(gate, true)?))
                
            }
            GateSerialized::Ellipse { x_param, y_param, left, top, right, bottom, label_position } => {
                let parameters = (x_param.clone(), y_param.clone());
                let geom = create_omiq_ellipse_geometry((*left).into(), (*right).into(), (*top).into(), x_param, y_param)?;
                let label_position = label_position.map(|p| LabelPosition {
                    offset_x: p.x as f32,
                    offset_y: p.y as f32,
                });
                let gate = flow_gates::Gate {
                    id: id.clone(),
                    name: name.to_string(),
                    geometry: geom,
                    mode: gate_mode,
                    parameters,
                    label_position,
                };
                Ok(Arc::new(EllipseGate::try_new(gate, true)?))
            }
            GateSerialized::Line { x_param, y_param, f1min, f1max, label_position } => {
                let parameters = (x_param.clone(), y_param.clone());
                let max = (*f1max as f32, f32::MAX);
                let min = (*f1min as f32, f32::MIN);
                let coords = vec![min, max];
                let geom = flow_gates::geometry::create_rectangle_geometry(coords, x_param, y_param)?;
                let label_position = label_position.map(|p| LabelPosition {
                    offset_x: p.x as f32,
                    offset_y: p.y as f32,
                });
                let gate = flow_gates::Gate {
                    id: id.clone(),
                    name: name.to_string(),
                    geometry: geom,
                    mode: gate_mode,
                    parameters,
                    label_position,
                };
                Ok(Arc::new(LineGate::try_new(gate, 0f32, true)?))
            }
            GateSerialized::Unknown => todo!(),
        }
    }
}

#[derive(Deserialize, Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Point {
    #[serde(rename = "f1Val", default)]
    pub x: f64,
    #[serde(rename = "f2Val", default)]
    pub y: f64,
}

impl From<(f32, f32)> for Point {
    fn from(coords: (f32, f32)) -> Self {
        Point {
            x: coords.0 as f64,
            y: coords.1 as f64,
        }
    }
}

impl From<Point> for (f32, f32) {
    fn from(p: Point) -> Self {
        (p.x as f32, p.y as f32)
    }
}

impl From<Point> for (f64, f64) {
    fn from(p: Point) -> Self {
        (p.x, p.y)
    }
}

fn create_omiq_ellipse_geometry(
    left: (f64, f64),
    right: (f64, f64),
    top: (f64, f64),
    x_param: &str,
    y_param: &str,
) -> Result<GateGeometry, GateError> {
    // 1. Center is the average of opposite nodes
    // Using left/right is safe even if they are dragged across each other
    let cx = (left.0 + right.0) / 2.0;
    let cy = (left.1 + right.1) / 2.0;

    // 2. Define the Conjugate Vectors
    // These vectors represent the affine skew from the center to the nodes
    let dx1 = right.0 - cx;
    let dy1 = right.1 - cy;
    
    let dx2 = top.0 - cx;
    let dy2 = top.1 - cy;

    // 3. Matrix Multiplication (MM^T)
    // We construct the covariance matrix of these vectors to find the true axes
    let e = dx1 * dx1 + dx2 * dx2;
    let f = dx1 * dy1 + dx2 * dy2;
    let g = dy1 * dy1 + dy2 * dy2;

    // 4. Eigenvalue Decomposition
    // The eigenvalues of this matrix give us the squared lengths of the major and minor axes
    let trace = e + g;
    let diff = ((e - g).powi(2) + 4.0 * f * f).sqrt();

    let lambda1 = (trace + diff) / 2.0;
    let lambda2 = (trace - diff) / 2.0;

    // We take the square root to get the actual radii
    let radius_x = lambda1.max(0.0).sqrt();
    let radius_y = lambda2.max(0.0).sqrt();

    // 5. Calculate the True Rotation Angle
    // The angle is derived from the eigenvector corresponding to lambda1
    let angle = if f == 0.0 {
        if e > g { 0.0 } else { std::f64::consts::PI / 2.0 }
    } else {
        // Rust's atan2 takes (y, x)
        (lambda1 - e).atan2(f)
    };

    // 6. Final Node Setup for your Library
    let mut center_node = GateNode::new("ellipse_center");
    center_node.set_coordinate(Arc::from(x_param), cx as f32);
    center_node.set_coordinate(Arc::from(y_param), cy as f32);

    Ok(GateGeometry::Ellipse {
        center: center_node,
        radius_x: radius_x as f32,
        radius_y: radius_y as f32,
        angle: angle as f32,
    })
}