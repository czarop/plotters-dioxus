
use flow_fcs::TransformType;

use flow_gates::Gate;
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::{sync::Arc};



use crate::plotters_dioxus::{
    axis_info::{asinh_reverse_f32, asinh_transform_f32},
    gates::{
        gate_drag::PointDragData, gate_draw_helpers::{self
        }, gate_single::LineGate, gate_types::{ GateRenderShape, ShapeType}
    }, plot_helpers::PlotMapper,
};



type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(PartialEq, Clone)]
pub struct BisectorGate {
    gates: FxIndexMap<Arc<str>, LineGate>,
    id: Arc<str>,
    center_point: (f32, f32),
    axis_matched: bool,
    parameters: (Arc<str>, Arc<str>),
}

impl BisectorGate {
    pub fn try_new(plot_map: &PlotMapper, id: Arc<str>, click_loc: (f32, f32), x_axis_param: Arc<str>, y_axis_param: Arc<str>) -> anyhow::Result<Self> {
        let mut gate_map = FxIndexMap::default();
        let parameters = (x_axis_param.clone(), y_axis_param.clone());
        let click_data = plot_map.data_to_pixel(click_loc.0, click_loc.1, None, None);
        let geos = gate_draw_helpers::line::create_default_bisector(
                plot_map, 
                click_loc.0, 
                &x_axis_param, 
                &y_axis_param
            )?;
            let id_left = format!("{id}_L");
            let id_right = format!("{id}_R");
            let id_left_arc: Arc<str> = Arc::from(id_left.as_str());
            let id_right_arc: Arc<str> = Arc::from(id_right.as_str());
            let gate_left = Gate{ 
                id: id_left_arc.clone(), 
                name: id_left, 
                geometry: geos.0, 
                mode: flow_gates::GateMode::Global, 
                parameters: parameters.clone(), 
                label_position: None 
            };
            let gate_right = Gate{ 
                id: id_right_arc.clone(), 
                name: id_right, 
                geometry: geos.1, 
                mode: flow_gates::GateMode::Global, 
                parameters, 
                label_position: None 
            };
        let lg_l = LineGate::try_new(gate_left, click_data.1)?;
        let lg_r = LineGate::try_new(gate_right, click_data.1)?;
        gate_map.insert(id_left_arc, lg_l);
        gate_map.insert(id_right_arc, lg_r);
        Ok(Self {
            gates: gate_map,
            id,
            center_point: click_loc,
            axis_matched: true,
            parameters: (x_axis_param, y_axis_param),
        })
    }

    fn clone_with_gates(&self, gates: FxIndexMap<Arc<str>, LineGate>) -> Self {
        Self {
            gates,
            id: self.id.clone(),
            center_point: self.center_point,
            axis_matched: self.axis_matched,
            parameters: self.parameters.clone(),
        }
    }

}



impl super::gate_traits::DrawableGate for BisectorGate {
    fn get_points(&self) -> Vec<(f32, f32)> {
        todo!()
    }

