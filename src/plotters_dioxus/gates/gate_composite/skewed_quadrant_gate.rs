use flow_fcs::TransformType;

use flow_gates::{Gate, GateGeometry};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::{ops::RangeInclusive, sync::Arc};
use std::ops::Index;

use crate::plotters_dioxus::{
    gates::{
        gate_drag::PointDragData,
        gate_single::{polygon_gate::PolygonGate, rescale_helper_point},
        gate_traits::DrawableGate,
        gate_types::{DEFAULT_LINE, GateRenderShape, SELECTED_LINE, ShapeType},
    },
    plot_helpers::PlotMapper,
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(PartialEq, Clone)]
struct DataPoints {
    center: (f32, f32),
    left: (f32, f32),
    bottom: (f32, f32),
    right: (f32, f32),
    top: (f32, f32),
    x_data_range: RangeInclusive<f32>,
    y_data_range: RangeInclusive<f32>
}

impl DataPoints {
    fn new_from_click(cx: f32, cy: f32, plot_map: &PlotMapper) -> Self {
        let (xmin, xmax) = {let axis = plot_map.x_axis_min_max(); (*axis.start(), *axis.end())};
        let (ymin, ymax) = {let axis = plot_map.y_axis_min_max(); (*axis.start(), *axis.end())};
        let left = (xmin, cy);
        let right = (xmax, cy);
        let bottom = (cx, ymin);
        let top = (cx, ymax);
        println!("center: {:?} left: {:?}, bottom: {:?}, right: {:?}, top: {:?}", (cx, cy), left, bottom, right, top);
        Self::new_from_points((cx, cy), left, bottom, right, top, plot_map.x_data_min_max(), plot_map.y_data_min_max())
    }

    fn new_from_points(center: (f32, f32), left: (f32, f32), bottom: (f32, f32), right: (f32, f32), top: (f32, f32), x_data_range: RangeInclusive<f32>, y_data_range: RangeInclusive<f32>) -> Self {
        Self {
            center,
            left,
            bottom,
            right,
            top,
            x_data_range,
            y_data_range
        }
    }

