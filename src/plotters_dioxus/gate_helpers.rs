
use std::ops::Deref;

use crate::plotters_dioxus::PlotDrawable;

#[derive(PartialEq, Clone)]
pub enum GateDraft {
    Polygon {
        points: Vec<(f32, f32)>,
        x_param: String,
        y_param: String,
    },
    // You can add Rectangle or Ellipse drafts here later
}

impl PlotDrawable for GateDraft {
    fn get_points(&self) -> Vec<(f32, f32)> {
        match self {
            GateDraft::Polygon { points, .. } => points.clone(),
        }
    }
    fn is_finalised(&self) -> bool {
        false
    }
}

impl GateDraft {
    pub fn new_polygon(points: Vec<(f32, f32)>, x_param: &str, y_param: &str) -> Self {
        GateDraft::Polygon {
            points,
            x_param: x_param.to_string(),
            y_param: y_param.to_string(),
        }
    }
}

#[derive(PartialEq)]
pub struct GateFinal {
    inner: flow_gates::Gate,
    selected: bool,
}

impl GateFinal {
    pub fn new(gate: flow_gates::Gate, selected: bool) -> Self {
        GateFinal {
            inner: gate,
            selected,
        }
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, state: bool) {
        self.selected = state;
    }
}

impl Deref for GateFinal {
    type Target = flow_gates::Gate;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl PlotDrawable for GateFinal {
    fn get_points(&self) -> Vec<(f32, f32)> {
        self.inner.get_points()
    }

    fn is_finalised(&self) -> bool {
        return true;
    }
}


impl PlotDrawable for flow_gates::Gate {
    fn get_points(&self) -> Vec<(f32, f32)> {
        self.geometry.to_render_points(
            self.x_parameter_channel_name(),
            self.y_parameter_channel_name(),
        )
    }
    fn is_finalised(&self) -> bool {
        true
    }
}
