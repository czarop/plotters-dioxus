use std::sync::Arc;

use crate::plotters_dioxus::{
    PlotDrawable,
    gates::gate_styles::{DRAFT_LINE, GateShape},
};

#[derive(PartialEq, Clone)]
pub enum GateDraft {
    Polygon {
        points: Vec<(f32, f32)>,
        x_param: Arc<str>,
        y_param: Arc<str>,
    },
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

    fn draw_self(&self) -> Vec<GateShape> {
        match self {
            GateDraft::Polygon { points, .. } => draw_draft_polygon(points),
        }
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

fn draw_draft_polygon(points: &[(f32, f32)]) -> Vec<GateShape> {
    match points.len() {
        0 => vec![],
        1 => {
            let center = points[0];
            vec![GateShape::Circle {
                center,
                radius: 3.0,
                fill: "red",
            }]
        }
        2 => {
            let start = points[0];
            let end = points[1];

            vec![GateShape::PolyLine {
                points: vec![start, end],
                style: &DRAFT_LINE,
            }]
        }
        _ => {
            let mut points_local: Vec<(f32, f32)> = points.to_vec();
            // close the loop
            if let Some(first) = points_local.first() {
                points_local.push(first.clone());
            }

            vec![GateShape::Polygon {
                points: points_local,
                style: &DRAFT_LINE,
            }]
        }
    }
}
