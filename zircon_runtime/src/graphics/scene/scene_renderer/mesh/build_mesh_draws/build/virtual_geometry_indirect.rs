use std::collections::BTreeMap;
use std::sync::Arc;

use wgpu::util::DeviceExt;

use crate::core::framework::render::{
    RenderVirtualGeometryExecutionSegment, RenderVirtualGeometryExecutionState,
};
use crate::core::framework::scene::EntityId;
use crate::graphics::scene::scene_renderer::mesh::mesh_draw::VirtualGeometrySubmissionDetail;
use crate::graphics::types::ViewportRenderFrame;

use super::super::indexed_indirect_args::IndexedIndirectArgs;
use super::pending_mesh_draw::{
    draw_ref_input, segment_input, PendingMeshDraw, VirtualGeometryIndirectDrawRef,
    VirtualGeometryIndirectDrawRefInput, VirtualGeometryIndirectSegmentInput,
    VirtualGeometryIndirectSegmentKey,
};

pub(super) struct VirtualGeometryIndirectDrawPlan {
    pub(super) segment_count: u32,
    pub(super) args_count: u32,
    pub(super) args_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) submission_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) authority_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) draw_ref_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) segment_buffer: Option<Arc<wgpu::Buffer>>,
    pub(super) args_buffers: Vec<Option<Arc<wgpu::Buffer>>>,
    pub(super) args_offsets: Vec<Option<u64>>,
    pub(super) draw_ref_indices: Vec<Option<u32>>,
    pub(super) submission_tokens: Vec<Option<u32>>,
    pub(super) submission_details: Vec<Option<VirtualGeometrySubmissionDetail>>,
}

impl VirtualGeometryIndirectDrawPlan {
    fn empty(draw_count: usize) -> Self {
        Self {
            segment_count: 0,
            args_count: 0,
            args_buffer: None,
            submission_buffer: None,
            authority_buffer: None,
            draw_ref_buffer: None,
            segment_buffer: None,
            args_buffers: vec![None; draw_count],
            args_offsets: vec![None; draw_count],
            draw_ref_indices: vec![None; draw_count],
            submission_tokens: vec![None; draw_count],
            submission_details: vec![None; draw_count],
        }
    }
}

pub(super) fn build_virtual_geometry_indirect_draw_plan(
    device: &wgpu::Device,
    frame: &ViewportRenderFrame,
    virtual_geometry_enabled: bool,
    pending_draws: &mut Vec<PendingMeshDraw>,
) -> VirtualGeometryIndirectDrawPlan {
    if !virtual_geometry_enabled {
        return VirtualGeometryIndirectDrawPlan::empty(pending_draws.len());
    }
    let Some(snapshot) = frame.virtual_geometry_debug_snapshot.as_ref() else {
        return VirtualGeometryIndirectDrawPlan::empty(pending_draws.len());
    };
    if snapshot.execution_segments.is_empty() {
        return VirtualGeometryIndirectDrawPlan::empty(pending_draws.len());
    }

    let segments_by_entity = execution_segments_by_entity(&snapshot.execution_segments);
    if segments_by_entity.is_empty() {
        return VirtualGeometryIndirectDrawPlan::empty(pending_draws.len());
    }

    let draw_segments =
        expand_pending_draws_for_execution_segments(pending_draws, &segments_by_entity);
    let Some(indirect_inputs) = build_indirect_inputs(pending_draws, &draw_segments) else {
        return VirtualGeometryIndirectDrawPlan::empty(pending_draws.len());
    };

    let args_buffer = create_buffer(
        device,
        "zircon-vg-mesh-indirect-args",
        bytemuck::cast_slice(&indirect_inputs.args),
        indirect_args_usage(),
    );
    let args_buffers = draw_segments
        .iter()
        .map(|segment| segment.as_ref().map(|_| Arc::clone(&args_buffer)))
        .collect::<Vec<_>>();
    let submission_buffer = create_buffer(
        device,
        "zircon-vg-mesh-indirect-submissions",
        bytemuck::cast_slice(&indirect_inputs.submission_tokens),
        metadata_buffer_usage(),
    );
    let authority_buffer = create_buffer(
        device,
        "zircon-vg-mesh-indirect-authority",
        bytemuck::cast_slice(&indirect_inputs.authority_words),
        metadata_buffer_usage(),
    );
    let draw_ref_buffer = create_buffer(
        device,
        "zircon-vg-mesh-indirect-draw-refs",
        bytemuck::cast_slice(&indirect_inputs.draw_refs),
        metadata_buffer_usage(),
    );
    let segment_buffer = create_buffer(
        device,
        "zircon-vg-mesh-indirect-segments",
        bytemuck::cast_slice(&indirect_inputs.segments),
        metadata_buffer_usage(),
    );

    VirtualGeometryIndirectDrawPlan {
        segment_count: saturated_u32_len(indirect_inputs.segments.len()),
        args_count: saturated_u32_len(indirect_inputs.args.len()),
        args_buffer: Some(args_buffer),
        submission_buffer: Some(submission_buffer),
        authority_buffer: Some(authority_buffer),
        draw_ref_buffer: Some(draw_ref_buffer),
        segment_buffer: Some(segment_buffer),
        args_buffers,
        args_offsets: indirect_inputs.args_offsets,
        draw_ref_indices: indirect_inputs.draw_ref_indices,
        submission_tokens: indirect_inputs.per_draw_submission_tokens,
        submission_details: indirect_inputs.submission_details,
    }
}

