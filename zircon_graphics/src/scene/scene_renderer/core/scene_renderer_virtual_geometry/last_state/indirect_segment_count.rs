use super::super::super::scene_renderer::SceneRenderer;

impl SceneRenderer {
    #[cfg(test)]
    pub(crate) fn last_virtual_geometry_indirect_segment_count(&self) -> u32 {
        self.last_virtual_geometry_indirect_segment_count
    }
}
