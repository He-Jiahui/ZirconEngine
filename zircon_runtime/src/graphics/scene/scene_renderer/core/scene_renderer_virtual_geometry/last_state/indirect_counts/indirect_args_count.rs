use crate::graphics::scene::scene_renderer::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_indirect_args_count(&self) -> u32 {
        self.last_virtual_geometry_indirect_args_count
    }
}
