use std::sync::Arc;

use super::scene_renderer_advanced_plugin_outputs::SceneRendererAdvancedPluginOutputs;

#[cfg_attr(not(test), allow(dead_code))]
impl SceneRendererAdvancedPluginOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_draw_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect().indirect_draw_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_buffer_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect().indirect_buffer_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect().indirect_segment_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_args_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect().indirect_args_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect().execution_segment_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_page_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect().execution_page_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_resident_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect()
            .execution_resident_segment_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_pending_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect()
            .execution_pending_segment_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_missing_segment_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect()
            .execution_missing_segment_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_repeated_draw_count(
        &self,
    ) -> u32 {
        self.virtual_geometry_indirect()
            .execution_repeated_draw_count()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_mesh_draw_submission_order(
        &self,
    ) -> &Vec<(Option<u32>, u64, u32)> {
        self.virtual_geometry_indirect()
            .mesh_draw_submission_order()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_mesh_draw_submission_records(
        &self,
    ) -> &Vec<(u64, u32, u32, usize)> {
        self.virtual_geometry_indirect()
            .mesh_draw_submission_records()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_mesh_draw_submission_token_records(
        &self,
    ) -> &Vec<(u64, u32, u32, u32, usize)> {
        self.virtual_geometry_indirect()
            .mesh_draw_submission_token_records()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_args_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect().indirect_args_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_submission_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect()
            .indirect_submission_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_authority_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect().indirect_authority_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_draw_refs_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect().indirect_draw_refs_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_segments_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect().indirect_segments_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_execution_submission_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect()
            .indirect_execution_submission_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_execution_args_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect()
            .indirect_execution_args_buffer()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_indirect_execution_authority_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        self.virtual_geometry_indirect()
            .indirect_execution_authority_buffer()
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_submission_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_submission_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_authority_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_authority_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_mesh_draw_submission_token_records(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_mesh_draw_submission_token_records();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_mesh_draw_submission_records(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_mesh_draw_submission_records();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_args_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_args_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_draw_refs_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_draw_refs_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_segments_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_segments_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_execution_submission_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_execution_submission_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_execution_args_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_execution_args_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_virtual_geometry_indirect_execution_authority_buffer(
        &mut self,
    ) {
        self.virtual_geometry_indirect_mut()
            .clear_indirect_execution_authority_buffer();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn virtual_geometry_execution_indirect_offsets(
        &self,
    ) -> Vec<u64> {
        self.virtual_geometry_indirect()
            .execution_indirect_offsets()
    }
}
