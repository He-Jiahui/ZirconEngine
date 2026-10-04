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
/// 后端身份、能力与排序层级；合并器依据输出中的 order 协调多个命中来源。
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

/// 从共享 RayMap 产生每指针命中组的后端边界；管线负责跨后端排序、遮挡与事件派发。
pub trait PickingBackend: Send + Sync {
    fn info(&self) -> PickingBackendInfo;
    fn collect_hits(&self, rays: &RayMap) -> Vec<PointerHits>;
}
