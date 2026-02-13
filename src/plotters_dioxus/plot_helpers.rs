use flow_fcs::TransformType;
use flow_gates::transforms::{
    get_plotting_area, pixel_to_raw, pixel_to_raw_y, raw_to_pixel, raw_to_pixel_y,
};
use std::{ops::RangeInclusive, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct PlotMapper {
    view_width: f32,
    view_height: f32,
    x_data_range: RangeInclusive<f32>,
    y_data_range: RangeInclusive<f32>,
    x_transform: TransformType,
    y_transform: TransformType,
    x_pix_range: std::ops::Range<u32>,
    y_pix_range: std::ops::Range<u32>,
}

impl PlotMapper {
    pub fn new(
        width: f32,
        height: f32,
        x_range: RangeInclusive<f32>,
        y_range: RangeInclusive<f32>,
        x_transform: TransformType,
        y_transform: TransformType,
    ) -> Self {
        let (x_pix_range, y_pix_range) = get_plotting_area(width as u32, height as u32);

        Self {
            view_width: width,
            view_height: height,
            x_data_range: x_range,
            y_data_range: y_range,
            x_transform,
            y_transform,
            x_pix_range,
            y_pix_range,
        }
    }

    pub fn get_data_tolerance(&self, pixel_slop: f32) -> (f32, f32) {
        let x_span = self.x_data_range.end() - self.x_data_range.start();
        let y_span = self.y_data_range.end() - self.y_data_range.start();

        let plot_w = (self.x_pix_range.end - self.x_pix_range.start) as f32;
        let plot_h = (self.y_pix_range.end - self.y_pix_range.start) as f32;

        (
            (pixel_slop / plot_w) * x_span.abs(),
            (pixel_slop / plot_h) * y_span.abs(),
        )
    }

    pub fn pixel_to_data(
        &self,
        px: f32,
        py: f32,
        x_t: Option<TransformType>,
        y_t: Option<TransformType>,
    ) -> (f32, f32) {
        let xt;
        if x_t.is_none() {
            xt = TransformType::Linear;
        } else {
            xt = x_t.unwrap();
        }
        let yt;
        if y_t.is_none() {
            yt = TransformType::Linear;
        } else {
            yt = y_t.unwrap();
        }

        let dx = pixel_to_raw(px, &self.x_data_range, &self.x_pix_range, &xt);
        let dy = pixel_to_raw_y(py, &self.y_data_range, &self.y_pix_range, &yt);

        (dx, dy)
    }

    pub fn data_to_pixel(
        &self,
        dx: f32,
        dy: f32,
        x_t: Option<TransformType>,
        y_t: Option<TransformType>,
    ) -> (f32, f32) {
        let xt;
        if x_t.is_none() {
            xt = TransformType::Linear;
        } else {
            xt = x_t.unwrap();
        }
        let yt;
        if y_t.is_none() {
            yt = TransformType::Linear;
        } else {
            yt = y_t.unwrap();
        }

        let px = raw_to_pixel(dx, &self.x_data_range, &self.x_pix_range, &xt);
        let py = raw_to_pixel_y(dy, &self.y_data_range, &self.y_pix_range, &yt);

        (px, py)
    }

    pub fn width(&self) -> f32 {
        self.view_width
    }
    pub fn height(&self) -> f32 {
        self.view_height
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Param {
    pub marker: Arc<str>,
    pub fluoro: Arc<str>,
}

impl std::fmt::Display for Param {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.marker == self.fluoro {
            write!(f, "{}", self.marker)
        } else {
            let trimmed;
            if self.fluoro.ends_with("-A") {
                trimmed = &self.fluoro[..self.fluoro.len().saturating_sub(2)]
            } else {
                trimmed = &self.fluoro
            }
            write!(f, "{}-{}", self.marker, trimmed)
        }
    }
}