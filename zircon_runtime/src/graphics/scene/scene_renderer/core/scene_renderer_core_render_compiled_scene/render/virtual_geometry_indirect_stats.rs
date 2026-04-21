use std::collections::HashSet;
use std::sync::Arc;

use crate::core::framework::render::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
};
use crate::graphics::scene::scene_renderer::mesh::MeshDraw;
use crate::graphics::types::VirtualGeometryClusterSelection;

use super::virtual_geometry_executed_cluster_selection_pass::execute_virtual_geometry_executed_cluster_selection_pass;
use super::virtual_geometry_hardware_rasterization_pass::{
    execute_virtual_geometry_hardware_rasterization_pass,
    VirtualGeometryHardwareRasterizationPassOutput,
};
use super::virtual_geometry_visbuffer64_pass::{
    execute_virtual_geometry_visbuffer64_pass, VirtualGeometryVisBuffer64PassOutput,
};

pub(super) struct VirtualGeometryIndirectStats {
    pub(super) draw_count: u32,
    pub(super) buffer_count: u32,
    pub(super) segment_count: u32,
    pub(super) execution_segment_count: u32,
    pub(super) execution_page_count: u32,
    pub(super) execution_resident_segment_count: u32,
    pub(super) execution_pending_segment_count: u32,
    pub(super) execution_missing_segment_count: u32,
    pub(super) execution_repeated_draw_count: u32,
    pub(super) execution_indirect_offsets: Vec<u64>,
    pub(super) execution_segments: Vec<RenderVirtualGeometryExecutionSegment>,
    pub(super) executed_selected_cluster_count: u32,
    pub(super) executed_selected_cluster_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) hardware_rasterization_pass: VirtualGeometryHardwareRasterizationPassOutput,
    pub(super) visbuffer64_pass: VirtualGeometryVisBuffer64PassOutput,
    pub(super) draw_submission_order: Vec<(Option<u32>, u64, u32)>,
    pub(super) draw_submission_records: Vec<(u64, u32, u32, usize)>,
    pub(super) draw_submission_token_records: Vec<(u64, u32, u32, u32, usize)>,
    pub(super) args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) args_count: u32,
    pub(super) submission_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) authority_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) segment_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) execution_submission_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) execution_args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) execution_authority_buffer: Option<Arc<wgpu::Buffer>>,
}

