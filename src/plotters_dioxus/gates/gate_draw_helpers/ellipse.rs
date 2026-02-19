use flow_gates::{GateGeometry, GateNode};

use crate::plotters_dioxus::{gates::{
    gate_drag::PointDragData,
    gate_styles::{DRAGGED_LINE, DrawingStyle, GateShape, ShapeType},
}, plot_helpers::PlotMapper};

pub fn is_point_on_ellipse_perimeter(
    point: (f32, f32),
    center: (f32, f32),
    rx: f32,
    ry: f32,
    angle_rad: f32,
    tolerance: (f32, f32),
) -> Option<f32> {
    // 1. Pre-calculate rotation once
    let (sin_a, cos_a) = (-angle_rad).sin_cos();

    // 2. Translate and Rotate point into local ellipse space
    let dx = point.0 - center.0;
    let dy = point.1 - center.1;
    let local_x = dx * cos_a - dy * sin_a;
    let local_y = dx * sin_a + dy * cos_a;

    // 3. Normalized distance check (The Ellipse Equation: (x/rx)^2 + (y/ry)^2 = 1)
    // We use a "fat" perimeter check by comparing the normalized distance to 1.0
    let norm_x = local_x / rx;
    let norm_y = local_y / ry;
    let dist_sq = norm_x * norm_x + norm_y * norm_y;

    // Estimate thickness based on tolerance
    // This is much faster than finding the exact nearest coordinate
    let norm_tol = (tolerance.0 / rx).max(tolerance.1 / ry);

    if (dist_sq.sqrt() - 1.0).abs() <= norm_tol {
        // Only do the expensive math if we are actually near the edge
        let theta = local_y.atan2(local_x);
        let nearest_world_x = center.0 + (rx * theta.cos() * -cos_a - ry * theta.sin() * sin_a);
        let nearest_world_y = center.1 + (rx * theta.cos() * sin_a + ry * theta.sin() * -cos_a);

        let actual_dist = f32::hypot(point.0 - nearest_world_x, point.1 - nearest_world_y);
        Some(actual_dist)
    } else {
        None
    }
}

pub fn draw_elipse(
    center: (f32, f32),
    rx: f32,
    ry: f32,
    angle_rotation: f32,
    style: &'static DrawingStyle,
    shape_type: ShapeType,
) -> Vec<GateShape> {
    let degrees_rotation = -angle_rotation.to_degrees();
    vec![GateShape::Ellipse {
        center,
        radius_x: rx,
        radius_y: ry,
        degrees_rotation,
        style,
        shape_type,
    }]
}
pub fn calculate_ellipse_nodes(cx: f32, cy: f32, rx: f32, ry: f32, angle_rad: f32) -> Vec<(f32, f32)> {
    let (sin_a, cos_a) = angle_rad.sin_cos();

    vec![
        (cx, cy),                                   // 0. Center
        (cx + rx * cos_a, cy + rx * sin_a),         // 1. Right (Local X+)
        (cx + ry * sin_a, cy - ry * cos_a),         // 2. Top (Local Y-)
        (cx - rx * cos_a, cy - rx * sin_a),         // 3. Left (Local X-)
        (cx - ry * sin_a, cy + ry * cos_a),         // 4. Bottom (Local Y+)
    ]
}

pub fn draw_ghost_point_for_ellipse(
    curr_geo: &GateGeometry,
    drag_data: &PointDragData,
    x_param: &str,
    y_param: &str,
) -> Option<Vec<GateShape>> {
    let (cursor_x, cursor_y) = drag_data.loc();

    if let GateGeometry::Ellipse { center, radius_x, radius_y, angle } = curr_geo {
        let cx = center.get_coordinate(x_param).unwrap_or_default();
        let cy = center.get_coordinate(y_param).unwrap_or_default();
        let index = drag_data.point_index();

        let (new_rx, new_ry, new_angle) = calculate_projected_radii(
            (cursor_x, cursor_y), (cx, cy), *radius_x, *radius_y, *angle, index,
        );

        let (sin_n, cos_n) = new_angle.sin_cos();

        let ghost_circle_pos = match index {
            0 => (cursor_x, cursor_y),
            1 | 3 => {
                let proj = (cursor_x - cx) * cos_n + (cursor_y - cy) * sin_n;
                (cx + proj * cos_n, cy + proj * sin_n)
            }
            2 | 4 => {
                let proj = (cursor_x - cx) * sin_n - (cursor_y - cy) * cos_n;
                (cx + proj * sin_n, cy - proj * cos_n)
            }
            5 => {
                // let dist = new_ry + 20.0;
                // (cx - dist * sin_n, cy + dist * cos_n)
                let handle_distance = new_ry + 20.0; // Distance from center to the handle
    
                // Position the ghost dot relative to the rotated top of the ellipse
                // (cx - dist * sin, cy + dist * cos)
                let gx = cx - handle_distance * sin_n;
                let gy = cy + handle_distance * cos_n;
                
                (gx, gy)
            }
            _ => (cursor_x, cursor_y),
        };

        return Some(vec![
            GateShape::Circle {
                center: ghost_circle_pos,
                radius: 5.0,
                fill: "yellow",
                shape_type: ShapeType::GhostPoint,
            },
            GateShape::Ellipse {
                center: (cx, cy),
                radius_x: new_rx,
                radius_y: new_ry,
                degrees_rotation: (-new_angle).to_degrees(), // Standard SVG degrees
                style: &DRAGGED_LINE,
                shape_type: ShapeType::GhostPoint,
            },
        ]);
    }
    None
}

