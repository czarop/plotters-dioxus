#![allow(non_snake_case)]
use std::{ops::RangeInclusive, sync::Arc};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use dioxus::prelude::*;

use plotters::coord::Shift;
use plotters::prelude::*;
use plotters_bitmap::BitMapBackend;

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, render::RenderConfig,
};

use crate::plotters_dioxus::{AxisInfo, draw_gates::GateLayer, plot_helpers::PlotMapper};

pub type DioxusDrawingArea<'a> = DrawingArea<BitMapBackend<'a>, Shift>;

#[component]
pub fn PseudoColourPlot(
    data: ReadSignal<Vec<(f32, f32)>>,
    size: ReadSignal<(u32, u32)>,
    x_axis_info: ReadSignal<AxisInfo>,
    y_axis_info: ReadSignal<AxisInfo>,
    parental_gate_id: ReadSignal<Option<Arc<str>>>,
) -> Element {
    let mut plot_image_src = use_signal(|| String::new());
    let mut plot_map = use_signal(|| None::<PlotMapper>);
    use_context_provider::<Signal<Option<PlotMapper>>>(|| plot_map);

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
            .range(x_axis_info.axis_lower..=x_axis_info.axis_upper)
            .transform(x_axis_info.transform.clone())
            .label(&x_axis_info.param.to_string())
            .build()
            .expect("axis options failed");
        let y_axis_options = flow_plots::AxisOptions::new()
            .range(y_axis_info.axis_lower..=y_axis_info.axis_upper)
            .transform(y_axis_info.transform.clone())
            .label(y_axis_info.param.to_string())
            .build()
            .expect("axis options failed");

        let actual_ranges = flow_plots::create_axis_specs(
            &x_axis_options.range,
            &y_axis_options.range,
            &x_axis_info.transform,
            &y_axis_info.transform,
        )
        .expect("should not fail");

        let (inc_x, inc_y) = {
            (
                actual_ranges.0.start..=actual_ranges.0.end,
                actual_ranges.1.start..=actual_ranges.1.end,
            )
        };

        println!("X Bounds are: {}, {}", inc_x.start(), inc_x.end());

        let mapper = PlotMapper::new(
            width as f32,
            height as f32,
            inc_x,
            inc_y,
            RangeInclusive::new(x_axis_info.data_lower, x_axis_info.data_upper),
            RangeInclusive::new(y_axis_info.data_lower, y_axis_info.data_upper),
            x_axis_info.transform.clone(),
            y_axis_info.transform.clone(),
        );
        let options = DensityPlotOptions::new()
            .base(base_options)
            .colormap(ColorMaps::Jet)
            .x_axis(x_axis_options)
            .y_axis(y_axis_options)
            .point_size(0.35)
            .build()
            .expect("shouldn't fail");

        let mut render_config = RenderConfig::default();

        let plot_data = plot
            .render(data().into(), &options, &mut render_config)
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
                x_channel: x_axis_info().param.fluoro.clone(),
                y_channel: y_axis_info().param.fluoro.clone(),
                parental_gate_id,

            }
        }
    }
}
