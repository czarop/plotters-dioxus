use crate::plotters_dioxus::gates::gate_traits::DrawableGate;
use crate::plotters_dioxus::{
    gates::{
        gate_drag::PointDragData,
        gate_types::{DRAGGED_LINE, DrawingStyle, GREY_LINE_DASHED, GateRenderShape, ShapeType},
    },
    plot_helpers::PlotMapper,
};
use flow_gates::GateGeometry;

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

pub fn create_default_bisector(
    plot_map: &PlotMapper,
    cx_raw: f32,
    x_channel: &str,
    y_channel: &str,
) -> anyhow::Result<(GateGeometry, GateGeometry)> {


    let (y_min, y_max) = plot_map.y_axis_min_max();
    let (x_min, x_max) = plot_map.x_axis_min_max();

    let max_left = plot_map.pixel_to_data(cx_raw, y_max, None, None);
    let min_left = plot_map.pixel_to_data(x_min, y_min, None, None);
    let coords = vec![min_left, max_left];
    let g1 = flow_gates::geometry::create_rectangle_geometry(coords, x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create rectangle geometry"))?;

    let max_right = plot_map.pixel_to_data(x_max, y_max, None, None);
    let min_right = plot_map.pixel_to_data(cx_raw, y_min, None, None);
    let coords = vec![min_right, max_right];
    let g2 = flow_gates::geometry::create_rectangle_geometry(coords, x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create rectangle geometry"))?;

    Ok((g1, g2))

}

pub fn bounds_to_svg_line(
    min: (f32, f32),
    max: (f32, f32),
    loc: f32,
    axis_matched: bool,
) -> (f32, f32, f32) {
    if axis_matched {
        let width = (max.0 - min.0).abs();
        let x = min.0;
        let y = loc;
        (x, y, width)
    } else {
        let width = (max.1 - min.1).abs();
        let x = loc;
        let y = min.1;
        (x, y, width)
    }
}

pub fn bounds_to_line_coords(
    min: (f32, f32),
    max: (f32, f32),
    loc: f32,
    axis_matched: bool,
) -> ((f32, f32), (f32, f32)) {
    if axis_matched {
        let width = (max.0 - min.0).abs();
        let x1 = min.0;
        let x2 = x1 + width;
        let y = loc;
        return ((x1, y), (x2, y));
    } else {
        let width = (max.1 - min.1).abs();
        let x = loc;

        let y1 = min.1;
        let y2 = y1 + width;
        return ((x, y1), (x, y2));
    }
}

pub fn draw_line(
    min: (f32, f32),
    max: (f32, f32),
    y_coord: f32,
    style: &'static DrawingStyle,
    shape_type: ShapeType,
    point_drag_data: &Option<PointDragData>,
    axis_matched: bool,
) -> Vec<GateRenderShape> {
    println!("({}, {}), ({}, {})", min.0, min.1, max.0, max.1);
    let coords = bounds_to_svg_line(min, max, y_coord, axis_matched);
    if axis_matched {
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

        vec![GateRenderShape::Line {
            x1: x1,
            y1: y,
            x2: x2,
            y2: y,
            style,
            shape_type,
        }]
    } else {
        let mut y1 = coords.1;
        let mut y2 = y1 + coords.2;

        if let Some(pdd) = point_drag_data {
            match pdd.point_index() {
                0 => {
                    y1 = pdd.loc().1;
                }
                1 => {
                    y2 = pdd.loc().1;
                }
                _ => unreachable!(),
            }
        }
        vec![GateRenderShape::Line {
            x1: y_coord,
            y1: y1,
            x2: y_coord,
            y2: y2,
            style,
            shape_type,
        }]
    }
}

pub fn draw_circles_for_line(
    min: (f32, f32),
    max: (f32, f32),
    loc: f32,
    point_drag_data: &Option<PointDragData>,
    axis_matched: bool,
) -> Vec<GateRenderShape> {
    let mut coords = bounds_to_line_coords(min, max, loc, axis_matched);
    let style;
    if let Some(pdd) = point_drag_data {
        style = &DRAGGED_LINE;
        match pdd.point_index() {
            0 => match axis_matched {
                true => coords.0.0 = pdd.loc().0,
                false => coords.0.1 = pdd.loc().1,
            },
            1 => match axis_matched {
                true => coords.1.0 = pdd.loc().0,
                false => coords.1.1 = pdd.loc().1,
            },
            _ => unreachable!(),
        }
    } else {
        style = &GREY_LINE_DASHED;
    }

    let mut x1 = coords.0.0;
    let mut y1 = coords.0.1;
    let mut x2 = coords.1.0;
    let mut y2 = coords.1.1;

    let (l1, l2, c1, c2) = match axis_matched {
        true => {
            y1 = min.1;
            y2 = max.1;

            (
                GateRenderShape::Line {
                    x1: x1,
                    y1: y1,
                    x2: x1,
                    y2: y2,
                    style,
                    shape_type: ShapeType::GhostPoint,
                },
                GateRenderShape::Line {
                    x1: x2,
                    y1: y1,
                    x2: x2,
                    y2: y2,
                    style,
                    shape_type: ShapeType::GhostPoint,
                },
                GateRenderShape::Circle {
                    center: (coords.0.0, coords.0.1),
                    radius: 3.0,
                    fill: "red",
                    shape_type: ShapeType::Point(0),
                },
                GateRenderShape::Circle {
                    center: (coords.1.0, coords.1.1),
                    radius: 3.0,
                    fill: "red",
                    shape_type: ShapeType::Point(1),
                },
            )
        }
        false => {
            x1 = min.0;
            x2 = max.0;

            (
                GateRenderShape::Line {
                    x1: x1,
                    y1: y1,
                    x2: x2,
                    y2: y1,
                    style,
                    shape_type: ShapeType::GhostPoint,
                },
                GateRenderShape::Line {
                    x1: x1,
                    y1: y2,
                    x2: x2,
                    y2: y2,
                    style,
                    shape_type: ShapeType::GhostPoint,
                },
                GateRenderShape::Circle {
                    center: (coords.0.0, coords.0.1),
                    radius: 3.0,
                    fill: "red",
                    shape_type: ShapeType::Point(0),
                },
                GateRenderShape::Circle {
                    center: (coords.1.0, coords.1.1),
                    radius: 3.0,
                    fill: "red",
                    shape_type: ShapeType::Point(1),
                },
            )
        }
    };

    vec![l1, l2, c1, c2]
}

pub fn is_point_on_line(
    shape: &crate::plotters_dioxus::gates::gate_single::LineGate,
    point: (f32, f32),
    tolerance: (f32, f32),
    axis_matched: bool,
) -> Option<f32> {
    let rect_bounds = shape.get_points();
    if rect_bounds.len() != 4 {
        return None;
    }

    let (min, max) = (rect_bounds[0], rect_bounds[2]);

    let line_coords = bounds_to_line_coords(min, max, shape.height, axis_matched);

    if let Some(dis) = shape.is_near_segment(point, line_coords.0, line_coords.1, tolerance) {
        return Some(dis);
    }
    None
}

pub fn update_line_geometry(
    mut current_rect_points: Vec<(f32, f32)>,
    new_point: (f32, f32),
    point_index: usize,
    x_param: &str,
    y_param: &str,
    axis_matched: bool,
) -> anyhow::Result<GateGeometry> {
    let n = current_rect_points.len();

    if point_index >= n {
        return Err(anyhow::anyhow!(
            "invalid point index for rectangle geometry"
        ));
    }

    let current = new_point;

    match axis_matched {
        true => {
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
        }
        false => {
            // [bottom-left, bottom-right, top-right, top-left]
            let (idx_before, idx_after) = match point_index {
                0 => {
                    //top - the rectangle is now rotated 90 degrees!
                    (1, 0)
                }
                1 => {
                    //bottom
                    (2, 3)
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "invalid point index for rectangle geometry"
                    ));
                }
            };

            let p_prev = current_rect_points[idx_before];
            let p_next = current_rect_points[idx_after];

            let prev = (p_prev.0, current.1);
            let next = (p_next.0, current.1);

            current_rect_points[idx_before] = prev;
            current_rect_points[idx_after] = next;
        }
    }

    flow_gates::geometry::create_rectangle_geometry(current_rect_points, x_param, y_param)
        .map_err(|_| anyhow::anyhow!("failed to update rectangle geometry"))
}
