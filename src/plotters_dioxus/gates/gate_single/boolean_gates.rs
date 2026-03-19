use std::sync::Arc;

use flow_gates::{BooleanOperation, GateBuilder, GateGeometry, GateMode};

pub struct BooleanGate {
    inner: flow_gates::Gate,
}

impl BooleanGate {
    pub fn new(
        id: Arc<str>,
        linked_gate_ids: Vec<Arc<str>>,
        operation: BooleanOperation,
    ) -> anyhow::Result<Self> {
        let mut gate = GateBuilder::new(id.clone(), id.to_string())
            .mode(GateMode::global())
            .build()?;
        let geom = GateGeometry::Boolean {
            operation,
            operands: linked_gate_ids,
        };
        gate.geometry = geom;
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

    pub fn get_id(&self) -> Arc<str> {
        self.inner.id.clone()
    }

    pub fn get_name(&self) -> &str {
        &self.inner.name
    }

    pub fn get_mode(&self) -> &GateMode {
        &self.inner.mode
    }
}
