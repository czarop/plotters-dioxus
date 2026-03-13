use flow_fcs::TransformType;

use flow_gates::{Gate, GateGeometry};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::{
    ops::{Index, RangeInclusive},
    sync::Arc,
};

// use crate::plotters_dioxus::{
//     gates::{
//         gate_drag::PointDragData,
//         gate_single::polygon_gate::PolygonGate,
//         gate_traits::DrawableGate,
//         gate_types::{DEFAULT_LINE, GateRenderShape, SELECTED_LINE, ShapeType},
//     },
//     plot_helpers::PlotMapper,
// };

// type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

// #[derive(PartialEq, Clone)]
// pub struct QuadrantGate {
//     gates: FxIndexMap<Arc<str>, PolygonGate>,
//     id: Arc<str>,
//     points: (f32, f32),
//     x_data_range: RangeInclusive<f32>,
//     y_data_range: RangeInclusive<f32>,
//     x_axis_range: RangeInclusive<f32>,
//     y_axis_range: RangeInclusive<f32>,
//     axis_matched: bool,
//     parameters: (Arc<str>, Arc<str>),
// }

// impl QuadrantGate {
//     pub fn try_new_from_raw_coord(
//         plot_map: &PlotMapper,
//         id: Arc<str>,
//         click_loc_raw: (f32, f32),
//         x_axis_param: Arc<str>,
//         y_axis_param: Arc<str>,
//     ) -> anyhow::Result<Self> {
//         let click_loc_data = plot_map.pixel_to_data(click_loc_raw.0, click_loc_raw.1, None, None);

//         QuadrantGate::try_new_from_data_coord(
//             id,
//             click_loc_data,
//             x_axis_param,
//             y_axis_param,
//             true,
//             None,
//             plot_map.x_data_min_max(),
//             plot_map.y_data_min_max(),
//             plot_map.x_axis_min_max(),
//             plot_map.y_axis_min_max(),
//         )
//     }

//     fn try_new_from_data_coord(
//         id: Arc<str>,
//         click_loc_data: (f32, f32),
//         x_axis_param: Arc<str>,
//         y_axis_param: Arc<str>,
//         axis_matched: bool,
//         subgate_ids: Option<Vec<Arc<str>>>,
//         x_data_range: RangeInclusive<f32>,
//         y_data_range: RangeInclusive<f32>,
//         x_axis_range: RangeInclusive<f32>,
//         y_axis_range: RangeInclusive<f32>,
//     ) -> anyhow::Result<Self> {
//         let mut gate_map = FxIndexMap::default();
//         let parameters = (x_axis_param.clone(), y_axis_param.clone());

//         let geos = create_quadrant_geos(
//             click_loc_data.0,
//             click_loc_data.1,
//             &x_axis_param,
//             &y_axis_param,
//             &x_data_range,
//             &y_data_range,
//             &x_axis_range,
//             &y_axis_range,
//         )?;

//         let (
//             id_bottom_left,
//             id_bottom_right,
//             id_top_right,
//             id_top_left,
//             id_bottom_left_arc,
//             id_bottom_right_arc,
//             id_top_right_arc,
//             id_top_left_arc,
//         ) = if let Some(subgate_ids) = subgate_ids {
//             (
//                 subgate_ids[0].to_string(),
//                 subgate_ids[1].to_string(),
//                 subgate_ids[2].to_string(),
//                 subgate_ids[3].to_string(),
//                 subgate_ids[0].clone(),
//                 subgate_ids[1].clone(),
//                 subgate_ids[2].clone(),
//                 subgate_ids[3].clone(),
//             )
//         } else {
//             let (a, b, c, d) = (
//                 format!("{id}_BL"),
//                 format!("{id}_BR"),
//                 format!("{id}_TR"),
//                 format!("{id}_TL"),
//             );

//             let (astr, bstr, cstr, dstr) = (a.as_str(), b.as_str(), c.as_str(), d.as_str());

//             (
//                 a.clone(),
//                 b.clone(),
//                 c.clone(),
//                 d.clone(),
//                 Arc::from(astr),
//                 Arc::from(bstr),
//                 Arc::from(cstr),
//                 Arc::from(dstr),
//             )
//         };

