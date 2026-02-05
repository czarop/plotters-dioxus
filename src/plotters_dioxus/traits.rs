use std::f32::INFINITY;

use crate::plotters_dioxus::plot_helpers::{GateShape, PlotMapper};

pub trait PlotDrawable {
    fn get_points(&self) -> Vec<(f32, f32)>;
    fn is_finalised(&self) -> bool;

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
    fn is_point_on_perimeter(&self, point: (f32, f32), tolerance: (f32, f32)) -> Option<f32> {
        let points = self.get_points();
        let mut closest = INFINITY;
        for segment in points.windows(2) {
            let (p1, p2) = (segment[0], segment[1]);
            if let Some(dis) = self.is_near_segment(point, p1, p2, tolerance) {
                if dis < closest {
                    closest = dis;
                }
            }
        }
        if closest == INFINITY {
            return None;
        } else {
            return Some(closest);
        }
    }

    fn draw_self(&self, mapper: &PlotMapper) -> Vec<GateShape>;
    fn draw_self_selected(&self, mapper: &PlotMapper) -> Vec<GateShape>;
    fn draw_ghost_point(
        &self, 
        mapper: &PlotMapper, 
        point_idx: usize, 
        new_pos: (f32, f32)
    ) -> Vec<GateShape>;

    
    fn draw_ghost_move(
        &self, 
        mapper: &PlotMapper, 
        delta: (f32, f32)
    ) -> Vec<GateShape>;
}
