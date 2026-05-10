use super::config::RenderBackendConfig;

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
