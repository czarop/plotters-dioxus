use polars::prelude::*;
use rustfft::{FftPlanner, num_complex::Complex};

pub struct DensityGrid {
    pub counts: Vec<f32>, // row-major, shape [n_bins x n_bins]
    pub n_bins: usize,
    pub x_range: (f64, f64),
    pub y_range: (f64, f64),
}

impl DensityGrid {
    pub fn from_column(
        xs: &Column,
        ys: &Column,
        n_bins: usize,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
        let xs = xs.f64().unwrap();
        let ys = ys.f64().unwrap();

        let mut counts = vec![0.0f32; n_bins * n_bins];

        let x_scale = n_bins as f64 / (x_range.1 - x_range.0);
        let y_scale = n_bins as f64 / (y_range.1 - y_range.0);

        for (x, y) in xs.into_iter().zip(ys.into_iter()) {
            let (Some(x), Some(y)) = (x, y) else { continue };

            let col = ((x - x_range.0) * x_scale) as isize;
            let row = ((y - y_range.0) * y_scale) as isize;

            if col >= 0 && col < n_bins as isize && row >= 0 && row < n_bins as isize {
                counts[row as usize * n_bins + col as usize] += 1.0;
            }
        }

        Self {
            counts,
            n_bins,
            x_range,
            y_range,
        }
    }

    /// Bin width in data-space units — used to convert peak offset back to data coords
    pub fn bin_width_x(&self) -> f64 {
        (self.x_range.1 - self.x_range.0) / self.n_bins as f64
    }

    pub fn bin_width_y(&self) -> f64 {
        (self.y_range.1 - self.y_range.0) / self.n_bins as f64
    }
}

/// Simple separable Gaussian blur applied in-place
pub fn gaussian_blur(grid: &mut DensityGrid, sigma: f32) {
    let n = grid.n_bins;
    let kernel = make_gaussian_kernel(sigma);
    let k = kernel.len();
    let half = k / 2;
    let mut tmp = vec![0.0f32; n * n];

    // Horizontal pass
    for row in 0..n {
        for col in 0..n {
            let mut val = 0.0f32;
            for (ki, &kw) in kernel.iter().enumerate() {
                let src_col = col as isize + ki as isize - half as isize;
                if src_col >= 0 && src_col < n as isize {
                    val += grid.counts[row * n + src_col as usize] * kw;
                }
            }
            tmp[row * n + col] = val;
        }
    }

    // Vertical pass
    for row in 0..n {
        for col in 0..n {
            let mut val = 0.0f32;
            for (ki, &kw) in kernel.iter().enumerate() {
                let src_row = row as isize + ki as isize - half as isize;
                if src_row >= 0 && src_row < n as isize {
                    val += tmp[src_row as usize * n + col] * kw;
                }
            }
            grid.counts[row * n + col] = val;
        }
    }
}

fn make_gaussian_kernel(sigma: f32) -> Vec<f32> {
    let radius = (3.0 * sigma).ceil() as usize;
    let size = 2 * radius + 1;
    let mut k: Vec<f32> = (0..size)
        .map(|i| {
            let x = i as f32 - radius as f32;
            (-x * x / (2.0 * sigma * sigma)).exp()
        })
        .collect();
    let sum: f32 = k.iter().sum();
    k.iter_mut().for_each(|v| *v /= sum);
    k
}

pub struct TranslationVector {
    pub dx_bins: i32, // positive = test is shifted right relative to QC
    pub dy_bins: i32,
    pub dx_data: f64, // in data-space units (e.g. arcsinh-transformed)
    pub dy_data: f64,
    pub peak_strength: f32, // for confidence scoring
}

fn fft2d(data: &mut Vec<Complex<f32>>, n: usize, planner: &mut FftPlanner<f32>, inverse: bool) {
    let fft = if inverse {
        planner.plan_fft_inverse(n)
    } else {
        planner.plan_fft_forward(n)
    };

    // FFT along rows
    for row in 0..n {
        let slice = &mut data[row * n..(row + 1) * n];
        fft.process(slice);
    }

    // FFT along columns — extract, process, write back
    let mut col_buf = vec![Complex::new(0.0f32, 0.0); n];
    for col in 0..n {
        for row in 0..n {
            col_buf[row] = data[row * n + col];
        }
        fft.process(&mut col_buf);
        for row in 0..n {
            data[row * n + col] = col_buf[row];
        }
    }

    if inverse {
        let norm = (n * n) as f32;
        data.iter_mut().for_each(|v| *v /= norm);
    }
}

