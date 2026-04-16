use zircon_math::UVec2;

use super::surface_state::SurfaceState;

impl SurfaceState {
    pub(crate) fn resize(&mut self, device: &wgpu::Device, size: UVec2) {
        let size = UVec2::new(size.x.max(1), size.y.max(1));
        self.size = size;
        self.config.width = size.x;
        self.config.height = size.y;
        self.surface.configure(device, &self.config);
    }
}
