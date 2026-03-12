use std::sync::Arc;

use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{GateGeometry, create_ellipse_geometry};

use crate::plotters_dioxus::{
    gates::{
        gate_drag::{GateDragData, PointDragData},
        gate_single::{draw_circles_for_selected_gate, rescale_helper},
        gate_traits::DrawableGate,
        gate_types::{DEFAULT_LINE, GateRenderShape, SELECTED_LINE, ShapeType},
    },
    plot_helpers::PlotMapper,
};

#[derive(PartialEq, Clone)]
pub struct EllipseGate {
    pub inner: flow_gates::Gate,
    points: Vec<(f32, f32)>,
}

impl EllipseGate {
    pub fn try_new(gate: flow_gates::Gate) -> anyhow::Result<Self> {
        let p = {
            if let GateGeometry::Ellipse {
                center,
                radius_x,
                radius_y,
                angle,
            } = &gate.geometry
            {
                let cx = center.get_coordinate(&gate.parameters.0);
                let cy = center.get_coordinate(&gate.parameters.1);
                if let (Some(cx), Some(cy)) = (cx, cy) {
                    calculate_ellipse_nodes(cx, cy, *radius_x, *radius_y, *angle)
                } else {
                    return Err(anyhow!("Invalid points for Ellipse Gate"));
                }
            } else {
                return Err(anyhow!("Invalid geometry for Ellipse Gate"));
            }
        };
        Ok(Self {
            inner: gate,
            points: p,
        })
    }

    fn get_points(&self) -> Vec<(f32, f32)> {
        if let GateGeometry::Ellipse {
            center,
            radius_x,
            radius_y,
            angle,
        } = &self.inner.geometry
        {
            let cx = center.get_coordinate(&self.inner.parameters.0);
            let cy = center.get_coordinate(&self.inner.parameters.1);
            if let (Some(cx), Some(cy)) = (cx, cy) {
                return calculate_ellipse_nodes(cx, cy, *radius_x, *radius_y, *angle);
            }
        }
        vec![]
    }
}

impl DrawableGate for EllipseGate {
    fn clone_box(&self) -> Box<dyn DrawableGate> {
        Box::new(self.clone())
    }

    fn get_id(&self) -> Arc<str> {
        self.inner.id.clone()
    }
    fn is_composite(&self) -> bool {
        false
    }
    fn get_params(&self) -> (Arc<str>, Arc<str>) {
        self.inner.parameters.clone()
    }

    fn is_point_on_perimeter(
        &self,
        point: (f32, f32),
        tolerance: (f32, f32),
        _mapper: &PlotMapper,
    ) -> Option<f32> {
        if let GateGeometry::Ellipse {
            center,
            radius_x,
            radius_y,
            angle,
        } = &self.inner.geometry
        {
            let cx = center
                .get_coordinate(&self.inner.parameters.0)
                .unwrap_or_default();
            let cy = center
                .get_coordinate(&self.inner.parameters.1)
                .unwrap_or_default();
            return is_point_on_ellipse_perimeter(
                point,
                (cx, cy),
                *radius_x,
                *radius_y,
                *angle,
                tolerance,
            );
        }
        None
    }