pub fn calculate_projected_radii(
    cursor: (f32, f32),
    center: (f32, f32),
    current_rx: f32,
    current_ry: f32,
    current_angle_rad: f32,
    point_index: usize,
) -> (f32, f32, f32) {
    let dx = cursor.0 - center.0;
    let dy = cursor.1 - center.1;
    let (sin_a, cos_a) = current_angle_rad.sin_cos();

    match point_index {
        1 | 3 => { // Horizontal Axis (Right/Left)
            let rx = (dx * cos_a + dy * sin_a).abs();
            (rx, current_ry, current_angle_rad)
        }
        2 | 4 => { // Vertical Axis (Top/Bottom)
            // Projects cursor onto the Minor Axis vector (sin, -cos)
            let ry = (dx * sin_a - dy * cos_a).abs();
            (current_rx, ry, current_angle_rad)
        }
        // 5 => { // Rotation Handle
        //     let mouse_angle = dy.atan2(dx); 
        //     let new_angle = mouse_angle + std::f32::consts::FRAC_PI_2; 
        //     (current_rx, current_ry, new_angle)
        // }
        5 => { // Rotation Handle
            let mouse_angle = dy.atan2(dx); 
            // In Y-up Data Space, Top is +PI/2. 
            // We subtract PI/2 to align the Ellipse's 0-degree axis.
            let new_angle = mouse_angle - std::f32::consts::FRAC_PI_2; 
            (current_rx, current_ry, new_angle)
        }
        _ => (current_rx, current_ry, current_angle_rad),
    }
}

pub fn update_ellipse_geometry(
    center: &GateNode,
    old_rx: f32,
    old_ry: f32,
    old_angle: f32,
    new_point: (f32, f32),
    point_index: usize,
    x_param: &str,
    y_param: &str,
) -> anyhow::Result<GateGeometry> {
    let cx = center.get_coordinate(x_param).unwrap_or(0.0);
    let cy = center.get_coordinate(y_param).unwrap_or(0.0);

    let (final_cx, final_cy, final_rx, final_ry, final_angle) = if point_index == 0 {
        (new_point.0, new_point.1, old_rx, old_ry, old_angle)
    } else {
        let (rx, ry, angle) = calculate_projected_radii(new_point, (cx, cy), old_rx, old_ry, old_angle, point_index);
        (cx, cy, rx, ry, angle)
    };

    // CALL THE HELPER HERE
    let sanitized_points = calculate_ellipse_nodes_y_up(final_cx, final_cy, final_rx, final_ry, final_angle);

    Ok(flow_gates::create_ellipse_geometry(sanitized_points, x_param, y_param)?)
}

pub fn calculate_ellipse_nodes_y_up(cx: f32, cy: f32, rx: f32, ry: f32, angle_rad: f32) -> Vec<(f32, f32)> {
    let (sin_a, cos_a) = angle_rad.sin_cos();

    vec![
        (cx, cy),                                   // 0. Center
        (cx + rx * cos_a, cy + rx * sin_a),         // 1. Right (Local X+)
        (cx - ry * sin_a, cy + ry * cos_a),         // 2. Top (Local Y+)
        (cx - rx * cos_a, cy - rx * sin_a),         // 3. Left (Local X-)
        (cx + ry * sin_a, cy - ry * cos_a),         // 4. Bottom (Local Y-)
    ]
}

pub fn create_default_ellipse(plot_map: &PlotMapper, cx_raw: f32, cy_raw: f32, rx_raw: f32, ry_raw: f32, x_channel: &str, y_channel: &str) -> anyhow::Result<GateGeometry> {
    let data_coords = plot_map
        .pixel_to_data(cx_raw, cy_raw, None, None);
    let (click_x, click_y) = data_coords;

    let edge_x_data = plot_map
        .pixel_to_data(cx_raw + rx_raw, cy_raw, None, None);
    let edge_y_data = plot_map
        .pixel_to_data(cx_raw, cy_raw + ry_raw, None, None);
    let rx = (edge_x_data.0 - click_x).abs();
    let ry = (edge_y_data.1 - click_y).abs();
    let coords = vec![
        (click_x, click_y),
        (click_x + rx, click_y),
        (click_x, click_y + ry),
        (click_x - rx, click_y),
        (click_x, click_y - ry),
    ];
    flow_gates::geometry::create_ellipse_geometry(
            coords,
            x_channel,
            y_channel,
        )
        .map_err(|_| anyhow::anyhow!("failed to create ellipse geometry"))
}