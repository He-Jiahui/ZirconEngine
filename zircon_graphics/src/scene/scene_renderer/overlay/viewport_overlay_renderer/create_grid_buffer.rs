use wgpu::util::DeviceExt;

use super::super::super::primitives::build_grid_vertices;

pub(in crate::scene::scene_renderer::overlay::viewport_overlay_renderer) fn create_grid_buffer(
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
