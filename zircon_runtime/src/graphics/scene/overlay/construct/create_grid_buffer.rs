use wgpu::util::DeviceExt;

use super::super::super::super::primitives::build_grid_vertices;

/// 渲染器构造时一次性上传世界网格；后续帧只切换可见性，不更新网格几何。
pub(in crate::graphics::scene::scene_renderer::overlay::viewport_overlay_renderer) fn create_grid_buffer(
    device: &wgpu::Device,
) -> (wgpu::Buffer, u32) {
    let grid_vertices = build_grid_vertices();
    let grid_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-grid-buffer"),
        contents: bytemuck::cast_slice(&grid_vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    (grid_vertex_buffer, grid_vertices.len() as u32)
}
