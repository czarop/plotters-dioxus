use crate::plotters_dioxus::gates::gate_store::Id;

#[derive(PartialEq, Clone)]
pub enum ShapeType {
    Gate(Id),
    Point(usize),
    GhostGate((f32, f32)),
    GhostPoint,
    DraftGate,
    Rotation,
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
    Ellipse {
        center: (f32, f32),
        radius_x: f32,
        radius_y: f32,
        degrees_rotation: f32,
        style: &'static DrawingStyle,
        shape_type: ShapeType,
    },
    Handle {
        center: (f32, f32),
        size: f32,
        shape_center: (f32, f32),
        shape_type: ShapeType,
    }
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
            GateShape::Ellipse { center, radius_x, radius_y, degrees_rotation, style:_, shape_type:_ } => GateShape::Ellipse 
            { center: *center, radius_x: *radius_x, radius_y: *radius_y, degrees_rotation: *degrees_rotation, style: style, shape_type: shape_type.clone() },
            GateShape::Handle { center, size, shape_center, shape_type:_ } => Self::Handle {
                center: *center, size: *size, shape_center: *shape_center, shape_type: shape_type.clone()
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
            },
            GateShape::Ellipse {
                center, radius_x, radius_y, degrees_rotation, style, shape_type,
            } => {
                let c = (center.0 - offset.0, center.1 - offset.1);

                Self::Ellipse {
                    center: c,
                    radius_x: *radius_x,
                    radius_y: *radius_y,
                    degrees_rotation: *degrees_rotation,
                                        style: style,
                    shape_type: shape_type.clone(),
                }
            }
            GateShape::Handle { center, size, shape_center, shape_type } => {
                
                Self::Handle {
                center: (center.0 + offset.0, center.1 + offset.1), shape_center: *shape_center, size: *size, shape_type: shape_type.clone()
            }},
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
