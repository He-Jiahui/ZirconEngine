use super::{
    VirtualGeometryPrepareCluster, VirtualGeometryPrepareClusterState, VirtualGeometryPrepareFrame,
    VirtualGeometryPrepareIndirectDraw, VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};

#[test]
fn virtual_geometry_unified_indirect_synthesizes_fallback_cluster_slices_when_segments_are_absent()
{
    let frame = VirtualGeometryPrepareFrame {
        visible_entities: vec![2],
        visible_clusters: vec![
            VirtualGeometryPrepareCluster {
                entity: 2,
                cluster_id: 20,
                page_id: 300,
                lod_level: 0,
                resident_slot: Some(2),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryPrepareCluster {
                entity: 2,
                cluster_id: 21,
                page_id: 301,
                lod_level: 0,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        cluster_draw_segments: Vec::new(),
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 300,
            slot: 2,
            size_bytes: 4096,
        }],
        pending_page_requests: vec![VirtualGeometryPrepareRequest {
            page_id: 301,
            size_bytes: 4096,
            generation: 1,
            frontier_rank: 0,
            assigned_slot: Some(1),
            recycled_page_id: None,
        }],
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    };

    assert_eq!(
        frame.unified_indirect_draws(),
        vec![
            VirtualGeometryPrepareIndirectDraw {
                entity: 2,
                page_id: 301,
                cluster_start_ordinal: 1,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: Some(1),
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            VirtualGeometryPrepareIndirectDraw {
                entity: 2,
                page_id: 300,
                cluster_start_ordinal: 0,
                cluster_span_count: 1,
                cluster_total_count: 2,
                lineage_depth: 0,
                lod_level: 0,
                frontier_rank: 0,
                resident_slot: Some(2),
                submission_slot: Some(2),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
        ],
        "expected unified indirect ownership itself to synthesize per-cluster fallback slices from visible clusters while preserving entity-local cluster ordinals even after authoritative submission ordering reorders the resulting draws"
    );
}
