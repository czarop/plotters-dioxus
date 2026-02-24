use flow_gates::GateGeometry;
use crate::plotters_dioxus::gates::gate_traits::DrawableGate;
use crate::plotters_dioxus::{
    
    gates::{gate_drag::PointDragData, gate_types::{DRAGGED_LINE, DrawingStyle, GateRenderShape, ShapeType}},
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
) -> Vec<GateRenderShape> {
    let (x, y, width, height) = bounds_to_svg_rect(min, max);
    vec![GateRenderShape::Rectangle {
        x,
        y,
        width,
        height,
        style,
        shape_type,
    }]
}

pub fn is_point_on_rectangle_perimeter(
    shape: &crate::plotters_dioxus::gates::gate_single::RectangleGate,
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

pub fn draw_ghost_point_for_rectangle(
    drag_data: &PointDragData,
    main_points: &[(f32, f32)],
) -> Option<Vec<GateRenderShape>> {
    // [bottom-left, bottom-right, top-right, top-left]
    let idx = drag_data.point_index();
    let current = drag_data.loc();

    let (x, y, width, height) = match idx {
    0 => { // Bottom-Left dragged -> Anchor is Top-Right (Index 2)
        let anchor = main_points[2];
        let x = current.0.min(anchor.0);
        let y = current.1.max(anchor.1); // In data space, Top is Max Y
        let w = (current.0 - anchor.0).abs();
        let h = (current.1 - anchor.1).abs();
        (x, y, w, h)
    }
    1 => { // Bottom-Right dragged -> Anchor is Top-Left (Index 3)
        let anchor = main_points[3];
        let x = current.0.min(anchor.0);
        let y = current.1.max(anchor.1);
        let w = (current.0 - anchor.0).abs();
        let h = (current.1 - anchor.1).abs();
        (x, y, w, h)
    }
    2 => { // Top-Right dragged -> Anchor is Bottom-Left (Index 0)
        let anchor = main_points[0];
        let x = current.0.min(anchor.0);
        let y = current.1.max(anchor.1);
        let w = (current.0 - anchor.0).abs();
        let h = (current.1 - anchor.1).abs();
        (x, y, w, h)
    }
    3 => { // Top-Left dragged -> Anchor is Bottom-Right (Index 1)
        let anchor = main_points[1];
        let x = current.0.min(anchor.0);
        let y = current.1.max(anchor.1);
        let w = (current.0 - anchor.0).abs();
        let h = (current.1 - anchor.1).abs();
        (x, y, w, h)
    }
    _ => unreachable!(),
};

    let new_rect = GateRenderShape::Rectangle {
        x,
        y,
        width,
        height,
        style: &DRAGGED_LINE,
        shape_type: ShapeType::GhostPoint,
    };

    let point_curr = GateRenderShape::Circle {
        center: current,
        radius: 5.0,
        fill: "yellow",
        shape_type: ShapeType::GhostPoint,
    };

    Some(vec![new_rect, point_curr])
}

pub fn update_rectangle_geometry(
    mut current_points: Vec<(f32, f32)>,
    new_point: (f32, f32),
    point_index: usize,
    x_param: &str,
    y_param: &str,
) -> anyhow::Result<GateGeometry> {

    let n = current_points.len();

    if point_index >= n {
        return Err(anyhow::anyhow!("invalid point index for rectangle geometry"));
    }

    
    let idx_before = (point_index + n - 1) % n;
    let idx_after = (point_index + 1) % n;
    
    let p_prev = current_points[idx_before];
    let p_next = current_points[idx_after];

    let prev ;
    let current = new_point;
    let next ;
    
    match point_index {
        0 => {
            //top-left, bottom-left, bottom-right
            prev = (current.0, p_prev.1);
            next = (p_next.0, current.1);
        },
        1 => {
            //bottom-left, bottom-right, top-right
            prev = (p_prev.0, current.1);
            next = (current.0, p_next.1);
        },
        2 => {
            //bottom-right, top-right, top-left
            prev = (current.0, p_prev.1);
            next = (p_next.0, current.1);
        },
        3 => {
            //top-right, top-left, bottom-left
            prev = (p_prev.0, current.1);
            next = (current.0, p_next.1);
        },
        _ => return Err(anyhow::anyhow!("invalid point index for rectangle geometry")),
    }

    current_points[point_index] = new_point;
    current_points[idx_before] = prev;
    current_points[idx_after] = next;

    flow_gates::geometry::create_rectangle_geometry(current_points, x_param, y_param)
        .map_err(|_| anyhow::anyhow!("failed to update rectangle geometry"))
}