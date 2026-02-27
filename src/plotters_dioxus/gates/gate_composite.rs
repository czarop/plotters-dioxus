use anyhow::anyhow;
use flow_fcs::TransformType;

use flow_gates::{Gate, GateGeometry};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::sync::Arc;

use crate::plotters_dioxus::{
    gates::{
        gate_drag::PointDragData,
        gate_draw_helpers::{self},
        gate_single::{LineGate, draw_circles_for_selected_gate},
        gate_traits::DrawableGate,
        gate_types::{DEFAULT_LINE, GREY_LINE_DASHED, GateRenderShape, SELECTED_LINE, ShapeType},
    },
    plot_helpers::PlotMapper,
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(PartialEq, Clone, Debug)]
struct BisectorPoints {
    xmin: f32,
    xmax: f32,
    ymin: f32,
    ymax: f32,
    cx: f32,
    cy: f32,
}

#[derive(PartialEq, Clone)]
pub struct BisectorGate {
    gates: FxIndexMap<Arc<str>, LineGate>,
    id: Arc<str>,
    // center_point: (f32, f32),
    points: BisectorPoints,
    axis_matched: bool,
    parameters: (Arc<str>, Arc<str>),
}

impl BisectorGate {
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
        let (xmin, xmax) = plot_map.x_axis_min_max();
        let (ymin, ymax) = plot_map.y_axis_min_max();
        let geos = gate_draw_helpers::line::create_default_bisector(
            plot_map,
            click_loc.0,
            &x_axis_param,
            &y_axis_param,
        )?;
        let id_left = format!("{id}_L");
        let id_right = format!("{id}_R");
        let id_left_arc: Arc<str> = Arc::from(id_left.as_str());
        let id_right_arc: Arc<str> = Arc::from(id_right.as_str());
        let gate_left = Gate {
            id: id_left_arc.clone(),
            name: id_left,
            geometry: geos.0,
            mode: flow_gates::GateMode::Global,
            parameters: parameters.clone(),
            label_position: None,
        };
        let gate_right = Gate {
            id: id_right_arc.clone(),
            name: id_right,
            geometry: geos.1,
            mode: flow_gates::GateMode::Global,
            parameters,
            label_position: None,
        };
        let lg_l = LineGate::try_new(gate_left, click_data.1)?;
        let lg_r = LineGate::try_new(gate_right, click_data.1)?;
        gate_map.insert(id_left_arc, lg_l);
        gate_map.insert(id_right_arc, lg_r);

        let points = BisectorPoints {
            xmin,
            xmax,
            ymin,
            ymax,
            cx: click_data.0,
            cy: click_data.1,
        };

        Ok(Self {
            gates: gate_map,
            id,
            points,
            axis_matched: true,
            parameters: (x_axis_param, y_axis_param),
        })
    }

    fn clone_with_gates(&self, gates: FxIndexMap<Arc<str>, LineGate>) -> Box<dyn DrawableGate> {
        self.clone_with_gates_and_height(gates, self.points.cx, self.points.cy)
    }

    fn clone_with_gates_and_height(
        &self,
        gates: FxIndexMap<Arc<str>, LineGate>,
        cx: f32,
        cy: f32,
    ) -> Box<dyn DrawableGate> {
        let (xmin, xmax, ymin, ymax) = {
            let min;
            let max;
            if let Some((_, left_gate)) = gates.get_index(0) {
                min = left_gate.get_points()[0];
            } else {
                unreachable!()
            }
            if let Some((_, right_gate)) = gates.get_index(1) {
                max = right_gate.get_points()[2];
            } else {
                unreachable!()
            }
            (min.0, max.0, min.1, max.1)
        };
        println!("{} {} {} {} {} {}", xmin, ymin, xmax, ymax, cx, cy);
        let new_points = BisectorPoints {
            xmin,
            xmax,
            ymin,
            ymax,
            cx,
            cy,
        };

        Box::new(Self {
            gates,
            id: self.id.clone(),
            points: new_points,
            axis_matched: self.axis_matched,
            parameters: self.parameters.clone(),
        })
    }

    pub fn get_subgate_map(&self) -> &FxIndexMap<Arc<str>, LineGate> {
        &self.gates
    }
}

impl super::gate_traits::DrawableGate for BisectorGate {
    fn get_points(&self) -> Vec<(f32, f32)> {
        let p = &self.points;
        return vec![(p.xmin, p.ymin), (p.cx, p.cy), (p.xmax, p.ymax)];
    }

