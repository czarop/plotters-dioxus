use crate::gate_store::Id;

#[derive(PartialEq, Clone)]
pub enum ShapeType {
    Gate(Id),
    Point(usize),
    GhostGate((f32, f32)),
    GhostPoint,
    DraftGate,
}

#[derive(PartialEq, Clone)]
pub enum GateShape {
    PolyLine {
        points: Vec<(f32, f32)>,
        style: &'static DrawingStyle,
        shape_type: ShapeType,
    },
    Circle {
        center: (f32, f32),
        radius: f32,
        fill: &'static str,
        shape_type: ShapeType,
    },
    Polygon {
        points: Vec<(f32, f32)>,
        style: &'static DrawingStyle,
        shape_type: ShapeType,
    },
}

impl GateShape {
    pub fn clone_with_type(&self, style: &'static DrawingStyle, shape_type: ShapeType) -> Self {
        match self {
            GateShape::PolyLine {
                points,
                style: _,
                shape_type: _,
            } => Self::PolyLine {
                points: points.clone(),
                style: style,
                shape_type: shape_type,
            },
            GateShape::Circle {
                center,
                radius,
                fill,
                shape_type: _,
            } => Self::Circle {
                center: *center,
                radius: *radius,
                fill: fill,
                shape_type: shape_type.clone(),
            },
            GateShape::Polygon {
                points,
                style: _,
                shape_type: _,
            } => Self::Polygon {
                points: points.clone(),
                style: style,
                shape_type: shape_type.clone(),
            },
        }
    }

    pub fn clone_with_offset(&self, offset: (f32, f32), style: &'static DrawingStyle) -> Self {
        match self {
            GateShape::PolyLine {
                points,
                style: _,
                shape_type,
            } => {
                let p = points
                    .iter()
                    .map(|(x, y)| (x + offset.0, y + offset.1))
                    .collect();
                Self::PolyLine {
                    points: p,
                    style: style,
                    shape_type: shape_type.clone(),
                }
            }
            GateShape::Circle {
                center,
                radius,
                fill,
                shape_type,
            } => Self::Circle {
                center: (center.0 + offset.0, center.1 + offset.1),
                radius: *radius,
                fill: fill,
                shape_type: shape_type.clone(),
            },
            GateShape::Polygon {
                points,
                style: _,
                shape_type,
            } => {
                let p = points
                    .iter()
                    .map(|(x, y)| (x - offset.0, y - offset.1))
                    .collect();
                Self::Polygon {
                    points: p,
                    style: style,
                    shape_type: shape_type.clone(),
                }
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct DrawingStyle {
    pub stroke: &'static str,
    pub fill: &'static str,
    pub stroke_width: f32,
    pub dashed: bool,
}

pub static DRAFT_LINE: DrawingStyle = DrawingStyle {
    stroke: "red",
    fill: "rgba(0, 255, 255, 0.2)",
    stroke_width: 2.0,
    dashed: false,
};

pub static DEFAULT_LINE: DrawingStyle = DrawingStyle {
    stroke: "cyan",
    fill: "rgba(0, 255, 255, 0.2)",
    stroke_width: 2.0,
    dashed: false,
};

pub static SELECTED_LINE: DrawingStyle = DrawingStyle {
    stroke: "orange",
    fill: "rgba(0, 255, 255, 0.2)",
    stroke_width: 2.0,
    dashed: false,
};

pub static DRAGGED_LINE: DrawingStyle = DrawingStyle {
    stroke: "yellow",
    fill: "none",
    stroke_width: 2.0,
    dashed: true,
};

pub static DRAGGED_GATE: DrawingStyle = DrawingStyle {
    stroke: "grey",
    fill: "none",
    stroke_width: 2.0,
    dashed: false,
};
