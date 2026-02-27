use std::sync::Arc;

use anyhow::anyhow;
use flow_fcs::TransformType;
use flow_gates::{
    GateGeometry, create_ellipse_geometry, create_polygon_geometry, create_rectangle_geometry,
};

use crate::plotters_dioxus::{
    axis_info::{asinh_reverse_f32, asinh_transform_f32},
    gates::{
        gate_drag::{GateDragData, PointDragData}, gate_draw_helpers::{
            ellipse::{
                calculate_ellipse_nodes, draw_elipse, draw_ghost_point_for_ellipse,
                is_point_on_ellipse_perimeter, update_ellipse_geometry,
            },
            line::{draw_circles_for_line, draw_line, is_point_on_line, update_line_geometry},
            polygon::{draw_ghost_point_for_polygon, draw_polygon, is_point_on_polygon_perimeter},
            rectangle::{
                draw_ghost_point_for_rectangle, draw_rectangle, is_point_on_rectangle_perimeter,
                update_rectangle_geometry,
            },
        }, gate_traits::DrawableGate, gate_types::{DEFAULT_LINE, GateRenderShape, SELECTED_LINE, ShapeType}
    },
};

#[derive(PartialEq, Clone)]
pub struct RectangleGate {
    inner: flow_gates::Gate,
    points: Vec<(f32, f32)>,
}

impl RectangleGate {
    pub fn try_new(gate: flow_gates::Gate) -> anyhow::Result<Self> {
        let p;
        if let GateGeometry::Rectangle { min, max } = &gate.geometry {
            let (x1, y1) = (
                min.get_coordinate(&gate.parameters.0),
                min.get_coordinate(&gate.parameters.1),
            );
            let (x2, y2) = (
                max.get_coordinate(&gate.parameters.0),
                max.get_coordinate(&gate.parameters.1),
            );
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (x1, y1, x2, y2) {
                p = vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)];
            } else {
                return Err(anyhow!(
                    "Invalid geometry for Rectangle Gate: invalid parameters"
                ));
            }
        } else {
            return Err(anyhow!(
                "Invalid geometry for Rectangle Gate: missing coordinates"
            ));
        }
        Ok(Self {
            inner: gate,
            points: p,
        })
    }
}

impl super::gate_traits::DrawableGate for RectangleGate {
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

    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        is_point_on_rectangle_perimeter(self, point, tolerance)
    }

    fn match_to_plot_axis(
        &self,
        plot_x: &str,
        plot_y: &str,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        let (x, y) = (&self.inner.parameters.0, &self.inner.parameters.1);
        if plot_x == x.as_ref() && plot_y == y.as_ref() {
            return Ok(None);
        }
        if plot_x == y.as_ref() && plot_y == x.as_ref() {
            let pts: Vec<_> = self.get_points().into_iter().map(|(x, y)| (y, x)).collect();
            let new_geometry = create_rectangle_geometry(pts, y, x)?;
            let new_parameters = (y.clone(), x.clone());
            let new_gate = flow_gates::Gate {
                id: self.inner.id.clone(),
                parameters: new_parameters,
                geometry: new_geometry,
                label_position: self.inner.label_position.clone(),
                name: self.inner.name.clone(),
                mode: self.inner.mode.clone(),
            };
            return Ok(Some(Box::new(RectangleGate::try_new(new_gate)?)));
        }
        Err(anyhow!("Axis mismatch for Rectangle Gate"))
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let p = self.get_points();
        let new_geometry = update_rectangle_geometry(
            p,
            new_point,
            point_index,
            &self.inner.parameters.0,
            &self.inner.parameters.1,
        )?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(RectangleGate::try_new(new_gate)?))
    }

    fn replace_points(
        &self,
        gate_drag_data: GateDragData,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let x_offset = gate_drag_data.offset().0;
        let y_offset = gate_drag_data.offset().1;
        let points = self
            .get_points()
            .into_iter()
            .map(|(x, y)| (x - x_offset, y - y_offset))
            .collect();
        let new_geometry =
            create_rectangle_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(RectangleGate::try_new(new_gate)?))
    }

    fn rotate_gate(&self, _mouse_pos: (f32, f32)) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        Ok(None)
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: Arc<str>,
        old: &TransformType,
        new: &TransformType,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let points = rescale_helper(
            &self.get_points(),
            &param,
            &self.inner.parameters.0,
            old,
            new,
        )?;
        let new_geometry =
            create_rectangle_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(RectangleGate::try_new(new_gate)?))
    }

    fn get_points(&self) -> Vec<(f32, f32)> {
        self.points.clone()
    }

    fn is_finalised(&self) -> bool {
        true
    }

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
    ) -> Vec<GateRenderShape> {
        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };
        let pts = &self.points;
        let main = draw_rectangle(
            pts[0],
            pts[2],
            style,
            ShapeType::Gate(self.inner.id.clone()),
        );
        let selected = if is_selected {
            Some(draw_circles_for_selected_gate(&pts, 0))
        } else {
            None
        };
        let ghost = drag_point
            .as_ref()
            .and_then(|d| draw_ghost_point_for_rectangle(d, &pts));
        crate::collate_vecs!(main, selected, ghost)
    }
}

