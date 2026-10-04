use super::PreparedIconDraw;

/// 同帧图标候选和几何回退集合；图标上传由帧事务管理，不由此值确认。
pub(crate) struct PreparedSceneGizmoPass {
    pub(crate) line_buffer: Option<(wgpu::Buffer, u32)>,
    pub(crate) icon_draws: Vec<PreparedIconDraw>,
}