    fn match_to_plot_axis(
        &self,
        plot_x: &str,
        plot_y: &str,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        let (x, y) = (&self.inner.parameters.0, &self.inner.parameters.1);
        if plot_x == x.as_ref() && *plot_y == *y.as_ref() {
            return Ok(None);
        }
        if plot_x == y.as_ref() && plot_y == x.as_ref() {
            let p = self.get_points();
            let mirrored = vec![
                (p[0].1, p[0].0),
                (p[3].1, p[3].0),
                (p[4].1, p[4].0),
                (p[1].1, p[1].0),
                (p[2].1, p[2].0),
            ];
            let new_geometry = create_ellipse_geometry(mirrored, y, x)?;
            let new_parameters = (y.clone(), x.clone());
            let new_gate = flow_gates::Gate {
                id: self.inner.id.clone(),
                parameters: new_parameters,
                geometry: new_geometry,
                label_position: self.inner.label_position.clone(),
                name: self.inner.name.clone(),
                mode: self.inner.mode.clone(),
            };
            return Ok(Some(Box::new(EllipseGate::try_new(new_gate)?)));
        }

        Err(anyhow!("Axis mismatch for Ellipse Gate"))
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
        _mapper: &PlotMapper,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let new_geometry;
        if let GateGeometry::Ellipse {
            center,
            radius_x,
            radius_y,
            angle,
        } = &self.inner.geometry
        {
            new_geometry = update_ellipse_geometry(
                center,
                *radius_x,
                *radius_y,
                *angle,
                new_point,
                point_index,
                &self.inner.parameters.0,
                &self.inner.parameters.1,
            )?;
        } else {
            return Err(anyhow!("Error replacing point in Ellipse"));
        }
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(EllipseGate::try_new(new_gate)?))
    }

    fn replace_points(
        &self,
        gate_drag_data: GateDragData,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        let x_offset = gate_drag_data.offset().0;
        let y_offset = gate_drag_data.offset().1;
        let points = self
            .get_points()
            .into_iter()
            .map(|(x, y)| (x - x_offset, y - y_offset))
            .collect();

        let new_geometry =
            create_ellipse_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Some(Box::new(EllipseGate::try_new(new_gate)?)))
    }

    fn rotate_gate(&self, mouse_pos: (f32, f32)) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        let new_geometry;
        if let GateGeometry::Ellipse {
            center,
            radius_x,
            radius_y,
            angle,
        } = &self.inner.geometry
        {
            new_geometry = update_ellipse_geometry(
                center,
                *radius_x,
                *radius_y,
                *angle,
                mouse_pos,
                5,
                &self.inner.parameters.0,
                &self.inner.parameters.1,
            )?;
        } else {
            return Err(anyhow!("Error rotating Ellipse"));
        }
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Some(Box::new(EllipseGate::try_new(new_gate)?)))
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: Arc<str>,
        old: &TransformType,
        new: &TransformType,
        _data_range: (f32, f32)
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let points = rescale_helper(
            &self.get_points(),
            &param,
            &self.inner.parameters.0,
            old,
            new,
        )?;
        let new_geometry =
            create_ellipse_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(EllipseGate::try_new(new_gate)?))
    }

    
    fn is_finalised(&self) -> bool {
        true
    }
    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
        _plot_map: &PlotMapper,
    ) -> Vec<GateRenderShape> {
        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };
        let pts = self.get_points();
        if let GateGeometry::Ellipse {
            radius_x,
            radius_y,
            angle,
            ..
        } = &self.inner.geometry
        {
            let main = draw_elipse(
                pts[0],
                *radius_x,
                *radius_y,
                *angle,
                style,
                ShapeType::Gate(self.inner.id.clone()),
            );
            let selected = if is_selected {
                let mut c = draw_circles_for_selected_gate(&pts[1..], 1);
                c.push(GateRenderShape::Handle {
                    center: (pts[0].0, pts[0].1 + *radius_y),
                    size: 5.0,
                    shape_center: pts[0],
                    shape_type: ShapeType::Rotation(*angle),
                });
                Some(c)
            } else {
                None
            };
            let ghost = drag_point.as_ref().and_then(|d| {
                draw_ghost_point_for_ellipse(
                    &self.inner.geometry,
                    d,
                    &self.inner.parameters.0,
                    &self.inner.parameters.1,
                )
            });
            return crate::collate_vecs!(main, selected, ghost);
        }
        vec![]
    }
    
    fn get_gate_ref(&self, _id: Option<&str>) -> Option<&flow_gates::Gate>  {
        Some(&self.inner)
    }
    fn get_inner_gate_ids(&self) -> Vec<Arc<str>>{
        vec![self.inner.id.clone()]
    }
    fn recalculate_gate_for_new_axis_limits(
        &self,
        _param: std::sync::Arc<str>,
        _lower: f32,
        _upper: f32,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        Ok(None)
    }
}

