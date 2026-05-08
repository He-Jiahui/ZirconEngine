use super::{PointerHits, RayMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PickingBackendCapability {
    CpuRayCast,
    OverlayShapes,
    RenderableBounds,
    Ui,
    GpuPicking,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PickingBackendInfo {
    pub name: String,
    pub capabilities: Vec<PickingBackendCapability>,
    pub order: f32,
}

impl PickingBackendInfo {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            capabilities: Vec::new(),
            order: 0.0,
        }
    }

    pub fn with_capability(mut self, capability: PickingBackendCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_order(mut self, order: f32) -> Self {
        self.order = order;
        self
    }

    pub fn supports(&self, capability: PickingBackendCapability) -> bool {
        self.capabilities.contains(&capability)
    }
}

pub trait PickingBackend: Send + Sync {
    fn info(&self) -> PickingBackendInfo;
    fn collect_hits(&self, rays: &RayMap) -> Vec<PointerHits>;
}
