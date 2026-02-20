use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{
    GateGeometry, create_ellipse_geometry, create_polygon_geometry, create_rectangle_geometry,
    geometry, types::LabelPosition,
};
use rustc_hash::FxHashMap;
use std::{ops::Deref, sync::Arc};

use crate::plotters_dioxus::{
    PlotDrawable,
    axis_info::{asinh_reverse_f32, asinh_transform_f32},
    gates::{
        gate_drag::PointDragData,
        gate_draw_helpers::{
            ellipse::{
                calculate_ellipse_nodes, draw_elipse, draw_ghost_point_for_ellipse,
                is_point_on_ellipse_perimeter, update_ellipse_geometry,
            },
            polygon::{draw_ghost_point_for_polygon, draw_polygon, is_point_on_polygon_perimeter},
            rectangle::{
                draw_ghost_point_for_rectangle, draw_rectangle, is_point_on_rectangle_perimeter,
                update_rectangle_geometry,
            },
        },
        gate_traits::GateTrait,
        gate_types::{DEFAULT_LINE, GateRenderShape, GateType, SELECTED_LINE, ShapeType},
    },
};

#[derive(PartialEq, Clone)]
pub struct CompositeGate {
    gates: Vec<flow_gates::Gate>,
    pub id: Arc<str>,
    id_map: FxHashMap<Arc<str>, usize>,
    selected: bool,
    drag_point: Option<PointDragData>,
    gate_type: GateType,
}

impl CompositeGate {
    pub fn new(id: Arc<str>, parametersgate_type: GateType) -> Self {
        match gate_type {
            GateType::Bisector => {}
            GateType::Quadrant => todo!(),
            GateType::FlexiQuadrant => todo!(),
            _ => panic!("Invalid gate type for composite gate"),
        }

        CompositeGate {
            gates,
            id,
            selected,
            drag_point: None,
            gate_type,
        }
    }

    fn swap_axis(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn to_render_points(&self, x_param: &str, y_param: &str) -> Vec<(f32, f32)> {
        vec![]
    }

    // returns a modified list of nodes, e.g. to drop the center node that is included in the ellipse
    fn get_points_for_nodes(&self) -> Vec<(f32, f32)> {
        vec![]
    }
}

// impl Deref for CompositeGate {
//     type Target = flow_gates::Gate;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

impl super::gate_traits::DrawableGate for CompositeGate {}

impl GateTrait for CompositeGate {
    fn get_params(&self) -> (Arc<str>, Arc<str>) {
        self.inner.parameters.clone()
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
        match &self.inner.geometry {
            GateGeometry::Polygon { .. } => is_point_on_polygon_perimeter(self, point, tolerance),
            GateGeometry::Rectangle { .. } => {
                is_point_on_rectangle_perimeter(self, point, tolerance)
            }
            GateGeometry::Ellipse {
                center,
                radius_x: rx,
                radius_y: ry,
                angle,
            } => {
                let x = center
                    .get_coordinate(self.x_parameter_channel_name())
                    .unwrap_or_default();
                let y = center
                    .get_coordinate(self.y_parameter_channel_name())
                    .unwrap_or_default();
                is_point_on_ellipse_perimeter(point, (x, y), *rx, *ry, *angle, tolerance)
            }
            GateGeometry::Boolean { .. } => todo!(),
        }
    }

    fn match_to_plot_axis(&mut self, plot_x_param: &str, plot_y_param: &str) -> anyhow::Result<()> {
        if plot_x_param == self.x_parameter_channel_name()
            && plot_y_param == self.y_parameter_channel_name()
        {
            return Ok(());
        } else if plot_x_param == self.y_parameter_channel_name()
            && plot_y_param == self.x_parameter_channel_name()
        {
            self.swap_axis()
        } else {
            Err(anyhow!(
                "Gate {} not applicable for plot axis {} and {}",
                self.id,
                plot_x_param,
                plot_y_param
            ))
        }
    }

    fn recalculate_gate_for_rescaled_axis(
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
                TransformType::Arcsinh { cofactor } => {
                    asinh_reverse_f32(*val, *cofactor).unwrap_or(*val)
                }
                TransformType::Linear => *val,
                _ => *val,
            };

            *val = match new_transform {
                TransformType::Arcsinh { cofactor } => {
                    asinh_transform_f32(raw, *cofactor).unwrap_or(raw)
                }
                TransformType::Linear => raw,
                _ => raw,
            };
        }

        let new_geo = match &self.geometry {
            GateGeometry::Polygon { .. } => geometry::create_polygon_geometry(
                points,
                self.x_parameter_channel_name(),
                self.y_parameter_channel_name(),
            )?,
            GateGeometry::Rectangle { .. } => geometry::create_rectangle_geometry(
                points,
                self.x_parameter_channel_name(),
                self.y_parameter_channel_name(),
            )?,
            GateGeometry::Ellipse { .. } => geometry::create_ellipse_geometry(
                points,
                self.x_parameter_channel_name(),
                self.y_parameter_channel_name(),
            )?,
            GateGeometry::Boolean { .. } => todo!(),
        };

