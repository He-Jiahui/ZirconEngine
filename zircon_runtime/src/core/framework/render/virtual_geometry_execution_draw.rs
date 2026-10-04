use super::RenderVirtualGeometryExecutionSegment;

/// 虚拟几何队列把可执行绘制投影到后端提交时的中立记录。
/// 间接参数可用性和执行段必须随同一帧决策传递，诊断计数才对应实际 draw。
#[derive(Clone)]
pub struct RenderVirtualGeometryExecutionDraw {
    pub indirect_args_buffer_available: bool,
    pub indirect_args_offset: u64,
    pub uses_indirect_draw: bool,
    pub execution_selection_key: Option<(u64, u32)>,
    pub execution_segment: RenderVirtualGeometryExecutionSegment,
    pub submission_order_record: Option<(Option<u32>, u64, u32)>,
    pub draw_submission_record: Option<(u64, u32, u32, usize)>,
    pub draw_submission_token_record: Option<(u64, u32, u32, u32, usize)>,
    pub execution_draw_ref_index: u32,
}
