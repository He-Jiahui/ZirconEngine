use std::sync::Arc;

use winit::window::Window;
use zircon_math::UVec2;

pub(crate) struct SurfaceState {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) size: UVec2,
    #[allow(dead_code)]
    pub(crate) window: Arc<dyn Window>,
}
