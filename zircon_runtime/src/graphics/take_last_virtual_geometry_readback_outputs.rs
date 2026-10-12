use crate::core::framework::render::RenderVirtualGeometryReadbackOutputs;

use super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn take_last_virtual_geometry_readback_outputs(
        &mut self,
    ) -> RenderVirtualGeometryReadbackOutputs {
        self.advanced_plugin_outputs
            .take_virtual_geometry_readback_outputs()
    }
}
