use std::{f32::INFINITY, sync::Arc};

use dioxus::prelude::*;
use flow_fcs::TransformType;

use crate::gate_editor::plots::axis_store::Param;

pub fn asinh_transform_f32(value: f32, cofactor: f32) -> anyhow::Result<f32> {
    if value.is_nan() || value.is_infinite() {
        return Err(anyhow::anyhow!("Value {value} cannot be arcsinh transform"));
    }
    if cofactor == 0_f32 {
        return Err(anyhow::anyhow!(
            "Cofactor {cofactor} cannot be used for arcsinh transform"
        ));
    }
    Ok((value / cofactor).asinh())
}

pub fn asinh_reverse_f32(transformed_value: f32, cofactor: f32) -> anyhow::Result<f32> {
    if transformed_value.is_nan() || transformed_value.is_infinite() {
        return Err(anyhow::anyhow!(
            "Transformed value {transformed_value} is invalid"
        ));
    }
    if cofactor == 0_f32 {
        return Err(anyhow::anyhow!("Cofactor {cofactor} cannot be zero"));
    }
    Ok(transformed_value.sinh() * cofactor)
}

pub fn asinh_to_asinh(value: f32, old_cofactor: f32, new_cofactor: f32) -> anyhow::Result<f32> {
    let untransformed = asinh_reverse_f32(value, old_cofactor)?;
    asinh_transform_f32(untransformed, new_cofactor)
}

#[derive(Debug, Clone, PartialEq, Props)]
pub struct AxisInfo {
    pub param: Param,
    pub axis_lower: f32,
    pub axis_upper: f32,
    pub data_lower: f32,
    pub data_upper: f32,
    pub transform: flow_fcs::TransformType,
}

impl Default for AxisInfo {
    fn default() -> Self {
        Self {
            param: Param {
                marker: Arc::from(""),
                fluoro: Arc::from(""),
            },
            axis_lower: 0_f32,
            axis_upper: 4194304_f32,
            data_lower: 0_f32,
            data_upper: 4194304_f32,
            transform: flow_fcs::TransformType::Linear,
        }
    }
}

impl AxisInfo {
    pub fn new_from_raw(
        param: Param,
        lower_raw: f32,
        upper_raw: f32,
        data_lower: f32,
        data_upper: f32,
        transform: TransformType,
    ) -> Self {
        match transform {
            TransformType::Linear => Self {
                param,
                axis_lower: lower_raw,
                axis_upper: upper_raw,
                data_lower,
                data_upper,
                transform,
            },
            TransformType::Arcsinh { cofactor } => {
                let lower = asinh_transform_f32(lower_raw, cofactor).unwrap_or(0f32);
                let upper = asinh_transform_f32(upper_raw, cofactor).unwrap_or(INFINITY);
                let data_lower = asinh_transform_f32(data_lower, cofactor).unwrap_or(0f32);
                let data_upper = asinh_transform_f32(data_upper, cofactor).unwrap_or(INFINITY);
                Self {
                    param,
                    axis_lower: lower,
                    axis_upper: upper,
                    data_lower,
                    data_upper,
                    transform,
                }
            }
            TransformType::Biexponential {
                top_of_scale: _,
                positive_decades: _,
                negative_decades: _,
                width: _,
            } => todo!(),
        }
    }

    pub fn into_archsinh(&self, cofactor: f32) -> anyhow::Result<Self> {
        let old_lower = self.axis_lower;
        let old_upper = self.axis_upper;
        let old_dl = self.data_lower;
        let old_du = self.data_upper;
        let transform = TransformType::Arcsinh { cofactor };
        let new_self = match self.transform {
            flow_fcs::TransformType::Arcsinh {
                cofactor: old_cofactor,
            } => {
                let lower = asinh_to_asinh(old_lower, old_cofactor, cofactor)?;
                let upper = asinh_to_asinh(old_upper, old_cofactor, cofactor)?;
                let data_lower = asinh_to_asinh(old_dl, old_cofactor, cofactor)?;
                let data_upper = asinh_to_asinh(old_du, old_cofactor, cofactor)?;
                Self {
                    param: self.param.clone(),
                    axis_lower: lower,
                    axis_upper: upper,
                    data_lower,
                    data_upper,
                    transform,
                }
            }
            _ => {
                let lower = asinh_transform_f32(old_lower, cofactor)?;
                let upper = asinh_transform_f32(old_upper, cofactor)?;
                let data_lower = asinh_transform_f32(old_dl, cofactor)?;
                let data_upper = asinh_transform_f32(old_du, cofactor)?;
                Self {
                    param: self.param.clone(),
                    axis_lower: lower,
                    axis_upper: upper,
                    data_lower,
                    data_upper,
                    transform,
                }
            }
        };
        Ok(new_self)
    }

