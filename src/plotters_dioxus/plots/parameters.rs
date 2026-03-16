use anyhow::anyhow;
use core::f32;
use dioxus::prelude::*;
use flow_fcs::TransformType;
use flow_gates::{EventIndex, transforms::{
    get_plotting_area, pixel_to_raw, pixel_to_raw_y, raw_to_pixel, raw_to_pixel_y,
}};
use rustc_hash::FxHashMap;
use std::{
    ops::RangeInclusive,
    sync::{Arc, RwLock},
};

use crate::plotters_dioxus::{AxisInfo, gates::GateId};

#[derive(Clone, Debug, PartialEq)]
pub struct PlotMapper {
    view_width: f32,
    view_height: f32,
    x_data_axis_range: RangeInclusive<f32>,
    y_data_axis_range: RangeInclusive<f32>,
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
        x_data_axis_range: RangeInclusive<f32>,
        y_data_axis_range: RangeInclusive<f32>,
        x_data_range: RangeInclusive<f32>,
        y_data_range: RangeInclusive<f32>,
        x_transform: TransformType,
        y_transform: TransformType,
    ) -> Self {
        let (x_pix_range, y_pix_range) = get_plotting_area(width as u32, height as u32);

        Self {
            view_width: width,
            view_height: height,
            x_data_axis_range,
            y_data_axis_range,
            x_transform,
            y_transform,
            x_data_range,
            y_data_range,
            x_pix_range,
            y_pix_range,
        }
    }

    pub fn get_data_tolerance(&self, pixel_slop: f32) -> (f32, f32) {
        let x_span = self.x_data_axis_range.end() - self.x_data_axis_range.start();
        let y_span = self.y_data_axis_range.end() - self.y_data_axis_range.start();

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
        let dx_raw = pixel_to_raw(px, &self.x_data_axis_range, &self.x_pix_range, &xt);
        let dy_raw = pixel_to_raw_y(py, &self.y_data_axis_range, &self.y_pix_range, &yt);
        (dx_raw, dy_raw)
    }

    pub fn pixel_x_to_data(&self, x: f32, t: Option<TransformType>) -> f32 {
        let xt;
        if t.is_none() {
            xt = TransformType::Linear;
        } else {
            xt = t.unwrap();
        }
        let dx = pixel_to_raw(x, &self.x_data_axis_range, &self.x_pix_range, &xt);
        dx
    }

    pub fn pixel_y_to_data(&self, y: f32, t: Option<TransformType>) -> f32 {
        let yt;
        if t.is_none() {
            yt = TransformType::Linear;
        } else {
            yt = t.unwrap();
        }
        let dy = pixel_to_raw_y(y, &self.y_data_axis_range, &self.y_pix_range, &yt);
        dy
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

        let px = raw_to_pixel(dx, &self.x_data_axis_range, &self.x_pix_range, &xt);
        let py = raw_to_pixel_y(dy, &self.y_data_axis_range, &self.y_pix_range, &yt);

        (px, py)
    }

    pub fn width(&self) -> f32 {
        self.view_width
    }
    pub fn height(&self) -> f32 {
        self.view_height
    }

    pub fn x_axis_min_max(&self) -> RangeInclusive<f32> {
        self.x_data_axis_range.clone()
    }

    pub fn y_axis_min_max(&self) -> RangeInclusive<f32> {
        self.y_data_axis_range.clone()
    }

    pub fn x_data_min_max(&self) -> RangeInclusive<f32> {
        self.x_data_range.clone()
    }

    pub fn y_data_min_max(&self) -> RangeInclusive<f32> {
        self.y_data_range.clone()
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

#[derive(Clone)]
pub struct EventIndexMapped {
    pub event_index: Arc<EventIndex>,
    pub index_map: Arc<Vec<usize>>
}

impl PartialEq for EventIndexMapped {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.event_index, &other.event_index) 
        &&  Arc::ptr_eq(&self.index_map, &other.index_map) 
    }
}

#[derive(Default, Store, Clone)]
pub struct PlotStore {
    pub settings: Arc<RwLock<FxHashMap<Arc<str>, AxisInfo>>>,
    pub event_index_map: Option<EventIndexMapped>
}

