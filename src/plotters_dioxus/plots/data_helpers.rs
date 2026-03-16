use std::sync::Arc;

use crate::plotters_dioxus::gates::gate_store::GateStateStoreExt;
use crate::plotters_dioxus::gates::{GateState, gate_traits::DrawableGate};
use dioxus::prelude::*;
use dioxus::stores::Store;
use flow_fcs::{Fcs};
use flow_gates::{EventIndex, Gate};

use polars::prelude::*;
use tokio::task;

pub async fn get_flow_data(path: std::path::PathBuf) -> Result<Arc<Fcs>, Arc<anyhow::Error>> {
    task::spawn_blocking(move || {
        let fcs_file = Fcs::open(path.to_str().unwrap_or_default())?;
        Ok(Arc::new(fcs_file))
    })
    .await
    .map_err(|e| Arc::new(e.into()))?
}

pub async fn get_filtered_dataframe(
    df: Arc<DataFrame>,
    parental_gate_id: &Option<Arc<str>>,
) -> Result<Arc<DataFrame>, anyhow::Error> {
    let df_clone = df.clone();
    let gate_store: Store<GateState> = use_context::<Store<GateState>>();
    let gate_chain: Option<Vec<(Arc<str>, Arc<dyn DrawableGate>)>> =
        if let Some(parent) = parental_gate_id {
            let arcs: Vec<(Arc<str>, Arc<dyn DrawableGate>)> = gate_store
                .hierarchy()
                .peek()
                .get_chain_to_root(parent)
                .iter()
                .filter_map(|id| {
                    gate_store
                        .primary_and_subgate_registry()
                        .peek()
                        .get(id)
                        .map(|g| (id.clone(), g.clone()))
                })
                .collect();

            if arcs.is_empty() { None } else { Some(arcs) }
        } else {
            None
        };

    task::spawn_blocking(move || -> Result<Arc<DataFrame>, anyhow::Error> {
        if let Some(chain) = gate_chain {
            let gate_refs: Vec<&Gate> = chain
                .iter()
                .filter_map(|(id, gate)| gate.get_gate_ref(Some(&id)))
                .collect();

            // 1. Get the final narrowed mask for the whole hierarchy
            let mask = super::super::gates::gate_filtering::filter_events_by_hierarchy_to_mask(
                &df, &gate_refs,
            )?;
            // 2. Filter the dataframe once at the end
            Ok(df.filter(&mask)?.into())
        } else {
            Ok(df_clone)
        }

    })
    .await?
}

pub async fn zip_cols_from_filtered_df(
    df: Arc<DataFrame>,
    col1_name: Arc<str>,
    col2_name: Arc<str>,
) -> Result<Vec<(f32, f32)>, anyhow::Error> {
    let df_clone = df.clone();

    task::spawn_blocking(move || -> Result<Vec<(f32, f32)>, anyhow::Error> {
        let x_series = df_clone.column(&col1_name)?.f32()?;
        let y_series = df_clone.column(&col2_name)?.f32()?;

        let zipped_cols: Vec<(f32, f32)> = x_series
            .into_iter()
            .zip(y_series.into_iter())
            .filter_map(|(x, y)| match (x, y) {
                (Some(vx), Some(vy)) => Some((vx, vy)),
                _ => None,
            })
            .collect();

        Ok(zipped_cols)
        })
    .await?
}

pub fn get_event_mask_from_scaled_df(
    scaled_df: Arc<DataFrame>,
    col1_name: Arc<str>,
    col2_name: Arc<str>,
) -> anyhow::Result<Arc<EventIndex>> {
    let col1_name = col1_name.clone();
    let col2_name = col2_name.clone();

    let x_rechunked = scaled_df.column(&col1_name)?.f32()?.rechunk();
    let y_rechunked = scaled_df.column(&col2_name)?.f32()?.rechunk();
    let x_slice = x_rechunked
        .cont_slice()
        .map_err(|_| anyhow::anyhow!("Failed to get contiguous slice for X"))?;
    let y_slice = y_rechunked
        .cont_slice()
        .map_err(|_| anyhow::anyhow!("Failed to get contiguous slice for Y"))?;


    match EventIndex::build(x_slice, y_slice){
        Ok(index) => Ok(Arc::new(index)),
        Err(e) => Err(anyhow::anyhow!("{e}")),
    }

}

