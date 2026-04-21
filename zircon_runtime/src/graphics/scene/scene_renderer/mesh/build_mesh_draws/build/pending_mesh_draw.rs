use std::sync::Arc;

use crate::core::framework::scene::EntityId;
use crate::core::math::Vec4;
use bytemuck::{Pod, Zeroable};

use crate::graphics::types::{
    VirtualGeometryClusterRasterDraw, VirtualGeometryPrepareClusterState,
};

pub(super) struct PendingMeshDraw {
    pub(super) mesh: Arc<crate::graphics::scene::resources::GpuMeshResource>,
    pub(super) texture: Arc<crate::graphics::scene::resources::GpuTextureResource>,
    pub(super) pipeline_key: crate::graphics::scene::resources::PipelineKey,
    pub(super) model_matrix: [[f32; 4]; 4],
    pub(super) draw_tint: Vec4,
    pub(super) first_index: u32,
    pub(super) draw_index_count: u32,
    pub(super) indirect_draw_ref: Option<VirtualGeometryIndirectDrawRef>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct VirtualGeometryIndirectDrawRef {
    pub(super) mesh_index_count: u32,
    pub(super) mesh_signature: u64,
    pub(super) segment_key: VirtualGeometryIndirectSegmentKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct VirtualGeometryIndirectSegmentKey {
    pub(super) submission_index: u32,
    pub(super) instance_index: Option<u32>,
    pub(super) entity: EntityId,
    pub(super) page_id: u32,
    pub(super) cluster_start_ordinal: u32,
    pub(super) cluster_span_count: u32,
    pub(super) cluster_total_count: u32,
    pub(super) lineage_depth: u32,
    pub(super) lod_level: u8,
    pub(super) frontier_rank: u32,
    pub(super) submission_slot: Option<u32>,
    pub(super) state: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryIndirectSegmentInput {
    pub(super) cluster_start_ordinal: u32,
    pub(super) cluster_span_count: u32,
    pub(super) cluster_total_count: u32,
    pub(super) page_id: u32,
    pub(super) submission_slot: u32,
    pub(super) state: u32,
    pub(super) lineage_depth: u32,
    pub(super) lod_level: u32,
    pub(super) frontier_rank: u32,
    pub(super) submission_index: u32,
    pub(super) instance_index: u32,
    pub(super) entity_lo: u32,
    pub(super) entity_hi: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryIndirectDrawRefInput {
    pub(super) mesh_index_count: u32,
    pub(super) segment_index: u32,
    pub(super) segment_draw_ref_count: u32,
    pub(super) submission_token: u32,
}

pub(super) fn indirect_draw_ref_for_cluster_draw(
    entity: EntityId,
    mesh_index_count: u32,
    mesh_signature: u64,
    cluster_draw: VirtualGeometryClusterRasterDraw,
) -> VirtualGeometryIndirectDrawRef {
    VirtualGeometryIndirectDrawRef {
        mesh_index_count,
        mesh_signature,
        segment_key: segment_key_for_cluster_draw(entity, cluster_draw),
    }
}

pub(super) fn segment_key_for_cluster_draw(
    entity: EntityId,
    cluster_draw: VirtualGeometryClusterRasterDraw,
) -> VirtualGeometryIndirectSegmentKey {
    VirtualGeometryIndirectSegmentKey {
        submission_index: cluster_draw.submission_index,
        instance_index: cluster_draw.instance_index,
        entity,
        page_id: cluster_draw.page_id,
        cluster_start_ordinal: cluster_draw.entity_cluster_start_ordinal as u32,
        cluster_span_count: cluster_draw.entity_cluster_span_count.max(1) as u32,
        cluster_total_count: cluster_draw.entity_cluster_total_count.max(1) as u32,
        lineage_depth: cluster_draw.lineage_depth,
        lod_level: cluster_draw.lod_level,
        frontier_rank: cluster_draw.frontier_rank,
        submission_slot: cluster_draw.submission_slot,
        state: encode_cluster_state(cluster_draw.state),
    }
}

pub(super) fn segment_input(
    segment_key: VirtualGeometryIndirectSegmentKey,
) -> VirtualGeometryIndirectSegmentInput {
    VirtualGeometryIndirectSegmentInput {
        cluster_start_ordinal: segment_key.cluster_start_ordinal,
        cluster_span_count: segment_key.cluster_span_count,
        cluster_total_count: segment_key.cluster_total_count,
        page_id: segment_key.page_id,
        submission_slot: segment_key.submission_slot.unwrap_or_default(),
        state: segment_key.state,
        lineage_depth: segment_key.lineage_depth,
        lod_level: u32::from(segment_key.lod_level),
        frontier_rank: segment_key.frontier_rank,
        submission_index: segment_key.submission_index,
        instance_index: segment_key.instance_index.unwrap_or(u32::MAX),
        entity_lo: segment_key.entity as u32,
        entity_hi: (segment_key.entity >> 32) as u32,
    }
}

pub(super) fn draw_ref_input(
    mesh_index_count: u32,
    segment_index: u32,
    segment_draw_ref_count: u32,
    submission_token: u32,
) -> VirtualGeometryIndirectDrawRefInput {
    VirtualGeometryIndirectDrawRefInput {
        mesh_index_count,
        segment_index,
        segment_draw_ref_count,
        submission_token,
    }
}

fn encode_cluster_state(state: VirtualGeometryPrepareClusterState) -> u32 {
    match state {
        VirtualGeometryPrepareClusterState::Resident => 0,
        VirtualGeometryPrepareClusterState::PendingUpload => 1,
        VirtualGeometryPrepareClusterState::Missing => 2,
    }
}
