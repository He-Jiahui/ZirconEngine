use std::sync::Arc;

use super::RenderVirtualGeometryExecutionSegment;

#[derive(Clone)]
pub struct RenderVirtualGeometryExecutionDraw {
    pub indirect_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub indirect_args_offset: u64,
    pub uses_indirect_draw: bool,
    pub execution_selection_key: Option<(u64, u32)>,
    pub execution_segment: RenderVirtualGeometryExecutionSegment,
    pub submission_order_record: Option<(Option<u32>, u64, u32)>,
    pub draw_submission_record: Option<(u64, u32, u32, usize)>,
    pub draw_submission_token_record: Option<(u64, u32, u32, u32, usize)>,
    pub execution_draw_ref_index: u32,
}
