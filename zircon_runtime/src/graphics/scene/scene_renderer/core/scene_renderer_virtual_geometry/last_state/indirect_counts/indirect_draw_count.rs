use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_indirect_draw_count(&self) -> u32 {
        self.advanced_plugin_outputs
            .virtual_geometry_indirect_draw_count()
    }
}
