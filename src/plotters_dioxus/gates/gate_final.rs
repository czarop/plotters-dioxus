use std::{ops::Deref, sync::Arc};

use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{GateGeometry, create_ellipse_geometry, geometry};

use crate::plotters_dioxus::{
    PlotDrawable, axis_info::{asinh_reverse_f32, asinh_transform_f32}, gates::{
        gate_drag::{GateDragData, PointDragData},
        gate_styles::{
            DEFAULT_LINE, DRAGGED_LINE, DrawingStyle, GateShape, SELECTED_LINE,
            ShapeType,
        },
    }
};

#[derive(PartialEq, Clone)]
pub struct GateFinal {
    inner: flow_gates::Gate,
    selected: bool,
    drag_self: Option<GateDragData>,
    drag_point: Option<PointDragData>,
}

impl GateFinal {
    pub fn new(gate: flow_gates::Gate, selected: bool) -> Self {
        GateFinal {
            inner: gate,
            selected,
            drag_point: None,
            drag_self: None,
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, state: bool) {
        self.selected = state;
    }

    pub fn is_drag(&self) -> bool {
        self.drag_self.is_some()
    }

    pub fn set_drag_self(&mut self, drag_data: Option<GateDragData>) {
        self.drag_self = drag_data
    }

    pub fn is_drag_point(&self) -> bool {
        self.drag_point.is_some()
    }

    pub fn set_drag_point(&mut self, drag_data: Option<PointDragData>) {
        self.drag_point = drag_data;
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

        let mut gate = (self.inner).clone();
        gate.geometry = match &gate.geometry {
            GateGeometry::Polygon { .. } => geometry::create_polygon_geometry(points, gate.x_parameter_channel_name(), gate.y_parameter_channel_name())?,
            GateGeometry::Rectangle { .. } => geometry::create_rectangle_geometry(points, gate.x_parameter_channel_name(), gate.y_parameter_channel_name())?,
            GateGeometry::Ellipse { .. } => geometry::create_ellipse_geometry(points, gate.x_parameter_channel_name(), gate.y_parameter_channel_name())?,
            GateGeometry::Boolean { .. } => todo!(),
        };
        self.inner = gate;
        
        Ok(())
    }

    fn to_render_points(&self, x_param: &str, y_param: &str) -> Vec<(f32, f32)> {
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
            _ => todo!(),
        };
        let selected_points = {
            if self.is_selected() {
                Some(draw_circles_for_selected_gate(&*points_for_nodes))
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

fn is_point_on_ellipse_perimeter(
    &self, 
    click_pt: (f32, f32), 
    center: (f32, f32), 
    rx: f32, 
    ry: f32, 
    angle_rad: f32,
    tolerance: f32
) -> Option<f32> {
    // 1. Translate point relative to center
    let dx = click_pt.0 - center.0;
    let dy = click_pt.1 - center.1;

    // 2. Rotate point back (Inverse Rotation)
    // If ellipse is rotated by θ, rotate point by -θ
    let cos_a = (-angle_rad).cos();
    let sin_a = (-angle_rad).sin();
    let local_x = dx * cos_a - dy * sin_a;
    let local_y = dx * sin_a + dy * cos_a;

    // 3. The "Ellipse Equation" check
    // A point (x,y) is ON the perimeter if (x/rx)^2 + (y/ry)^2 == 1
    // We calculate the "distance" from 1.0
    
    // Calculate the angle of the click point relative to the local center
    let click_angle = local_y.atan2(local_x);
    
    // Find the closest point actually ON the ellipse at that same angle
    let on_ellipse_x = rx * click_angle.cos();
    let on_ellipse_y = ry * click_angle.sin();

    // 4. Distance check
    let dist_sq = (local_x - on_ellipse_x).powi(2) + (local_y - on_ellipse_y).powi(2);
    let dist = dist_sq.sqrt();

    if dist <= tolerance {
        Some(dist)
    } else {
        None
    }
}

fn draw_elipse(center: (f32, f32), rx: f32, ry: f32, angle_rotation: f32, style: &'static DrawingStyle,
    shape_type: ShapeType,) -> Vec<GateShape> {
        let degrees_rotation = angle_rotation.to_degrees();
        vec![GateShape::Ellipse { center, radius_x: rx, radius_y: ry, degrees_rotation, style, shape_type }]
    }

pub fn calculate_ellipse_nodes(cx: f32, cy: f32, rx: f32, ry: f32, angle_rad: f32) -> Vec<(f32, f32)> {
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    vec![
        (cx, cy),
        // Right node (Local: rx, 0)
        (cx + rx * cos_a, cy + rx * sin_a),
        // Top node (Local: 0, -ry) -> assuming standard screen coords where Y is down
        (cx + ry * sin_a, cy - ry * cos_a),
        // Left node (Local: -rx, 0)
        (cx - rx * cos_a, cy - rx * sin_a),
        // Bottom node (Local: 0, ry)
        (cx - ry * sin_a, cy + ry * cos_a),
    ]
}


fn draw_circles_for_selected_gate(points: &[(f32, f32)]) -> Vec<GateShape> {
    points
        .iter()
        .enumerate()
        .map(|(idx, p)| GateShape::Circle {
            center: *p,
            radius: 3.0,
            fill: "red",
            shape_type: ShapeType::Point(idx),
        })
        .collect()
}

fn draw_polygon(
    points: &[(f32, f32)],
    style: &'static DrawingStyle,
    shape_type: ShapeType,
) -> Vec<GateShape> {
    vec![GateShape::Polygon {
        points: points.to_vec(),
        style: style,
        shape_type,
    }]
}

fn draw_ghost_point_for_polygon(
    drag_data: &PointDragData,
    main_points: &[(f32, f32)],
) -> Option<Vec<GateShape>> {
    let idx = drag_data.point_index();
    let n = main_points.len();

    let idx_before = (idx + n - 1) % n;
    let idx_after = (idx + 1) % n;
    let p_prev = main_points[idx_before];
    let p_next = main_points[idx_after];

    let prev = (p_prev.0, p_prev.1);
    let current = drag_data.loc();
    let next = (p_next.0, p_next.1);

    let line = GateShape::PolyLine {
        points: vec![prev, current, next],
        style: &DRAGGED_LINE,
        shape_type: ShapeType::GhostPoint,
    };
    let point = GateShape::Circle {
        center: current,
        radius: 5.0,
        fill: "yellow",
        shape_type: ShapeType::GhostPoint,
    };
    Some(vec![line, point])
}

fn draw_ghost_point_for_ellipse(
    curr_geo: &GateGeometry,
    drag_data: &PointDragData,
    x_param: &str,
    y_param: &str
) -> Option<Vec<GateShape>>{
    let (cursor_x, cursor_y) = drag_data.loc();
    if let GateGeometry::Ellipse { center: current_center, radius_x: current_rx, radius_y: current_ry, angle: current_angle } = curr_geo {
        let current_cx =  current_center.get_coordinate(x_param).unwrap_or_default();
        let current_cy =  current_center.get_coordinate(y_param).unwrap_or_default();
        
        // 1. Pre-calculate trig for the constant angle
        let cos_a = current_angle.cos();
        let sin_a = current_angle.sin();

        // 2. Identify new dimensions
        // We calculate new_rx/ry based on distance from the CURRENT center 
        // to the NEW cursor position.
        let (new_rx, new_ry) = match drag_data.point_index() {
            1 | 3 => {
                // Dragging Right or Left changes horizontal radius
                let dist = f32::hypot(cursor_x - current_cx, cursor_y - current_cy);
                (dist, *current_ry)
            }
            2 | 4 => {
                // Dragging Top or Bottom changes vertical radius
                let dist = f32::hypot(cursor_x - current_cx, cursor_y - current_cy);
                (*current_rx, dist)
            }
            _ => (*current_rx, *current_ry),
        };

        // 3. Generate the 5-point Vec for your 'create_ellipse_geometry' function
        // Even though you don't use the center as a handle, the function requires it.
        let ghost_points: Vec<(f32, f32)> = vec![
            (current_cx, current_cy),                              // 0: Center (Stable)
            (current_cx + new_rx * cos_a, current_cy + new_rx * sin_a), // 1: Right
            (current_cx + new_ry * sin_a, current_cy - new_ry * cos_a), // 2: Top
            (current_cx - new_rx * cos_a, current_cy - new_rx * sin_a), // 3: Left
            (current_cx - new_ry * sin_a, current_cy + new_ry * cos_a), // 4: Bottom
        ];

        if let Ok(GateGeometry::Ellipse { center, radius_x, radius_y, angle }) = create_ellipse_geometry(ghost_points, &x_param, &y_param) {
            
            let x = center.get_coordinate(x_param).unwrap_or_default();
            let y = center.get_coordinate(y_param).unwrap_or_default();
            
            return Some(vec![
                GateShape::Circle {
            center: (cursor_x, cursor_y),
            radius: 5.0,
            fill: "yellow",
            shape_type: ShapeType::GhostPoint,
        },
        GateShape::Ellipse { 
            center: (x, y), 
            radius_x, 
            radius_y, 
            degrees_rotation: angle.to_degrees(), 
            style: &DRAGGED_LINE, 
            shape_type: ShapeType::GhostPoint }
            ]);
        } else {
            return None;
        }
    }
    None

}
