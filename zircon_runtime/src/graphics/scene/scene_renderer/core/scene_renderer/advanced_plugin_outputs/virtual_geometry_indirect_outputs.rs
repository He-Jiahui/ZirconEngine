use std::sync::Arc;

use crate::graphics::scene::scene_renderer::core::scene_renderer::VirtualGeometryIndirectOutputUpdate;

#[derive(Default)]
pub(super) struct VirtualGeometryIndirectOutputs {
    indirect_draw_count: u32,
    indirect_buffer_count: u32,
    indirect_segment_count: u32,
    execution_segment_count: u32,
    execution_page_count: u32,
    execution_resident_segment_count: u32,
    execution_pending_segment_count: u32,
    execution_missing_segment_count: u32,
    execution_repeated_draw_count: u32,
    execution_indirect_offsets: Vec<u64>,
    mesh_draw_submission_order: Vec<(Option<u32>, u64, u32)>,
    mesh_draw_submission_records: Vec<(u64, u32, u32, usize)>,
    mesh_draw_submission_token_records: Vec<(u64, u32, u32, u32, usize)>,
    indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_args_count: u32,
    indirect_submission_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_authority_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_draw_refs_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_segments_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_execution_submission_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_execution_args_buffer: Option<Arc<wgpu::Buffer>>,
    indirect_execution_authority_buffer: Option<Arc<wgpu::Buffer>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl VirtualGeometryIndirectOutputs {
    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_draw_count(&self) -> u32 {
        self.indirect_draw_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_buffer_count(&self) -> u32 {
        self.indirect_buffer_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_segment_count(&self) -> u32 {
        self.indirect_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_args_count(&self) -> u32 {
        self.indirect_args_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execution_segment_count(&self) -> u32 {
        self.execution_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execution_page_count(&self) -> u32 {
        self.execution_page_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execution_resident_segment_count(
        &self,
    ) -> u32 {
        self.execution_resident_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execution_pending_segment_count(
        &self,
    ) -> u32 {
        self.execution_pending_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execution_missing_segment_count(
        &self,
    ) -> u32 {
        self.execution_missing_segment_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn execution_repeated_draw_count(
        &self,
    ) -> u32 {
        self.execution_repeated_draw_count
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn mesh_draw_submission_order(
        &self,
    ) -> &Vec<(Option<u32>, u64, u32)> {
        &self.mesh_draw_submission_order
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn mesh_draw_submission_records(
        &self,
    ) -> &Vec<(u64, u32, u32, usize)> {
        &self.mesh_draw_submission_records
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn mesh_draw_submission_token_records(
        &self,
    ) -> &Vec<(u64, u32, u32, u32, usize)> {
        &self.mesh_draw_submission_token_records
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_args_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_args_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_submission_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_submission_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_authority_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_authority_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_draw_refs_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_draw_refs_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_segments_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_segments_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_execution_submission_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_execution_submission_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_execution_args_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_execution_args_buffer
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn indirect_execution_authority_buffer(
        &self,
    ) -> &Option<Arc<wgpu::Buffer>> {
        &self.indirect_execution_authority_buffer
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_submission_buffer(
        &mut self,
    ) {
        self.indirect_submission_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_authority_buffer(
        &mut self,
    ) {
        self.indirect_authority_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_mesh_draw_submission_token_records(
        &mut self,
    ) {
        self.mesh_draw_submission_token_records.clear();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_mesh_draw_submission_records(
        &mut self,
    ) {
        self.mesh_draw_submission_records.clear();
        self.mesh_draw_submission_order.clear();
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_args_buffer(&mut self) {
        self.indirect_args_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_draw_refs_buffer(
        &mut self,
    ) {
        self.indirect_draw_refs_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_segments_buffer(
        &mut self,
    ) {
        self.indirect_segments_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_execution_submission_buffer(
        &mut self,
    ) {
        self.indirect_execution_submission_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_execution_args_buffer(
        &mut self,
    ) {
        self.indirect_execution_args_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn clear_indirect_execution_authority_buffer(
        &mut self,
    ) {
        self.indirect_execution_authority_buffer = None;
    }

    #[cfg(test)]
    pub(in crate::graphics::scene::scene_renderer::core) fn execution_indirect_offsets(
        &self,
    ) -> Vec<u64> {
        self.execution_indirect_offsets.clone()
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn store(
        &mut self,
        update: VirtualGeometryIndirectOutputUpdate,
    ) {
        self.indirect_draw_count = update.indirect_draw_count;
        self.indirect_buffer_count = update.indirect_buffer_count;
        self.indirect_segment_count = update.indirect_segment_count;
        self.execution_segment_count = update.execution_segment_count;
        self.execution_page_count = update.execution_page_count;
        self.execution_resident_segment_count = update.execution_resident_segment_count;
        self.execution_pending_segment_count = update.execution_pending_segment_count;
        self.execution_missing_segment_count = update.execution_missing_segment_count;
        self.execution_repeated_draw_count = update.execution_repeated_draw_count;
        self.execution_indirect_offsets = update.execution_indirect_offsets;
        self.mesh_draw_submission_order = update.mesh_draw_submission_order;
        self.mesh_draw_submission_records = update.mesh_draw_submission_records;
        self.mesh_draw_submission_token_records = update.mesh_draw_submission_token_records;
        self.indirect_args_buffer = update.indirect_args_buffer;
        self.indirect_args_count = update.indirect_args_count;
        self.indirect_submission_buffer = update.indirect_submission_buffer;
        self.indirect_authority_buffer = update.indirect_authority_buffer;
        self.indirect_draw_refs_buffer = update.indirect_draw_refs_buffer;
        self.indirect_segments_buffer = update.indirect_segments_buffer;
        self.indirect_execution_submission_buffer = update.indirect_execution_submission_buffer;
        self.indirect_execution_args_buffer = update.indirect_execution_args_buffer;
        self.indirect_execution_authority_buffer = update.indirect_execution_authority_buffer;
    }
}