#[store(pub name = PlotStoreImplExt)]
impl<Lens> Store<PlotStore, Lens> {
    fn add_new_axis_settings(&mut self, p: &Param, fcs_file: &flow_fcs::Fcs) {
        self.settings()
            .write()
            .write()
            .expect("Lock poisoned")
            .entry(p.fluoro.clone())
            .or_insert_with(|| {
                // Determine transform based on channel metadata
                let transform = fcs_file
                    .parameters
                    .get(p.fluoro.as_ref())
                    .map(|t| {
                        if t.is_fluorescence() {
                            TransformType::Arcsinh { cofactor: 6000.0 }
                        } else {
                            TransformType::Linear
                        }
                    })
                    .unwrap_or(TransformType::Linear);

                // Set logical lower bounds based on transform type
                let lower = if matches!(transform, TransformType::Linear) {
                    0.0
                } else {
                    -10000.0
                };

                let (data_lower, data_upper) = fcs_file
                    .get_minmax_of_parameter(&p.fluoro)
                    .expect("couldn't find parameter");

                AxisInfo::new_from_raw(
                    p.clone(),
                    lower,
                    4194304.0,
                    data_lower,
                    data_upper,
                    transform,
                )
            });
    }

    fn update_cofactor(
        &mut self,
        id: &Arc<str>,
        cofactor: f32,
    ) -> anyhow::Result<(AxisInfo, AxisInfo)> {
        let mut old = None;
        let mut new = None;

        match self.settings().write().write() {
            Ok(mut w) => {
                w.entry(id.clone()).and_modify(|axis| {
                    if let TransformType::Arcsinh { .. } = axis.transform {
                        let old_axis = std::mem::take(axis);
                        let new_axis = (&old_axis)
                            .into_archsinh(cofactor)
                            .unwrap_or(old_axis.clone());
                        new = Some(new_axis.clone());
                        old = Some(old_axis);
                        *axis = new_axis;
                    }
                });
            }
            Err(_) => return Err(anyhow!("Could not get write lock")),
        }

        if new.is_some() && old.is_some() {
            return Ok((old.unwrap(), new.unwrap()));
        }

        Err(anyhow!("Could not find axis"))
    }

    fn update_lower(
        &mut self,
        id: &GateId,
        lower: f32,
    ) -> anyhow::Result<(f32, f32, TransformType)> {
        let mut old_upper = None;
        let mut new_lower = None;
        let mut transform = None;
        self.settings()
            .write()
            .write()
            .expect("lock poisoned")
            .entry(id.clone())
            .and_modify(|axis_arc| {
                old_upper = Some(axis_arc.axis_upper);
                let new_axis_data = axis_arc.into_new_lower(lower);
                new_lower = Some(new_axis_data.axis_lower);
                transform = Some(new_axis_data.transform.clone());
                *axis_arc = new_axis_data;
            });

        if let (Some(upper), Some(lower), Some(transform)) = (old_upper, new_lower, transform) {
            Ok((lower, upper, transform))
        } else {
            Err(anyhow!("error modifying axis for {}", id.clone()))
        }
    }
    fn update_upper(
        &mut self,
        id: &GateId,
        upper: f32,
    ) -> anyhow::Result<(f32, f32, TransformType)> {
        let mut new_upper = None;
        let mut old_lower = None;
        let mut transform = None;

        self.settings()
            .write()
            .write()
            .expect("lock poisoned")
            .entry(id.clone())
            .and_modify(|axis_arc| {
                old_lower = Some(axis_arc.axis_lower);
                let new_axis_data = axis_arc.into_new_upper(upper);
                new_upper = Some(new_axis_data.axis_upper);
                transform = Some(new_axis_data.transform.clone());
                *axis_arc = new_axis_data;
            });

        if let (Some(upper), Some(lower), Some(transform)) = (new_upper, old_lower, transform) {
            Ok((lower, upper, transform))
        } else {
            Err(anyhow!("error modifying axis for {}", id.clone()))
        }
    }
}
