use flow_gates::GateGeometry;

use crate::plotters_dioxus::{
    PlotDrawable,
    gates::gate_styles::{DrawingStyle, GateShape, ShapeType},
    plot_helpers::PlotMapper,
};

pub fn create_default_rectangle(
    plot_map: &PlotMapper,
    cx_raw: f32,
    cy_raw: f32,
    width_raw: f32,
    height_raw: f32,
    x_channel: &str,
    y_channel: &str,
) -> anyhow::Result<GateGeometry> {
    let half_width = width_raw / 2f32;
    let half_height = height_raw / 2f32;

    let max = plot_map.pixel_to_data(cx_raw + half_width, cy_raw + half_height, None, None);
    let min = plot_map.pixel_to_data(cx_raw - half_width, cy_raw - half_height, None, None);
    let coords = vec![min, max];
    flow_gates::geometry::create_rectangle_geometry(coords, x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create rectangle geometry"))
}

pub fn bounds_to_svg_rect(min: (f32, f32), max: (f32, f32)) -> (f32, f32, f32, f32) {
    let width = (max.0 - min.0).abs();
    let height = (max.1 - min.1).abs();
    let x = min.0;
    let y = max.1;
    (x, y, width, height)
}

pub fn map_rect_to_pixels(
    data_x: f32,
    data_y: f32,
    data_width: f32,
    data_height: f32,
    mapper: &PlotMapper,
) -> (f32, f32, f32, f32) {
    // 1. Identify the two data-space corners
    // (Assuming data_y is the "top" and data_x is the "left")
    let x_min = data_x;
    let x_max = data_x + data_width;
    let y_max = data_y;
    let y_min = data_y - data_height;

    // 2. Map both points to pixel space
    let (p1_x, p1_y) = mapper.data_to_pixel(x_min, y_max, None, None);
    let (p2_x, p2_y) = mapper.data_to_pixel(x_max, y_min, None, None);

    // 3. Calculate SVG attributes from the mapped pixels
    // We use .min() and .abs() because screen Y is inverted
    let rect_x = p1_x.min(p2_x);
    let rect_y = p1_y.min(p2_y);
    let rect_width = (p1_x - p2_x).abs();
    let rect_height = (p1_y - p2_y).abs();

    (rect_x, rect_y, rect_width, rect_height)
}

pub fn draw_rectangle(
    min: (f32, f32),
    max: (f32, f32),
    style: &'static DrawingStyle,
    shape_type: ShapeType,
) -> Vec<GateShape> {
    let (x, y, width, height) = bounds_to_svg_rect(min, max);
    vec![GateShape::Rectangle {
        x,
        y,
        width,
        height,
        style,
        shape_type,
    }]
}

pub fn is_point_on_rectangle_perimeter(
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
