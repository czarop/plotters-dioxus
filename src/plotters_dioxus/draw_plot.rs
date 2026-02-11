#![allow(non_snake_case)]
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use dioxus::prelude::*;
use flow_fcs::TransformType;
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;

use std::sync::Arc;

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, render::RenderConfig,
};

use crate::plotters_dioxus::{draw_gates::GateLayer, plot_helpers::PlotMapper};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

pub fn asinh_transform_f32(value: f32, cofactor: f32) -> anyhow::Result<f32> {
    if value.is_nan() || value.is_infinite() {
        return Err(anyhow::anyhow!("Value {value} cannot be arcsinh transform"));
    }
    if cofactor == 0_f32 {
        return Err(anyhow::anyhow!("Cofactor {cofactor} cannot be used for arcsinh transform"));
    }
    Ok((value / cofactor).asinh())
}

pub fn asinh_reverse_f32(transformed_value: f32, cofactor: f32) -> anyhow::Result<f32> {
    if transformed_value.is_nan() || transformed_value.is_infinite() {
        return Err(anyhow::anyhow!("Transformed value {transformed_value} is invalid"));
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
    pub title: Arc<str>,
    pub lower: f32,
    pub upper: f32,
    pub transform: flow_fcs::TransformType,
}

impl Default for AxisInfo{
    fn default() -> Self {
        Self { title: Default::default(), lower: 0_f32, upper: 4194304_f32, transform: flow_fcs::TransformType::Linear }
    }
}

impl AxisInfo {
    pub fn into_archsinh(&self, cofactor: f32) -> anyhow::Result<Self> {
        
        let old_lower = self.lower;
        let old_upper = self.upper;
        let transform = TransformType::Arcsinh{cofactor};
        let new_self = match self.transform {
            
            flow_fcs::TransformType::Arcsinh { cofactor: old_cofactor } => {
                let lower = asinh_to_asinh(old_lower, old_cofactor, cofactor)?;
                let upper = asinh_to_asinh(old_upper, old_cofactor, cofactor)?;
                Self { title: self.title.clone(), lower, upper, transform  }
            },
            _ => {
                let lower = asinh_transform_f32(old_lower, cofactor)?;
                let upper = asinh_transform_f32(old_upper, cofactor)?;
                Self{ title: self.title.clone(), lower, upper, transform }
            },
        };
        Ok(new_self)
    }

    pub fn into_linear(&self) -> anyhow::Result<Self> {
        
        let old_lower = self.lower;
        let old_upper = self.upper;
        let transform = TransformType::Linear;
        let new_self = match self.transform {
            TransformType::Linear => self.clone(),
            
            TransformType::Arcsinh { cofactor: old_cofactor } => {
                let old_cofactor = old_cofactor;
                let upper_untransformed = asinh_reverse_f32(old_upper, old_cofactor)?;
                let lower_untransformed = asinh_reverse_f32(old_lower, old_cofactor)?;
                Self { title: self.title.clone(), lower: lower_untransformed, upper: upper_untransformed, transform  }
            },
            TransformType::Biexponential { .. } => {
                Self{ title: self.title.clone(), lower: old_lower, upper: old_upper, transform }
            },
        };
        Ok(new_self)
    }

    pub fn is_linear(&self) -> bool {
        match self.transform {
            TransformType::Linear => true,
            _ => false
        }
    }
}

#[component]
pub fn PseudoColourPlot(
    #[props] data: ReadSignal<Arc<Vec<(f32, f32)>>>,
    #[props] size: ReadSignal<(u32, u32)>,
    #[props] x_axis_info: ReadSignal<AxisInfo>,
    #[props] y_axis_info: ReadSignal<AxisInfo>,
) -> Element {
    let mut plot_image_src = use_signal(|| String::new());
    let mut plot_map = use_signal(|| None::<PlotMapper>);

    use_effect(move || {
        let x_axis_info = x_axis_info();
        let y_axis_info = y_axis_info();
        let (width, height) = size();
        let data = data.clone();

        let plot = DensityPlot::new();
        let base_options = BasePlotOptions::new()
            .width(width)
            .height(height)
            .title("My Density Plot")
            .build()
            .expect("shouldn't fail");

        let x_axis_options = flow_plots::AxisOptions::new()
            .range(x_axis_info.lower..=x_axis_info.upper)
            .transform(x_axis_info.transform.clone())
            .label(&x_axis_info.title.to_string())
            .build()
            .expect("axis options failed");
        let y_axis_options = flow_plots::AxisOptions::new()
            .range(y_axis_info.lower..=y_axis_info.upper)
            .transform(y_axis_info.transform.clone())
            .label(y_axis_info.title.to_string())
            .build()
            .expect("axis options failed");
        let mapper = PlotMapper::new(
            width as f32,
            height as f32,
            x_axis_options.range.clone(),
            y_axis_options.range.clone(),
            x_axis_info.transform.clone(),
            y_axis_info.transform.clone(),
        );
        let options = DensityPlotOptions::new()
            .base(base_options)
            .colormap(ColorMaps::Jet)
            .x_axis(x_axis_options)
            .y_axis(y_axis_options)
            .build()
            .expect("shouldn't fail");

        let mut render_config = RenderConfig::default();

        let plot_data = plot
            .render(data(), &options, &mut render_config)
            .expect("failed to render plot");

        let base64_str = BASE64_STANDARD.encode(&plot_data);
        plot_image_src.set(format!("data:image/jpeg;base64,{}", base64_str));

        plot_map.set(Some(mapper));
    });

    rsx! {
        div { style: "position: relative; width: {size().0}px; height: {size().1}px;",
            img {
                style: "user-select: none; -webkit-user-select: none;",
                src: "{plot_image_src()}",
                width: "{size().0}",
                height: "{size().1}",
            }
            GateLayer {
                plot_map,
                x_channel: x_axis_info().title.clone(),
                y_channel: y_axis_info().title.clone(),
            
            }
        }
    }
}