fn execution_segments_by_entity(
    execution_segments: &[RenderVirtualGeometryExecutionSegment],
) -> BTreeMap<EntityId, Vec<RenderVirtualGeometryExecutionSegment>> {
    let mut segments = BTreeMap::<_, Vec<_>>::new();
    for segment in execution_segments {
        if segment.state == RenderVirtualGeometryExecutionState::Missing {
            continue;
        }
        segments
            .entry(segment.entity)
            .or_default()
            .push(segment.clone());
    }
    segments
}

fn expand_pending_draws_for_execution_segments(
    pending_draws: &mut Vec<PendingMeshDraw>,
    segments_by_entity: &BTreeMap<EntityId, Vec<RenderVirtualGeometryExecutionSegment>>,
) -> Vec<Option<RenderVirtualGeometryExecutionSegment>> {
    let mut expanded_draws = Vec::with_capacity(pending_draws.len());
    let mut draw_segments = Vec::with_capacity(pending_draws.len());

    for pending_draw in pending_draws.drain(..) {
        let Some(entity_segments) = segments_by_entity.get(&pending_draw.source_entity) else {
            expanded_draws.push(pending_draw);
            draw_segments.push(None);
            continue;
        };
        for segment in entity_segments {
            let mut draw = pending_draw.clone();
            draw.indirect_draw_ref = Some(indirect_draw_ref_for_segment(segment));
            expanded_draws.push(draw);
            draw_segments.push(Some(segment.clone()));
        }
    }

    *pending_draws = expanded_draws;
    draw_segments
}

struct VirtualGeometryIndirectInputs {
    args: Vec<IndexedIndirectArgs>,
    segments: Vec<VirtualGeometryIndirectSegmentInput>,
    draw_refs: Vec<VirtualGeometryIndirectDrawRefInput>,
    submission_tokens: Vec<u32>,
    authority_words: Vec<u32>,
    args_offsets: Vec<Option<u64>>,
    draw_ref_indices: Vec<Option<u32>>,
    per_draw_submission_tokens: Vec<Option<u32>>,
    submission_details: Vec<Option<VirtualGeometrySubmissionDetail>>,
}

