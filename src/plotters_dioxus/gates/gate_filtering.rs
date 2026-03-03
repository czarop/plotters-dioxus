use flow_fcs::Fcs;
use flow_gates::{Gate, GateGeometry};
use polars::prelude::*;

pub fn filter_events_to_mask(fcs: &Fcs, gate: &Gate) -> anyhow::Result<BooleanChunked> {
    let df = &fcs.data_frame;
    let x_param = gate.x_parameter_channel_name();
    let y_param = gate.y_parameter_channel_name();

    match &gate.geometry {
        GateGeometry::Rectangle { min, max } => {
            // Polars native SIMD comparison - Extremely Fast
            let x_series = df.column(x_param)?.f32()?;
            let y_series = df.column(y_param)?.f32()?;

            let (minx, miny, maxx, maxy) = {
                (
                    min.get_coordinate(x_param)
                        .ok_or(anyhow::anyhow!("x_coord not found"))?,
                    min.get_coordinate(y_param)
                        .ok_or(anyhow::anyhow!("y_coord not found"))?,
                    max.get_coordinate(x_param)
                        .ok_or(anyhow::anyhow!("x_coord not found"))?,
                    max.get_coordinate(y_param)
                        .ok_or(anyhow::anyhow!("y_coord not found"))?,
                )
            };

            // This generates the mask in one pass with zero manual indexing
            let mask =
                x_series.gt(minx) & x_series.lt(maxx) & y_series.gt(miny) & y_series.lt(maxy);

            Ok(mask)
        }
        GateGeometry::Ellipse {
            center,
            radius_x,
            radius_y,
            angle,
        } => {
            // 1. EXTRACTION: Get coordinates from the HashMap once
            let h = center
                .get_coordinate(&x_param)
                .ok_or_else(|| anyhow::anyhow!("Missing X"))?;
            let k = center
                .get_coordinate(&y_param)
                .ok_or_else(|| anyhow::anyhow!("Missing Y"))?;

            // 2. PRE-CALCULATION: Trig and Bounding Box
            let cos_a = angle.cos();
            let sin_a = angle.sin();

            // Calculate Bounding Box (AABB)
            let x_extent = ((radius_x * cos_a).powi(2) + (radius_y * sin_a).powi(2)).sqrt();
            let y_extent = ((radius_x * sin_a).powi(2) + (radius_y * cos_a).powi(2)).sqrt();

            let min_x = h - x_extent;
            let max_x = h + x_extent;
            let min_y = k - y_extent;
            let max_y = k + y_extent;

            // Pre-calc quadratic coefficients for the rotated formula
            let r_x2 = radius_x.powi(2);
            let r_y2 = radius_y.powi(2);
            let a_coeff = cos_a.powi(2) / r_x2 + sin_a.powi(2) / r_y2;
            let b_coeff = 2.0 * sin_a * cos_a * (1.0 / r_x2 - 1.0 / r_y2);
            let c_coeff = sin_a.powi(2) / r_x2 + cos_a.powi(2) / r_y2;

            // 3. SCAN: 10 Million Rows
            let x_series = df.column(x_param)?.f32()?;
            let y_series = df.column(y_param)?.f32()?;
            let xs = x_series.cont_slice()?;
            let ys = y_series.cont_slice()?;

            let mask: BooleanChunked = xs
                .iter()
                .zip(ys.iter())
                .map(|(&px, &py)| {
                    // STEP A: Cheap Bounding Box Pre-Check
                    if px < min_x || px > max_x || py < min_y || py > max_y {
                        return false;
                    }

                    // STEP B: Precise Rotated Ellipse Math
                    let dx = px - h;
                    let dy = py - k;
                    (a_coeff * dx * dx + b_coeff * dx * dy + c_coeff * dy * dy) <= 1.0
                })
                .collect();

            Ok(mask.with_name("mask".into()))
        }
        GateGeometry::Polygon { nodes, .. } => {
            let coords: Vec<(f32, f32)> = nodes
                .iter()
                .filter_map(|node| {
                    let x = node.get_coordinate(&x_param)?;
                    let y = node.get_coordinate(&y_param)?;
                    Some((x, y))
                })
                .collect();

            if coords.len() < 3 {
                return Ok(BooleanChunked::full("mask".into(), false, df.height()));
            }

            // 2. Pre-calculate the Bounding Box (AABB) from our flat coords
            let mut min_x = f32::MAX;
            let mut max_x = f32::MIN;
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;

            for (x, y) in &coords {
                if *x < min_x {
                    min_x = *x;
                }
                if *x > max_x {
                    max_x = *x;
                }
                if *y < min_y {
                    min_y = *y;
                }
                if *y > max_y {
                    max_y = *y;
                }
            }

            // 3. Get raw data slices
            let x_series = df.column(x_param)?.f32()?;
            let y_series = df.column(y_param)?.f32()?;
            let xs = x_series.cont_slice()?;
            let ys = y_series.cont_slice()?;

            // 4. Vectorized Scan
            let mask: BooleanChunked = xs
                .iter()
                .zip(ys.iter())
                .map(|(&px, &py)| {
                    // Fast Bounding Box Reject
                    if px < min_x || px > max_x || py < min_y || py > max_y {
                        return false;
                    }

                    // Ray Casting logic using our flat coords
                    let mut inside = false;
                    let mut j = coords.len() - 1;
                    for i in 0..coords.len() {
                        let (v_ix, v_iy) = coords[i];
                        let (v_jx, v_jy) = coords[j];

                        if ((v_iy > py) != (v_jy > py))
                            && (px < (v_jx - v_ix) * (py - v_iy) / (v_jy - v_iy) + v_ix)
                        {
                            inside = !inside;
                        }
                        j = i;
                    }
                    inside
                })
                .collect();

            Ok(mask.with_name("mask".into()))
        }
        _ => {
            // Fallback for complex polygons where an index actually helps
            let indices = flow_gates::filter_events_by_gate(fcs, gate, None)?;

            // Convert indices to mask (slightly slower, but necessary for complex shapes)
            let mut mask_vec = vec![false; df.height()];
            for idx in indices {
                mask_vec[idx] = true;
            }
            Ok(BooleanChunked::from_slice("mask".into(), &mask_vec))
        }
    }
}

pub fn filter_events_by_hierarchy_to_mask(
    fcs: &Fcs,
    gate_chain: &[&Gate],
) -> Result<BooleanChunked, anyhow::Error> {
    let event_count = fcs.data_frame.height();

    // Start with "True" for everyone (Identity mask)
    let mut final_mask = BooleanChunked::full("mask".into(), true, event_count);

    for gate in gate_chain {
        // Generate the mask for the current gate ONLY
        let gate_mask = filter_events_to_mask(fcs, gate)?;

        // Bitwise AND: narrow the population
        final_mask = final_mask & gate_mask;
    }

    Ok(final_mask)
}