#[derive(PartialEq, Clone)]
pub struct PolygonGate {
    inner: flow_gates::Gate,
    points: Vec<(f32, f32)>,
}

impl PolygonGate {
    pub fn try_new(gate: flow_gates::Gate) -> anyhow::Result<Self> {
        let p;
        if let GateGeometry::Polygon { nodes, .. } = &gate.geometry {
            p = nodes
                .iter()
                .filter_map(|n| {
                    Some((
                        n.get_coordinate(&gate.parameters.0)?,
                        n.get_coordinate(&gate.parameters.1)?,
                    ))
                })
                .collect();
        } else {
            return Err(anyhow!("Invalid geometry for Polygon Gate"));
        }
        Ok(Self {
            inner: gate,
            points: p,
        })
    }
}

impl super::gate_traits::DrawableGate for PolygonGate {
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

    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        is_point_on_polygon_perimeter(self, point, tolerance)
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
            let pts: Vec<_> = self.get_points().into_iter().map(|(x, y)| (y, x)).collect();
            let new_geometry = create_polygon_geometry(pts, y, x)?;
            let new_parameters = (y.clone(), x.clone());
            let new_gate = flow_gates::Gate {
                id: self.inner.id.clone(),
                parameters: new_parameters,
                geometry: new_geometry,
                label_position: self.inner.label_position.clone(),
                name: self.inner.name.clone(),
                mode: self.inner.mode.clone(),
            };
            return Ok(Some(Box::new(PolygonGate::try_new(new_gate)?)));
        }
        Err(anyhow!("Axis mismatch for Polygon Gate"))
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let mut p = self.get_points();
        if point_index < p.len() {
            p[point_index] = new_point;
        }
        let new_geometry =
            create_polygon_geometry(p, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(PolygonGate::try_new(new_gate)?))
    }

    fn replace_points(
        &self,
        gate_drag_data: GateDragData,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let x_offset = gate_drag_data.offset().0;
        let y_offset = gate_drag_data.offset().1;
        let points = self
            .get_points()
            .into_iter()
            .map(|(x, y)| (x - x_offset, y - y_offset))
            .collect();
        let new_geometry =
            create_polygon_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(PolygonGate::try_new(new_gate)?))
    }

    fn rotate_gate(&self, _mouse_pos: (f32, f32)) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        Ok(None)
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: Arc<str>,
        old: &TransformType,
        new: &TransformType,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let points = rescale_helper(
            &self.get_points(),
            &param,
            &self.inner.parameters.0,
            old,
            new,
        )?;
        let new_geometry =
            create_polygon_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        Ok(Box::new(PolygonGate::try_new(new_gate)?))
    }

    fn get_points(&self) -> Vec<(f32, f32)> {
        if let GateGeometry::Polygon { nodes, .. } = &self.inner.geometry {
            return nodes
                .iter()
                .filter_map(|n| {
                    Some((
                        n.get_coordinate(&self.inner.parameters.0)?,
                        n.get_coordinate(&self.inner.parameters.1)?,
                    ))
                })
                .collect();
        }
        vec![]
    }
    fn is_finalised(&self) -> bool {
        true
    }
    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
    ) -> Vec<GateRenderShape> {
        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };
        let pts = self.get_points();
        let main = draw_polygon(&pts, style, ShapeType::Gate(self.inner.id.clone()));
        let selected = if is_selected {
            Some(draw_circles_for_selected_gate(&pts, 0))
        } else {
            None
        };
        let ghost = drag_point
            .as_ref()
            .and_then(|d| draw_ghost_point_for_polygon(d, &pts));
        crate::collate_vecs!(main, selected, ghost)
    }
}

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
}