    fn is_finalised(&self) -> bool {
        true
    }

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
    ) -> Vec<GateRenderShape> {
        todo!()
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
        for gate in self.gates.values(){
            match gate.is_point_on_perimeter(point, tolerance) {
                Some(t) => return Some(t),
                None => continue,
            }
        }
        None
    }

    fn match_to_plot_axis(
        &self,
        plot_x_param: &str,
        plot_y_param: &str,
    ) -> anyhow::Result<Option<Box<dyn super::gate_traits::DrawableGate>>> {
        todo!()
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<Box<dyn super::gate_traits::DrawableGate>> {
        let new_gate_map = FxIndexMap::default();

        for gate in self.gates.values_mut(){
            match gate.clone_line_for_rescaled_axis(param, old_transform, new_transform) {
                Ok(g) => {
                    new_gate_map.insert(gate.get_id(), g);
                },
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn rotate_gate(
        &self,
        mouse_position: (f32, f32),
    ) -> anyhow::Result<Option<Box<dyn super::gate_traits::DrawableGate>>> {
        todo!()
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
    ) -> anyhow::Result<Box<dyn super::gate_traits::DrawableGate>> {
        todo!()
    }

    fn replace_points(&self, gate_drag_data: super::gate_drag::GateDragData)
    -> anyhow::Result<Box<dyn super::gate_traits::DrawableGate>> {
        todo!()
    }

    fn clone_box(&self) -> Box<dyn super::gate_traits::DrawableGate> {
        todo!()
    }
}

// impl GateTrait for BisectorGate {
//     fn get_params(&self) -> (Arc<str>, Arc<str>) {
//         self.parameters.clone()
//     }

//     fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
    //     for gate in self.gates.values(){
    //         match gate.is_point_on_perimeter(point, tolerance) {
    //             Some(t) => return Some(t),
    //             None => continue,
    //         }
    //     }
    //     None
    // }

//     fn match_to_plot_axis(&mut self, plot_x_param: &str, plot_y_param: &str) -> anyhow::Result<()> {
        // for gate in self.gates.values_mut(){
        //     match gate.match_to_plot_axis(plot_x_param, plot_y_param) {
        //         Ok(()) => continue,
        //         Err(e) => return Err(e),
        //     }
        // }
        // Ok(())
//     }

//     fn recalculate_gate_for_rescaled_axis(
//         &mut self,
//         param: std::sync::Arc<str>,
//         old_transform: &TransformType,
//         new_transform: &TransformType,
//     ) -> anyhow::Result<()> {
//         for gate in self.gates.values_mut(){
//             match gate.recalculate_gate_for_rescaled_axis(param.clone(), old_transform, new_transform) {
//                 Ok(()) => continue,
//                 Err(e) => return Err(e),
//             }
//         }
//         Ok(())
//     }

//     fn replace_point(&mut self, new_point: (f32, f32), point_index: usize) -> anyhow::Result<()> {
        
//         let mut points = self.get_points();
//         if point_index < points.len() { points[point_index] = new_point; }

//         todo!("replace geometries for sub gates based on new points for bisector gate");

//         Ok(())
//     }

//     fn replace_points(&mut self, points: Vec<(f32, f32)>) -> anyhow::Result<()> {
//         if points.len() != 1 { return Err(anyhow!("Bisector gate should only have 1 point"));}
//         self.replace_point(points[0], 0)

//     }

//     fn rotate_gate(&mut self, _mouse_position: (f32, f32)) -> anyhow::Result<()> {
//         Err(anyhow!("Bisector cannot be rotated"))
//     }

//     fn get_id(&self) -> Arc<str> {
//         return self.id.clone();
//     }

//     fn is_composite(&self) -> bool {
//         true
//     }
// }

// impl PlotDrawable for BisectorGate {
//     fn get_points(&self) -> Vec<(f32, f32)> {
//         self.draw_points.clone()
//     }

//     fn is_finalised(&self) -> bool {
//         return true;
//     }

//     fn draw_self(&self) -> Vec<GateRenderShape> { // draw from self points - not from sub gates
//         // let gate_line_style = if self.is_selected() {
//         //     &SELECTED_LINE
//         // } else {
//         //     &DEFAULT_LINE
//         // };

//         // let main_points = self.get_points();
//         // let points_for_nodes = self.get_points_for_nodes();
//         // let mut index_offset = 0;
//         // let main_gate = match &self.inner.geometry {
//         //     GateGeometry::Polygon { .. } => draw_polygon(
//         //         &main_points,
//         //         gate_line_style,
//         //         ShapeType::Gate(self.id.clone()),
//         //     ),
//         //     GateGeometry::Ellipse {
//         //         center,
//         //         radius_x,
//         //         radius_y,
//         //         angle,
//         //     } => {
//         //         index_offset = 1;
//         //         let x = center
//         //             .get_coordinate(self.x_parameter_channel_name())
//         //             .unwrap_or_default();
//         //         let y = center
//         //             .get_coordinate(self.y_parameter_channel_name())
//         //             .unwrap_or_default();
//         //         draw_elipse(
//         //             (x, y),
//         //             *radius_x,
//         //             *radius_y,
//         //             *angle,
//         //             gate_line_style,
//         //             ShapeType::Gate(self.id.clone()),
//         //         )
//         //     }
//         //     GateGeometry::Rectangle { .. } => draw_rectangle(
//         //         main_points[0],
//         //         main_points[2],
//         //         gate_line_style,
//         //         ShapeType::Gate(self.id.clone()),
//         //     ),
//         //     _ => todo!(),
//         // };
//         // let selected_points = {
//         //     if self.is_selected() {
//         //         let mut circles = draw_circles_for_selected_gate(&*points_for_nodes, index_offset);
//         //         if let GateGeometry::Ellipse {
//         //             center,
//         //             radius_x: _,
//         //             radius_y: ry,
//         //             angle,
//         //         } = &self.geometry
//         //         {
//         //             let cx = center
//         //                 .get_coordinate(self.x_parameter_channel_name())
//         //                 .unwrap_or_default();
//         //             let cy = center
//         //                 .get_coordinate(self.y_parameter_channel_name())
//         //                 .unwrap_or_default();
//         //             let unrotated_top = (cx, cy + *ry);

//         //             let rotation = GateRenderShape::Handle {
//         //                 // center: points_for_nodes[3],
//         //                 center: unrotated_top,
//         //                 size: 5_f32,
//         //                 shape_center: main_points[0],
//         //                 shape_type: ShapeType::Rotation(*angle),
//         //             };
//         //             circles.push(rotation);
//         //         }
//         //         Some(circles)
//         //     } else {
//         //         None
//         //     }
//         // };
//         // let ghost_point = {
//         //     if let Some(drag_data) = self.drag_point {
//         //         None
//         //     } else {
//         //         None
//         //     }
//         // };

//         // let items_to_render = crate::collate_vecs!(main_gate, selected_points, ghost_point,);

//         // items_to_render
//         vec![]
//     }
// }

fn draw_circles_for_selected_gate(
    points: &[(f32, f32)],
    index_offset: usize,
) -> Vec<GateRenderShape> {
    points
        .iter()
        .enumerate()
        .map(|(idx, p)| GateRenderShape::Circle {
            center: *p,
            radius: 3.0,
            fill: "red",
            shape_type: ShapeType::Point(idx + index_offset),
        })
        .collect()
}