//         let gate_bottom_left = Gate {
//             id: id_bottom_left_arc.clone(),
//             name: id_bottom_left,
//             geometry: geos.0,
//             mode: flow_gates::GateMode::Global,
//             parameters: parameters.clone(),
//             label_position: None,
//         };
//         let gate_bottom_right = Gate {
//             id: id_bottom_right_arc.clone(),
//             name: id_bottom_right,
//             geometry: geos.1,
//             mode: flow_gates::GateMode::Global,
//             parameters: parameters.clone(),
//             label_position: None,
//         };

//         let gate_top_right = Gate {
//             id: id_top_right_arc.clone(),
//             name: id_top_right,
//             geometry: geos.2,
//             mode: flow_gates::GateMode::Global,
//             parameters: parameters.clone(),
//             label_position: None,
//         };
//         let gate_top_left = Gate {
//             id: id_top_left_arc.clone(),
//             name: id_top_left,
//             geometry: geos.3,
//             mode: flow_gates::GateMode::Global,
//             parameters: parameters,
//             label_position: None,
//         };

//         let lg_tl = PolygonGate::try_new(gate_top_left)?;
//         let lg_tr = PolygonGate::try_new(gate_top_right)?;
//         let lg_bl = PolygonGate::try_new(gate_bottom_left)?;
//         let lg_br = PolygonGate::try_new(gate_bottom_right)?;
//         // [bottom-left, bottom-right, top-right, top-left]
//         gate_map.insert(id_bottom_left_arc, lg_bl);
//         gate_map.insert(id_bottom_right_arc, lg_br);
//         gate_map.insert(id_top_right_arc, lg_tr);
//         gate_map.insert(id_top_left_arc, lg_tl);

//         let points = click_loc_data;

//         Ok(Self {
//             gates: gate_map,
//             id,
//             points,
//             x_data_range,
//             y_data_range,
//             x_axis_range,
//             y_axis_range,
//             axis_matched: axis_matched,
//             parameters: (x_axis_param, y_axis_param),
//         })
//     }

//     fn clone_with_gates(
//         &self,
//         gates: FxIndexMap<Arc<str>, PolygonGate>,
//         swap_axis: bool,
//     ) -> Box<dyn DrawableGate> {
//         if swap_axis {
//             let new_parameters = (self.parameters.1.clone(), self.parameters.0.clone());
//             let new_points = (self.points.1, self.points.0);
//             Box::new(Self {
//                 gates,
//                 id: self.id.clone(),
//                 points: new_points,
//                 x_data_range: self.y_data_range.clone(),
//                 y_data_range: self.x_data_range.clone(),
//                 x_axis_range: self.y_axis_range.clone(),
//                 y_axis_range: self.x_axis_range.clone(),
//                 axis_matched: !self.axis_matched,
//                 parameters: new_parameters,
//             })
//         } else {
//             Box::new(Self {
//                 gates,
//                 id: self.id.clone(),
//                 points: self.points,
//                 x_data_range: self.x_data_range.clone(),
//                 y_data_range: self.y_data_range.clone(),
//                 x_axis_range: self.x_axis_range.clone(),
//                 y_axis_range: self.y_axis_range.clone(),
//                 axis_matched: self.axis_matched,
//                 parameters: self.parameters.clone(),
//             })
//         }
//     }

//     fn clone_with_point(
//         &self,
//         cx: f32,
//         cy: f32,
//         x_data_range: RangeInclusive<f32>,
//         y_data_range: RangeInclusive<f32>,
//         x_axis_range: RangeInclusive<f32>,
//         y_axis_range: RangeInclusive<f32>,
//     ) -> anyhow::Result<Self> {
//         let (x_axis_param, y_axis_param) = self.parameters.clone();

//         let subgate_bl_id = self.gates.index(0).get_id();
//         let subgate_br_id = self.gates.index(1).get_id();
//         let subgate_tr_id = self.gates.index(2).get_id();
//         let subgate_tl_id = self.gates.index(3).get_id();

//         let gate_ids = vec![subgate_bl_id, subgate_br_id, subgate_tr_id, subgate_tl_id];

