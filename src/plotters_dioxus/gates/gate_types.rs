use crate::plotters_dioxus::gates::gate_store::Id;

use anyhow::anyhow;
use flow_gates::{GateGeometry};

use crate::plotters_dioxus::plot_helpers::PlotMapper;


#[derive(Clone, PartialEq, Copy)]
pub enum GateType {
    Polygon,
    Ellipse,
    Rectangle,
    Line,
    Bisector,
    Quadrant,
    FlexiQuadrant,
}

// convert this to create GateFinal
// GateFinal will store a GateClass containing it's gate(s)
// GateFinal's drawself will return a composite list of shapes from the sub-gates
// decide how to work with ID's - GateFinal should have its own ID so it can be retrieved from the store
// when a sub-gate is clicked on it 
impl GateType {

    pub fn is_composite(&self) -> bool {
        matches!(self, GateType::Bisector | GateType::Quadrant | GateType::FlexiQuadrant)
    }
    
    pub fn is_single(&self) -> bool {
        !self.is_composite()
    }

    pub fn to_gate_geometry(
        &self,
        mapper: &PlotMapper,
        click_x: f32,
        click_y: f32,
        x_param: &str,
        y_param: &str,
        points: Option<Vec<(f32, f32)>>,
    ) -> anyhow::Result<GateGeometry> {
        match self {
            GateType::Polygon => flow_gates::geometry::create_polygon_geometry(
                                            points.ok_or(anyhow!("points not provided for polygon gate"))?,
                                            x_param,
                                            y_param,
                                        )
                                        .map_err(|_| anyhow!("failed to create polygon geometry")),
            GateType::Ellipse => {
                crate::plotters_dioxus::gates::gate_draw_helpers::ellipse::create_default_ellipse(
                                    &mapper,
                                    click_x,
                                    click_y,
                                    50f32,
                                    30f32,
                                    x_param,
                                    y_param,
                                )
            },
            GateType::Rectangle => {
                crate::plotters_dioxus::gates::gate_draw_helpers::rectangle::create_default_rectangle(
                                    &mapper,
                                    click_x,
                                    click_y,
                                    50f32,
                                    50f32,
                                    x_param,
                                    y_param,
                                )
            },
            GateType::Line => todo!(),
            GateType::Bisector => todo!(),
            GateType::Quadrant => todo!(),
            GateType::FlexiQuadrant => todo!(),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum ShapeType {
    Gate(Id),
    Point(usize),
    GhostGate((f32, f32)),
    GhostPoint,
    DraftGate,
    Rotation(f32),
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
    },
    Rectangle {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
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
                    points: p,
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
