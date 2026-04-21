#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryVisBuffer64Entry;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_gpu_readback_visbuffer64(
        &self,
    ) -> Option<(u64, Vec<RenderVirtualGeometryVisBuffer64Entry>)> {
        self.last_virtual_geometry_gpu_readback
            .as_ref()
            .map(|readback| {
                (
                    readback.visbuffer64_clear_value,
                    readback.visbuffer64_entries.clone(),
                )
            })
    }
}
