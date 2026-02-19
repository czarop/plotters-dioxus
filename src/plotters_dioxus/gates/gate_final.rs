use std::{ops::Deref, sync::Arc};

use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{GateGeometry, create_ellipse_geometry, create_polygon_geometry, create_rectangle_geometry, geometry, types::LabelPosition};

use crate::plotters_dioxus::{
    PlotDrawable, axis_info::{asinh_reverse_f32, asinh_transform_f32}, gates::{
        gate_drag::PointDragData, gate_draw_helpers::{ellipse::{calculate_ellipse_nodes, draw_elipse, draw_ghost_point_for_ellipse, is_point_on_ellipse_perimeter, update_ellipse_geometry}, polygon::{draw_ghost_point_for_polygon, draw_polygon, is_point_on_polygon_perimeter}, rectangle::{draw_rectangle, is_point_on_rectangle_perimeter}}, gate_styles::{
            DEFAULT_LINE, GateShape, SELECTED_LINE,
            ShapeType,
        }
    }
};

#[derive(PartialEq, Clone)]
pub struct GateFinal {
    inner: flow_gates::Gate,
    selected: bool,
    drag_point: Option<PointDragData>,

}

impl GateFinal {
    pub fn new(gate: flow_gates::Gate, selected: bool) -> Self {
        GateFinal {
            inner: gate,
            selected,
            drag_point: None,
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, state: bool) {
        self.selected = state;
    }


    pub fn is_drag_point(&self) -> bool {
        self.drag_point.is_some()
    }

    pub fn set_drag_point(&mut self, drag_data: Option<PointDragData>) {
        self.drag_point = drag_data;
    }

    pub fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        match &self.inner.geometry{
            GateGeometry::Polygon { .. } => is_point_on_polygon_perimeter(self, point, tolerance),
            GateGeometry::Rectangle { .. } => is_point_on_rectangle_perimeter(self, point, tolerance),
            GateGeometry::Ellipse { center, radius_x: rx, radius_y: ry, angle  } => {
                let x = center.get_coordinate(self.x_parameter_channel_name()).unwrap_or_default();
                let y = center.get_coordinate(self.y_parameter_channel_name()).unwrap_or_default();
                is_point_on_ellipse_perimeter(point, (x, y), *rx, *ry, *angle, tolerance)
            },
            GateGeometry::Boolean { .. } => todo!(),
        }
    }

    pub fn match_to_plot_axis(&mut self, plot_x_param: &str, plot_y_param: &str) -> anyhow::Result<()> {
        if plot_x_param == self.x_parameter_channel_name() && plot_y_param == self.y_parameter_channel_name() {
            return Ok(());
        } else if plot_x_param == self.y_parameter_channel_name() && plot_y_param == self.x_parameter_channel_name() {
            self.swap_axis()
        } else {
            Err(anyhow!("Gate {} not applicable for plot axis {} and {}", self.id, plot_x_param, plot_y_param))
        }
    }

    fn swap_axis(&mut self) -> anyhow::Result<()> {
        let (curr_x, curr_y) = &self.inner.parameters;
        let new_params = (curr_y.clone(), curr_x.clone());
        
        let (x_param, y_param) = &new_params;

        let swapped_points = self.to_render_points(curr_y, curr_x);
        let new_geo = match &self.geometry {
            GateGeometry::Polygon { .. } => None,
            GateGeometry::Rectangle { .. } => Some(create_rectangle_geometry(swapped_points, x_param, y_param)?),
            GateGeometry::Ellipse { center, radius_x, radius_y, angle } => {
                //format: [center, right/end, top, left/start, bottom]
                if let (Some(cx), Some(cy)) = (
                    center.get_coordinate(curr_x),
                    center.get_coordinate(curr_y),
                ) {
                    let pts = calculate_ellipse_nodes(cx, cy, *radius_x, *radius_y, *angle);
    
        
                    let reflect = |(x, y): (f32, f32)| (y, x);

                    let mirrored_pts = vec![
                        reflect(pts[0]),
                        reflect(pts[3]),
                        reflect(pts[4]),
                        reflect(pts[1]),
                        reflect(pts[2]),
                    ];
                    Some(create_ellipse_geometry(mirrored_pts, x_param, y_param)?)
                } else {
                    panic!("failed to make ellipse geometry")
                }
                
            },
            GateGeometry::Boolean { .. } => todo!(),
        };

        let new_label_poition = match &self.inner.label_position{
            Some(pos) => {
                Some(LabelPosition{ offset_x: pos.offset_y, offset_y: pos.offset_x })
            },
            None => None,
        };

        self.inner.parameters = new_params;
        if new_geo.is_some() {
            self.inner.geometry = new_geo.unwrap();
        }
        
        self.inner.label_position = new_label_poition;

        self.set_drag_point(None);
        self.set_selected(false);

        Ok(())
    }

