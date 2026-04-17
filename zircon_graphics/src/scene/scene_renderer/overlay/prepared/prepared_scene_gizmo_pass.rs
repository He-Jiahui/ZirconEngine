use super::PreparedIconDraw;

pub(crate) struct PreparedSceneGizmoPass {
    pub(crate) line_buffer: Option<(wgpu::Buffer, u32)>,
    pub(crate) icon_draws: Vec<PreparedIconDraw>,
}
