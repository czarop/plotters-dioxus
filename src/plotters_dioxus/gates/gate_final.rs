use std::{ops::Deref};

use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{GateGeometry, geometry};

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
}

impl Deref for GateFinal {
    type Target = flow_gates::Gate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PlotDrawable for GateFinal {
    fn get_points(&self) -> Vec<(f32, f32)> {
        self.inner.geometry.to_render_points(
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
        let main_gate = match &self.inner.geometry {
            GateGeometry::Polygon {
                ..
            } => draw_polygon(
                &main_points,
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
            ),
            GateGeometry::Ellipse { .. } => draw_elipse(
                &main_points[0],
                &main_points[1].0,
                &main_points[1].1,
                gate_line_style,
                ShapeType::Gate(self.id.clone()),
            ),
            _ => todo!(),
        };
        let selected_points = {
            if self.is_selected() {
                Some(draw_circles_for_selected_polygon(&main_points))
            } else {
                None
            }
        };
        let ghost_point = {
            if let Some(drag_data) = self.drag_point {
                match &self.inner.geometry {
                    flow_gates::GateGeometry::Polygon {
                        nodes: _,
                        closed: _,
                    } => draw_ghost_point_for_polygon(&drag_data, &main_points),
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

fn draw_elipse(center: (f32, f32), rx: f32, ry: f32, style: &'static DrawingStyle,
    shape_type: ShapeType,) -> Vec<GateShape> {
        vec![GateShape::Ellipse { center, radius_x: rx, radius_y: ry, style, shape_type }]
    }

fn draw_circles_for_selected_polygon(points: &[(f32, f32)]) -> Vec<GateShape> {
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
