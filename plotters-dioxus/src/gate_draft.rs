pub enum GateDraft {
    Polygon {
        points: Vec<(f32, f32)>,
        x_param: String,
        y_param: String,
    },
    // You can add Rectangle or Ellipse drafts here later
}

impl flow_plots::plots::traits::PlotDrawable for GateDraft {
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
