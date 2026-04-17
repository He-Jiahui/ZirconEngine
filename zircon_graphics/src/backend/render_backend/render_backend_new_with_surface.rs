use std::sync::Arc;

use winit::window::Window;
use zircon_math::UVec2;

use crate::types::GraphicsError;

use super::render_backend::RenderBackend;
use super::request_device::request_device;
use super::surface_state::SurfaceState;

impl RenderBackend {
    pub(crate) fn new_with_surface(
        window: Arc<dyn Window>,
    ) -> Result<(Self, SurfaceState), GraphicsError> {
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone())?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .map_err(|_| GraphicsError::NoAdapter)?;
        let (device, queue) = request_device(&adapter)?;
        let surface_size = window.surface_size();
        let size = UVec2::new(surface_size.width.max(1), surface_size.height.max(1));
        let capabilities = surface.get_capabilities(&adapter);
        let format = capabilities
            .formats
            .iter()
            .copied()
            .find(wgpu::TextureFormat::is_srgb)
            .unwrap_or(capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.x,
            height: size.y,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
            },
            SurfaceState {
                surface,
                config,
                size,
                window,
            },
        ))
    }
}