//         QuadrantGate::try_new_from_data_coord(
//             self.id.clone(),
//             (cx, cy),
//             x_axis_param,
//             y_axis_param,
//             self.axis_matched,
//             Some(gate_ids),
//             x_data_range,
//             y_data_range,
//             x_axis_range,
//             y_axis_range,
//         )
//     }

//     pub fn get_subgate_map(&self) -> &FxIndexMap<Arc<str>, PolygonGate> {
//         &self.gates
//     }
// }

// impl super::super::gate_traits::DrawableGate for QuadrantGate {
//     fn is_finalised(&self) -> bool {
//         true
//     }

//     fn draw_self(
//         &self,
//         is_selected: bool,
//         drag_point: Option<PointDragData>,
//         plot_map: &PlotMapper,
//     ) -> Vec<GateRenderShape> {
//         let (min, max) = {
//             let (xmin, xmax) = {
//                 let axis = plot_map.x_axis_min_max();
//                 (*axis.start(), *axis.end())
//             };
//             let (ymin, ymax) = {
//                 let axis = plot_map.y_axis_min_max();
//                 (*axis.start(), *axis.end())
//             };
//             ((xmin, ymin), (xmax, ymax))
//         };
//         let mut center = self.points;

//         if let Some(dd) = drag_point {
//             center = dd.loc();
//         };

//         let style = if is_selected {
//             &SELECTED_LINE
//         } else {
//             &DEFAULT_LINE
//         };

//         let main = {
//             let horizontal = GateRenderShape::Line {
//                 x1: min.0,
//                 y1: center.1,
//                 x2: max.0,
//                 y2: center.1,
//                 style,
//                 shape_type: ShapeType::UndraggableLine,
//             };

//             let vertical = GateRenderShape::Line {
//                 x1: center.0,
//                 y1: min.1,
//                 x2: center.0,
//                 y2: max.1,
//                 style,
//                 shape_type: ShapeType::UndraggableLine,
//             };

//             Some(vec![horizontal, vertical])
//         };

//         let selected = if is_selected {
//             let p = GateRenderShape::Circle {
//                 center,
//                 radius: 3.0,
//                 fill: "red",
//                 shape_type: ShapeType::UndraggablePoint(0),
//             };

//             Some(vec![p])
//         } else {
//             None
//         };

//         crate::collate_vecs!(main, selected)
//     }

//     fn is_composite(&self) -> bool {
//         true
//     }

//     fn get_id(&self) -> Arc<str> {
//         self.id.clone()
//     }

//     fn get_params(&self) -> (Arc<str>, Arc<str>) {
//         self.parameters.clone()
//     }

//     fn is_point_on_perimeter(
//         &self,
//         point: (f32, f32),
//         tolerance: (f32, f32),
//         plot_map: &PlotMapper,
//     ) -> Option<f32> {
//         let (xmin, xmax) = {
//             let axis = plot_map.x_axis_min_max();
//             (*axis.start(), *axis.end())
//         };
//         let (ymin, ymax) = {
//             let axis = plot_map.y_axis_min_max();
//             (*axis.start(), *axis.end())
//         };
//         let (cx, cy) = self.points;

//         let mut closest = std::f32::INFINITY;

//         if let Some(dis) = self.is_near_segment(point, (xmin, cy), (xmax, cy), tolerance) {
//             closest = closest.min(dis);
//         }
//         if let Some(dis) = self.is_near_segment(point, (cx, ymin), (cx, ymax), tolerance) {
//             closest = closest.min(dis);
//         }

//         if closest == std::f32::INFINITY {
//             return None;
//         } else {
//             return Some(closest);
//         }
//     }

//     fn match_to_plot_axis(
//         &self,
//         plot_x_param: &str,
//         plot_y_param: &str,
//     ) -> anyhow::Result<Option<Box<dyn super::super::gate_traits::DrawableGate>>> {
//         let mut new_gate_map = FxIndexMap::default();
//         let mut swap_axis = false;
//         for gate in self.gates.values() {
//             match gate.clone_polygon_for_axis_swap(plot_x_param, plot_y_param) {
//                 Ok(Some(g)) => {
//                     swap_axis = true;
//                     new_gate_map.insert(gate.get_id(), g);
//                 }
//                 Ok(None) => {
//                     return Ok(None);
//                 }
//                 Err(e) => return Err(e),
//             }
//         }
//         Ok(Some(self.clone_with_gates(new_gate_map, swap_axis)))
//     }

