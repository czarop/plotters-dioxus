use flow_gates::{GateGeometry, GateNode};

use crate::plotters_dioxus::gates::{
    gate_drag::PointDragData,
    gate_styles::{DRAGGED_LINE, DrawingStyle, GateShape, ShapeType},
};

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
    let degrees_rotation = angle_rotation.to_degrees();
    vec![GateShape::Ellipse {
        center,
        radius_x: rx,
        radius_y: ry,
        degrees_rotation,
        style,
        shape_type,
    }]
}

pub fn calculate_ellipse_nodes(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    angle_rad: f32,
) -> Vec<(f32, f32)> {
    let (sin_a, cos_a) = angle_rad.sin_cos();

    vec![
        // 0. Center
        (cx, cy),
        // 1. Right (rx, 0)
        (cx + rx * cos_a, cy + rx * sin_a),
        // 2. Top (0, -ry)
        (cx + ry * sin_a, cy - ry * cos_a),
        // 3. Left (-rx, 0)
        (cx - rx * cos_a, cy - rx * sin_a),
        // 4. Bottom (0, ry)
        (cx - ry * sin_a, cy + ry * cos_a),
    ]
}

pub fn draw_ghost_point_for_ellipse(
    curr_geo: &GateGeometry,
    drag_data: &PointDragData,
    x_param: &str,
    y_param: &str,
) -> Option<Vec<GateShape>> {
    let (cursor_x, cursor_y) = drag_data.loc();

    if let GateGeometry::Ellipse {
        center: current_center,
        radius_x,
        radius_y,
        angle,
    } = curr_geo
    {
        let current_cx = current_center.get_coordinate(x_param).unwrap_or_default();
        let current_cy = current_center.get_coordinate(y_param).unwrap_or_default();
        let index = drag_data.point_index();

        // 1. Use the shared helper for radii
        let (new_rx, new_ry, new_angle) = calculate_projected_radii(
            (cursor_x, cursor_y),
            (current_cx, current_cy),
            *radius_x,
            *radius_y,
            *angle,
            index,
        );

        // 2. Calculate the snapped ghost point position
        let (sin_n, cos_n) = new_angle.sin_cos();
        // let dx = cursor_x - current_cx;
        // let dy = cursor_y - current_cy;

        // let ghost_circle_pos = match index {
        //     0 => (cursor_x, cursor_y), // Center moves freely
        //     1 | 3 => {
        //         let proj_x = dx * cos_a + dy * sin_a;
        //         (current_cx + proj_x * cos_a, current_cy + proj_x * sin_a)
        //     }
        //     2 | 4 => {
        //         let proj_y = dx * -sin_a + dy * cos_a;
        //         (current_cx + proj_y * -sin_a, current_cy + proj_y * cos_a)
        //     }
        //     _ => (cursor_x, cursor_y),
        // };

        let ghost_circle_pos = match index {
            0 => (cursor_x, cursor_y),
            1 | 3 => {
                // Snap to Major Axis
                let dx = cursor_x - current_cx;
                let dy = cursor_y - current_cy;
                let proj = dx * cos_n + dy * sin_n;
                (current_cx + proj * cos_n, current_cy + proj * sin_n)
            }
            2 | 4 => {
                // Snap to Minor Axis
                let dx = cursor_x - current_cx;
                let dy = cursor_y - current_cy;
                let proj = dx * -sin_n + dy * cos_n;
                (current_cx + proj * -sin_n, current_cy + proj * cos_n)
            }
            5 => {
                // Snap to Rotation Orbit
                // The dot stays at a fixed distance (e.g., ry + 20) but follows the angle
                let dist = new_ry + 20.0;
                (current_cx + dist * sin_n, current_cy - dist * cos_n)
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
                center: (current_cx, current_cy),
                radius_x: new_rx,
                radius_y: new_ry,
                degrees_rotation: angle.to_degrees(),
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
        // Handle Horizontal Axis (Right/Left)
        1 | 3 => {
            let rx = (dx * cos_a + dy * sin_a).abs();
            (rx, current_ry, current_angle_rad)
        }
        // Handle Vertical Axis (Top/Bottom)
        2 | 4 => {
            let ry = (dx * -sin_a + dy * cos_a).abs();
            (current_rx, ry, current_angle_rad)
        }
        // Handle Rotation Handle
        5 => {
            // Calculate the angle from center to cursor.
            // Since our 'Top' handle (index 2) is at -90 degrees (or -PI/2)
            // relative to the local X axis, we adjust the atan2 result.
            let mouse_angle = dy.atan2(dx);
            let new_angle = mouse_angle + std::f32::consts::FRAC_PI_2;

            (current_rx, current_ry, new_angle)
        }
        // Default (Center or no-op)
        _ => (current_rx, current_ry, current_angle_rad),
    }
}

// pub fn update_ellipse_geometry(
//     center: &GateNode,
//     old_rx: f32,
//     old_ry: f32,
//     old_angle: f32,
//     new_point: (f32, f32),
//     point_index: usize,
//     x_param: &str,
//     y_param: &str,
// ) -> anyhow::Result<GateGeometry> {
//     println!("called");
//     let cx = center.get_coordinate(x_param).unwrap_or(0.0);
//     let cy = center.get_coordinate(y_param).unwrap_or(0.0);

//     // 1. Calculate new radii (and eventually angle)
//     let (rx, ry, new_angle) =
//         calculate_projected_radii(new_point, (cx, cy), old_rx, old_ry, old_angle, point_index);

//     // 2. Reconstruct the 5 points the library expects
//     let (sin_a, cos_a) = new_angle.sin_cos();
//     let sanitized_points = vec![
//         (cx, cy),
//         (cx + rx * cos_a, cy + rx * sin_a), // Right
//         (cx + ry * sin_a, cy - ry * cos_a), // Top
//         (cx - rx * cos_a, cy - rx * sin_a), // Left
//         (cx - ry * sin_a, cy + ry * cos_a), // Bottom
//     ];

//     Ok(flow_gates::create_ellipse_geometry(
//         sanitized_points,
//         x_param,
//         y_param,
//     )?)
// }

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

    // If point_index is 0, we are moving the center (Translation)
    if point_index == 0 {
        // Re-construct with new center, same radii/angle
        let (sin_a, cos_a) = old_angle.sin_cos();
        let sanitized_points = vec![
            new_point, // The new center
            (new_point.0 + old_rx * cos_a, new_point.1 + old_rx * sin_a),
            (new_point.0 + old_ry * sin_a, new_point.1 - old_ry * cos_a),
            (new_point.0 - old_rx * cos_a, new_point.1 - old_rx * sin_a),
            (new_point.0 - old_ry * sin_a, new_point.1 + old_ry * cos_a),
        ];
        return Ok(flow_gates::create_ellipse_geometry(
            sanitized_points,
            x_param,
            y_param,
        )?);
    }

    // Otherwise, we are resizing (1-4) or rotating (5)
    let (rx, ry, new_angle) =
        calculate_projected_radii(new_point, (cx, cy), old_rx, old_ry, old_angle, point_index);

    let (sin_a, cos_a) = new_angle.sin_cos();
    let sanitized_points = vec![
        (cx, cy),
        (cx + rx * cos_a, cy + rx * sin_a),
        (cx + ry * sin_a, cy - ry * cos_a),
        (cx - rx * cos_a, cy - rx * sin_a),
        (cx - ry * sin_a, cy + ry * cos_a),
    ];

    Ok(flow_gates::create_ellipse_geometry(
        sanitized_points,
        x_param,
        y_param,
    )?)
}
