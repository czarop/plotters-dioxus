use base64::{Engine as _, prelude::BASE64_STANDARD};

use ndarray::Array2;
// use ndarray_ndimage::gaussian_filter;
use crate::colormap::ColorMap;
use png::{BitDepth, ColorType, Encoder};
use rayon::prelude::*;
use std::io::Cursor;

pub struct DensityPlot {
    pub bins: Array2<u32>, // Event counts per bin
    pub grid_size: usize,  // e.g., 256
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
}

impl DensityPlot {
    pub fn new(
        x_data: &[f64],
        y_data: &[f64],
        grid_size: usize,
        x_range: Option<(f64, f64)>,
        y_range: Option<(f64, f64)>,
    ) -> Self {
        assert_eq!(x_data.len(), y_data.len());

        // Auto-calculate ranges if not provided
        let x_range = x_range.unwrap_or_else(|| {
            let min = x_data.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = x_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (min, max)
        });

        let y_range = y_range.unwrap_or_else(|| {
            let min = y_data.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = y_data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            (min, max)
        });

        let mut bins = Array2::<u32>::zeros((grid_size, grid_size));

        // Parallel binning for 1M events
        let x_scale = (grid_size as f64) / (x_range.1 - x_range.0);
        let y_scale = (grid_size as f64) / (y_range.1 - y_range.0);

        // Chunk the data to avoid contention on bins
        let chunk_size = 10_000;
        let local_bins: Vec<Array2<u32>> = (0..x_data.len())
            .step_by(chunk_size)
            .par_bridge()
            .map(|start| {
                let end = (start + chunk_size).min(x_data.len());
                let mut local = Array2::<u32>::zeros((grid_size, grid_size));

                for i in start..end {
                    let x_bin = ((x_data[i] - x_range.0) * x_scale) as isize;
                    // let y_bin = ((y_data[i] - y_range.0) * y_scale) as isize;
                    let y_bin = (grid_size as isize - 1) - ((y_data[i] - y_range.0) * y_scale) as isize;

                    // Bounds check
                    if x_bin >= 0
                        && x_bin < grid_size as isize
                        && y_bin >= 0
                        && y_bin < grid_size as isize
                    {
                        local[[y_bin as usize, x_bin as usize]] += 1;
                    }
                }
                local
            })
            .collect();

        // Merge local histograms
        for local in local_bins {
            bins += &local;
        }

        Self {
            bins,
            grid_size,
            x_range,
            y_range,
        }
    }

    /// Apply log transform for better visualization (like FlowJo)
    pub fn to_log_density(&self) -> Array2<f64> {
        self.bins.mapv(|count| {
            if count > 0 {
                (count as f64).ln_1p() // log(1 + count)
            } else {
                0.0
            }
        })
    }

    /// Apply Gaussian smoothing (optional, for prettier plots)
    // pub fn smooth(&self, sigma: f64) -> Array2<f64> {
    //     gaussian_blur(&self.bins.mapv(|x| x as f64), sigma)
    // }

    /// Convert to RGB image with colormap
    pub fn to_rgb(&self, colormap: &ColorMap) -> Vec<u8> {
        let density = self.to_log_density();
        let max_density = density.iter().cloned().fold(0.0, f64::max);

        let mut rgb = Vec::with_capacity(self.grid_size * self.grid_size * 3);

        for &val in density.iter() {
            let normalized = if max_density > 0.0 {
                val / max_density
            } else {
                0.0
            };
            let color = colormap.get_color(normalized);
            rgb.extend_from_slice(&[color.r, color.g, color.b]);
        }

        rgb
    }
}

pub fn density_plot_to_base64(
    points: &[(f64, f64)], // Changed from separate x_data, y_data
    grid_size: usize,
    colormap: &ColorMap,
) -> Result<String, Box<dyn std::error::Error>> {
    let x_range = Some((-2.0, 6.0));
    let y_range = Some((-2.0, 6.0));
    // Unzip the points into separate x and y vectors
    let (x_data, y_data): (Vec<f64>, Vec<f64>) = points.iter().copied().unzip();

    // Create density plot
    let plot = DensityPlot::new(&x_data, &y_data, grid_size, x_range, y_range);

    // Get RGB data
    let rgb_data = plot.to_rgb(colormap);

    // Encode to PNG
    let mut png_data = Vec::new();
    {
        let mut encoder = Encoder::new(
            Cursor::new(&mut png_data),
            grid_size as u32,
            grid_size as u32,
        );
        encoder.set_color(ColorType::Rgb);
        encoder.set_depth(BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(&rgb_data)?;
    }

    // Convert to base64 data URI
    let buffer_base64 = BASE64_STANDARD.encode(&png_data);
    Ok(format!("data:image/png;base64,{}", buffer_base64))
}

// fn gaussian_blur(data: &Array2<f64>, sigma: f64) -> Array2<f64> {
//     let mut output = data.clone();
//     &mut gaussian_filter(
//         &data.view(),
//         &mut output.view_mut(),
//         &[sigma, sigma], // sigma for each axis
//         0,               // order (0 = just blur, not derivative)
//         ndarray_ndimage::BorderMode::Constant(0.0),
//     );
//     output
// }

// fn gaussian_kernel_2d(size: usize, sigma: f64) -> Array2<f64> {
//     let center = size / 2;
//     let mut kernel = Array2::zeros((size, size));
//     let mut sum = 0.0;

//     for i in 0..size {
//         for j in 0..size {
//             let x = i as f64 - center as f64;
//             let y = j as f64 - center as f64;
//             let val = (-(x * x + y * y) / (2.0 * sigma * sigma)).exp();
//             kernel[[i, j]] = val;
//             sum += val;
//         }
//     }

//     kernel / sum // Normalize
// }
