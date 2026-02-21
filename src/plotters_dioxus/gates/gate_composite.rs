use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{
    GateGeometry, create_ellipse_geometry, create_polygon_geometry, create_rectangle_geometry,
    geometry,
};
use indexmap::IndexMap;
use rustc_hash::FxBuildHasher;
use std::{sync::Arc};



use crate::plotters_dioxus::{
    PlotDrawable,
    axis_info::{asinh_reverse_f32, asinh_transform_f32},
    gates::{
        gate_drag::PointDragData, gate_draw_helpers::{
            ellipse::{
                draw_elipse, draw_ghost_point_for_ellipse,
                is_point_on_ellipse_perimeter, update_ellipse_geometry,
            },
            polygon::{draw_ghost_point_for_polygon, draw_polygon, is_point_on_polygon_perimeter},
            rectangle::{
                draw_ghost_point_for_rectangle, draw_rectangle, is_point_on_rectangle_perimeter,
                update_rectangle_geometry,
            },
        }, gate_single::RectangleGate, gate_traits::GateTrait, gate_types::{DEFAULT_LINE, GateRenderShape, GateType, SELECTED_LINE, ShapeType}
    },
};

type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;

#[derive(PartialEq, Clone)]
pub struct BisectorGate {
    gates: FxIndexMap<Arc<str>, RectangleGate>,
    pub id: Arc<str>,
    draw_points: Vec<(f32, f32)>,
    selected: bool,
    drag_point: Option<PointDragData>,
    parameters: (Arc<str>, Arc<str>),
}

impl BisectorGate {
    pub fn new(id: Arc<str>, click_loc: (f32, f32), x_axis_param: Arc<str>, y_axis_param: Arc<str>) -> Self {
        let gate_map = FxIndexMap::default();
        Self {
            gates: gate_map,
            id,
            selected: false,
            drag_point: None,
            parameters: (x_axis_param, y_axis_param),
        }
    }

}



impl super::gate_traits::DrawableGate for BisectorGate {}

impl GateTrait for BisectorGate {
    fn get_params(&self) -> (Arc<str>, Arc<str>) {
        self.parameters.clone()
    }

    fn is_selected(&self) -> bool {
        self.selected
    }

    fn set_selected(&mut self, state: bool) {
        self.selected = state;
    }

    fn is_drag_point(&self) -> bool {
        self.drag_point.is_some()
    }

    fn set_drag_point(&mut self, drag_data: Option<PointDragData>) {
        self.drag_point = drag_data;
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

    fn match_to_plot_axis(&mut self, plot_x_param: &str, plot_y_param: &str) -> anyhow::Result<()> {
        for gate in self.gates.values_mut(){
            match gate.match_to_plot_axis(plot_x_param, plot_y_param) {
                Ok(()) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn recalculate_gate_for_rescaled_axis(
        &mut self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<()> {
        for gate in self.gates.values_mut(){
            match gate.recalculate_gate_for_rescaled_axis(param.clone(), old_transform, new_transform) {
                Ok(()) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn replace_point(&mut self, new_point: (f32, f32), point_index: usize) -> anyhow::Result<()> {
        
        let mut points = self.get_points();
        if point_index < points.len() { points[point_index] = new_point; }

        todo!("replace geometries for sub gates based on new points for bisector gate");

        Ok(())
    }

    fn replace_points(&mut self, points: Vec<(f32, f32)>) -> anyhow::Result<()> {
        if points.len() != 1 { return Err(anyhow!("Bisector gate should only have 1 point"));}
        self.replace_point(points[0], 0)

    }

    fn rotate_gate(&mut self, _mouse_position: (f32, f32)) -> anyhow::Result<()> {
        Err(anyhow!("Bisector cannot be rotated"))
    }

    fn get_id(&self) -> Arc<str> {
        return self.id.clone();
    }

    fn is_composite(&self) -> bool {
        true
    }
}

impl PlotDrawable for BisectorGate {
    fn get_points(&self) -> Vec<(f32, f32)> {
        self.draw_points.clone()
    }

    fn is_finalised(&self) -> bool {
        return true;
    }

    fn draw_self(&self) -> Vec<GateRenderShape> { // draw from self points - not from sub gates
        // let gate_line_style = if self.is_selected() {
        //     &SELECTED_LINE
        // } else {
        //     &DEFAULT_LINE
        // };

        // let main_points = self.get_points();
        // let points_for_nodes = self.get_points_for_nodes();
        // let mut index_offset = 0;
        // let main_gate = match &self.inner.geometry {
        //     GateGeometry::Polygon { .. } => draw_polygon(
        //         &main_points,
        //         gate_line_style,
        //         ShapeType::Gate(self.id.clone()),
        //     ),
        //     GateGeometry::Ellipse {
        //         center,
        //         radius_x,
        //         radius_y,
        //         angle,
        //     } => {
        //         index_offset = 1;
        //         let x = center
        //             .get_coordinate(self.x_parameter_channel_name())
        //             .unwrap_or_default();
        //         let y = center
        //             .get_coordinate(self.y_parameter_channel_name())
        //             .unwrap_or_default();
        //         draw_elipse(
        //             (x, y),
        //             *radius_x,
        //             *radius_y,
        //             *angle,
        //             gate_line_style,
        //             ShapeType::Gate(self.id.clone()),
        //         )
        //     }
        //     GateGeometry::Rectangle { .. } => draw_rectangle(
        //         main_points[0],
        //         main_points[2],
        //         gate_line_style,
        //         ShapeType::Gate(self.id.clone()),
        //     ),
        //     _ => todo!(),
        // };
        // let selected_points = {
        //     if self.is_selected() {
        //         let mut circles = draw_circles_for_selected_gate(&*points_for_nodes, index_offset);
        //         if let GateGeometry::Ellipse {
        //             center,
        //             radius_x: _,
        //             radius_y: ry,
        //             angle,
        //         } = &self.geometry
        //         {
        //             let cx = center
        //                 .get_coordinate(self.x_parameter_channel_name())
        //                 .unwrap_or_default();
        //             let cy = center
        //                 .get_coordinate(self.y_parameter_channel_name())
        //                 .unwrap_or_default();
        //             let unrotated_top = (cx, cy + *ry);

        //             let rotation = GateRenderShape::Handle {
        //                 // center: points_for_nodes[3],
        //                 center: unrotated_top,
        //                 size: 5_f32,
        //                 shape_center: main_points[0],
        //                 shape_type: ShapeType::Rotation(*angle),
        //             };
        //             circles.push(rotation);
        //         }
        //         Some(circles)
        //     } else {
        //         None
        //     }
        // };
        // let ghost_point = {
        //     if let Some(drag_data) = self.drag_point {
        //         None
        //     } else {
        //         None
        //     }
        // };

        // let items_to_render = crate::collate_vecs!(main_gate, selected_points, ghost_point,);

        // items_to_render
        vec![]
    }
}

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