//     fn recalculate_gate_for_rescaled_axis(
//         &self,
//         param: std::sync::Arc<str>,
//         old_transform: &TransformType,
//         new_transform: &TransformType,
//         data_range: (f32, f32),
//         axis_range: (f32, f32),
//     ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
//         let (x_param, _) = &self.parameters;
//         let is_x = x_param == &param;
//         let (cx, cy) = crate::plotters_dioxus::gates::gate_single::rescale_helper_point(
//             self.points,
//             &param,
//             x_param,
//             old_transform,
//             new_transform,
//         )?;

//         Ok(Box::new(self.clone_with_point(
//             cx,
//             cy,
//             if is_x {
//                 RangeInclusive::new(data_range.0, data_range.1)
//             } else {
//                 self.x_data_range.clone()
//             },
//             if !is_x {
//                 RangeInclusive::new(data_range.0, data_range.1)
//             } else {
//                 self.y_data_range.clone()
//             },
//             if is_x {
//                 RangeInclusive::new(axis_range.0, axis_range.1)
//             } else {
//                 self.x_axis_range.clone()
//             },
//             if !is_x {
//                 RangeInclusive::new(axis_range.0, axis_range.1)
//             } else {
//                 self.y_axis_range.clone()
//             },
//         )?))
//     }

//     fn rotate_gate(
//         &self,
//         _mouse_position: (f32, f32),
//     ) -> anyhow::Result<Option<Box<dyn super::super::gate_traits::DrawableGate>>> {
//         Ok(None)
//     }

//     fn replace_point(
//         &self,
//         new_point: (f32, f32),
//         _point_index: usize,
//         _mapper: &PlotMapper,
//     ) -> anyhow::Result<Box<dyn super::super::gate_traits::DrawableGate>> {
//         Ok(Box::new(self.clone_with_point(
//             new_point.0,
//             new_point.1,
//             self.x_data_range.clone(),
//             self.y_data_range.clone(),
//             self.x_axis_range.clone(),
//             self.y_axis_range.clone(),
//         )?))
//     }

//     fn replace_points(
//         &self,
//         _gate_drag_data: super::super::gate_drag::GateDragData,
//     ) -> anyhow::Result<Option<Box<dyn super::super::gate_traits::DrawableGate>>> {
//         Ok(None)
//     }

//     fn clone_box(&self) -> Box<dyn super::super::gate_traits::DrawableGate> {
//         Box::new(self.clone())
//     }

//     fn get_gate_ref(&self, id: Option<&str>) -> Option<&Gate> {
//         if let Some(id) = id {
//             if let Some(g) = self.gates.get(id) {
//                 g.get_gate_ref(None)
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
//     fn get_inner_gate_ids(&self) -> Vec<Arc<str>> {
//         self.gates.keys().map(|k| k.clone()).collect()
//     }
// }

// fn create_quadrant_geos(
//     cx: f32,
//     cy: f32,
//     x_channel: &str,
//     y_channel: &str,
//     x_data_range: &RangeInclusive<f32>,
//     y_data_range: &RangeInclusive<f32>,
//     x_axis_range: &RangeInclusive<f32>,
//     y_axis_range: &RangeInclusive<f32>,
// ) -> anyhow::Result<(GateGeometry, GateGeometry, GateGeometry, GateGeometry)> {
//     let center = (cx, cy);
//     let (x_min, x_max) = (*x_axis_range.start(), *x_axis_range.end());
//     let (y_min, y_max) = (*y_axis_range.start(), *y_axis_range.end());
//     let (x_data_min, x_data_max) = (
//         x_data_range.start().min(x_min),
//         x_data_range.end().max(x_max),
//     );
//     let (y_data_min, y_data_max) = (
//         y_data_range.start().min(y_min),
//         y_data_range.end().max(y_max),
//     );

//     let bl1 = (x_data_min, y_data_min);
//     let bl2 = (center.0, y_data_min);
//     let bl3 = center;
//     let bl4 = (x_data_min, center.1);
//     let bl = flow_gates::geometry::create_polygon_geometry(
//         vec![bl1, bl2, bl3, bl4],
//         x_channel,
//         y_channel,
//     )
//     .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

