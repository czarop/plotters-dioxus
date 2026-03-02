use flow_fcs::TransformType;

use flow_gates::{Gate, GateGeometry};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::sync::Arc;

use crate::plotters_dioxus::{
    gates::{
        gate_drag::PointDragData,
        gate_single::{polygon_gate::PolygonGate},
        gate_traits::DrawableGate,
        gate_types::{DEFAULT_LINE, GREY_LINE_DASHED, GateRenderShape, SELECTED_LINE, ShapeType},
    },
    plot_helpers::PlotMapper,
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(PartialEq, Clone)]
pub struct QuadrantGate {
    gates: FxIndexMap<Arc<str>, PolygonGate>,
    id: Arc<str>,
    points: (f32, f32),
    axis_matched: bool,
    parameters: (Arc<str>, Arc<str>),
}

impl QuadrantGate {
    pub fn try_new(
        plot_map: &PlotMapper,
        id: Arc<str>,
        click_loc: (f32, f32),
        x_axis_param: Arc<str>,
        y_axis_param: Arc<str>,
    ) -> anyhow::Result<Self> {
        let mut gate_map = FxIndexMap::default();
        let parameters = (x_axis_param.clone(), y_axis_param.clone());
        let click_data = plot_map.pixel_to_data(click_loc.0, click_loc.1, None, None);

        let geos = create_default_quadrant(plot_map, click_loc.0, click_loc.1, &x_axis_param, &y_axis_param)?;
        let id_top_left = format!("{id}_TL");
        let id_top_right = format!("{id}_TR");
        let id_bottom_left = format!("{id}_BL");
        let id_bottom_right = format!("{id}_BR");
        let id_top_left_arc: Arc<str> = Arc::from(id_top_left.as_str());
        let id_top_right_arc: Arc<str> = Arc::from(id_top_right.as_str());
        let id_bottom_left_arc: Arc<str> = Arc::from(id_bottom_left.as_str());
        let id_bottom_right_arc: Arc<str> = Arc::from(id_bottom_right.as_str());
        let gate_bottom_left = Gate {
            id: id_bottom_left_arc.clone(),
            name: id_bottom_left,
            geometry: geos.0,
            mode: flow_gates::GateMode::Global,
            parameters: parameters.clone(),
            label_position: None,
        };
        let gate_bottom_right = Gate {
            id: id_bottom_right_arc.clone(),
            name: id_bottom_right,
            geometry: geos.1,
            mode: flow_gates::GateMode::Global,
            parameters: parameters.clone(),
            label_position: None,
        };
        
        let gate_top_right = Gate {
            id: id_top_right_arc.clone(),
            name: id_top_right,
            geometry: geos.2,
            mode: flow_gates::GateMode::Global,
            parameters: parameters.clone(),
            label_position: None,
        };
        let gate_top_left = Gate {
            id: id_top_left_arc.clone(),
            name: id_top_left,
            geometry: geos.3,
            mode: flow_gates::GateMode::Global,
            parameters: parameters,
            label_position: None,
        };

        
        let lg_tl = PolygonGate::try_new(gate_top_left)?;
        let lg_tr = PolygonGate::try_new(gate_top_right)?;
        let lg_bl = PolygonGate::try_new(gate_bottom_left)?;
        let lg_br = PolygonGate::try_new(gate_bottom_right)?;
        // [bottom-left, bottom-right, top-right, top-left]
        gate_map.insert(id_bottom_left_arc, lg_bl);
        gate_map.insert(id_bottom_right_arc, lg_br);
        gate_map.insert(id_top_right_arc, lg_tr);
        gate_map.insert(id_top_left_arc, lg_tl);

        let points = click_data;

        Ok(Self {
            gates: gate_map,
            id,
            points,
            axis_matched: true,
            parameters: (x_axis_param, y_axis_param),
        })
    }

    fn clone_with_gates(
        &self,
        gates: FxIndexMap<Arc<str>, PolygonGate>,
        swap_axis: bool,
    ) -> Box<dyn DrawableGate> {
        if swap_axis {
            let new_parameters = (self.parameters.1.clone(), self.parameters.0.clone());
            let new_points = (self.points.1, self.points.0);
            Box::new(Self {
                gates,
                id: self.id.clone(),
                points: new_points,
                axis_matched: !self.axis_matched,
                parameters: new_parameters,
            })
        } else {
            Box::new(Self {
                gates,
                id: self.id.clone(),
                points: self.points,
                axis_matched: self.axis_matched,
                parameters: self.parameters.clone(),
            })
        }
    }

