use std::sync::Arc;

use flow_fcs::TransformType;

use crate::plotters_dioxus::{PlotDrawable, gates::gate_drag::PointDragData};

pub trait DrawableGate: GateTrait + PlotDrawable {}

pub trait GateTrait {
    fn is_composite(&self) -> bool;

    fn get_id(&self) -> Arc<str>;

    fn get_params(&self) -> (Arc<str>, Arc<str>);

    fn is_selected(&self) -> bool;

    fn set_selected(&mut self, state: bool);

    fn is_drag_point(&self) -> bool;

    fn set_drag_point(&mut self, drag_data: Option<PointDragData>);

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

    fn replace_points(&mut self, points: Vec<(f32, f32)>) -> anyhow::Result<()>;
}