impl super::gate_traits::DrawableGate for EllipseGate {
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

    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
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
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
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
        Ok(Box::new(EllipseGate::try_new(new_gate)?))
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
    fn is_finalised(&self) -> bool {
        true
    }
    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
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
}

#[derive(PartialEq, Clone)]
pub struct LineGate {
    pub inner: flow_gates::Gate,
    points: Vec<(f32, f32)>,
    pub height: f32,
    pub axis_matched: bool,
}

impl LineGate {
    pub fn try_new(gate: flow_gates::Gate, height: f32) -> anyhow::Result<Self> {
        let p = {
            if let GateGeometry::Rectangle { min, max } = &gate.geometry {
                let (x1, y1) = (
                    min.get_coordinate(&gate.parameters.0),
                    min.get_coordinate(&gate.parameters.1),
                );
                let (x2, y2) = (
                    max.get_coordinate(&gate.parameters.0),
                    max.get_coordinate(&gate.parameters.1),
                );
                if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (x1, y1, x2, y2) {
                    vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)]
                } else {
                    return Err(anyhow!("Invalid points for Line Gate"));
                }
            } else {
                return Err(anyhow!("Invalid points for Line Gate"));
            }
        };

        Ok(Self {
            inner: gate,
            points: p,
            height: height,
            axis_matched: true,
        })
    }

    pub fn clone_line_for_rescaled_axis(
        &self,
        param: Arc<str>,
        old: &TransformType,
        new: &TransformType,
    ) -> anyhow::Result<Self> {
        let points = rescale_helper(
            &self.get_points(),
            &param,
            &self.inner.parameters.0,
            old,
            new,
        )?;
        let new_geometry =
            create_rectangle_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.inner.parameters.clone(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        let mut line = LineGate::try_new(new_gate, self.height)?;
        line.axis_matched = self.axis_matched;
        Ok(line)
    }

    pub fn clone_line_for_axis_swap(&self,
        plot_x: &str,
        plot_y: &str,
    ) -> anyhow::Result<Option<Self>> {
        let (x, y) = (&self.inner.parameters.0, &self.inner.parameters.1);
        if plot_x == x.as_ref() && *plot_y == *y.as_ref() {
            return Ok(None);
        }

        if plot_x == y.as_ref() && plot_y == x.as_ref() {
            let pts: Vec<_> = self.get_points().into_iter().map(|(x, y)| (y, x)).collect();
            let new_geometry = create_rectangle_geometry(pts, y, x)?;
            let new_parameters = (y.clone(), x.clone());
            let new_axis_matched = !self.axis_matched;
            let new_gate = flow_gates::Gate {
                id: self.inner.id.clone(),
                parameters: new_parameters,
                geometry: new_geometry,
                label_position: self.inner.label_position.clone(),
                name: self.inner.name.clone(),
                mode: self.inner.mode.clone(),
            };
            let mut new_line = LineGate::try_new(new_gate, self.height)?;
            new_line.axis_matched = new_axis_matched;
            println!("axis not matched, rotating gate");
            return Ok(Some(new_line));
        }
        Err(anyhow!("Axis mismatch for Line Gate"))
    }

    pub fn clone_line_for_new_point(&self,
        new_point: (f32, f32),
        point_index: usize,) -> anyhow::Result<Self> {
        let p = self.get_points();
        let new_geometry = update_line_geometry(
            p,
            new_point,
            point_index,
            &self.inner.parameters.0,
            &self.inner.parameters.1,
            self.axis_matched,
        )?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.get_params(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        let mut new_line = LineGate::try_new(new_gate, self.height)?;
        new_line.axis_matched = self.axis_matched;
        return Ok(new_line);
    }
}

impl super::gate_traits::DrawableGate for LineGate {
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

    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        is_point_on_line(self, point, tolerance, self.axis_matched)
    }

    fn match_to_plot_axis(
        &self,
        plot_x: &str,
        plot_y: &str,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        let line = self.clone_line_for_axis_swap(plot_x, plot_y)?;
        match line{
            Some(l) => Ok(Some(Box::new(l))),
            None => Ok(None),
        }
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let line = self.clone_line_for_new_point(new_point, point_index)?;
        return Ok(Box::new(line));
    }

    fn replace_points(
        &self,
        gate_drag_data: GateDragData,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let x_offset = gate_drag_data.offset().0;
        let y_offset = gate_drag_data.offset().1;
        let height;
        let points: Vec<(f32, f32)> = match self.axis_matched {
            true => {
                height = gate_drag_data.current_loc().1;
                self.get_points()
                    .into_iter()
                    .map(|(x, y)| (x - x_offset, y))
                    .collect()
            }
            false => {
                height = gate_drag_data.current_loc().0;
                self.get_points()
                    .into_iter()
                    .map(|(x, y)| (x, y - y_offset))
                    .collect()
            }
        };

        if points.len() != 4 {
            return Err(anyhow!("Line gate geometry must have exactly 4 points"));
        }
        let new_geometry =
            create_rectangle_geometry(points, &self.inner.parameters.0, &self.inner.parameters.1)?;
        let new_gate = flow_gates::Gate {
            id: self.inner.id.clone(),
            parameters: self.get_params(),
            geometry: new_geometry,
            label_position: self.inner.label_position.clone(),
            name: self.inner.name.clone(),
            mode: self.inner.mode.clone(),
        };
        let mut new_line = LineGate::try_new(new_gate, height)?;
        new_line.axis_matched = self.axis_matched;
        return Ok(Box::new(new_line));
    }

    fn rotate_gate(&self, _mouse_pos: (f32, f32)) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        Ok(None)
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: Arc<str>,
        old: &TransformType,
        new: &TransformType,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        let line = self.clone_line_for_rescaled_axis(param, old, new)?;
        Ok(Box::new(line))
    }

    fn get_points(&self) -> Vec<(f32, f32)> {
        if let GateGeometry::Rectangle { min, max } = &self.inner.geometry {
            let (x1, y1) = (
                min.get_coordinate(&self.inner.parameters.0),
                min.get_coordinate(&self.inner.parameters.1),
            );
            let (x2, y2) = (
                max.get_coordinate(&self.inner.parameters.0),
                max.get_coordinate(&self.inner.parameters.1),
            );
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (x1, y1, x2, y2) {
                return vec![(x1, y1), (x2, y1), (x2, y2), (x1, y2)];
            }
        }
        vec![]
    }
    fn is_finalised(&self) -> bool {
        true
    }
    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
    ) -> Vec<GateRenderShape> {
        let style = if is_selected {
            &SELECTED_LINE
        } else {
            &DEFAULT_LINE
        };
        let pts = self.get_points();
        let main = draw_line(
            pts[0],
            pts[2],
            self.height,
            style,
            ShapeType::Gate(self.inner.id.clone()),
            &drag_point,
            self.axis_matched,
        );
        let selected = if is_selected {
            Some(draw_circles_for_line(
                pts[0],
                pts[2],
                self.height,
                &drag_point,
                self.axis_matched,
            ))
        } else {
            None
        };
        crate::collate_vecs!(main, selected)
    }
}

pub fn draw_circles_for_selected_gate(
    points: &[(f32, f32)],
    index_offset: usize,
) -> Vec<GateRenderShape> {
    points
        .iter()
        .enumerate()
        .map(|(idx, p)| GateRenderShape::Circle {
            center: *p,
            radius: 3.0,
            fill: "red",
            shape_type: ShapeType::Point(idx + index_offset),
        })
        .collect()
}

fn rescale_helper(
    pts: &[(f32, f32)],
    param: &str,
    x_param: &str,
    old: &TransformType,
    new: &TransformType,
) -> anyhow::Result<Vec<(f32, f32)>> {
    let is_x = x_param == param;
    let mut new_pts = pts.to_vec();
    for p in new_pts.iter_mut() {
        let val = if is_x { &mut p.0 } else { &mut p.1 };
        let raw = match old {
            TransformType::Arcsinh { cofactor } => {
                asinh_reverse_f32(*val, *cofactor).unwrap_or(*val)
            }
            _ => *val,
        };
        *val = match new {
            TransformType::Arcsinh { cofactor } => {
                asinh_transform_f32(raw, *cofactor).unwrap_or(raw)
            }
            _ => raw,
        };
    }
    Ok(new_pts)
}
