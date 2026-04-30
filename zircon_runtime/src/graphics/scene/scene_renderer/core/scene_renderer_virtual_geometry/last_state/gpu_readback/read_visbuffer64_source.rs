#[cfg(test)]
use crate::core::framework::render::RenderVirtualGeometryVisBuffer64Source;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_visbuffer64_source(
        &self,
    ) -> RenderVirtualGeometryVisBuffer64Source {
        self.advanced_plugin_outputs
            .virtual_geometry_visbuffer64_source()
    }
}
