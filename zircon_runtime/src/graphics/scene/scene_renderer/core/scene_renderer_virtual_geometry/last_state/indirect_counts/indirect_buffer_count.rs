use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_indirect_buffer_count(&self) -> u32 {
        self.last_virtual_geometry_indirect_buffer_count
    }
}
