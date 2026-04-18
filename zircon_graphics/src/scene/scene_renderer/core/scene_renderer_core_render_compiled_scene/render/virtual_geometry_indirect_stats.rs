use std::collections::HashSet;
use std::sync::Arc;

use crate::scene::scene_renderer::mesh::MeshDraw;

pub(super) struct VirtualGeometryIndirectStats {
    pub(super) draw_count: u32,
    pub(super) buffer_count: u32,
    pub(super) segment_count: u32,
    pub(super) args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) args_count: u32,
    pub(super) draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) segment_buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn virtual_geometry_indirect_stats(
    mesh_draws: &[MeshDraw],
    args_count: u32,
    segment_count: u32,
    draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    segment_buffer: Option<Arc<wgpu::Buffer>>,
) -> VirtualGeometryIndirectStats {
    let draw_count = mesh_draws
        .iter()
        .filter(|draw| draw.uses_indirect_draw())
        .count() as u32;
    let buffer_count = mesh_draws
        .iter()
        .filter_map(|draw| {
            draw.indirect_args_buffer
                .as_ref()
                .map(|buffer| Arc::as_ptr(buffer) as usize)
        })
        .collect::<HashSet<_>>()
        .len() as u32;
    let args_buffer = mesh_draws
        .iter()
        .find_map(|draw| draw.indirect_args_buffer.as_ref().map(Arc::clone));

    VirtualGeometryIndirectStats {
        draw_count,
        buffer_count,
        segment_count,
        args_buffer,
        args_count,
        draw_ref_buffer,
        segment_buffer,
    }
}