//     let br1 = (center.0, y_data_min);
//     let br2 = (x_data_max, y_data_min);
//     let br3 = (x_data_max, center.1);
//     let br4 = center;
//     let br = flow_gates::geometry::create_polygon_geometry(
//         vec![br1, br2, br3, br4],
//         x_channel,
//         y_channel,
//     )
//     .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

//     let tr1 = center;
//     let tr2 = br3;
//     let tr3 = (x_data_max, y_data_max);
//     let tr4 = (center.0, y_data_max);
//     let tr = flow_gates::geometry::create_polygon_geometry(
//         vec![tr1, tr2, tr3, tr4],
//         x_channel,
//         y_channel,
//     )
//     .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

//     let tl1 = (x_data_min, center.1);
//     let tl2 = center;
//     let tl3 = (center.0, y_data_max);
//     let tl4 = (x_data_min, y_data_max);
//     let tl = flow_gates::geometry::create_polygon_geometry(
//         vec![tl1, tl2, tl3, tl4],
//         x_channel,
//         y_channel,
//     )
//     .map_err(|_| anyhow::anyhow!("failed to create polygon geometry"))?;

//     Ok((bl, br, tr, tl))
// }


use anyhow::Result;

// Assuming these are available in your crate scope

use crate::plotters_dioxus::{gates::{gate_composite::skewed_quadrant_gate::{DataPoints, create_skewed_quadrant_geos}, gate_drag::{GateDragData, PointDragData}, gate_single::{polygon_gate::PolygonGate, rescale_helper_point}, gate_traits::DrawableGate, gate_types::{DEFAULT_LINE, GateRenderShape, SELECTED_LINE, ShapeType}}, plot_helpers::PlotMapper};
type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(PartialEq, Clone)]
pub struct QuadrantGate {
    gates: FxIndexMap<Arc<str>, PolygonGate>,
    id: Arc<str>,
    points: DataPoints,
    axis_matched: bool,
    parameters: (Arc<str>, Arc<str>),
}

impl QuadrantGate {
    pub fn try_new_from_raw_coord(
        plot_map: &PlotMapper,
        id: Arc<str>,
        click_loc_raw: (f32, f32),
        x_axis_param: Arc<str>,
        y_axis_param: Arc<str>,
    ) -> Result<Self> {
        let (cx, cy) = plot_map.pixel_to_data(click_loc_raw.0, click_loc_raw.1, None, None);
        let points = DataPoints::new_from_click(cx, cy, plot_map);

        Self::try_new_from_data_points(
            id,
            points,
            x_axis_param,
            y_axis_param,
            true,
            None,
        )
    }

    fn try_new_from_data_points(
        id: Arc<str>,
        mut data_points: DataPoints,
        x_axis_param: Arc<str>,
        y_axis_param: Arc<str>,
        axis_matched: bool,
        subgate_ids: Option<Vec<Arc<str>>>,
    ) -> Result<Self> {
        // FORCE ORTHOGONALITY: Overwrite any skew with center alignment
        data_points.left.1 = data_points.center.1;
        data_points.right.1 = data_points.center.1;
        data_points.top.0 = data_points.center.0;
        data_points.bottom.0 = data_points.center.0;

        let mut gate_map = FxIndexMap::default();
        let parameters = (x_axis_param.clone(), y_axis_param.clone());
        
        // Reuse the skewed geometry generator (orthogonal is just 0 skew)
        let geos = create_skewed_quadrant_geos(data_points.clone(), &x_axis_param, &y_axis_param)?;

        let sub_ids = if let Some(ids) = subgate_ids {
            ids
        } else {
            vec![
                Arc::from(format!("{id}_BL")),
                Arc::from(format!("{id}_BR")),
                Arc::from(format!("{id}_TR")),
                Arc::from(format!("{id}_TL")),
            ]
        };

        let names = [format!("{id}_BL"), format!("{id}_BR"), format!("{id}_TR"), format!("{id}_TL")];

        for (i, (id_arc, name)) in sub_ids.into_iter().zip(names.into_iter()).enumerate() {
            let geo = match i {
                0 => geos.0.clone(),
                1 => geos.1.clone(),
                2 => geos.2.clone(),
                _ => geos.3.clone(),
            };
            let g = Gate {
                id: id_arc.clone(),
                name,
                geometry: geo,
                mode: flow_gates::GateMode::Global,
                parameters: parameters.clone(),
                label_position: None,
            };
            gate_map.insert(id_arc, PolygonGate::try_new(g)?);
        }

        Ok(Self {
            gates: gate_map,
            id,
            points: data_points,
            axis_matched,
            parameters,
        })
    }