    fn is_finalised(&self) -> bool {
        true
    }

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
    ) -> Vec<GateRenderShape> {
        let points = self.get_points();
        let (min, center, max) = (points[0], points[1], points[2]);
        let center_tab_height = (max.1 - min.1) * 0.02;
        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };

        let main = {
            let left = GateRenderShape::Line {
                x1: min.0,
                y1: center.1,
                x2: center.0,
                y2: center.1,
                style,
                shape_type: ShapeType::Gate(self.id.clone()),
            };

            let right = GateRenderShape::Line {
                x1: center.0,
                y1: center.1,
                x2: max.0,
                y2: center.1,
                style,
                shape_type: ShapeType::Gate(self.id.clone()),
            };

            let center_tab = GateRenderShape::Line {
                x1: center.0,
                y1: center.1 - center_tab_height,
                x2: center.0,
                y2: center.1 + center_tab_height,
                style,
                shape_type: ShapeType::Gate(self.id.clone()),
            };

            Some(vec![left, right, center_tab])
        };

        let ghost = {};

        let selected = if is_selected {
            let mut p = draw_circles_for_selected_gate(&[center], 0);
            if drag_point.is_none() {
                let line = GateRenderShape::Line {
                    x1: center.0,
                    y1: min.1,
                    x2: center.0,
                    y2: max.1,
                    style: &GREY_LINE_DASHED,
                    shape_type: ShapeType::Gate(self.id.clone()),
                };
                p.push(line);
            }
            Some(p)
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

    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        let min = (self.points.xmin, self.points.cy);
        let max = (self.points.xmax, self.points.cy);
        if let Some(dis) = self.is_near_segment(point, min, max, tolerance) {
            return Some(dis);
        }
        None
    }

    fn match_to_plot_axis(
        &self,
        plot_x_param: &str,
        plot_y_param: &str,
    ) -> anyhow::Result<Option<Box<dyn super::gate_traits::DrawableGate>>> {
        let mut new_gate_map = FxIndexMap::default();

        for gate in self.gates.values() {
            match gate.clone_line_for_axis_swap(plot_x_param, plot_y_param) {
                Ok(Some(g)) => {
                    new_gate_map.insert(gate.get_id(), g);
                }
                Ok(None) => {
                    return Ok(None);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(Some(self.clone_with_gates(new_gate_map)))
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<Box<dyn super::gate_traits::DrawableGate>> {
        let mut new_gate_map = FxIndexMap::default();

        for gate in self.gates.values() {
            match gate.clone_line_for_rescaled_axis(param.clone(), old_transform, new_transform) {
                Ok(g) => {
                    new_gate_map.insert(gate.get_id(), g);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(self.clone_with_gates(new_gate_map))
    }

    fn rotate_gate(
        &self,
        _mouse_position: (f32, f32),
    ) -> anyhow::Result<Option<Box<dyn super::gate_traits::DrawableGate>>> {
        Ok(None)
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        _point_index: usize,
    ) -> anyhow::Result<Box<dyn super::gate_traits::DrawableGate>> {
        let mut new_gate_map = FxIndexMap::default();
        // let delta = (new_point.0 - self.points.cx, new_point.1 - self.points.cy);

        // for (i, (id, gate)) in self.gates.iter().enumerate(){
        //     let p_index = if i == 0 {1} else if i == 1 {0} else {return Err(anyhow!("invalid point"))};
        //     let old_point = gate.get_points()[p_index];
        //     println!("{:?}", old_point);
        //     let new_p = (old_point.0 - delta.0, old_point.1);
        //     println!("{:?}", new_point);
        //     let new_gate = gate.clone_line_for_new_point(new_p, p_index)?;
        //     new_gate_map.insert(id.clone(), new_gate);
        // }

        let new_cx = new_point.0;
        for (i, (id, gate)) in self.gates.iter().enumerate() {
            // Find which point of the sub-line connects to the center
            let p_index = if i == 0 { 1 } else { 0 };

            // Instead of calculating a delta, just tell the sub-gate:
            // "Your connecting point is now exactly at the center X"
            let old_p = gate.get_points()[p_index];
            let target_p = (new_cx, old_p.1); // Keep the Y constant

            let new_gate = gate.clone_line_for_new_point(target_p, p_index)?;
            new_gate_map.insert(id.clone(), new_gate);
        }

        Ok(self.clone_with_gates_and_height(new_gate_map, new_point.0, self.points.cy))
    }

    fn replace_points(
        &self,
        gate_drag_data: super::gate_drag::GateDragData,
    ) -> anyhow::Result<Box<dyn super::gate_traits::DrawableGate>> {
        let (_, y_offset) = gate_drag_data.offset();
        let mut new_self = self.clone();
        let mut new_points = self.points.clone();
        println!("{:#?}", &new_points);
        new_points.cy = self.points.cy - y_offset;
        new_self.points = new_points;
        println!("{:#?}", &new_self.points);
        Ok(Box::new(new_self))
    }

    fn clone_box(&self) -> Box<dyn super::gate_traits::DrawableGate> {
        Box::new(self.clone())
    }
}