    pub fn recalculate_gate_for_rescaled_axis(
        &mut self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<()> {
        
        let is_x = self.x_parameter_channel_name() == &*param;
        let is_y = self.y_parameter_channel_name() == &*param;
        
        if !is_x && !is_y {
            return Err(anyhow!("Param does not match gate {}!", &self.name));
        }

        let mut points = self.get_points();

        for p in points.iter_mut() {
            let val = if is_x { &mut p.0 } else { &mut p.1 };
            let raw = match old_transform {
                TransformType::Arcsinh { cofactor } => asinh_reverse_f32(*val, *cofactor).unwrap_or(*val),
                TransformType::Linear => *val,
                _ => *val,
            };

            *val = match new_transform {
                TransformType::Arcsinh { cofactor } => asinh_transform_f32(raw, *cofactor).unwrap_or(raw),
                TransformType::Linear => raw,
                _ => raw,
            };
        }


        let new_geo = match &self.geometry {
            GateGeometry::Polygon { .. } => geometry::create_polygon_geometry(points, self.x_parameter_channel_name(), self.y_parameter_channel_name())?,
            GateGeometry::Rectangle { .. } => geometry::create_rectangle_geometry(points, self.x_parameter_channel_name(), self.y_parameter_channel_name())?,
            GateGeometry::Ellipse { .. } => geometry::create_ellipse_geometry(points, self.x_parameter_channel_name(), self.y_parameter_channel_name())?,
            GateGeometry::Boolean { .. } => todo!(),
        };

        self.inner.geometry = new_geo;
        Ok(())
    }

    fn to_render_points(&self, x_param: &str, y_param: &str) -> Vec<(f32, f32)> {
        println!("called {} {}", x_param, y_param);
        match &self.inner.geometry {
            GateGeometry::Polygon { nodes, .. } => nodes
                .iter()
                .filter_map(|n| Some((n.get_coordinate(x_param)?, n.get_coordinate(y_param)?)))
                .collect(),
            GateGeometry::Rectangle { min, max } => {
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                    min.get_coordinate(x_param),
                    min.get_coordinate(y_param),
                    max.get_coordinate(x_param),
                    max.get_coordinate(y_param),
                ) {
                    // Create the 4 corners of the rectangle in sequence
                    vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)]
                } else {
                    vec![]
                }
            }
            GateGeometry::Ellipse {
                center,
                radius_x,
                radius_y,
                angle,
            } => {
                //format: [center, right/end, top, left/start, bottom]
                if let (Some(cx), Some(cy)) = (
                    center.get_coordinate(x_param),
                    center.get_coordinate(y_param),
                ) {
                    calculate_ellipse_nodes(cx, cy, *radius_x, *radius_y, *angle)
                } else {
                    panic!("failed to make ellipse geometry")
                }
            }
            GateGeometry::Boolean { .. } => vec![],
        }
    }


    // returns a modified list of nodes, e.g. to drop the center node that is included in the ellipse
    fn get_points_for_nodes(&self) -> Vec<(f32, f32)> {
        let p = self.to_render_points(
                self.x_parameter_channel_name(),
                self.y_parameter_channel_name(),
            );
        
        match self.inner.geometry {
            GateGeometry::Ellipse { .. } => {
                p[1..].to_vec()
            },
            _ => p,
        }
        
    }

    pub fn rotate_gate(&mut self, mouse_position: (f32, f32)) -> anyhow::Result<()> {
        let x_param = self.x_parameter_channel_name();
        let y_param = self.y_parameter_channel_name();
        match &self.inner.geometry {
            GateGeometry::Polygon { .. } => todo!(),
            GateGeometry::Rectangle { .. } => todo!(),
            GateGeometry::Ellipse { center, radius_x, radius_y, angle } => {
                self.inner.geometry = update_ellipse_geometry(center, *radius_x, *radius_y, *angle, mouse_position, 5, x_param, y_param)?;
            },
            GateGeometry::Boolean { .. } => todo!(),
        };

        Ok(())
    }

    pub fn replace_point(&mut self, new_point: (f32, f32), point_index: usize) -> anyhow::Result<()> {
        let x_param = self.x_parameter_channel_name();
        let y_param = self.y_parameter_channel_name();

        match &self.inner.geometry {
            GateGeometry::Polygon { .. } => {
                let mut p = self.get_points();
                if point_index < p.len() {
                    p[point_index] = new_point;
                }
                self.inner.geometry = create_polygon_geometry(p, x_param, y_param)?;
            },
            GateGeometry::Rectangle { .. } => todo!(),
            GateGeometry::Ellipse { center, radius_x, radius_y, angle } => {

                self.inner.geometry = update_ellipse_geometry(center, *radius_x, *radius_y, *angle, new_point, point_index, x_param, y_param)?;
            },
            GateGeometry::Boolean { .. } => todo!(),
        };

        Ok(())
    }

    pub fn replace_points(&mut self, points: Vec<(f32, f32)>) -> anyhow::Result<()> {
        let x_param = self.x_parameter_channel_name();
        let y_param = self.y_parameter_channel_name();

        let geometry = match &self.inner.geometry {
            GateGeometry::Polygon { .. } => {
                create_polygon_geometry(points, x_param, y_param)
            },
            GateGeometry::Rectangle { .. } => create_rectangle_geometry(points, x_param, y_param),
            GateGeometry::Ellipse { .. } => {
                
                create_ellipse_geometry(points, x_param, y_param)
            },
            GateGeometry::Boolean { .. } => todo!(),
        }?;
        self.inner.geometry = geometry;
        Ok(())
    }

}

