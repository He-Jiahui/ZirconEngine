use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::primitives::IconVertex;

/// 将已判定可用贴图的 gizmo 图标顶点固化为本帧 GPU 缓冲；空图标不生成绘制命令。
pub(crate) fn build_icon_buffer(
    device: &wgpu::Device,
    label: &str,
    vertices: &[IconVertex],
) -> Option<(wgpu::Buffer, u32)> {
    if vertices.is_empty() {
        return None;
    }
    let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    Some((buffer, vertices.len() as u32))
}