pub fn cross_correlate(qc: &DensityGrid, test: &DensityGrid) -> TranslationVector {
    let n = qc.n_bins;
    assert_eq!(n, test.n_bins);

    let mut planner = FftPlanner::<f32>::new();

    let mut a: Vec<Complex<f32>> = qc.counts.iter().map(|&v| Complex::new(v, 0.0)).collect();
    let mut b: Vec<Complex<f32>> = test.counts.iter().map(|&v| Complex::new(v, 0.0)).collect();

    fft2d(&mut a, n, &mut planner, false);
    fft2d(&mut b, n, &mut planner, false);

    let mut product: Vec<Complex<f32>> = a
        .iter()
        .zip(b.iter())
        .map(|(ai, bi)| bi * ai.conj())
        .collect();

    fft2d(&mut product, n, &mut planner, true);

    let (peak_idx, peak_val) = product
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.re.partial_cmp(&b.re).unwrap())
        .unwrap();

    let raw_row = (peak_idx / n) as i32;
    let raw_col = (peak_idx % n) as i32;
    let n_i = n as i32;

    let dy_bins = if raw_row > n_i / 2 {
        raw_row - n_i
    } else {
        raw_row
    };
    let dx_bins = if raw_col > n_i / 2 {
        raw_col - n_i
    } else {
        raw_col
    };

    TranslationVector {
        dx_bins,
        dy_bins,
        dx_data: dx_bins as f64 * qc.bin_width_x(),
        dy_data: dy_bins as f64 * qc.bin_width_y(),
        peak_strength: peak_val.re,
    }
}

pub struct GateRules {
    pub lock_x: bool,
    pub lock_y: bool,
    pub max_translation: Option<f64>, // in data-space units
}

pub fn apply_constraints(mut t: TranslationVector, rules: &GateRules) -> TranslationVector {
    if rules.lock_x {
        t.dx_data = 0.0;
        t.dx_bins = 0;
    }
    if rules.lock_y {
        t.dy_data = 0.0;
        t.dy_bins = 0;
    }

    if let Some(max_t) = rules.max_translation {
        let mag = (t.dx_data.powi(2) + t.dy_data.powi(2)).sqrt();
        if mag > max_t {
            let scale = max_t / mag;
            t.dx_data *= scale;
            t.dy_data *= scale;
        }
    }

    t
}

pub fn compute_gate_translation(
    qc_parent_events: (&Column, &Column),
    test_parent_events: (&Column, &Column),
    axis_x_range: (f64, f64),
    axis_y_range: (f64, f64),
    rules: &GateRules,
    n_bins: usize,
    blur_sigma: f32,
) -> TranslationVector {
    let mut qc_grid = DensityGrid::from_column(
        qc_parent_events.0,
        qc_parent_events.1,
        n_bins,
        axis_x_range,
        axis_y_range,
    );
    let mut test_grid = DensityGrid::from_column(
        test_parent_events.0,
        test_parent_events.1,
        n_bins,
        axis_x_range,
        axis_y_range,
    );

    gaussian_blur(&mut qc_grid, blur_sigma);
    gaussian_blur(&mut test_grid, blur_sigma);

    let translation = cross_correlate(&qc_grid, &test_grid);
    apply_constraints(translation, rules)
}

use rand::prelude::*;
use rand_distr::Normal;