impl Deref for GateFinal {
    type Target = flow_gates::Gate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PlotDrawable for GateFinal {
    fn get_points(&self) -> Vec<(f32, f32)> {
        self.to_render_points(
            self.x_parameter_channel_name(),
            self.y_parameter_channel_name(),
        )
    }

    fn is_finalised(&self) -> bool {
        return true;
    }

    fn draw_self(&self) -> Vec<GateShape> {
        let gate_line_style = if self.is_selected() {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };

        let main_points = self.get_points();
        let points_for_nodes = self.get_points_for_nodes();
        let mut index_offset = 0;
        let main_gate = match &self.inner.geometry {
            GateGeometry::Polygon {
                ..
            } => {
                draw_polygon(
                &main_points,
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
            )},
            GateGeometry::Ellipse { center, radius_x, radius_y, angle} => 
            {
                index_offset = 1;
                let x = center.get_coordinate(self.x_parameter_channel_name()).unwrap_or_default();
                let y = center.get_coordinate(self.y_parameter_channel_name()).unwrap_or_default();
            draw_elipse(
                (x, y),
                *radius_x,
                *radius_y,
                *angle,
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
            )},
            GateGeometry::Rectangle { min, max } => {
                draw_rectangle(main_points[0],
                    main_points[2],
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
                )
            }
            _ => todo!(),
        };
        let selected_points = {
            if self.is_selected() {
                let mut circles = draw_circles_for_selected_gate(&*points_for_nodes, index_offset);
                if let GateGeometry::Ellipse { center, radius_x: _, radius_y: ry, angle } = &self.geometry {
                    let cx = center.get_coordinate(self.x_parameter_channel_name()).unwrap_or_default();
                    let cy = center.get_coordinate(self.y_parameter_channel_name()).unwrap_or_default();
                    let unrotated_top = (cx, cy + *ry);

                    let rotation = GateShape::Handle { 
                        // center: points_for_nodes[3], 
                        center: unrotated_top,
                        size: 5_f32, 
                        shape_center: main_points[0], 
                        shape_type: ShapeType::Rotation(*angle) };
                    circles.push(rotation);
                }
                Some(circles)
            } else {
                None
            }
        };
        let ghost_point = {
            if let Some(drag_data) = self.drag_point {
                match &self.inner.geometry {
                    flow_gates::GateGeometry::Polygon {
                        ..
                    } => draw_ghost_point_for_polygon(&drag_data, &main_points),
                    GateGeometry::Ellipse { .. } => {
                        draw_ghost_point_for_ellipse(&self.inner.geometry, &drag_data, self.x_parameter_channel_name(), self.y_parameter_channel_name())
                    }
                    _ => todo!(),
                }
            } else {
                None
            }
        };

        let items_to_render = crate::collate_vecs!(main_gate, selected_points, ghost_point,);

        items_to_render
    }

    
}



fn draw_circles_for_selected_gate(points: &[(f32, f32)], index_offset: usize) -> Vec<GateShape> {
    points
        .iter()
        .enumerate()
        .map(|(idx, p)| GateShape::Circle {
            center: *p,
            radius: 3.0,
            fill: "red",
            shape_type: ShapeType::Point(idx + index_offset),
        })
        .collect()
}




