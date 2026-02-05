use flow_fcs::TransformType;
use std::ops::RangeInclusive;
// Import the library functions you discovered
use flow_gates::transforms::{
    get_plotting_area, pixel_to_raw, pixel_to_raw_y, raw_to_pixel, raw_to_pixel_y,
};

// #[derive(Clone, Debug, PartialEq)]
// pub struct PlotMapper {
//     pub view_width: f32,
//     pub view_height: f32,
//     pub x_data_min: f32,
//     pub x_data_max: f32,
//     pub y_data_min: f32,
//     pub y_data_max: f32,
//     pub plot_left: f32,
//     pub plot_width: f32,
//     pub plot_top: f32,
//     pub plot_height: f32,
// }

// impl PlotMapper {
//     pub fn from_plot_helper(
//         helper: &flow_plots::render::plothelper::PlotHelper,
//         width: f32,
//         height: f32,
//     ) -> Self {
//         Self {
//             view_width: width,
//             view_height: height,
//             x_data_min: helper.x_data_min,
//             x_data_max: helper.x_data_max,
//             y_data_min: helper.y_data_min,
//             y_data_max: helper.y_data_max,
//             plot_width: helper.plot_width,
//             plot_height: helper.plot_height,
//             plot_left: helper.plot_left,
//             plot_top: helper.plot_top,
//         }
//     }
//     /// Helper to get the internal plotting ranges as the library expects them
//     fn get_ranges(
//         &self,
//     ) -> (
//         RangeInclusive<f32>,
//         RangeInclusive<f32>,
//         (std::ops::Range<u32>, std::ops::Range<u32>),
//     ) {
//         let x_data_range = self.x_data_min..=self.x_data_max;
//         let y_data_range = self.y_data_min..=self.y_data_max;
//         let pixel_ranges = get_plotting_area(self.view_width as u32, self.view_height as u32);
//         (x_data_range, y_data_range, pixel_ranges)
//     }

//     /// Convert a single pixel click to raw data space
//     pub fn pixel_to_data(
//         &self,
//         click_x: f32,
//         click_y: f32,
//         x_transform: Option<&TransformType>,
//         y_transform: Option<&TransformType>,
//     ) -> Option<(f32, f32)> {
//         let (x_data_range, y_data_range, (x_pix_range, y_pix_range)) = self.get_ranges();

//         let x_transform_local;
//         let y_transform_local;
//         if x_transform.is_none() {
//             x_transform_local = &TransformType::Linear;
//         } else {
//             x_transform_local = x_transform.unwrap()
//         }
//         if y_transform.is_none() {
//             y_transform_local = &TransformType::Linear;
//         } else {
//             y_transform_local = y_transform.unwrap()
//         }

//         // The library handles clamping and coordinate inversion internally
//         let data_x = pixel_to_raw(click_x, &x_data_range, &x_pix_range, x_transform_local);
//         let data_y = pixel_to_raw_y(click_y, &y_data_range, &y_pix_range, y_transform_local);

//         Some((data_x, data_y))
//     }

//     /// Convert a single raw data point to pixel coordinates
//     pub fn data_to_pixel(
//         &self,
//         data_x: f32,
//         data_y: f32,
//         x_transform: Option<&TransformType>,
//         y_transform: Option<&TransformType>,
//     ) -> (f32, f32) {
//         let x_transform_local;
//         let y_transform_local;
//         if x_transform.is_none() {
//             x_transform_local = &TransformType::Linear;
//         } else {
//             x_transform_local = x_transform.unwrap()
//         }
//         if y_transform.is_none() {
//             y_transform_local = &TransformType::Linear;
//         } else {
//             y_transform_local = y_transform.unwrap()
//         }

//         let (x_data_range, y_data_range, (x_pix_range, y_pix_range)) = self.get_ranges();

//         let pix_x = raw_to_pixel(data_x, &x_data_range, &x_pix_range, x_transform_local);
//         let pix_y = raw_to_pixel_y(data_y, &y_data_range, &y_pix_range, y_transform_local);

