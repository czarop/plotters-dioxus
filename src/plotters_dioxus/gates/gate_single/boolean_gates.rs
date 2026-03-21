use std::sync::Arc;

use flow_gates::{BooleanOperation, GateGeometry};

use crate::plotters_dioxus::gates::gate_traits::DrawableGate;

#[derive(PartialEq, Clone)]
pub struct BooleanGate {
    inner: flow_gates::Gate,
}

impl BooleanGate {
    pub fn new(
        id: Arc<str>,
        name: String,
        linked_gate_ids: Vec<Arc<str>>,
        operation: BooleanOperation,
        x_param: Arc<str>,
        y_param: Arc<str>,
    ) -> anyhow::Result<Self> {
        let geom = GateGeometry::Boolean {
            operation,
            operands: linked_gate_ids,
        };
        let gate = flow_gates::Gate::new(id.clone(), name, geom, x_param, y_param);
        
        Ok(Self { inner: gate })
    }

    pub fn get_operation(&self) -> BooleanOperation {
        let GateGeometry::Boolean { operation, .. } = self.inner.geometry else {
            unreachable!()
        };
        operation
    }

    pub fn get_operands(&self) -> &[Arc<str>] {
        let GateGeometry::Boolean { operands, .. } = &self.inner.geometry else {
            unreachable!()
        };
        operands
    }

}

impl DrawableGate for BooleanGate {
    fn get_gate_ref(&self, _id: Option<&str>) -> Option<&flow_gates::Gate> {
        Some(&self.inner)
    }
    fn get_name(&self) -> &str {
        &self.inner.name
    }
    fn get_inner_gate_ids(&self) -> Vec<Arc<str>> {
        vec![]
    }

    fn is_finalised(&self) -> bool {
        true
    }

    fn draw_self(
        &self,
        _is_selected: bool,
        _drag_point: Option<crate::plotters_dioxus::gates::gate_drag::PointDragData>,
        _plot_map: &crate::plotters_dioxus::plots::parameters::PlotMapper,
        _gate_stats: &Option<crate::plotters_dioxus::gates::gate_types::GateStats>
    ) -> Vec<crate::plotters_dioxus::gates::gate_types::GateRenderShape> {
        vec![]
    }

    fn is_composite(&self) -> bool {
        false
    }

    fn get_id(&self) -> Arc<str> {
        self.inner.id.clone()
    }

    fn get_params(&self) -> (Arc<str>, Arc<str>) {
        self.inner.parameters.clone()
    }

    fn is_point_on_perimeter(
        &self,
        point: (f32, f32),
        tolerance: (f32, f32),
        mapper: &crate::plotters_dioxus::plots::parameters::PlotMapper,
    ) -> Option<f32> {
        None
    }

    fn match_to_plot_axis(
        &self,
        plot_x: &str,
        plot_y: &str,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        let (x, y) = (&self.inner.parameters.0, &self.inner.parameters.1);
        if plot_x == x.as_ref() && *plot_y == *y.as_ref() {
            return Ok(None);
        }

        let mut new_self = self.clone();
        let new_params = (y.clone(), x.clone());
        new_self.inner.parameters = new_params;
        Ok(Some(Box::new(new_self)))
        
    }

    fn recalculate_gate_for_rescaled_axis(
        &self,
        param: std::sync::Arc<str>,
        old_transform: &flow_fcs::TransformType,
        new_transform: &flow_fcs::TransformType,
        data_range: (f32, f32),
        axis_range: (f32, f32),
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        return Ok(self.clone_box())
    }

    fn rotate_gate(
        &self,
        mouse_position: (f32, f32),
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        return Ok(None)
    }

    fn replace_point(
        &self,
        new_point: (f32, f32),
        point_index: usize,
        plot_map: &crate::plotters_dioxus::plots::parameters::PlotMapper,
    ) -> anyhow::Result<Box<dyn DrawableGate>> {
        return Ok(self.clone_box())
    }

    fn replace_points(
        &self,
        gate_drag_data: crate::plotters_dioxus::gates::gate_drag::GateDragData,
    ) -> anyhow::Result<Option<Box<dyn DrawableGate>>> {
        return Ok(None)
    }
    
    fn clone_box(&self) -> Box<dyn DrawableGate> {
        Box::new(self.clone())
    }

}
