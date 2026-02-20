use crate::plotters_dioxus::{
    PlotDrawable,
    gates::{
        gate_drag::PointDragData,
        gate_types::{DRAGGED_LINE, DrawingStyle, GateShape, ShapeType},
    },
};

pub fn draw_polygon(
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

pub fn draw_ghost_point_for_polygon(
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

pub fn is_point_on_polygon_perimeter(
    shape: &dyn PlotDrawable,
    point: (f32, f32),
    tolerance: (f32, f32),
) -> Option<f32> {
    let points = shape.get_points();
    if points.len() < 2 {
        return None;
    }
    let mut closest = std::f32::INFINITY;
    for segment in points.windows(2) {
        if let Some(dis) = shape.is_near_segment(point, segment[0], segment[1], tolerance) {
            closest = closest.min(dis);
        }
    }
    // close the loop if required:
    let first = points[0];
    let last = points[points.len() - 1];

    if first != last {
        if let Some(dis) = shape.is_near_segment(point, last, first, tolerance) {
            closest = closest.min(dis);
        }
    }
    if closest == std::f32::INFINITY {
        return None;
    } else {
        return Some(closest);
    }
}
