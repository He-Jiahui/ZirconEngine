use super::PreparedSceneGizmoPass;

pub(crate) struct PreparedOverlayBuffers {
    pub(crate) selection_buffer: Option<(wgpu::Buffer, u32)>,
    pub(crate) wireframe_buffer: Option<(wgpu::Buffer, u32)>,
    pub(crate) scene_gizmo: PreparedSceneGizmoPass,
    pub(crate) handle_buffer: Option<(wgpu::Buffer, u32)>,
}
