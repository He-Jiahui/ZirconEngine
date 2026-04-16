use crate::scene::scene_renderer::VirtualGeometryGpuReadback;

use super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn take_last_virtual_geometry_gpu_readback(
        &mut self,
    ) -> Option<VirtualGeometryGpuReadback> {
        self.last_virtual_geometry_gpu_readback.take()
    }
}
