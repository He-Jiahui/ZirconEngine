use crate::graphics::types::GraphicsError;

use super::config::RenderBackendConfig;
use super::render_backend::RenderBackend;
use super::request_device::request_device;

impl RenderBackend {
    pub(crate) fn new_offscreen() -> Result<Self, GraphicsError> {
        let config = RenderBackendConfig::from_environment();
        let instance = wgpu::Instance::new(config.instance_descriptor());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|_| GraphicsError::NoAdapter)?;
        let (device, queue) = request_device(&adapter)?;
        let backend_name = format!("wgpu({})", adapter.get_info().backend.to_str());

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            backend_name,
            config,
        })
    }
}
