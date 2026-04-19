use crate::graphics::types::GraphicsError;

use super::render_backend::RenderBackend;
use super::request_device::request_device;

impl RenderBackend {
    pub(crate) fn new_offscreen() -> Result<Self, GraphicsError> {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .map_err(|_| GraphicsError::NoAdapter)?;
        let (device, queue) = request_device(&adapter)?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
