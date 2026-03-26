#![allow(non_snake_case)]
use std::{ops::RangeInclusive, sync::Arc};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use dioxus::prelude::*;

use flow_plots::{
    BasePlotOptions, ColorMaps, DensityPlot, DensityPlotOptions, Plot, render::RenderConfig
};

use crate::gate_editor::{AxisInfo, gates::{draw_gates::GateLayer}, plots::axis_store::PlotMapper};

#[component]
pub fn PseudoColourPlot(
    data: ReadSignal<Vec<(f32, f32)>>,
    size: ReadSignal<(u32, u32)>,
    x_axis_info: ReadSignal<AxisInfo>,
    y_axis_info: ReadSignal<AxisInfo>,
    parental_gate_id: ReadSignal<Option<Arc<str>>>,
) -> Element {
    // let mut plot_image_src = use_signal(|| String::new());
    let mut plot_map = use_signal(|| None::<Arc<PlotMapper>>);
    use_context_provider::<Signal<Option<Arc<PlotMapper>>>>(|| plot_map);

    let render_result = use_resource(move || { 

        async move {
        let x_axis_info = x_axis_info();
        let y_axis_info = y_axis_info();
        let (width, height) = size();
        let data = data.clone()();

        let result = tokio::task::spawn_blocking(move || -> Result<(String, Arc<PlotMapper>), anyhow::Error> {
        let plot = DensityPlot::new();
        let base_options = BasePlotOptions::new()
            .width(width)
            .height(height)
            .title("My Density Plot")
            // .show_colorbar(false)
            .build()?;

        let x_axis_options = flow_plots::AxisOptions::new()
            .range(x_axis_info.axis_lower..=x_axis_info.axis_upper)
            .transform(x_axis_info.transform.clone())
            .label(&x_axis_info.param.to_string())
            .build()?;
        let y_axis_options = flow_plots::AxisOptions::new()
            .range(y_axis_info.axis_lower..=y_axis_info.axis_upper)
            .transform(y_axis_info.transform.clone())
            .label(y_axis_info.param.to_string())
            .build()?;

        let actual_ranges = flow_plots::create_axis_specs(
            &x_axis_options.range,
            &y_axis_options.range,
            &x_axis_info.transform,
            &y_axis_info.transform,
        )?;

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
            // .plot_type(PlotType::Density)
            .colormap(ColorMaps::Jet)
            .x_axis(x_axis_options)
            .y_axis(y_axis_options)
            .point_size(0.35)
            .build()?;

        let mut render_config = RenderConfig::default();
        let data_final: flow_plots::ScatterPlotData = data.into();
        let plot_data = plot
            .render(data_final, &options, &mut render_config)?;

        let base64_str = BASE64_STANDARD.encode(&plot_data);
        Ok((format!("data:image/jpeg;base64,{}", base64_str), Arc::new(mapper)))
    }).await;

    match result{
        Ok(r) => r,
        Err(e) => Err(anyhow::anyhow!("Failed to generate plot {}", e)),
    }


} });


    rsx! {
        match &*render_result.read() {
            Some(Ok((data, map))) => {
                plot_map.set(Some(map.clone()));
                let size = size();
                rsx! {
                    div { style: "position: relative; width: {size.0}px; height: {size.1}px;",
                        img {
                            style: "user-select: none; -webkit-user-select: none;",
                            src: "{data}",
                            width: "{size.0}",
                            height: "{size.1}",
                        }
                        GateLayer {
                            x_channel: x_axis_info().param.fluoro.clone(),
                            y_channel: y_axis_info().param.fluoro.clone(),
                            parental_gate_id,

                        }
                    }
                }

            }
            Some(Err(e)) => {
                rsx! {
                    {e.to_string()}
                }
            }
            None => {
                let size = size();
                let style = format!("width: {}px; height: {}px;", size.0, size.1);
                rsx! {
                    div { style, class: "spinner-container",
                        div { class: "spinner" }
                        span { style: "margin-top: 10px; font-size: 12px; color: #666;", "Rendering Plot..." }
                    }
                }
            }
        }

    }
}
