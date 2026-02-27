use std::sync::Arc;

use flow_fcs::TransformType;

use crate::plotters_dioxus::gates::{
    GateId, gate_drag::{GateDragData, PointDragData}, gate_types::GateRenderShape
};

pub trait DrawableGate {
    fn get_points(&self) -> Vec<(f32, f32)>;
    fn is_finalised(&self) -> bool;

    fn draw_self(
        &self,
        is_selected: bool,
        drag_point: Option<PointDragData>,
    ) -> Vec<GateRenderShape>;

    fn is_near_segment(
        &self,
        m: (f32, f32),
        a: (f32, f32),
        b: (f32, f32),
        tolerance: (f32, f32),
    ) -> Option<f32> {
        let (tol_x, tol_y) = tolerance;
        let dx = b.0 - a.0;
        let dy = b.1 - a.1;
        let length_sq = dx * dx + dy * dy;

        // 1. Find the nearest point on the segment
        let t_clamped = if length_sq == 0.0 {
            0.0
        } else {
            (((m.0 - a.0) * dx + (m.1 - a.1) * dy) / length_sq).clamp(0.0, 1.0)
        };

        let nearest_x = a.0 + t_clamped * dx;
        let nearest_y = a.1 + t_clamped * dy;

        // 2. Check the rectangular tolerance box
        let diff_x = (m.0 - nearest_x).abs();
        let diff_y = (m.1 - nearest_y).abs();

        if diff_x <= tol_x && diff_y <= tol_y {
            // 3. Return the actual Euclidean distance in data space
            let actual_dist = (diff_x.powi(2) + diff_y.powi(2)).sqrt();
            Some(actual_dist)
        } else {
            None
        }
    }

    fn is_composite(&self) -> bool;

    fn get_id(&self) -> Arc<str>;

    fn get_params(&self) -> (Arc<str>, Arc<str>);

    // fn is_selected(&self) -> bool;

    // fn set_selected(&mut self, state: bool);

    // fn is_drag_point(&self) -> bool;

    // fn set_drag_point(&mut self, drag_data: Option<PointDragData>);

    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32>;

    fn match_to_plot_axis(
        &self,
        plot_x_param: &str,
        plot_y_param: &str,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>>;

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<Box<dyn DrawableGate>>;

    fn rotate_gate(
        &self,
        mouse_position: (f32, f32),
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>>;

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
    ) -> anyhow::Result<Box<dyn DrawableGate>>;

    fn replace_points(&self, gate_drag_data: GateDragData)
    -> anyhow::Result<Box<dyn DrawableGate>>;

    fn clone_box(&self) -> Box<dyn DrawableGate>;
}

impl Clone for Box<dyn DrawableGate> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
