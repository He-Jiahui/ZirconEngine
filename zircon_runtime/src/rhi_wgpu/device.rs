use crate::rhi::{CommandList, RenderBackendCaps, RenderDevice, RenderQueueClass};

use super::capabilities::wgpu_backend_caps;

#[derive(Clone, Debug)]
pub struct WgpuRenderDevice {
    caps: RenderBackendCaps,
}

impl WgpuRenderDevice {
    pub fn new_headless() -> Self {
        Self {
            caps: wgpu_backend_caps("wgpu", wgpu::Features::empty(), false),
        }
    }

    pub fn new_with_surface_support() -> Self {
        Self {
            caps: wgpu_backend_caps("wgpu", wgpu::Features::empty(), true),
        }
    }
}

impl RenderDevice for WgpuRenderDevice {
    fn caps(&self) -> &RenderBackendCaps {
        &self.caps
    }
}

#[derive(Clone, Debug)]
pub struct WgpuCommandList {
    queue_class: RenderQueueClass,
    label: Option<String>,
}

impl WgpuCommandList {
    pub fn new(queue_class: RenderQueueClass, label: impl Into<String>) -> Self {
        Self {
            queue_class,
            label: Some(label.into()),
        }
    }
}

impl CommandList for WgpuCommandList {
    fn queue_class(&self) -> RenderQueueClass {
        self.queue_class
    }

    fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}