fn make_test_data() -> (DataFrame, DataFrame) {
    let mut rng = StdRng::seed_from_u64(42);

    // --- QC: three clusters ---
    // Cluster A: main population, centre (1.5, 1.2)
    // Cluster B: CD4+ T cells,    centre (2.8, 2.1)
    // Cluster C: debris/dim,      centre (0.4, 0.3)

    let clusters_qc: &[(f64, f64, f64, f64, usize)] = &[
        (1.5, 1.2, 0.15, 0.15, 3000),
        (2.8, 2.1, 0.20, 0.18, 800),
        (0.4, 0.3, 0.25, 0.20, 1500),
    ];

    let (qc_x, qc_y) = sample_clusters(clusters_qc, &mut rng);

    // --- Test: same clusters shifted by known amounts ---
    // Cluster A shifted (+0.3, +0.2)
    // Cluster B shifted (+0.3, +0.2)  <- same global shift
    // Cluster C shifted (+0.3, +0.2)

    let clusters_test: &[(f64, f64, f64, f64, usize)] = &[
        (1.5 + 0.3, 1.2 + 0.2, 0.15, 0.15, 2800),
        (2.8 + 0.3, 2.1 + 0.2, 0.20, 0.18, 750),
        (0.4 + 0.3, 0.3 + 0.2, 0.25, 0.20, 1400),
    ];

    let (test_x, test_y) = sample_clusters(clusters_test, &mut rng);

    let qc = df![
        "x" => qc_x,
        "y" => qc_y,
    ]
    .unwrap();

    let test = df![
        "x" => test_x,
        "y" => test_y,
    ]
    .unwrap();

    (qc, test)
}

fn sample_clusters(
    clusters: &[(f64, f64, f64, f64, usize)],
    rng: &mut StdRng,
) -> (Vec<f64>, Vec<f64>) {
    let mut xs = Vec::new();
    let mut ys = Vec::new();

    for &(cx, cy, sx, sy, n) in clusters {
        let dist_x = Normal::new(cx, sx).unwrap();
        let dist_y = Normal::new(cy, sy).unwrap();
        for _ in 0..n {
            xs.push(dist_x.sample(rng));
            ys.push(dist_y.sample(rng));
        }
    }

    (xs, ys)
}

#[cfg(test)]
mod tests {
    use super::*;
    // cargo test -- --nocapture
    #[test]
    fn test_cross_correlation_translation() {
        let (qc, test) = make_test_data();

        let qc_x = qc.column("x").unwrap();
        let qc_y = qc.column("y").unwrap();
        let test_x = test.column("x").unwrap();
        let test_y = test.column("y").unwrap();

        let axis_x_range = (-1.0, 4.5);
        let axis_y_range = (-1.0, 4.5);
        let n_bins = 64;
        let blur_sigma = 2.0;

        let rules = GateRules {
            lock_x: false,
            lock_y: false,
            max_translation: None,
        };

        let result = compute_gate_translation(
            (qc_x, qc_y),
            (test_x, test_y),
            axis_x_range,
            axis_y_range,
            &rules,
            n_bins,
            blur_sigma,
        );

        let bin_width_x = (axis_x_range.1 - axis_x_range.0) / n_bins as f64;
        let bin_width_y = (axis_y_range.1 - axis_y_range.0) / n_bins as f64;

        println!("--- Cross-Correlation Gate Alignment Result ---");
        println!(
            "Translation (bins):      dx={}, dy={}",
            result.dx_bins, result.dy_bins
        );
        println!(
            "Translation (data):      dx={:.4}, dy={:.4}",
            result.dx_data, result.dy_data
        );
        println!("Expected:                dx≈0.3000, dy≈0.2000");
        println!(
            "Error:                   dx={:.4}, dy={:.4}",
            (result.dx_data - 0.3).abs(),
            (result.dy_data - 0.2).abs()
        );
        println!(
            "Bin width (data units):  x={:.4}, y={:.4}",
            bin_width_x, bin_width_y
        );
        println!("Peak strength:           {:.6}", result.peak_strength);

        // Should recover the shift to within one bin width
        assert!(
            (result.dx_data - 0.3).abs() <= bin_width_x,
            "dx={:.4} not within one bin of 0.3",
            result.dx_data
        );
        assert!(
            (result.dy_data - 0.2).abs() <= bin_width_y,
            "dy={:.4} not within one bin of 0.2",
            result.dy_data
        );
    }
}
