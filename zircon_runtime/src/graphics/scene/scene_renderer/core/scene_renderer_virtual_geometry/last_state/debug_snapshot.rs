use crate::core::framework::render::RenderVirtualGeometryDebugSnapshot;
use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_debug_snapshot(
        &self,
    ) -> Option<RenderVirtualGeometryDebugSnapshot> {
        self.advanced_plugin_outputs
            .virtual_geometry_debug_snapshot
            .clone()
    }
}
