
use std::{ops::Deref, sync::Arc};

use crate::plotters_dioxus::PlotDrawable;

#[derive(PartialEq, Clone)]
pub enum GateDraft {
    Polygon {
        points: Vec<(f32, f32)>,
        x_param: Arc<str>,
        y_param: Arc<str>,
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
    
    fn draw_self(&self, mapper: &super::plot_helpers::PlotMapper) -> Vec<super::plot_helpers::GateShape> {
        todo!()
    }
    
}

impl GateDraft {
    pub fn new_polygon(points: Vec<(f32, f32)>, x_param: Arc<str>, y_param: Arc<str>) -> Self {
        GateDraft::Polygon {
            points,
            x_param: x_param,
            y_param: y_param,
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct GateFinal {
    inner: Arc<flow_gates::Gate>,
    selected: bool,
}

impl GateFinal {
    pub fn new(gate: flow_gates::Gate, selected: bool) -> Self {
        GateFinal {
            inner: Arc::new(gate),
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
        self.inner.geometry.to_render_points(
            self.x_parameter_channel_name(),
            self.y_parameter_channel_name(),
        )
    }

    fn is_finalised(&self) -> bool {
        return true;
    }
    
    fn draw_self(&self, mapper: &super::plot_helpers::PlotMapper) -> Vec<super::plot_helpers::GateShape> {
        todo!()
    }
    
    

}

