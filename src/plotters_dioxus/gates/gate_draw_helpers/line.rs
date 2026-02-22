use flow_gates::GateGeometry;

use crate::plotters_dioxus::{
    PlotDrawable,
    gates::{
        gate_drag::PointDragData,
        gate_types::{DRAGGED_LINE, DrawingStyle, GateRenderShape, ShapeType},
    },
    plot_helpers::PlotMapper,
};

pub fn create_default_line(
    plot_map: &PlotMapper,
    cx_raw: f32,
    width_raw: f32,
    x_channel: &str,
    y_channel: &str,
) -> anyhow::Result<GateGeometry> {
    let half_width = width_raw / 2f32;

    let (y_min, y_max) = plot_map.y_axis_min_max();

    let max = plot_map.pixel_to_data(cx_raw + half_width, y_max, None, None);
    let min = plot_map.pixel_to_data(cx_raw - half_width, y_min, None, None);
    let coords = vec![min, max];
    flow_gates::geometry::create_rectangle_geometry(coords, x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create rectangle geometry"))
}

pub fn bounds_to_svg_line(min: (f32, f32), max: (f32, f32), y_coord: f32) -> (f32, f32, f32) {
    let width = (max.0 - min.0).abs();
    let x = min.0;
    let y = y_coord;
    (x, y, width)
}

pub fn bounds_to_line_coords(min: (f32, f32), max: (f32, f32), y_coord: f32) -> ((f32, f32),(f32, f32)) {
    let width = (max.0 - min.0).abs();
    let x1 = min.0;
    let x2 = x1 + width;
    let y = y_coord;
    ((x1, y), (x2, y))
}

// pub fn map_line_to_pixels(
//     data_x: f32,
//     data_y: f32,
//     data_width: f32,
//     data_height: f32,
//     mapper: &PlotMapper,
// ) -> (f32, f32, f32, f32) {
//     // 1. Identify the two data-space corners
//     // (Assuming data_y is the "top" and data_x is the "left")
//     let x_min = data_x;
//     let x_max = data_x + data_width;
//     let y_max = data_y;
//     let y_min = data_y - data_height;

//     // 2. Map both points to pixel space
//     let (p1_x, p1_y) = mapper.data_to_pixel(x_min, y_max, None, None);
//     let (p2_x, p2_y) = mapper.data_to_pixel(x_max, y_min, None, None);

//     // 3. Calculate SVG attributes from the mapped pixels
//     // We use .min() and .abs() because screen Y is inverted
//     let rect_x = p1_x.min(p2_x);
//     let rect_y = p1_y.min(p2_y);
//     let rect_width = (p1_x - p2_x).abs();
//     let rect_height = (p1_y - p2_y).abs();

//     (rect_x, rect_y, rect_width, rect_height)
// }

pub fn draw_line(
    min: (f32, f32),
    max: (f32, f32),
    y_coord: f32,
    style: &'static DrawingStyle,
    shape_type: ShapeType,
) -> Vec<GateRenderShape> {
    let (x, y, width) = bounds_to_svg_line(min, max, y_coord);
    vec![GateRenderShape::Line {
        x1: x,
        y1: y,
        x2: x + width,
        y2: y,
        style,
        shape_type,
    }]
}

pub fn is_point_on_line(
    shape: & crate::plotters_dioxus::gates::gate_single::LineGate,
    point: (f32, f32),
    tolerance: (f32, f32),
) -> Option<f32> {
    let rect_bounds = shape.get_points();
    if rect_bounds.len() != 4 {
        return None;
    }

    let (min, max) = (rect_bounds[0], rect_bounds[2]);

    let line_coords = bounds_to_line_coords(min, max, shape.y_coord);

    if let Some(dis) = shape.is_near_segment(point, line_coords.0, line_coords.1, tolerance) {
        return Some(dis);
    }
    None
}

pub fn draw_ghost_point_for_line(
    drag_data: &PointDragData,
    y_coord: f32,
    current_rect_bounds: &[(f32, f32)],
) -> Option<Vec<GateRenderShape>> {
    // [left, right]
    let idx = drag_data.point_index();
    let (current_x, _) = drag_data.loc();

    if current_rect_bounds.len() != 4 {
        return None;
    }

    let (min, max) = (current_rect_bounds[0], current_rect_bounds[2]);

    let ((cx1, cy1), (cx2, cy2)) = bounds_to_line_coords(min, max, y_coord);

    let (x1, y1, x2, y2) = match idx {
        0 => {
            
            ((cx1 - current_x).abs(), cy1, cx2, cy2)
        }
        1 => {
            (cx1, cy1, (cx2 - current_x).abs(), cy2)
        }
        _ => unreachable!(),
    };

    let new_line = GateRenderShape::Line {
        x1,
        y1,
        x2,
        y2,
        style: &DRAGGED_LINE,
        shape_type: ShapeType::GhostPoint,
    };

    let left_curr = GateRenderShape::Circle {
        center: (cx1, cy1),
        radius: 5.0,
        fill: "yellow",
        shape_type: ShapeType::GhostPoint,
    };

    let right_curr = GateRenderShape::Circle {
        center: (cx2, cy2),
        radius: 5.0,
        fill: "yellow",
        shape_type: ShapeType::GhostPoint,
    };

    Some(vec![new_line, left_curr, right_curr])
}

pub fn update_line_geometry( // not done
    mut current_rect_points: Vec<(f32, f32)>,
    new_point: (f32, f32),
    point_index: usize,
    x_param: &str,
    y_param: &str,
) -> anyhow::Result<GateGeometry> {
    let n = current_rect_points.len();

    if point_index >= n {
        return Err(anyhow::anyhow!(
            "invalid point index for rectangle geometry"
        ));
    }

    let idx_before = (point_index + n - 1) % n;
    let idx_after = (point_index + 1) % n;

    let p_prev = current_rect_points[idx_before];
    let p_next = current_rect_points[idx_after];

    let prev;
    let current = new_point;
    let next;

    match point_index {
        0 => {
            //top-left, bottom-left, bottom-right
            prev = (current.0, p_prev.1);
            next = (p_next.0, current.1);
        }
        1 => {
            //bottom-left, bottom-right, top-right
            prev = (p_prev.0, current.1);
            next = (current.0, p_next.1);
        }
        2 => {
            //bottom-right, top-right, top-left
            prev = (current.0, p_prev.1);
            next = (p_next.0, current.1);
        }
        3 => {
            //top-right, top-left, bottom-left
            prev = (p_prev.0, current.1);
            next = (current.0, p_next.1);
        }
        _ => {
            return Err(anyhow::anyhow!(
                "invalid point index for rectangle geometry"
            ));
        }
    }

    current_rect_points[point_index] = new_point;
    current_rect_points[idx_before] = prev;
    current_rect_points[idx_after] = next;

    flow_gates::geometry::create_rectangle_geometry(current_rect_points, x_param, y_param)
        .map_err(|_| anyhow::anyhow!("failed to update rectangle geometry"))
}
