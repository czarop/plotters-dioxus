use std::sync::Arc;

use flow_fcs::TransformType;

use crate::plotters_dioxus::{gates::{gate_drag::{GateDragData, PointDragData}, gate_types::GateRenderShape}};

pub trait DrawableGate{

    fn get_points(&self) -> Vec<(f32, f32)>;
    fn is_finalised(&self) -> bool;

    fn draw_self(&self) -> Vec<GateRenderShape>;

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

    fn match_to_plot_axis(&mut self, plot_x_param: &str, plot_y_param: &str) -> anyhow::Result<()>;

    fn recalculate_gate_for_rescaled_axis(
        &mut self,
        param: std::sync::Arc<str>,
        old_transform: &TransformType,
        new_transform: &TransformType,
    ) -> anyhow::Result<()>;

    fn rotate_gate(&mut self, mouse_position: (f32, f32)) -> anyhow::Result<()>;

    fn replace_point(&mut self, new_point: (f32, f32), point_index: usize) -> anyhow::Result<()>;

    fn replace_points(&mut self, gate_drag_data: GateDragData) -> anyhow::Result<()>;
}
