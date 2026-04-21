use crate::graphics::scene::scene_renderer::core::SceneRenderer;

impl SceneRenderer {
    pub(crate) fn last_virtual_geometry_execution_segment_count(&self) -> u32 {
        self.last_virtual_geometry_execution_segment_count
    }

    pub(crate) fn last_virtual_geometry_execution_page_count(&self) -> u32 {
        self.last_virtual_geometry_execution_page_count
    }

    pub(crate) fn last_virtual_geometry_execution_resident_segment_count(&self) -> u32 {
        self.last_virtual_geometry_execution_resident_segment_count
    }

    pub(crate) fn last_virtual_geometry_execution_pending_segment_count(&self) -> u32 {
        self.last_virtual_geometry_execution_pending_segment_count
    }

    pub(crate) fn last_virtual_geometry_execution_missing_segment_count(&self) -> u32 {
        self.last_virtual_geometry_execution_missing_segment_count
    }

    pub(crate) fn last_virtual_geometry_execution_repeated_draw_count(&self) -> u32 {
        self.last_virtual_geometry_execution_repeated_draw_count
    }

    #[cfg(test)]
    pub(crate) fn read_last_virtual_geometry_execution_indirect_offsets(&self) -> Vec<u64> {
        self.last_virtual_geometry_execution_indirect_offsets
            .clone()
    }
}
