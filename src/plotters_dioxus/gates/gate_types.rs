use std::sync::Arc;

use crate::plotters_dioxus::gates::gate_store::GateId;

// #[derive(Clone, PartialEq, Hash)]
// pub enum GateTypeID {
//     Primary(Arc<str>),
//     SubGate(Arc<str>, Arc<str>)
// }

// impl GateID {
//     pub fn get_primary_id(&self) -> Arc<str> {
//         match self{
//             GateID::Primary(i) => i.clone(),
//             GateID::SubGate(i, _) => i.clone(),
//         }
//     }

//     pub fn get_subgate_id(&self) -> Option<Arc<str>> {
//         match self{
//             GateID::Primary(_) => None,
//             GateID::SubGate(_, i) => Some(i.clone()),
//         }
//     }

//     pub fn is_subgate(&self) -> bool {
//         match self{
//             GateID::Primary(_) => false,
//             GateID::SubGate(_, _) => true,
//         }
//     }
// }

#[derive(Clone, PartialEq, Copy)]
pub enum GateType {
    Polygon,
    Ellipse,
    Rectangle,
    Line(Option<f32>),
    Bisector,
    Quadrant,
    SkewedQuadrant,
}

impl GateType {
    pub fn is_composite(&self) -> bool {
        matches!(
            self,
            GateType::Bisector | GateType::Quadrant | GateType::SkewedQuadrant
        )
    }

    pub fn is_single(&self) -> bool {
        !self.is_composite()
    }
}

#[derive(PartialEq, Clone)]
pub enum ShapeType {
    Gate(GateId),
    CompositeGate(GateId, bool),
    Point(usize),
    CompositePoint(usize, bool),
    GhostGate((f32, f32)),
    GhostPoint,
    DraftGate,
    Rotation(f32),
    UndraggableLine,
    UndraggablePoint(usize),
}

#[derive(PartialEq, Clone)]
pub enum GateRenderShape {
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
        points: Arc<Vec<(f32, f32)>>,
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
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        style: &'static DrawingStyle,
        shape_type: ShapeType,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        style: &'static DrawingStyle,
        shape_type: ShapeType,
    },
}