    pub fn into_linear(&self) -> anyhow::Result<Self> {
        let old_lower = self.axis_lower;
        let old_upper = self.axis_upper;
        let old_dl = self.data_lower;
        let old_du = self.data_upper;
        let transform = TransformType::Linear;
        let new_self = match self.transform {
            TransformType::Linear => self.clone(),

            TransformType::Arcsinh {
                cofactor: old_cofactor,
            } => {
                let old_cofactor = old_cofactor;
                let upper_untransformed = asinh_reverse_f32(old_upper, old_cofactor)?;
                let lower_untransformed = asinh_reverse_f32(old_lower, old_cofactor)?;
                let data_lower = asinh_reverse_f32(old_dl, old_cofactor)?;
                let data_upper = asinh_reverse_f32(old_du, old_cofactor)?;
                Self {
                    param: self.param.clone(),
                    axis_lower: lower_untransformed,
                    axis_upper: upper_untransformed,
                    data_lower,
                    data_upper,
                    transform,
                }
            }
            TransformType::Biexponential { .. } => Self {
                param: self.param.clone(),
                axis_lower: old_lower,
                axis_upper: old_upper,
                data_lower: old_dl,
                data_upper: old_du,
                transform,
            },
        };
        Ok(new_self)
    }

    pub fn is_linear(&self) -> bool {
        match self.transform {
            TransformType::Linear => true,
            _ => false,
        }
    }

    pub fn is_arcsinh(&self) -> bool {
        match self.transform {
            TransformType::Arcsinh { .. } => true,
            _ => false,
        }
    }

    pub fn get_untransformed_bounds(&self) -> (f32, f32) {
        match self.transform {
            TransformType::Arcsinh { cofactor } => (
                asinh_reverse_f32(self.axis_lower, cofactor).unwrap_or_default(),
                asinh_reverse_f32(self.axis_upper, cofactor).unwrap_or_default(),
            ),
            _ => (self.axis_lower, self.axis_upper),
        }
    }

    pub fn get_untransformed_lower(&self) -> f32 {
        match self.transform {
            TransformType::Arcsinh { cofactor } => {
                asinh_reverse_f32(self.axis_lower, cofactor).unwrap_or_default()
            }
            _ => self.axis_lower,
        }
    }

    pub fn get_untransformed_upper(&self) -> f32 {
        match self.transform {
            TransformType::Arcsinh { cofactor } => {
                asinh_reverse_f32(self.axis_upper, cofactor).unwrap_or_default()
            }
            _ => self.axis_upper,
        }
    }

    pub fn into_new_lower(&self, lower_raw: f32) -> Self {
        match self.transform {
            TransformType::Linear => Self {
                param: self.param.clone(),
                axis_lower: lower_raw,
                axis_upper: self.axis_upper,
                data_lower: self.data_lower,
                data_upper: self.data_upper,
                transform: self.transform.clone(),
            },
            TransformType::Arcsinh { cofactor } => {
                let new_lower = asinh_transform_f32(lower_raw, cofactor).unwrap_or(self.axis_lower);
                Self {
                    param: self.param.clone(),
                    axis_lower: new_lower,
                    axis_upper: self.axis_upper,
                    data_lower: self.data_lower,
                    data_upper: self.data_upper,
                    transform: self.transform.clone(),
                }
            }
            TransformType::Biexponential { .. } => todo!(),
        }
    }

    pub fn into_new_upper(&self, upper_raw: f32) -> Self {
        match self.transform {
            TransformType::Linear => Self {
                param: self.param.clone(),
                axis_lower: self.axis_lower,
                axis_upper: upper_raw,
                data_lower: self.data_lower,
                data_upper: self.data_upper,
                transform: self.transform.clone(),
            },
            TransformType::Arcsinh { cofactor } => {
                let new_upper = asinh_transform_f32(upper_raw, cofactor).unwrap_or(self.axis_upper);
                Self {
                    param: self.param.clone(),
                    axis_lower: self.axis_lower,
                    axis_upper: new_upper,
                    data_lower: self.data_lower,
                    data_upper: self.data_upper,
                    transform: self.transform.clone(),
                }
            }
            TransformType::Biexponential { .. } => todo!(),
        }
    }

    pub fn get_cofactor(&self) -> Option<f32> {
        match self.transform {
            TransformType::Linear => None,
            TransformType::Arcsinh { cofactor } => Some(cofactor),
            TransformType::Biexponential { .. } => None,
        }
    }
}