    fn clone_with_point(&self, data_points: DataPoints) -> Result<Self> {
        let gate_ids = self.gates.keys().cloned().collect();
        Self::try_new_from_data_points(
            self.id.clone(),
            data_points,
            self.parameters.0.clone(),
            self.parameters.1.clone(),
            self.axis_matched,
            Some(gate_ids),
        )
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
}

impl DrawableGate for QuadrantGate {
    fn is_finalised(&self) -> bool { true }
    fn is_composite(&self) -> bool { true }
    fn get_id(&self) -> Arc<str> { self.id.clone() }
    fn get_params(&self) -> (Arc<str>, Arc<str>) { self.parameters.clone() }
    fn clone_box(&self) -> Box<dyn DrawableGate> { Box::new(self.clone()) }

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
        plot_map: &PlotMapper,
    ) -> Vec<GateRenderShape> {
        let (xmin, xmax) = { let a = plot_map.x_axis_min_max(); (*a.start(), *a.end()) };
        let (ymin, ymax) = { let a = plot_map.y_axis_min_max(); (*a.start(), *a.end()) };

        let mut center = self.points.center;

        if let Some(dd) = drag_point {
            let x_span = (xmax - xmin).abs();
            let y_span = (ymax - ymin).abs();
            // Center-only clamp for QuadrantGate
            if dd.point_index() == 0 {
                center = (
                    dd.loc().0.clamp(xmin + x_span * 0.1, xmax - x_span * 0.1),
                    dd.loc().1.clamp(ymin + y_span * 0.1, ymax - y_span * 0.1),
                );
            }
        }

        let style = if is_selected { &SELECTED_LINE } else { &DEFAULT_LINE };

        // Draw perfectly horizontal and vertical lines crossing at center
        let mut shapes = vec![
            GateRenderShape::Line { x1: xmin, y1: center.1, x2: center.0, y2: center.1, style, shape_type: ShapeType::UndraggableLine },
            GateRenderShape::Line { x1: center.0, y1: center.1, x2: xmax, y2: center.1, style, shape_type: ShapeType::UndraggableLine },
            GateRenderShape::Line { x1: center.0, y1: ymin, x2: center.0, y2: center.1, style, shape_type: ShapeType::UndraggableLine },
            GateRenderShape::Line { x1: center.0, y1: center.1, x2: center.0, y2: ymax, style, shape_type: ShapeType::UndraggableLine },
        ];

        if is_selected {
            shapes.push(GateRenderShape::Circle {
                center,
                radius: 4.0,
                fill: "red",
                shape_type: ShapeType::UndraggablePoint(0),
            });
        }
        shapes
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
        mapper: &PlotMapper,
    ) -> Result<Box<dyn DrawableGate>> {
        if point_index != 0 { return Ok(Box::new(self.clone())); }

        let (xmin, xmax) = { let a = mapper.x_axis_min_max(); (*a.start(), *a.end()) };
        let (ymin, ymax) = { let a = mapper.y_axis_min_max(); (*a.start(), *a.end()) };

        let clamped_c = (
            new_point.0.clamp(xmin + (xmax - xmin) * 0.1, xmax - (xmax - xmin) * 0.1),
            new_point.1.clamp(ymin + (ymax - ymin) * 0.1, ymax - (ymax - ymin) * 0.1),
        );

        let new_pts = DataPoints {
            center: clamped_c,
            left: (xmin, clamped_c.1),
            right: (xmax, clamped_c.1),
            bottom: (clamped_c.0, ymin),
            top: (clamped_c.0, ymax),
            x_data_range: self.points.x_data_range.clone(),
            y_data_range: self.points.y_data_range.clone(),
        };

        Ok(Box::new(self.clone_with_point(new_pts)?))
    }