pub(super) fn virtual_geometry_indirect_stats(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    visbuffer64_pass_enabled: bool,
    cluster_selections: Option<&[VirtualGeometryClusterSelection]>,
    execution_draws: &[&MeshDraw],
    args_buffer: Option<Arc<wgpu::Buffer>>,
    args_count: u32,
    segment_count: u32,
    submission_buffer: Option<Arc<wgpu::Buffer>>,
    authority_buffer: Option<Arc<wgpu::Buffer>>,
    draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    segment_buffer: Option<Arc<wgpu::Buffer>>,
) -> VirtualGeometryIndirectStats {
    let indirect_execution_draws = execution_draws
        .iter()
        .copied()
        .filter(|draw| draw.uses_indirect_draw())
        .collect::<Vec<_>>();
    let execution_summary = execution_segment_summary(&indirect_execution_draws);
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
    let execution_args_buffer = indirect_execution_draws
        .iter()
        .find_map(|draw| draw.indirect_args_buffer.as_ref().map(Arc::clone));
    let draw_submission_order = execution_draws
        .iter()
        .filter_map(|draw| {
            let (entity, page_id) = draw.virtual_geometry_submission_key?;
            Some((
                draw.virtual_geometry_submission_detail
                    .and_then(|detail| detail.instance_index),
                entity,
                page_id,
            ))
        })
        .collect::<Vec<_>>();
    let execution_indirect_offsets = indirect_execution_draws
        .iter()
        .map(|draw| draw.indirect_args_offset)
        .collect::<Vec<_>>();
    let execution_segments = collect_execution_segments(&indirect_execution_draws);
    let executed_cluster_selection_pass = execute_virtual_geometry_executed_cluster_selection_pass(
        device,
        cluster_selections,
        &indirect_execution_draws,
    );
    let hardware_rasterization_pass = execute_virtual_geometry_hardware_rasterization_pass(
        device,
        visbuffer64_pass_enabled,
        &executed_cluster_selection_pass,
    );
    let visbuffer64_pass = execute_virtual_geometry_visbuffer64_pass(
        device,
        visbuffer64_pass_enabled,
        &executed_cluster_selection_pass,
    );
    let draw_submission_records = execution_draws
        .iter()
        .enumerate()
        .filter_map(|(draw_index, draw)| {
            let (entity, page_id) = draw.virtual_geometry_submission_key?;
            Some((
                entity,
                page_id,
                execution_draw_ref_index(
                    draw.virtual_geometry_submission_detail,
                    draw.indirect_args_offset,
                ),
                draw_index,
            ))
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
    let execution_submission_buffer = build_execution_submission_buffer(
        device,
        encoder,
        &indirect_execution_draws,
        submission_buffer.as_ref(),
    );
    let execution_authority_buffer = build_execution_authority_buffer(
        device,
        encoder,
        &indirect_execution_draws,
        authority_buffer.as_ref(),
    );
    VirtualGeometryIndirectStats {
        draw_count,
        buffer_count,
        segment_count,
        execution_segment_count: execution_summary.segment_count,
        execution_page_count: execution_summary.page_count,
        execution_resident_segment_count: execution_summary.resident_segment_count,
        execution_pending_segment_count: execution_summary.pending_segment_count,
        execution_missing_segment_count: execution_summary.missing_segment_count,
        execution_repeated_draw_count: execution_summary.repeated_draw_count,
        execution_indirect_offsets,
        execution_segments,
        executed_selected_cluster_count: executed_cluster_selection_pass.selected_cluster_count,
        executed_selected_cluster_buffer: executed_cluster_selection_pass
            .selected_cluster_buffer
            .clone(),
        hardware_rasterization_pass,
        visbuffer64_pass,
        draw_submission_order,
        draw_submission_records,
        draw_submission_token_records,
        args_buffer,
        args_count,
        submission_buffer,
        authority_buffer,
        draw_ref_buffer,
        segment_buffer,
        execution_submission_buffer,
        execution_args_buffer,
        execution_authority_buffer,
    }
}

fn build_execution_submission_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    indirect_execution_draws: &[&MeshDraw],
    shared_submission_buffer: Option<&Arc<wgpu::Buffer>>,
) -> Option<Arc<wgpu::Buffer>> {
    let record_stride_bytes = std::mem::size_of::<u32>() as u64;

    let shared_submission_buffer = shared_submission_buffer?;
    if indirect_execution_draws.is_empty() {
        return None;
    }

    let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-execution-submission-tokens"),
        size: (indirect_execution_draws.len() as u64) * record_stride_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));

    for (execution_index, draw) in indirect_execution_draws.iter().enumerate() {
        let draw_ref_index = execution_draw_ref_index(
            draw.virtual_geometry_submission_detail,
            draw.indirect_args_offset,
        ) as u64;
        encoder.copy_buffer_to_buffer(
            shared_submission_buffer,
            draw_ref_index * record_stride_bytes,
            &buffer,
            (execution_index as u64) * record_stride_bytes,
            record_stride_bytes,
        );
    }

    Some(buffer)
}

fn build_execution_authority_buffer(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    indirect_execution_draws: &[&MeshDraw],
    shared_authority_buffer: Option<&Arc<wgpu::Buffer>>,
) -> Option<Arc<wgpu::Buffer>> {
    const AUTHORITY_RECORD_WORD_COUNT: u64 = 15;
    let record_stride_bytes = (std::mem::size_of::<u32>() as u64) * AUTHORITY_RECORD_WORD_COUNT;

    let shared_authority_buffer = shared_authority_buffer?;
    if indirect_execution_draws.is_empty() {
        return None;
    }

    let buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-vg-indirect-execution-authority-records"),
        size: (indirect_execution_draws.len() as u64) * record_stride_bytes,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    }));

    for (execution_index, draw) in indirect_execution_draws.iter().enumerate() {
        let draw_ref_index = execution_draw_ref_index(
            draw.virtual_geometry_submission_detail,
            draw.indirect_args_offset,
        ) as u64;
        encoder.copy_buffer_to_buffer(
            shared_authority_buffer,
            draw_ref_index * record_stride_bytes,
            &buffer,
            (execution_index as u64) * record_stride_bytes,
            record_stride_bytes,
        );
    }

    Some(buffer)
}

