use std::collections::HashSet;
use std::sync::Arc;

use crate::graphics::scene::scene_renderer::mesh::MeshDraw;
use wgpu::util::DeviceExt;

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
    pub(super) execution_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) execution_records_buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn virtual_geometry_indirect_stats(
    device: &wgpu::Device,
    execution_draws: &[&MeshDraw],
    args_count: u32,
    segment_count: u32,
    submission_buffer: Option<Arc<wgpu::Buffer>>,
    draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    segment_buffer: Option<Arc<wgpu::Buffer>>,
) -> VirtualGeometryIndirectStats {
    const INDIRECT_ARGS_STRIDE_BYTES: u64 = (std::mem::size_of::<u32>() as u64) * 5;

    let indirect_execution_draws = execution_draws
        .iter()
        .copied()
        .filter(|draw| draw.uses_indirect_draw())
        .collect::<Vec<_>>();
    let draw_count = indirect_execution_draws.len() as u32;
    let buffer_count = execution_draws
        .iter()
        .filter_map(|draw| {
            draw.indirect_args_buffer
                .as_ref()
                .map(|buffer| Arc::as_ptr(buffer) as usize)
        })
        .collect::<HashSet<_>>()
        .len() as u32;
    let args_buffer = execution_draws
        .iter()
        .find_map(|draw| draw.indirect_args_buffer.as_ref().map(Arc::clone));
    let draw_submission_order = execution_draws
        .iter()
        .filter_map(|draw| draw.virtual_geometry_submission_key)
        .collect::<Vec<_>>();
    let draw_submission_records = execution_draws
        .iter()
        .enumerate()
        .filter_map(|(draw_index, draw)| {
            draw.virtual_geometry_submission_key
                .map(|(entity, page_id)| (entity, page_id, draw.indirect_args_offset, draw_index))
        })
        .collect::<Vec<_>>();
    let draw_submission_token_records = execution_draws
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
    let execution_draw_ref_indices = indirect_execution_draws
        .iter()
        .map(|draw| (draw.indirect_args_offset / INDIRECT_ARGS_STRIDE_BYTES) as u32)
        .collect::<Vec<_>>();
    let execution_buffer = (!execution_draw_ref_indices.is_empty()).then(|| {
        Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-vg-indirect-execution-draw-ref-indices"),
                contents: bytemuck::cast_slice(&execution_draw_ref_indices),
                usage: wgpu::BufferUsages::COPY_SRC,
            }),
        )
    });
    let execution_records = indirect_execution_draws
        .iter()
        .zip(execution_draw_ref_indices.iter().copied())
        .map(|(draw, draw_ref_index)| {
            let (fallback_entity, fallback_page_id) =
                draw.virtual_geometry_submission_key.unwrap_or((0, 0));
            let entity = draw
                .virtual_geometry_submission_detail
                .map(|detail| detail.entity)
                .unwrap_or(fallback_entity);
            let page_id = draw
                .virtual_geometry_submission_detail
                .map(|detail| detail.page_id)
                .unwrap_or(fallback_page_id);
            let submission_index = draw
                .virtual_geometry_submission_detail
                .map(|detail| detail.submission_index)
                .unwrap_or(0);
            let draw_ref_rank = draw
                .virtual_geometry_submission_detail
                .map(|detail| detail.draw_ref_rank)
                .unwrap_or(0);
            [
                draw_ref_index,
                page_id,
                submission_index,
                draw_ref_rank,
                entity as u32,
                (entity >> 32) as u32,
            ]
        })
        .collect::<Vec<_>>();
    let execution_records_buffer = (!execution_records.is_empty()).then(|| {
        Arc::new(
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-vg-indirect-execution-records"),
                contents: bytemuck::cast_slice(&execution_records),
                usage: wgpu::BufferUsages::COPY_SRC,
            }),
        )
    });

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
        execution_buffer,
        execution_records_buffer,
    }
}