    fn clone_with_gates_and_loc(
        &self,
        gates: FxIndexMap<Arc<str>, PolygonGate>,
        cx: f32,
        cy: f32,
    ) -> Box<dyn DrawableGate> {
        let new_points = (cx, cy);
        Box::new(Self {
            gates,
            id: self.id.clone(),
            points: new_points,
            axis_matched: self.axis_matched,
            parameters: self.parameters.clone(),
        })
    }

    pub fn get_subgate_map(&self) -> &FxIndexMap<Arc<str>, PolygonGate> {
        &self.gates
    }
}

impl super::super::gate_traits::DrawableGate for QuadrantGate {
    fn get_points(&self) -> Vec<(f32, f32)> {
        return vec![self.points];
    }

    fn is_finalised(&self) -> bool {
        true
    }

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
        plot_map: &PlotMapper,
    ) -> Vec<GateRenderShape> {
        let (min, max) = {
            let (xmin, xmax) = plot_map.x_axis_min_max();
            let (ymin, ymax) = plot_map.y_axis_min_max();
            ((xmin, ymin), (xmax, ymax))
        };
        let mut center = self.points;

        if let Some(dd) = drag_point {
            if self.axis_matched {
                center = (dd.loc().0, center.1);
            } else {
                center = (center.0, dd.loc().1);
            }
        };

        let center_tab_height = {
            if self.axis_matched {
                (max.1 - min.1) * 0.02
            } else {
                (max.0 - min.0) * 0.02
            }
        };
        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };

        let main = {
            let left = GateRenderShape::Line {
                x1: if self.axis_matched { min.0 } else { center.0 },
                y1: if self.axis_matched { center.1 } else { min.1 },
                x2: center.0,
                y2: center.1,
                style,
                shape_type: ShapeType::CompositeGate(self.id.clone(), self.axis_matched),
            };

            let right = GateRenderShape::Line {
                x1: center.0,
                y1: center.1,
                x2: if self.axis_matched { max.0 } else { center.0 },
                y2: if self.axis_matched { center.1 } else { max.1 },
                style,
                shape_type: ShapeType::CompositeGate(self.id.clone(), self.axis_matched),
            };

            let center_tab = GateRenderShape::Line {
                x1: if self.axis_matched {
                    center.0
                } else {
                    center.0 - center_tab_height
                },
                y1: if self.axis_matched {
                    center.1 - center_tab_height
                } else {
                    center.1
                },
                x2: if self.axis_matched {
                    center.0
                } else {
                    center.0 + center_tab_height
                },
                y2: if self.axis_matched {
                    center.1 + center_tab_height
                } else {
                    center.1
                },
                style,
                shape_type: ShapeType::CompositeGate(self.id.clone(), self.axis_matched),
            };

            Some(vec![left, right, center_tab])
        };

        let selected = if is_selected {
            let p = GateRenderShape::Circle {
                center,
                radius: 3.0,
                fill: "red",
                shape_type: ShapeType::CompositePoint(0, self.axis_matched),
            };
            let line = GateRenderShape::Line {
                x1: if self.axis_matched { center.0 } else { min.0 },
                y1: if self.axis_matched { min.1 } else { center.1 },
                x2: if self.axis_matched { center.0 } else { max.0 },
                y2: if self.axis_matched { max.1 } else { center.1 },
                style: &GREY_LINE_DASHED,
                shape_type: ShapeType::LineGuide,
            };

            Some(vec![line, p])
        } else {
            None
        };

        crate::collate_vecs!(main, selected)
    }

    fn is_composite(&self) -> bool {
        true
    }

    fn get_id(&self) -> Arc<str> {
        self.id.clone()
    }

    fn get_params(&self) -> (Arc<str>, Arc<str>) {
        self.parameters.clone()
    }

    fn is_point_on_perimeter(
        &self,
        point: (f32, f32),
        tolerance: (f32, f32),
        mapper: &PlotMapper,
    ) -> Option<f32> {
        let (xmin, xmax) = mapper.x_axis_min_max();
        let (ymin, ymax) = mapper.y_axis_min_max();
        let (cx, cy) = self.points;

        let mut closest = std::f32::INFINITY;

        if let Some(dis) = self.is_near_segment(point, (xmin, cy), (xmax, cy), tolerance) {
            closest = closest.min(dis);
        }
        if let Some(dis) = self.is_near_segment(point, (cx, ymin), (cx, ymax), tolerance) {
            closest = closest.min(dis);
        }
        
        if closest == std::f32::INFINITY {
            return None;
        } else {
            return Some(closest);
        }
    }

    fn match_to_plot_axis(
        &self,
        plot_x_param: &str,
        plot_y_param: &str,
    ) -> anyhow::Result<Option<Box<dyn super::super::gate_traits::DrawableGate>>> {
        let mut new_gate_map = FxIndexMap::default();
        let mut swap_axis = false;
        for gate in self.gates.values() {
            match gate.clone_polygon_for_axis_swap(plot_x_param, plot_y_param) {
                Ok(Some(g)) => {
                    swap_axis = true;
                    new_gate_map.insert(gate.get_id(), g);
                }
                Ok(None) => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(Some(self.clone_with_gates(
            new_gate_map,
            swap_axis,
        )))
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
        let mut new_gate_map = FxIndexMap::default();

        for gate in self.gates.values() {
            match gate.clone_polygon_for_rescaled_axis(param.clone(), old_transform, new_transform) {
                Ok(g) => {
                    new_gate_map.insert(gate.get_id(), g);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(self.clone_with_gates(new_gate_map, false))
    }

    fn rotate_gate(
        &self,
        _mouse_position: (f32, f32),
    ) -> anyhow::Result<Option<Box<dyn super::super::gate_traits::DrawableGate>>> {
        Ok(None)
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        _point_index: usize,
        mapper: &PlotMapper,
    ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
        let mut new_gate_map = FxIndexMap::default();
        let new_cx = new_point.0;
        let new_cy = new_point.1;
        for (i, (id, gate)) in self.gates.iter().enumerate() {
            // let p_index = if i == 0 { 1 } else { 0 };
            // let old_p = gate.get_points()[p_index];
            let p_index = if i == 0 { 1 } else { 0 };
            let old_p = gate.get_points()[p_index];
            println!("old points: {:?}", gate.get_points());
            let target_p = if self.axis_matched {
                (new_cx, old_p.1)
            } else {
                (old_p.0, new_cy)
            };
            println!(
                "old point {:?}, new point {:?} inserted at {}",
                old_p, target_p, p_index
            );
            let new_gate = gate.clone_polygon_for_new_point(target_p, p_index, mapper)?;
            println!("new points: {:?}", gate.get_points());
            new_gate_map.insert(id.clone(), new_gate);
        }
        if self.axis_matched {
            Ok(self.clone_with_gates_and_loc(new_gate_map, new_point.0, self.points.1))
        } else {
            Ok(self.clone_with_gates_and_loc(new_gate_map, self.points.0, new_point.1))
        }
    }

    fn replace_points(
        &self,
        gate_drag_data: super::super::gate_drag::GateDragData,
    ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
        let (x_offset, y_offset) = gate_drag_data.offset();
        let mut new_self = self.clone();
        let mut new_points = self.points.clone();
        if self.axis_matched {
            new_points.1 = self.points.1 - y_offset;
        } else {
            new_points.0 = self.points.0 - x_offset;
        }

        new_self.points = new_points;
        Ok(Box::new(new_self))
    }

    fn clone_box(&self) -> Box<dyn super::super::gate_traits::DrawableGate> {
        Box::new(self.clone())
    }
}

fn create_default_quadrant(
    plot_map: &PlotMapper,
    cx_raw: f32,
    cy_raw: f32,
    x_channel: &str,
    y_channel: &str,
) -> anyhow::Result<(GateGeometry, GateGeometry, GateGeometry, GateGeometry)> {
    let center = plot_map.pixel_to_data(cx_raw, cy_raw, None, None);
    
    let bl1 = (f32::MIN, f32::MIN);
    let bl2 = (center.0, f32::MIN);
    let bl3 = center;
    let bl4 = (f32::MIN, center.1);
    let bl = flow_gates::geometry::create_polygon_geometry(vec![bl1, bl2, bl3, bl4], x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

    let br1 = (center.0, f32::MIN);
    let br2 = (f32::MAX, f32::MIN);
    let br3 = (f32::MAX, center.1);
    let br4 = center;
    let br = flow_gates::geometry::create_polygon_geometry(vec![br1, br2, br3, br4], x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

    let tr1 = center;
    let tr2 = br3;
    let tr3 = (f32::MAX, f32::MAX);
    let tr4 = (center.0, f32::MAX);
    let tr = flow_gates::geometry::create_polygon_geometry(vec![tr1, tr2, tr3, tr4], x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

    let tl1 = (f32::MIN, center.1);
    let tl2 = center;
    let tl3 = (center.0, f32::MAX);
    let tl4 = (f32::MIN, f32::MAX);
    let tl = flow_gates::geometry::create_polygon_geometry(vec![tl1, tl2, tl3, tl4], x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

    
    Ok((bl, br, tr, tl))
}