        self.inner.geometry = new_geo;
        Ok(())
    }

    fn replace_point(&mut self, new_point: (f32, f32), point_index: usize) -> anyhow::Result<()> {
        let x_param = self.x_parameter_channel_name();
        let y_param = self.y_parameter_channel_name();

        match &self.inner.geometry {
            GateGeometry::Polygon { .. } => {
                let mut p = self.get_points();
                if point_index < p.len() {
                    p[point_index] = new_point;
                }
                self.inner.geometry = create_polygon_geometry(p, x_param, y_param)?;
            }
            GateGeometry::Rectangle { .. } => {
                let p = self.get_points();
                self.inner.geometry =
                    update_rectangle_geometry(p, new_point, point_index, x_param, y_param)?;
            }
            GateGeometry::Ellipse {
                center,
                radius_x,
                radius_y,
                angle,
            } => {
                self.inner.geometry = update_ellipse_geometry(
                    center,
                    *radius_x,
                    *radius_y,
                    *angle,
                    new_point,
                    point_index,
                    x_param,
                    y_param,
                )?;
            }
            GateGeometry::Boolean { .. } => todo!(),
        };

        Ok(())
    }

    fn replace_points(&mut self, points: Vec<(f32, f32)>) -> anyhow::Result<()> {
        let x_param = self.x_parameter_channel_name();
        let y_param = self.y_parameter_channel_name();

        let geometry = match &self.inner.geometry {
            GateGeometry::Polygon { .. } => create_polygon_geometry(points, x_param, y_param),
            GateGeometry::Rectangle { .. } => create_rectangle_geometry(points, x_param, y_param),
            GateGeometry::Ellipse { .. } => create_ellipse_geometry(points, x_param, y_param),
            GateGeometry::Boolean { .. } => todo!(),
        }?;
        self.inner.geometry = geometry;
        Ok(())
    }

    fn rotate_gate(&mut self, mouse_position: (f32, f32)) -> anyhow::Result<()> {
        let x_param = self.x_parameter_channel_name();
        let y_param = self.y_parameter_channel_name();
        match &self.inner.geometry {
            GateGeometry::Polygon { .. } => todo!(),
            GateGeometry::Rectangle { .. } => todo!(),
            GateGeometry::Ellipse {
                center,
                radius_x,
                radius_y,
                angle,
            } => {
                self.inner.geometry = update_ellipse_geometry(
                    center,
                    *radius_x,
                    *radius_y,
                    *angle,
                    mouse_position,
                    5,
                    x_param,
                    y_param,
                )?;
            }
            GateGeometry::Boolean { .. } => todo!(),
        };

        Ok(())
    }

    fn get_id(&self) -> Arc<str> {
        return self.id.clone();
    }

    fn is_composite(&self) -> bool {
        true
    }
}

impl PlotDrawable for CompositeGate {
    fn get_points(&self) -> Vec<(f32, f32)> {
        self.to_render_points(
            self.x_parameter_channel_name(),
            self.y_parameter_channel_name(),
        )
    }

    fn is_finalised(&self) -> bool {
        return true;
    }

    fn draw_self(&self) -> Vec<GateRenderShape> {
        let gate_line_style = if self.is_selected() {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };

        let main_points = self.get_points();
        let points_for_nodes = self.get_points_for_nodes();
        let mut index_offset = 0;
        let main_gate = match &self.inner.geometry {
            GateGeometry::Polygon { .. } => draw_polygon(
                &main_points,
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
            ),
            GateGeometry::Ellipse {
                center,
                radius_x,
                radius_y,
                angle,
            } => {
                index_offset = 1;
                let x = center
                    .get_coordinate(self.x_parameter_channel_name())
                    .unwrap_or_default();
                let y = center
                    .get_coordinate(self.y_parameter_channel_name())
                    .unwrap_or_default();
                draw_elipse(
                    (x, y),
                    *radius_x,
                    *radius_y,
                    *angle,
                    gate_line_style,
                    ShapeType::Gate(self.id.clone()),
                )
            }
            GateGeometry::Rectangle { .. } => draw_rectangle(
                main_points[0],
                main_points[2],
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
            ),
            _ => todo!(),
        };
        let selected_points = {
            if self.is_selected() {
                let mut circles = draw_circles_for_selected_gate(&*points_for_nodes, index_offset);
                if let GateGeometry::Ellipse {
                    center,
                    radius_x: _,
                    radius_y: ry,
                    angle,
                } = &self.geometry
                {
                    let cx = center
                        .get_coordinate(self.x_parameter_channel_name())
                        .unwrap_or_default();
                    let cy = center
                        .get_coordinate(self.y_parameter_channel_name())
                        .unwrap_or_default();
                    let unrotated_top = (cx, cy + *ry);

                    let rotation = GateRenderShape::Handle {
                        // center: points_for_nodes[3],
                        center: unrotated_top,
                        size: 5_f32,
                        shape_center: main_points[0],
                        shape_type: ShapeType::Rotation(*angle),
                    };
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
                    flow_gates::GateGeometry::Polygon { .. } => {
                        draw_ghost_point_for_polygon(&drag_data, &main_points)
                    }
                    GateGeometry::Ellipse { .. } => draw_ghost_point_for_ellipse(
                        &self.inner.geometry,
                        &drag_data,
                        self.x_parameter_channel_name(),
                        self.y_parameter_channel_name(),
                    ),
                    GateGeometry::Rectangle { .. } => {
                        draw_ghost_point_for_rectangle(&drag_data, &main_points)
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
