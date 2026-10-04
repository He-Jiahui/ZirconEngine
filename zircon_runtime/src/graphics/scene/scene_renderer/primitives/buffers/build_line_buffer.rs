use wgpu::util::DeviceExt;

use crate::graphics::scene::scene_renderer::primitives::LineVertex;

/// 将叠加层线段列表固化为本帧 GPU 缓冲；None 表示对应 pass 无可绘制内容。
pub(crate) fn build_line_buffer(
    device: &wgpu::Device,
    label: &str,
    vertices: &[LineVertex],
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