fn execution_draw_ref_index(
    submission_detail: Option<
        crate::graphics::scene::scene_renderer::mesh::VirtualGeometrySubmissionDetail,
    >,
    indirect_args_offset: u64,
) -> u32 {
    const INDIRECT_ARGS_STRIDE_BYTES: u64 = (std::mem::size_of::<u32>() as u64) * 5;

    submission_detail
        .map(|detail| detail.draw_ref_index)
        .unwrap_or_else(|| (indirect_args_offset / INDIRECT_ARGS_STRIDE_BYTES) as u32)
}

fn collect_execution_segments(
    indirect_execution_draws: &[&MeshDraw],
) -> Vec<RenderVirtualGeometryExecutionSegment> {
    indirect_execution_draws
        .iter()
        .enumerate()
        .map(|(draw_index, draw)| {
            let fallback_key = draw.virtual_geometry_submission_key.unwrap_or((0, 0));
            let detail = draw.virtual_geometry_submission_detail;
            let page_id = detail
                .map(|detail| detail.page_id)
                .unwrap_or(fallback_key.1);
            RenderVirtualGeometryExecutionSegment {
                original_index: draw_index as u32,
                instance_index: detail.and_then(|detail| detail.instance_index),
                entity: detail.map(|detail| detail.entity).unwrap_or(fallback_key.0),
                page_id,
                draw_ref_index: execution_draw_ref_index(detail, draw.indirect_args_offset),
                submission_index: detail.map(|detail| detail.submission_index),
                draw_ref_rank: detail.map(|detail| detail.draw_ref_rank),
                cluster_start_ordinal: detail
                    .map(|detail| detail.cluster_start_ordinal)
                    .unwrap_or_default(),
                cluster_span_count: detail.map(|detail| detail.cluster_span_count).unwrap_or(1),
                cluster_total_count: detail.map(|detail| detail.cluster_total_count).unwrap_or(1),
                submission_slot: detail.and_then(|detail| detail.submission_slot),
                state: detail
                    .map(|detail| map_execution_state(detail.state))
                    .unwrap_or(RenderVirtualGeometryExecutionState::Resident),
                lineage_depth: detail
                    .map(|detail| detail.lineage_depth)
                    .unwrap_or_default(),
                lod_level: detail.map(|detail| detail.lod_level).unwrap_or_default(),
                frontier_rank: detail
                    .map(|detail| detail.frontier_rank)
                    .unwrap_or_default(),
            }
        })
        .collect()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct ExecutionSegmentKey {
    instance_index: u32,
    entity: u64,
    page_id: u32,
    cluster_start_ordinal: u32,
    cluster_span_count: u32,
    cluster_total_count: u32,
    submission_slot: u32,
    state: u32,
    lineage_depth: u32,
    lod_level: u8,
    frontier_rank: u32,
}

#[derive(Default)]
struct ExecutionSegmentSummary {
    segment_count: u32,
    page_count: u32,
    resident_segment_count: u32,
    pending_segment_count: u32,
    missing_segment_count: u32,
    repeated_draw_count: u32,
}

fn execution_segment_summary(indirect_execution_draws: &[&MeshDraw]) -> ExecutionSegmentSummary {
    let mut segments = HashSet::new();
    let mut pages = HashSet::new();
    let mut resident_segment_count = 0;
    let mut pending_segment_count = 0;
    let mut missing_segment_count = 0;

    for draw in indirect_execution_draws {
        let fallback_key = draw.virtual_geometry_submission_key.unwrap_or((0, 0));
        let detail = draw.virtual_geometry_submission_detail;
        let page_id = detail
            .map(|detail| detail.page_id)
            .unwrap_or(fallback_key.1);
        let key = ExecutionSegmentKey {
            instance_index: detail
                .and_then(|detail| detail.instance_index)
                .unwrap_or(u32::MAX),
            entity: detail.map(|detail| detail.entity).unwrap_or(fallback_key.0),
            page_id,
            cluster_start_ordinal: detail
                .map(|detail| detail.cluster_start_ordinal)
                .unwrap_or_default(),
            cluster_span_count: detail.map(|detail| detail.cluster_span_count).unwrap_or(1),
            cluster_total_count: detail.map(|detail| detail.cluster_total_count).unwrap_or(1),
            submission_slot: detail
                .and_then(|detail| detail.submission_slot)
                .unwrap_or(u32::MAX),
            state: detail
                .map(|detail| encode_cluster_state(detail.state))
                .unwrap_or(0),
            lineage_depth: detail
                .map(|detail| detail.lineage_depth)
                .unwrap_or_default(),
            lod_level: detail.map(|detail| detail.lod_level).unwrap_or_default(),
            frontier_rank: detail
                .map(|detail| detail.frontier_rank)
                .unwrap_or_default(),
        };
        if segments.insert(key) {
            pages.insert(page_id);
            match detail.map(|detail| detail.state) {
                Some(crate::graphics::types::VirtualGeometryPrepareClusterState::Resident) => {
                    resident_segment_count += 1
                }
                Some(crate::graphics::types::VirtualGeometryPrepareClusterState::PendingUpload) => {
                    pending_segment_count += 1
                }
                Some(crate::graphics::types::VirtualGeometryPrepareClusterState::Missing) => {
                    missing_segment_count += 1
                }
                None => resident_segment_count += 1,
            }
        }
    }

    let segment_count = segments.len() as u32;
    ExecutionSegmentSummary {
        segment_count,
        page_count: pages.len() as u32,
        resident_segment_count,
        pending_segment_count,
        missing_segment_count,
        repeated_draw_count: (indirect_execution_draws.len() as u32).saturating_sub(segment_count),
    }
}

fn encode_cluster_state(state: crate::graphics::types::VirtualGeometryPrepareClusterState) -> u32 {
    match state {
        crate::graphics::types::VirtualGeometryPrepareClusterState::Resident => 0,
        crate::graphics::types::VirtualGeometryPrepareClusterState::PendingUpload => 1,
        crate::graphics::types::VirtualGeometryPrepareClusterState::Missing => 2,
    }
}

fn map_execution_state(
    state: crate::graphics::types::VirtualGeometryPrepareClusterState,
) -> RenderVirtualGeometryExecutionState {
    match state {
        crate::graphics::types::VirtualGeometryPrepareClusterState::Resident => {
            RenderVirtualGeometryExecutionState::Resident
        }
        crate::graphics::types::VirtualGeometryPrepareClusterState::PendingUpload => {
            RenderVirtualGeometryExecutionState::PendingUpload
        }
        crate::graphics::types::VirtualGeometryPrepareClusterState::Missing => {
            RenderVirtualGeometryExecutionState::Missing
        }
    }
}

#[cfg(test)]
mod tests {
    use super::execution_draw_ref_index;
    use crate::graphics::scene::scene_renderer::mesh::VirtualGeometrySubmissionDetail;

    const INDIRECT_ARGS_STRIDE_BYTES: u64 = (std::mem::size_of::<u32>() as u64) * 5;

    #[test]
    fn execution_draw_ref_index_prefers_explicit_submission_detail_source() {
        let submission_detail = VirtualGeometrySubmissionDetail {
            instance_index: Some(3),
            entity: 42,
            page_id: 300,
            submission_index: 7,
            draw_ref_rank: 2,
            draw_ref_index: 9,
            cluster_start_ordinal: 3,
            cluster_span_count: 1,
            cluster_total_count: 4,
            submission_slot: Some(5),
            state: crate::graphics::types::VirtualGeometryPrepareClusterState::Resident,
            lineage_depth: 2,
            lod_level: 1,
            frontier_rank: 6,
        };

        assert_eq!(
            execution_draw_ref_index(Some(submission_detail), 3 * INDIRECT_ARGS_STRIDE_BYTES),
            9,
            "expected execution ownership to keep the authoritative draw-ref index emitted by the shared submission truth instead of reconstructing it from indirect args offsets"
        );
    }

    #[test]
    fn execution_draw_ref_index_falls_back_to_indirect_args_offset_stride() {
        assert_eq!(
            execution_draw_ref_index(None, 4 * INDIRECT_ARGS_STRIDE_BYTES),
            4,
            "expected offset-based draw-ref recovery to remain as a compatibility fallback when explicit authoritative draw-ref truth is absent"
        );
    }
}
