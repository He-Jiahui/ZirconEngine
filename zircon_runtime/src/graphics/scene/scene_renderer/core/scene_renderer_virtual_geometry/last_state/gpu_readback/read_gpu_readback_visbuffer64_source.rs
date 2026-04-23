#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryVisBuffer64Source;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_gpu_readback_visbuffer64_source(
        &self,
    ) -> Option<RenderVirtualGeometryVisBuffer64Source> {
        self.last_virtual_geometry_gpu_readback
            .as_ref()
            .map(|readback| readback.visbuffer64_source)
    }
}
