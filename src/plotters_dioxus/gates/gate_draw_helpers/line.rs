use flow_gates::GateGeometry;

use crate::plotters_dioxus::{
    PlotDrawable,
    gates::{
        gate_drag::PointDragData,
        gate_types::{DRAGGED_LINE, GREY_LINE_DASHED, DrawingStyle, GateRenderShape, ShapeType},
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

pub fn draw_line(
    min: (f32, f32),
    max: (f32, f32),
    y_coord: f32,
    style: &'static DrawingStyle,
    shape_type: ShapeType,
    point_drag_data : &Option<PointDragData>,
    axis_matched: bool,
) -> Vec<GateRenderShape> {
    println!("({}, {}), ({}, {})", min.0, min.1, max.0, max.1);
    let coords = bounds_to_svg_line(min, max, y_coord);

    let mut x1 = coords.0;
    let mut x2 = coords.0 + coords.2;
    let y = coords.1;


    if let Some(pdd) = point_drag_data {
        match pdd.point_index() {
            0 => {
                x1 = pdd.loc().0;

                
            }
            1 => {
                x2 = pdd.loc().0;
            }
            _ => unreachable!(),
        }
    }

    println!("{}, {}, {}", x1, x2, y);

    if axis_matched {
    vec![GateRenderShape::Line {
        x1: x1,
        y1: y,
        x2: x2,
        y2: y,
        style,
        shape_type,
    }]
} else {
        vec![GateRenderShape::Line {
            x1: x1,
            y1: y,
            x2: x2,
            y2: y,
            style,
            shape_type,
        }]
}
}

pub fn draw_circles_for_line(min: (f32, f32), max: (f32, f32), y_coord: f32, point_drag_data: &Option<PointDragData>) -> Vec<GateRenderShape> {

    let mut coords = bounds_to_line_coords(min, max, y_coord);
    let style;
    if let Some(pdd) = point_drag_data {
        style = &DRAGGED_LINE;
        match pdd.point_index() {
            0 => {
                coords.0 .0 = pdd.loc().0;
            }
            1 => {
                coords.1 .0 = pdd.loc().0;
            }
            _ => unreachable!(),
        }
    } else {
        style = &GREY_LINE_DASHED;
    }

    let ((cx1, cy1), (cx2, cy2)) = coords;

    vec![
        GateRenderShape::Line { 
            x1: cx1, 
            y1: min.1, 
            x2: cx1, 
            y2: max.1, 
            style, 
            shape_type: ShapeType::GhostPoint },
        GateRenderShape::Line { 
            x1: cx2, 
            y1: min.1, 
            x2: cx2, 
            y2: max.1, 
            style, 
            shape_type: ShapeType::GhostPoint },
        GateRenderShape::Circle {
            center: (cx1, cy1),
            radius: 3.0,
            fill: "red",
            shape_type: ShapeType::Point(0),
        },
        GateRenderShape::Circle {
            center: (cx2, cy2),
            radius: 3.0,
            fill: "red",
            shape_type: ShapeType::Point(1),
        },
    ]
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

pub fn update_line_geometry(
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

    


    let current = new_point;


    // [bottom-left, bottom-right, top-right, top-left]
    let (idx_before, idx_after) = match point_index {
        0 => {
            //left
            (0, 3)
        }
        1 => {
            //right
            (1, 2)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "invalid point index for rectangle geometry"
            ));
        }
    };

    let p_prev = current_rect_points[idx_before];
    let p_next = current_rect_points[idx_after];

    let prev = (current.0, p_prev.1);
    let next = (current.0, p_next.1);

    current_rect_points[idx_before] = prev;
    current_rect_points[idx_after] = next;

    flow_gates::geometry::create_rectangle_geometry(current_rect_points, x_param, y_param)
        .map_err(|_| anyhow::anyhow!("failed to update rectangle geometry"))
}
