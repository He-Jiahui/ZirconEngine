use std::sync::Arc;

pub(in crate::graphics::scene::scene_renderer::core) struct VirtualGeometryIndirectOutputUpdate {
    pub(in crate::graphics::scene::scene_renderer::core) indirect_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_buffer_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_page_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_resident_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_pending_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_missing_segment_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_repeated_draw_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) execution_indirect_offsets: Vec<u64>,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_draw_submission_order:
        Vec<(Option<u32>, u64, u32)>,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_draw_submission_records:
        Vec<(u64, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) mesh_draw_submission_token_records:
        Vec<(u64, u32, u32, u32, usize)>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_args_count: u32,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_draw_refs_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_segments_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_execution_submission_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_execution_args_buffer:
        Option<Arc<wgpu::Buffer>>,
    pub(in crate::graphics::scene::scene_renderer::core) indirect_execution_authority_buffer:
        Option<Arc<wgpu::Buffer>>,
}
