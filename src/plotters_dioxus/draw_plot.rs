#![allow(non_snake_case)]
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use dioxus::prelude::*;

use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;

use std::{sync::Arc};

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, render::RenderConfig,
};

use crate::plotters_dioxus::{
    AxisInfo, draw_gates::GateLayer, plot_helpers::{PlotMapper}
};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

#[component]
pub fn PseudoColourPlot(
    data: ReadSignal<Vec<(f32, f32)>>,
    size: ReadSignal<(u32, u32)>,
    x_axis_info: ReadSignal<AxisInfo>,
    y_axis_info: ReadSignal<AxisInfo>,
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
            .label(&x_axis_info.param.to_string())
            .build()
            .expect("axis options failed");
        let y_axis_options = flow_plots::AxisOptions::new()
            .range(y_axis_info.lower..=y_axis_info.upper)
            .transform(y_axis_info.transform.clone())
            .label(y_axis_info.param.to_string())
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
                x_channel: x_axis_info().param.fluoro.clone(),
                y_channel: y_axis_info().param.fluoro.clone(),
            
            }
        }
    }
}