use crate::plotters_dioxus::gates::gate_types::{DRAGGED_LINE, DrawingStyle};
use flow_gates::GateNode;

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
) -> Vec<GateRenderShape> {
    let degrees_rotation = -angle_rotation.to_degrees();
    vec![GateRenderShape::Ellipse {
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
        (cx, cy),                           // 0. Center
        (cx + rx * cos_a, cy + rx * sin_a), // 1. Right (Local X+)
        (cx + ry * sin_a, cy - ry * cos_a), // 2. Top (Local Y-)
        (cx - rx * cos_a, cy - rx * sin_a), // 3. Left (Local X-)
        (cx - ry * sin_a, cy + ry * cos_a), // 4. Bottom (Local Y+)
    ]
}

pub fn draw_ghost_point_for_ellipse(
    curr_geo: &GateGeometry,
    drag_data: &PointDragData,
    x_param: &str,
    y_param: &str,
) -> Option<Vec<GateRenderShape>> {
    let (cursor_x, cursor_y) = drag_data.loc();

    if let GateGeometry::Ellipse {
        center,
        radius_x,
        radius_y,
        angle,
    } = curr_geo
    {
        let cx = center.get_coordinate(x_param).unwrap_or_default();
        let cy = center.get_coordinate(y_param).unwrap_or_default();
        let index = drag_data.point_index();

        let (new_rx, new_ry, new_angle) = calculate_projected_radii(
            (cursor_x, cursor_y),
            (cx, cy),
            *radius_x,
            *radius_y,
            *angle,
            index,
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
            GateRenderShape::Circle {
                center: ghost_circle_pos,
                radius: 5.0,
                fill: "yellow",
                shape_type: ShapeType::GhostPoint,
            },
            GateRenderShape::Ellipse {
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
        1 | 3 => {
            // Horizontal Axis (Right/Left)
            let rx = (dx * cos_a + dy * sin_a).abs();
            (rx, current_ry, current_angle_rad)
        }
        2 | 4 => {
            // Vertical Axis (Top/Bottom)
            // Projects cursor onto the Minor Axis vector (sin, -cos)
            let ry = (dx * sin_a - dy * cos_a).abs();
            (current_rx, ry, current_angle_rad)
        }
        // 5 => { // Rotation Handle
        //     let mouse_angle = dy.atan2(dx);
        //     let new_angle = mouse_angle + std::f32::consts::FRAC_PI_2;
        //     (current_rx, current_ry, new_angle)
        // }
        5 => {
            // Rotation Handle
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
        let (rx, ry, angle) =
            calculate_projected_radii(new_point, (cx, cy), old_rx, old_ry, old_angle, point_index);
        (cx, cy, rx, ry, angle)
    };

    // CALL THE HELPER HERE
    let sanitized_points =
        calculate_ellipse_nodes_y_up(final_cx, final_cy, final_rx, final_ry, final_angle);

    Ok(flow_gates::create_ellipse_geometry(
        sanitized_points,
        x_param,
        y_param,
    )?)
}

pub fn calculate_ellipse_nodes_y_up(
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    angle_rad: f32,
) -> Vec<(f32, f32)> {
    let (sin_a, cos_a) = angle_rad.sin_cos();

    vec![
        (cx, cy),                           // 0. Center
        (cx + rx * cos_a, cy + rx * sin_a), // 1. Right (Local X+)
        (cx - ry * sin_a, cy + ry * cos_a), // 2. Top (Local Y+)
        (cx - rx * cos_a, cy - rx * sin_a), // 3. Left (Local X-)
        (cx + ry * sin_a, cy - ry * cos_a), // 4. Bottom (Local Y-)
    ]
}

pub fn create_default_ellipse(
    plot_map: &PlotMapper,
    cx_raw: f32,
    cy_raw: f32,
    rx_raw: f32,
    ry_raw: f32,
    x_channel: &str,
    y_channel: &str,
) -> anyhow::Result<GateGeometry> {
    let data_coords = plot_map.pixel_to_data(cx_raw, cy_raw, None, None);
    let (click_x, click_y) = data_coords;

    let edge_x_data = plot_map.pixel_to_data(cx_raw + rx_raw, cy_raw, None, None);
    let edge_y_data = plot_map.pixel_to_data(cx_raw, cy_raw + ry_raw, None, None);
    let rx = (edge_x_data.0 - click_x).abs();
    let ry = (edge_y_data.1 - click_y).abs();
    let coords = vec![
        (click_x, click_y),
        (click_x + rx, click_y),
        (click_x, click_y + ry),
        (click_x - rx, click_y),
        (click_x, click_y - ry),
    ];
    flow_gates::geometry::create_ellipse_geometry(coords, x_channel, y_channel)
        .map_err(|_| anyhow::anyhow!("failed to create ellipse geometry"))
}