fn build_indirect_inputs(
    pending_draws: &[PendingMeshDraw],
    draw_segments: &[Option<RenderVirtualGeometryExecutionSegment>],
) -> Option<VirtualGeometryIndirectInputs> {
    let indirect_draw_count = draw_segments
        .iter()
        .filter(|segment| segment.is_some())
        .count();
    if indirect_draw_count == 0 {
        return None;
    }

    let indirect_args_stride = std::mem::size_of::<IndexedIndirectArgs>() as u64;
    let mut args = Vec::with_capacity(indirect_draw_count);
    let mut segments = Vec::with_capacity(indirect_draw_count);
    let mut draw_refs = Vec::with_capacity(indirect_draw_count);
    let mut submission_tokens = Vec::with_capacity(indirect_draw_count);
    let mut authority_words = Vec::with_capacity(indirect_draw_count * 4);
    let mut args_offsets = vec![None; pending_draws.len()];
    let mut draw_ref_indices = vec![None; pending_draws.len()];
    let mut per_draw_submission_tokens = vec![None; pending_draws.len()];
    let mut submission_details = vec![None; pending_draws.len()];

    for (draw_index, (pending_draw, segment)) in pending_draws.iter().zip(draw_segments).enumerate()
    {
        let Some(segment) = segment else {
            continue;
        };
        let draw_ref_index = saturated_u32_len(args.len());
        let args_offset = u64::from(draw_ref_index) * indirect_args_stride;
        let submission_index = segment.submission_index.unwrap_or(draw_ref_index);
        let draw_ref_rank = segment.draw_ref_rank.unwrap_or(draw_ref_index);
        let submission_token = submission_token(submission_index, draw_ref_rank);
        let segment_key = segment_key_for_execution_segment(segment);

        args.push(IndexedIndirectArgs {
            index_count: pending_draw.draw_index_count,
            instance_count: 1,
            first_index: pending_draw.first_index,
            base_vertex: 0,
            first_instance: 0,
        });
        segments.push(segment_input(segment_key));
        draw_refs.push(draw_ref_input(
            pending_draw.draw_index_count,
            draw_ref_index,
            1,
            submission_token,
        ));
        submission_tokens.push(submission_token);
        authority_words.extend_from_slice(&[
            submission_index,
            draw_ref_rank,
            segment.page_id,
            state_word(segment.state),
        ]);
        args_offsets[draw_index] = Some(args_offset);
        draw_ref_indices[draw_index] = Some(draw_ref_index);
        per_draw_submission_tokens[draw_index] = Some(submission_token);
        submission_details[draw_index] = Some(VirtualGeometrySubmissionDetail::new(
            segment.instance_index,
            segment.entity,
            segment.page_id,
            submission_index,
            draw_ref_rank,
            draw_ref_index,
            segment.cluster_start_ordinal,
            segment.cluster_span_count,
            segment.cluster_total_count,
            segment.submission_slot,
            segment.state,
            segment.lineage_depth,
            segment.lod_level,
            segment.frontier_rank,
        ));
    }

    Some(VirtualGeometryIndirectInputs {
        args,
        segments,
        draw_refs,
        submission_tokens,
        authority_words,
        args_offsets,
        draw_ref_indices,
        per_draw_submission_tokens,
        submission_details,
    })
}

fn indirect_draw_ref_for_segment(
    segment: &RenderVirtualGeometryExecutionSegment,
) -> VirtualGeometryIndirectDrawRef {
    VirtualGeometryIndirectDrawRef {
        segment_key: segment_key_for_execution_segment(segment),
    }
}

fn segment_key_for_execution_segment(
    segment: &RenderVirtualGeometryExecutionSegment,
) -> VirtualGeometryIndirectSegmentKey {
    VirtualGeometryIndirectSegmentKey {
        submission_index: segment.submission_index.unwrap_or(segment.draw_ref_index),
        instance_index: segment.instance_index,
        entity: segment.entity,
        page_id: segment.page_id,
        cluster_start_ordinal: segment.cluster_start_ordinal,
        cluster_span_count: segment.cluster_span_count,
        cluster_total_count: segment.cluster_total_count,
        lineage_depth: segment.lineage_depth,
        lod_level: segment.lod_level,
        frontier_rank: segment.frontier_rank,
        submission_slot: segment.submission_slot,
        state: state_word(segment.state),
    }
}

fn submission_token(submission_index: u32, draw_ref_rank: u32) -> u32 {
    (submission_index << 16) | (draw_ref_rank & 0xFFFF)
}

fn state_word(state: RenderVirtualGeometryExecutionState) -> u32 {
    match state {
        RenderVirtualGeometryExecutionState::Resident => 0,
        RenderVirtualGeometryExecutionState::PendingUpload => 1,
        RenderVirtualGeometryExecutionState::Missing => 2,
    }
}

fn saturated_u32_len(len: usize) -> u32 {
    u32::try_from(len).unwrap_or(u32::MAX)
}

fn create_buffer(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[u8],
    usage: wgpu::BufferUsages,
) -> Arc<wgpu::Buffer> {
    Arc::new(
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents,
            usage,
        }),
    )
}

fn indirect_args_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::INDIRECT
        | wgpu::BufferUsages::STORAGE
        | wgpu::BufferUsages::COPY_DST
        | wgpu::BufferUsages::COPY_SRC
}

fn metadata_buffer_usage() -> wgpu::BufferUsages {
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC
}
