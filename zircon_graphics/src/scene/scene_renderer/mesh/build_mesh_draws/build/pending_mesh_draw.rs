use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use zircon_math::Vec4;
use zircon_scene::EntityId;

use super::super::virtual_geometry_cluster_raster_draw::VirtualGeometryClusterRasterDraw;
use crate::types::VirtualGeometryPrepareClusterState;

pub(super) struct PendingMeshDraw {
    pub(super) mesh: Arc<crate::scene::resources::GpuMeshResource>,
    pub(super) texture: Arc<crate::scene::resources::GpuTextureResource>,
    pub(super) pipeline_key: crate::scene::resources::PipelineKey,
    pub(super) model_matrix: [[f32; 4]; 4],
    pub(super) draw_tint: Vec4,
    pub(super) first_index: u32,
    pub(super) draw_index_count: u32,
    pub(super) indirect_draw_ref: Option<VirtualGeometryIndirectDrawRef>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct VirtualGeometryIndirectDrawRef {
    pub(super) mesh_index_count: u32,
    pub(super) segment_key: VirtualGeometryIndirectSegmentKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct VirtualGeometryIndirectSegmentKey {
    pub(super) entity: EntityId,
    pub(super) page_id: u32,
    pub(super) cluster_start_ordinal: u32,
    pub(super) cluster_span_count: u32,
    pub(super) cluster_total_count: u32,
    pub(super) lod_level: u8,
    pub(super) resident_slot: Option<u32>,
    pub(super) state: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryIndirectSegmentInput {
    pub(super) cluster_start_ordinal: u32,
    pub(super) cluster_span_count: u32,
    pub(super) cluster_total_count: u32,
    pub(super) page_id: u32,
    pub(super) resident_slot: u32,
    pub(super) state: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct VirtualGeometryIndirectDrawRefInput {
    pub(super) mesh_index_count: u32,
    pub(super) segment_index: u32,
}

pub(super) fn full_mesh_indirect_draw_ref(
    entity: EntityId,
    mesh_index_count: u32,
) -> VirtualGeometryIndirectDrawRef {
    VirtualGeometryIndirectDrawRef {
        mesh_index_count,
        segment_key: VirtualGeometryIndirectSegmentKey {
            entity,
            page_id: 0,
            cluster_start_ordinal: 0,
            cluster_span_count: 1,
            cluster_total_count: 1,
            lod_level: 0,
            resident_slot: Some(0),
            state: encode_cluster_state(VirtualGeometryPrepareClusterState::Resident),
        },
    }
}

pub(super) fn indirect_draw_ref_for_cluster_draw(
    entity: EntityId,
    mesh_index_count: u32,
    cluster_draw: VirtualGeometryClusterRasterDraw,
) -> VirtualGeometryIndirectDrawRef {
    VirtualGeometryIndirectDrawRef {
        mesh_index_count,
        segment_key: VirtualGeometryIndirectSegmentKey {
            entity,
            page_id: cluster_draw.page_id,
            cluster_start_ordinal: cluster_draw.entity_cluster_start_ordinal as u32,
            cluster_span_count: cluster_draw.entity_cluster_span_count.max(1) as u32,
            cluster_total_count: cluster_draw.entity_cluster_total_count.max(1) as u32,
            lod_level: cluster_draw.lod_level,
            resident_slot: cluster_draw.resident_slot,
            state: encode_cluster_state(cluster_draw.state),
        },
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
        resident_slot: segment_key.resident_slot.unwrap_or_default(),
        state: segment_key.state,
    }
}

pub(super) fn draw_ref_input(
    draw_ref: VirtualGeometryIndirectDrawRef,
    segment_index: u32,
) -> VirtualGeometryIndirectDrawRefInput {
    VirtualGeometryIndirectDrawRefInput {
        mesh_index_count: draw_ref.mesh_index_count,
        segment_index,
    }
}

fn encode_cluster_state(state: VirtualGeometryPrepareClusterState) -> u32 {
    match state {
        VirtualGeometryPrepareClusterState::Resident => 0,
        VirtualGeometryPrepareClusterState::PendingUpload => 1,
        VirtualGeometryPrepareClusterState::Missing => 2,
    }
}