//         (pix_x, pix_y)
//     }

//     /// Transforms a batch of raw data coordinates into screen pixel coordinates.
//     pub fn map_data_to_pixels(
//         &self,
//         data_points: &[(f32, f32)],
//         x_transform: Option<&TransformType>,
//         y_transform: Option<&TransformType>,
//     ) -> Vec<(f32, f32)> {
//         let x_transform_local;
//         let y_transform_local;
//         if x_transform.is_none() {
//             x_transform_local = Some(&TransformType::Linear);
//         } else {
//             x_transform_local = x_transform;
//         }
//         if y_transform.is_none() {
//             y_transform_local = Some(&TransformType::Linear);
//         } else {
//             y_transform_local = y_transform;
//         }

//         data_points
//             .iter()
//             .map(|&(x, y)| self.data_to_pixel(x, y, x_transform_local, y_transform_local))
//             .collect()
//     }

//     /// Transforms a batch of screen pixels into raw data coordinates.
//     pub fn map_pixels_to_data(
//         &self,
//         pixel_points: &[(f32, f32)],
//         x_transform: Option<&TransformType>,
//         y_transform: Option<&TransformType>,
//     ) -> Vec<(f32, f32)> {
//         let x_transform_local;
//         let y_transform_local;
//         if x_transform.is_none() {
//             x_transform_local = Some(&TransformType::Linear);
//         } else {
//             x_transform_local = x_transform;
//         }
//         if y_transform.is_none() {
//             y_transform_local = Some(&TransformType::Linear);
//         } else {
//             y_transform_local = y_transform;
//         }
//         pixel_points
//             .iter()
//             .filter_map(|&(px, py)| {
//                 self.pixel_to_data(px, py, x_transform_local, y_transform_local)
//             })
//             .collect()
//     }

//     /// Calculates the data-space equivalent of a pixel distance (slop).
//     /// This ensures the hit-test area is consistent with the visual plot.
//     pub fn get_data_tolerance(&self, pixel_slop: f32) -> (f32, f32) {
//         // Calculate total data ranges (these should be in your transformed/arcsinh space)
//         let data_width = (self.x_data_max - self.x_data_min).abs();
//         let data_height = (self.y_data_max - self.y_data_min).abs();

//         // Ratio: (Pixel Distance / Total Pixels) * Total Data Range
//         // This calculates how much "data" exists per pixel, then multiplies by the slop.
//         let tx = (pixel_slop / self.view_width) * data_width;
//         let ty = (pixel_slop / self.view_height) * data_height;

//         (tx, ty)
//     }

//     pub fn map_to_svg(&self, x: f32, y: f32) -> (f32, f32) {
//         // 1. Normalize data to 0.0 - 1.0 range
//         let rel_x = (x - self.x_data_min) / (self.x_data_max - self.x_data_min);
//         let rel_y = (y - self.y_data_min) / (self.y_data_max - self.y_data_min);

//         // 2. Map to the "Plot Area" and add the margin offsets
//         let svg_x = self.plot_left + (rel_x * self.plot_width);

//         // 3. Flip Y: SVG 0 is top, Plotters Y-max is top
//         let svg_y = self.plot_top + ((1.0 - rel_y) * self.plot_height);

//         (svg_x, svg_y)
//     }
// }

#[derive(Clone, Debug, PartialEq)]
pub struct PlotMapper {
    view_width: f32,
    view_height: f32,
    x_data_range: RangeInclusive<f32>,
    y_data_range: RangeInclusive<f32>,
    x_transform: TransformType,
    y_transform: TransformType,
    // Pre-calculated pixel bounds for performance
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
            (pixel_slop / plot_h) * y_span.abs()
        )
    }


    pub fn pixel_to_data(&self, px: f32, py: f32, x_t: Option<TransformType>, y_t: Option<TransformType>) -> (f32, f32) {
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

    pub fn data_to_pixel(&self, dx: f32, dy: f32, x_t: Option<TransformType>, y_t: Option<TransformType>) -> (f32, f32) {
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