use super::config::RenderBackendConfig;
use crate::rhi::RenderBackendCaps;
use crate::rhi_wgpu::wgpu_backend_caps;

pub(crate) struct RenderBackend {
    #[allow(dead_code)]
    pub(crate) instance: wgpu::Instance,
    #[allow(dead_code)]
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) backend_name: String,
    #[allow(dead_code)]
    pub(crate) config: RenderBackendConfig,
}

impl RenderBackend {
    pub(crate) fn caps(&self) -> RenderBackendCaps {
        wgpu_backend_caps(
            self.backend_name.clone(),
            self.adapter.features(),
            self.device.limits(),
            false,
        )
    }
}