impl GateRenderShape {
    pub fn clone_with_type(&self, style: &'static DrawingStyle, shape_type: ShapeType) -> Self {
        match self {
            GateRenderShape::PolyLine {
                points,
                style: _,
                shape_type: _,
            } => Self::PolyLine {
                points: points.clone(),
                style: style,
                shape_type: shape_type,
            },
            GateRenderShape::Circle {
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
            GateRenderShape::Polygon {
                points,
                style: _,
                shape_type: _,
            } => Self::Polygon {
                points: points.clone(),
                style: style,
                shape_type: shape_type.clone(),
            },
            GateRenderShape::Ellipse {
                center,
                radius_x,
                radius_y,
                degrees_rotation,
                style: _,
                shape_type: _,
            } => GateRenderShape::Ellipse {
                center: *center,
                radius_x: *radius_x,
                radius_y: *radius_y,
                degrees_rotation: *degrees_rotation,
                style: style,
                shape_type: shape_type.clone(),
            },
            GateRenderShape::Handle {
                center,
                size,
                shape_center,
                shape_type: _,
            } => Self::Handle {
                center: *center,
                size: *size,
                shape_center: *shape_center,
                shape_type: shape_type.clone(),
            },
            GateRenderShape::Rectangle {
                x,
                y,
                width,
                height,
                style,
                shape_type: _,
            } => Self::Rectangle {
                x: *x,
                y: *y,
                width: *width,
                height: *height,
                style: *style,
                shape_type: shape_type.clone(),
            },
            GateRenderShape::Line {
                x1,
                y1,
                x2,
                y2,
                style,
                shape_type,
            } => Self::Line {
                x1: *x1,
                y1: *y1,
                x2: *x2,
                y2: *y2,
                style: *style,
                shape_type: shape_type.clone(),
            },
        }
    }

    pub fn clone_with_offset(&self, offset: (f32, f32), style: &'static DrawingStyle) -> Self {
        match self {
            GateRenderShape::PolyLine {
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
            GateRenderShape::Circle {
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
            GateRenderShape::Polygon {
                points,
                style: _,
                shape_type,
            } => {
                let p = points
                    .iter()
                    .map(|(x, y)| (x - offset.0, y - offset.1))
                    .collect();
                Self::Polygon {
                    points: Arc::new(p),
                    style: style,
                    shape_type: shape_type.clone(),
                }
            }
            GateRenderShape::Ellipse {
                center,
                radius_x,
                radius_y,
                degrees_rotation,
                style,
                shape_type,
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
            GateRenderShape::Handle {
                center,
                size,
                shape_center,
                shape_type,
            } => Self::Handle {
                center: (center.0 + offset.0, center.1 + offset.1),
                shape_center: *shape_center,
                size: *size,
                shape_type: shape_type.clone(),
            },
            GateRenderShape::Rectangle {
                x,
                y,
                width,
                height,
                style,
                shape_type,
            } => {
                let new_x = x + offset.0;
                let new_y = y + offset.1;
                Self::Rectangle {
                    x: new_x,
                    y: new_y,
                    width: *width,
                    height: *height,
                    style: *style,
                    shape_type: shape_type.clone(),
                }
            }
            GateRenderShape::Line {
                x1,
                y1,
                x2,
                y2,
                style,
                shape_type,
            } => Self::Line {
                x1: *x1 + offset.0,
                y1: *y1 + offset.1,
                x2: *x2 + offset.0,
                y2: *y2 + offset.1,
                style: *style,
                shape_type: shape_type.clone(),
            },
        }
    }

    pub fn is_composite(&self) -> bool {
        let st = match self {
            GateRenderShape::PolyLine { shape_type, .. }
            | GateRenderShape::Circle { shape_type, .. }
            | GateRenderShape::Polygon { shape_type, .. }
            | GateRenderShape::Ellipse { shape_type, .. }
            | GateRenderShape::Handle { shape_type, .. }
            | GateRenderShape::Rectangle { shape_type, .. }
            | GateRenderShape::Line { shape_type, .. } => shape_type,
        };

        matches!(
            st,
            ShapeType::CompositeGate { .. }
                | ShapeType::CompositePoint(..)
                | ShapeType::UndraggableLine
                | ShapeType::UndraggablePoint(..)
        )
    }

    pub fn is_undraggable(&self) -> bool {
        let st = match self {
            GateRenderShape::PolyLine { shape_type, .. }
            | GateRenderShape::Circle { shape_type, .. }
            | GateRenderShape::Polygon { shape_type, .. }
            | GateRenderShape::Ellipse { shape_type, .. }
            | GateRenderShape::Handle { shape_type, .. }
            | GateRenderShape::Rectangle { shape_type, .. }
            | GateRenderShape::Line { shape_type, .. } => shape_type,
        };

        matches!(
            st,
            ShapeType::UndraggableLine | ShapeType::UndraggablePoint(..)
        )
    }

    pub fn is_axis_matched(&self) -> bool {
        let st = match self {
            GateRenderShape::PolyLine { shape_type, .. }
            | GateRenderShape::Circle { shape_type, .. }
            | GateRenderShape::Polygon { shape_type, .. }
            | GateRenderShape::Ellipse { shape_type, .. }
            | GateRenderShape::Handle { shape_type, .. }
            | GateRenderShape::Rectangle { shape_type, .. }
            | GateRenderShape::Line { shape_type, .. } => shape_type,
        };

        matches!(
            st,
            ShapeType::CompositeGate(.., true) | ShapeType::CompositePoint(.., true)
        )
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

pub static GREY_LINE: DrawingStyle = DrawingStyle {
    stroke: "grey",
    fill: "none",
    stroke_width: 2.0,
    dashed: false,
};

pub static GREY_LINE_DASHED: DrawingStyle = DrawingStyle {
    stroke: "grey",
    fill: "none",
    stroke_width: 2.0,
    dashed: true,
};
