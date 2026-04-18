#![cfg(test)]

use crate::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn last_virtual_geometry_indirect_draw_count(&self) -> u32 {
        self.last_virtual_geometry_indirect_draw_count
    }
}