    fn recalculate_gate_for_new_axis_limits(
        &self,
        param: Arc<str>,
        lower: f32,
        upper: f32,
        _transform: &TransformType,
    ) -> Result<Option<Box<dyn DrawableGate>>> {
        let is_x = param == self.parameters.0;
        let mut new_points = self.points.clone();
        let buffer = (upper - lower).abs() * 0.1;

        if is_x {
            new_points.center.0 = new_points.center.0.clamp(lower + buffer, upper - buffer);
            new_points.left.0 = lower;
            new_points.right.0 = upper;
            new_points.top.0 = new_points.center.0;
            new_points.bottom.0 = new_points.center.0;
        } else {
            new_points.center.1 = new_points.center.1.clamp(lower + buffer, upper - buffer);
            new_points.top.1 = upper;
            new_points.bottom.1 = lower;
            new_points.left.1 = new_points.center.1;
            new_points.right.1 = new_points.center.1;
        }

        Ok(Some(Box::new(self.clone_with_point(new_points)?)))
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
        data_range: (f32, f32),
        axis_range: (f32, f32),
    ) -> Result<Box<dyn DrawableGate>> {
        let (x_param, _) = &self.parameters;
        let is_x = x_param == &param;

        let mut c = rescale_helper_point(self.points.center, &param, x_param, old_transform, new_transform)?;
        
        // Orthogonal quadrants only care about the center and the edges
        let new_lower = axis_range.0;
        let new_upper = axis_range.1;
        let buffer = (new_upper - new_lower).abs() * 0.1;

        if is_x {
            c.0 = c.0.clamp(new_lower + buffer, new_upper - buffer);
        } else {
            c.1 = c.1.clamp(new_lower + buffer, new_upper - buffer);
        }

        let new_pts = DataPoints {
            center: c,
            left: (if is_x { new_lower } else { self.points.left.0 }, c.1),
            right: (if is_x { new_upper } else { self.points.right.0 }, c.1),
            bottom: (c.0, if !is_x { new_lower } else { self.points.bottom.1 }),
            top: (c.0, if !is_x { new_upper } else { self.points.top.1 }),
            x_data_range: if is_x { RangeInclusive::new(data_range.0, data_range.1) } else { self.points.x_data_range.clone() },
            y_data_range: if !is_x { RangeInclusive::new(data_range.0, data_range.1) } else { self.points.y_data_range.clone() },
        };

        Ok(Box::new(self.clone_with_point(new_pts)?))
    }

    fn match_to_plot_axis(&self, plot_x_param: &str, plot_y_param: &str) -> Result<Option<Box<dyn DrawableGate>>> {
        let mut new_gate_map = FxIndexMap::default();
        let mut swap_axis = false;
        for gate in self.gates.values() {
            if let Some(g) = gate.clone_polygon_for_axis_swap(plot_x_param, plot_y_param)? {
                swap_axis = true;
                new_gate_map.insert(gate.get_id(), g);
            } else { return Ok(None); }
        }
        Ok(Some(self.clone_with_gates(new_gate_map, swap_axis)))
    }

    // Boilerplate trait passthroughs
    fn get_gate_ref(&self, id: Option<&str>) -> Option<&Gate> {
        id.and_then(|id| self.gates.get(id)).and_then(|g| g.get_gate_ref(None))
    }
    fn get_inner_gate_ids(&self) -> Vec<Arc<str>> { self.gates.keys().cloned().collect() }
    fn rotate_gate(&self, _: (f32, f32)) -> Result<Option<Box<dyn DrawableGate>>> { Ok(None) }
    fn replace_points(&self, _: GateDragData) -> Result<Option<Box<dyn DrawableGate>>> { Ok(None) }
    
    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32), plot_map: &PlotMapper) -> Option<f32> {
        let (xmin, xmax) = {
            let axis = plot_map.x_axis_min_max();
            (*axis.start(), *axis.end())
        };
        let (ymin, ymax) = {
            let axis = plot_map.y_axis_min_max();
            (*axis.start(), *axis.end())
        };

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
}