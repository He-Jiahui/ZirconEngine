use std::sync::Arc;

/// 单帧面向相机图标的绘制快照；绑定仍由缓存共享，几何只适用于准备时的相机。
pub(crate) struct PreparedIconDraw {
    pub(crate) bind_group: Arc<wgpu::BindGroup>,
    pub(crate) vertex_buffer: wgpu::Buffer,
    pub(crate) vertex_count: u32,
}
