use wgpu::util::DeviceExt;

use crate::scene::scene_renderer::primitives::LineVertex;

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