    fn clone_for_swap_axis(&self, prev_axis_matched: bool) -> Self {
        if !prev_axis_matched {
            Self {
                center: (self.center.1, self.center.0),
                left: (self.bottom.1, self.bottom.0),
                right: (self.top.1, self.top.0),
                bottom: (self.left.1, self.left.0),
                top: (self.right.1, self.right.0),
                x_data_range: self.y_data_range.clone(),
                y_data_range: self.x_data_range.clone()
            }
        } else {
            Self {
                center: (self.center.1, self.center.0),
                left: (self.bottom.1, self.bottom.0),
                right: (self.top.1, self.top.0),
                bottom: (self.left.1, self.left.0),
                top: (self.right.1, self.right.0),
                x_data_range: self.y_data_range.clone(),
                y_data_range: self.x_data_range.clone()
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct SkewedQuadrantGate {
    gates: FxIndexMap<Arc<str>, PolygonGate>,
    id: Arc<str>,
    points: DataPoints,
    axis_matched: bool,
    parameters: (Arc<str>, Arc<str>),
}

impl SkewedQuadrantGate {
    pub fn try_new_from_raw_coord(
        plot_map: &PlotMapper,
        id: Arc<str>,
        click_loc_raw: (f32, f32),
        x_axis_param: Arc<str>,
        y_axis_param: Arc<str>,
    ) -> anyhow::Result<Self> {
        let (cx, cy) = plot_map.pixel_to_data(click_loc_raw.0, click_loc_raw.1, None, None);
        let points = DataPoints::new_from_click(cx, cy, plot_map);

        SkewedQuadrantGate::try_new_from_data_points(id, points, x_axis_param, y_axis_param, true, None)
    }

    fn try_new_from_data_points(
        id: Arc<str>,
        data_points: DataPoints,
        x_axis_param: Arc<str>,
        y_axis_param: Arc<str>,
        axis_matched: bool,
        subgate_ids: Option<Vec<Arc<str>>>,
    ) -> anyhow::Result<Self> {
        let mut gate_map = FxIndexMap::default();
        let parameters = (x_axis_param.clone(), y_axis_param.clone());
        let geos = create_skewed_quadrant_geos(data_points.clone(), &x_axis_param, &y_axis_param)?;
        let (
            id_bottom_left,
            id_bottom_right,
            id_top_right, 
            id_top_left, 
            id_bottom_left_arc,
            id_bottom_right_arc,
            id_top_right_arc,
            id_top_left_arc
         ) = if let Some(subgate_ids) = subgate_ids {
            (
                subgate_ids[0].to_string(),
                subgate_ids[1].to_string(),
                subgate_ids[2].to_string(),
                subgate_ids[3].to_string(),
                subgate_ids[0].clone(),
                subgate_ids[1].clone(),
                subgate_ids[2].clone(),
                subgate_ids[3].clone(),
            )

        } else {
            let (a, b, c, d) = (
                format!("{id}_BL"),
                format!("{id}_BR"),
                format!("{id}_TR"),
                format!("{id}_TL"),
        );

            let (astr, bstr, cstr, dstr) = (
                a.as_str(),
                b.as_str(),
                c.as_str(),
                d.as_str()
            );

            (a.clone(),
            b.clone(),
            c.clone(),
            d.clone(),
            Arc::from(astr),
            Arc::from(bstr),
            Arc::from(cstr),
            Arc::from(dstr))
        };
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

        let points = data_points;

        Ok(Self {
            gates: gate_map,
            id,
            points,
            axis_matched: axis_matched,
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
            let new_points = self.points.clone_for_swap_axis(self.axis_matched);
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
                points: self.points.clone(),
                axis_matched: self.axis_matched,
                parameters: self.parameters.clone(),
            })
        }
    }

    fn clone_with_point(&self, data_points: DataPoints) -> anyhow::Result<Self> {
        let (x_axis_param, y_axis_param) = self.parameters.clone();
        let subgate_bl_id = self.gates.index(0).get_id();
        let subgate_br_id = self.gates.index(1).get_id();
        let subgate_tr_id = self.gates.index(2).get_id();
        let subgate_tl_id = self.gates.index(3).get_id();

        let gate_ids = vec![subgate_bl_id, subgate_br_id, subgate_tr_id, subgate_tl_id];
        SkewedQuadrantGate::try_new_from_data_points(
            self.id.clone(),
            data_points,
            x_axis_param,
            y_axis_param,
            self.axis_matched,
            Some(gate_ids),
        )
    }

    pub fn get_subgate_map(&self) -> &FxIndexMap<Arc<str>, PolygonGate> {
        &self.gates
    }
}

impl super::super::gate_traits::DrawableGate for SkewedQuadrantGate {
    fn is_finalised(&self) -> bool {
        true
    }

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
        plot_map: &PlotMapper,
    ) -> Vec<GateRenderShape> {
        let (xmin, xmax) = {let axis = plot_map.x_axis_min_max(); (*axis.start(), *axis.end())};
        let (ymin, ymax) = {let axis = plot_map.y_axis_min_max(); (*axis.start(), *axis.end())};

        let (mut left, mut right, mut top, mut bottom, mut center) = (
            self.points.left,
            self.points.right,
            self.points.top,
            self.points.bottom,
            self.points.center,
        );

        if let Some(dd) = drag_point {
            match dd.point_index() {
                0 => center = dd.loc(),
                1 => left.1 = dd.loc().1,
                2 => bottom.0 = dd.loc().0,
                3 => right.1 = dd.loc().1,
                4 => top.0 = dd.loc().0,
                _ => unreachable!(),
            }
        };

        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };

        let main = {
            let left = GateRenderShape::Line {
                x1: xmin,
                y1: left.1,
                x2: center.0,
                y2: center.1,
                style,
                shape_type: ShapeType::UndraggableLine,
            };
            let right = GateRenderShape::Line {
                x1: center.0,
                y1: center.1,
                x2: xmax,
                y2: right.1,
                style,
                shape_type: ShapeType::UndraggableLine,
            };

            let bottom = GateRenderShape::Line {
                x1: bottom.0,
                y1: ymin,
                x2: center.0,
                y2: center.1,
                style,
                shape_type: ShapeType::UndraggableLine,
            };

            let top = GateRenderShape::Line {
                x1: center.0,
                y1: center.1,
                x2: top.0,
                y2: ymax,
                style,
                shape_type: ShapeType::UndraggableLine,
            };

            Some(vec![left, right, top, bottom])
        };

        let selected = if is_selected {
            let c = GateRenderShape::Circle {
                center,
                radius: 3.0,
                fill: "red",
                shape_type: ShapeType::UndraggablePoint(0),
            };
            let l = GateRenderShape::Circle {
                center: (xmin, left.1),
                radius: 3.0,
                fill: "red",
                shape_type: ShapeType::UndraggablePoint(1),
            };
            let b = GateRenderShape::Circle {
                center: (bottom.0, ymin),
                radius: 3.0,
                fill: "red",
                shape_type: ShapeType::UndraggablePoint(2),
            };
            let r = GateRenderShape::Circle {
                center: (xmax, right.1),
                radius: 3.0,
                fill: "red",
                shape_type: ShapeType::UndraggablePoint(3),
            };

            let t = GateRenderShape::Circle {
                center: (top.0, ymax),
                radius: 3.0,
                fill: "red",
                shape_type: ShapeType::UndraggablePoint(4),
            };

            Some(vec![c, l, b, r, t])
        } else {
            None
        };

        // crate::collate_vecs!(main, selected)

        let mut inner_gates = vec![];
        for (i, (_, shape)) in self.gates.iter().enumerate(){
            if i == 0{
                let res = shape.draw_self(false, None, plot_map);
                inner_gates.extend_from_slice(&res);
            }
            
            
        }

        crate::collate_vecs!(inner_gates, main, selected)
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
        plot_map: &PlotMapper,
    ) -> Option<f32> {
        let (xmin, xmax) = {let axis = plot_map.x_axis_min_max(); (*axis.start(), *axis.end())};
        let (ymin, ymax) = {let axis = plot_map.y_axis_min_max(); (*axis.start(), *axis.end())};

        let (left, bottom, right, top, center) = {
            (
                (xmin, self.points.left),
                (self.points.bottom, ymin),
                (xmax, self.points.right),
                (self.points.top, ymax),
                self.points.center,
            )
        };

        let mut closest = std::f32::INFINITY;

        if let Some(dis) = self.is_near_segment(point, left.1, center, tolerance) {
            closest = closest.min(dis);
        }
        if let Some(dis) = self.is_near_segment(point, center, right.1, tolerance) {
            closest = closest.min(dis);
        }
        if let Some(dis) = self.is_near_segment(point, center, bottom.0, tolerance) {
            closest = closest.min(dis);
        }
        if let Some(dis) = self.is_near_segment(point, center, top.0, tolerance) {
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
        Ok(Some(self.clone_with_gates(new_gate_map, swap_axis)))
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
        data_range: (f32, f32)
    ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
        let (x_param, _) = &self.parameters;
        let is_x = x_param == &param;
        let c = crate::plotters_dioxus::gates::gate_single::rescale_helper_point(
            self.points.center,
            &param,
            x_param,
            old_transform,
            new_transform,
        )?;

        let (l, b, r, t) = {
            (
                rescale_helper_point(self.points.left, &param, x_param, old_transform, new_transform)?,
                rescale_helper_point(self.points.bottom, &param, x_param, old_transform, new_transform)?,
                rescale_helper_point(self.points.right, &param, x_param, old_transform, new_transform)?,
                rescale_helper_point(self.points.top, &param, x_param, old_transform, new_transform)?,
            )
        };

        let new = DataPoints {
            center: c,
            left: l,
            bottom: b,
            right: r,
            top: t,
            x_data_range: if is_x {RangeInclusive::new(data_range.0, data_range.1)} else {self.points.x_data_range.clone()},
            y_data_range: if !is_x {RangeInclusive::new(data_range.0, data_range.1)} else {self.points.y_data_range.clone()}
        };

        Ok(Box::new(self.clone_with_point(new)?))
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
        point_index: usize,
        mapper: &PlotMapper,
    ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
        let (xmin, xmax) = {let axis = mapper.x_axis_min_max(); (*axis.start(), *axis.end())};
        let (ymin, ymax) = {let axis = mapper.y_axis_min_max(); (*axis.start(), *axis.end())};

        let (c, l, r, t, b) = (
            self.points.center,
            self.points.left,
            self.points.right,
            self.points.top,
            self.points.bottom,
        );

        let new = match point_index {
            0 => DataPoints {
                center: new_point,
                left: l,
                bottom: b,
                right: r,
                top: t,
                x_data_range: self.points.x_data_range.clone(),
                y_data_range: self.points.y_data_range.clone(),
            },
            1 => DataPoints {
                center: c,
                left: (xmin, new_point.1),
                bottom: b,
                right: r,
                top: t,
                x_data_range: self.points.x_data_range.clone(),
                y_data_range: self.points.y_data_range.clone(),
            },
            2 => DataPoints {
                center: c,
                left: l,
                bottom: (new_point.0, ymin),
                right: r,
                top: t,
                x_data_range: self.points.x_data_range.clone(),
                y_data_range: self.points.y_data_range.clone(),
            },
            3 => DataPoints {
                center: c,
                left: l,
                bottom: b,
                right: (xmax, new_point.1),
                top: t,
                x_data_range: self.points.x_data_range.clone(),
                y_data_range: self.points.y_data_range.clone(),
            },
            4 => DataPoints {
                center: c,
                left: l,
                bottom: b,
                right: r,
                top: (new_point.0, ymax),
                x_data_range: self.points.x_data_range.clone(),
                y_data_range: self.points.y_data_range.clone(),
            },
            _ => unreachable!(),
        };
        Ok(Box::new(self.clone_with_point(new)?))
    }

    fn replace_points(
        &self,
        _gate_drag_data: super::super::gate_drag::GateDragData,
    ) -> anyhow::Result<Option<Box<dyn super::super::gate_traits::DrawableGate>>> {
        Ok(None)
    }

    fn clone_box(&self) -> Box<dyn super::super::gate_traits::DrawableGate> {
        Box::new(self.clone())
    }

    fn get_gate_ref(&self, id: Option<&str>) -> Option<&Gate> {

        if let Some(id) = id {
            if let Some(g) = self.gates.get(id){
                g.get_gate_ref(None)
            } else {
                None
            }
        } else {
            None
        }
    }
    fn get_inner_gate_ids(&self) -> Vec<Arc<str>>{
        self.gates.keys().map(|k|k.clone()).collect()
    }
}

fn create_skewed_quadrant_geos(
    data_points: DataPoints,
    x_channel: &str,
    y_channel: &str,
) -> anyhow::Result<(GateGeometry, GateGeometry, GateGeometry, GateGeometry)> {
    let c = data_points.center;
    let b_x = data_points.bottom.0;
    let l_y = data_points.left.1;
    let t_x = data_points.top.0;
    let r_y = data_points.right.1;

    let x_min = data_points.left.0;
    let x_max = data_points.right.0;
    let y_min = data_points.bottom.1;
    let y_max = data_points.top.1;

    let (x_data_min, x_data_max) = (data_points.x_data_range.start().min(x_min), data_points.x_data_range.end().max(x_max));
    let (y_data_min, y_data_max) = (data_points.y_data_range.start().min(y_min), data_points.y_data_range.end().max(y_max));

    // Bottom-Left (BL)
    let bl = flow_gates::geometry::create_polygon_geometry(
        vec![
            (x_data_min, y_data_min), 
            (b_x, y_data_min),        // Vertical spoke projected to bottom
            c,                       // Center
            (x_data_min, l_y)         // Horizontal spoke projected to left
        ],
        x_channel, y_channel,
    ).map_err(|_| anyhow::anyhow!("failed bl"))?;

    // Bottom-Right (BR)
    let br = flow_gates::geometry::create_polygon_geometry(
        vec![
            (b_x, y_data_min), 
            (x_data_max, y_data_min), 
            (x_data_max, r_y),        // Right spoke projected to data max
            c
        ],
        x_channel, y_channel,
    ).map_err(|_| anyhow::anyhow!("failed br"))?;

    // Top-Right (TR)
    let tr = flow_gates::geometry::create_polygon_geometry(
        vec![
            c, 
            (x_data_max, r_y), 
            (x_data_max, y_data_max), 
            (t_x, y_data_max)         // Top spoke projected to data max
        ],
        x_channel, y_channel,
    ).map_err(|_| anyhow::anyhow!("failed tr"))?;

    // Top-Left (TL)
    let tl = flow_gates::geometry::create_polygon_geometry(
        vec![
            (x_data_min, l_y), 
            c, 
            (t_x, y_data_max), 
            (x_data_min, y_data_max)
        ],
        x_channel, y_channel,
    ).map_err(|_| anyhow::anyhow!("failed tl"))?;

    println!("gates created");
    Ok((bl, br, tr, tl))
}