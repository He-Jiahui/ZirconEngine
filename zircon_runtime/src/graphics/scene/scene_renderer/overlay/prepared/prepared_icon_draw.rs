use std::sync::Arc;

pub(crate) struct PreparedIconDraw {
    pub(crate) bind_group: Arc<wgpu::BindGroup>,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) vertex_count: u32,
}
