use super::PreparedSceneGizmoPass;

/// 将准备阶段的辅助几何交给记录阶段；空缓冲表示该层本帧无可绘制数据。
pub(crate) struct PreparedOverlayBuffers {
    pub(crate) selection_buffer: Option<(wgpu::Buffer, u32)>,
    pub(crate) wireframe_buffer: Option<(wgpu::Buffer, u32)>,
    pub(crate) scene_gizmo: PreparedSceneGizmoPass,
    pub(crate) handle_buffer: Option<(wgpu::Buffer, u32)>,
}
