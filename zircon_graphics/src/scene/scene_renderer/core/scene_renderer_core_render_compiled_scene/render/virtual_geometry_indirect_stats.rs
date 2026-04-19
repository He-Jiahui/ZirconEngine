use std::collections::HashSet;
use std::sync::Arc;

use crate::scene::scene_renderer::mesh::MeshDraw;

pub(super) struct VirtualGeometryIndirectStats {
    pub(super) draw_count: u32,
    pub(super) buffer_count: u32,
    pub(super) segment_count: u32,
    pub(super) draw_submission_order: Vec<(u64, u32)>,
    pub(super) draw_submission_records: Vec<(u64, u32, u64, usize)>,
    pub(super) draw_submission_token_records: Vec<(u64, u32, u32, u32, usize)>,
    pub(super) args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) args_count: u32,
    pub(super) submission_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) segment_buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn virtual_geometry_indirect_stats(
    mesh_draws: &[MeshDraw],
    args_count: u32,
    segment_count: u32,
    submission_buffer: Option<Arc<wgpu::Buffer>>,
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
    let draw_submission_order = mesh_draws
        .iter()
        .filter_map(|draw| draw.virtual_geometry_submission_key)
        .collect::<Vec<_>>();
    let draw_submission_records = mesh_draws
        .iter()
        .enumerate()
        .filter_map(|(draw_index, draw)| {
            draw.virtual_geometry_submission_key.map(|(entity, page_id)| {
                (entity, page_id, draw.indirect_args_offset, draw_index)
            })
        })
        .collect::<Vec<_>>();
    let draw_submission_token_records = mesh_draws
        .iter()
        .enumerate()
        .filter_map(|(draw_index, draw)| {
            draw.virtual_geometry_submission_detail.map(|detail| {
                (
                    detail.entity,
                    detail.page_id,
                    detail.submission_index,
                    detail.draw_ref_rank,
                    draw_index,
                )
            })
        })
        .collect::<Vec<_>>();

    VirtualGeometryIndirectStats {
        draw_count,
        buffer_count,
        segment_count,
        draw_submission_order,
        draw_submission_records,
        draw_submission_token_records,
        args_buffer,
        args_count,
        submission_buffer,
        draw_ref_buffer,
        segment_buffer,
    }
}
