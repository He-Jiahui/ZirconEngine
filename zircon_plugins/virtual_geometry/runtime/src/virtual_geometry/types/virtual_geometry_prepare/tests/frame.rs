use super::VirtualGeometryPrepareFrame;
use crate::virtual_geometry::{
    VirtualGeometryClusterSelection, VirtualGeometryPrepareClusterState,
    VirtualGeometryPrepareDrawSegment, VirtualGeometryPreparePage, VirtualGeometryPrepareRequest,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugState,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometryVisBufferMark,
};
use zircon_runtime::core::math::{Transform, Vec3};

#[test]
fn prepare_frame_exposes_same_frame_visbuffer_marks_from_unified_draw_truth() {
    let entity = 101_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 1,
                hierarchy_node_id: None,
                page_id: 10,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 2,
                hierarchy_node_id: None,
                page_id: 20,
                lod_level: 10,
                parent_cluster_id: Some(1),
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.2,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 0,
            mesh_name: Some("PrepareVisBufferUnitTest".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            visualize_visbuffer: true,
            ..RenderVirtualGeometryDebugState::default()
        },
    };
    let frame = VirtualGeometryPrepareFrame {
        visible_entities: vec![entity],
        visible_clusters: Vec::new(),
        cluster_draw_segments: Vec::new(),
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 10,
            slot: 0,
            size_bytes: 4096,
        }],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    };

    assert_eq!(
        frame.same_frame_visbuffer_debug_marks(&extract),
        vec![RenderVirtualGeometryVisBufferMark {
            instance_index: Some(0),
            entity,
            cluster_id: 1,
            page_id: 10,
            lod_level: 10,
            state:
                zircon_runtime::core::framework::render::RenderVirtualGeometryExecutionState::Resident,
            color_rgba: [179, 212, 35, 255],
        }],
        "expected prepare-owned same-frame visbuffer marks to follow unified indirect draw truth before the stored renderer snapshot is backfilled from execution"
    );
}

#[test]
fn prepare_frame_exposes_cluster_selection_from_unified_draw_truth() {
    let entity = 101_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 1,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 1,
                hierarchy_node_id: None,
                page_id: 10,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 2,
                hierarchy_node_id: None,
                page_id: 20,
                lod_level: 10,
                parent_cluster_id: Some(1),
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.2,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 0,
            mesh_name: Some("PrepareClusterSelectionUnitTest".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState {
            visualize_visbuffer: true,
            ..RenderVirtualGeometryDebugState::default()
        },
    };
    let frame = VirtualGeometryPrepareFrame {
        visible_entities: vec![entity],
        visible_clusters: Vec::new(),
        cluster_draw_segments: Vec::new(),
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 10,
            slot: 0,
            size_bytes: 4096,
        }],
        pending_page_requests: Vec::new(),
        available_slots: Vec::new(),
        evictable_pages: Vec::new(),
    };

    assert_eq!(
        frame.selected_clusters(&extract),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity,
            cluster_id: 1,
            cluster_ordinal: 0,
            page_id: 10,
            lod_level: 10,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected prepare-owned cluster selection to expose the authoritative current-frame cluster worklist derived from unified indirect draw truth"
    );
}

#[test]
fn prepare_frame_cluster_selections_include_submission_metadata_from_unified_draw_truth() {
    let entity = 101_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 3,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 1,
                hierarchy_node_id: None,
                page_id: 10,
                lod_level: 10,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 2,
                hierarchy_node_id: None,
                page_id: 20,
                lod_level: 11,
                parent_cluster_id: Some(1),
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.2,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 3,
                hierarchy_node_id: None,
                page_id: 20,
                lod_level: 11,
                parent_cluster_id: Some(1),
                bounds_center: Vec3::new(2.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.15,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 0,
            mesh_name: Some("PrepareClusterSelectionMetadataUnitTest".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let frame = VirtualGeometryPrepareFrame {
        visible_entities: vec![entity],
        visible_clusters: vec![
            crate::virtual_geometry::VirtualGeometryPrepareCluster {
                entity,
                cluster_id: 1,
                page_id: 10,
                lod_level: 10,
                resident_slot: Some(5),
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            crate::virtual_geometry::VirtualGeometryPrepareCluster {
                entity,
                cluster_id: 2,
                page_id: 20,
                lod_level: 11,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            crate::virtual_geometry::VirtualGeometryPrepareCluster {
                entity,
                cluster_id: 3,
                page_id: 20,
                lod_level: 11,
                resident_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        cluster_draw_segments: vec![VirtualGeometryPrepareDrawSegment {
            entity,
            cluster_id: 2,
            page_id: 20,
            resident_slot: None,
            cluster_ordinal: 1,
            cluster_span_count: 2,
            cluster_count: 3,
            lineage_depth: 4,
            lod_level: 11,
            state: VirtualGeometryPrepareClusterState::PendingUpload,
        }],
        resident_pages: vec![VirtualGeometryPreparePage {
            page_id: 10,
            slot: 5,
            size_bytes: 4096,
        }],
        pending_page_requests: vec![VirtualGeometryPrepareRequest {
            page_id: 20,
            size_bytes: 8192,
            generation: 2,
            frontier_rank: 7,
            assigned_slot: Some(9),
            recycled_page_id: None,
        }],
        available_slots: vec![11],
        evictable_pages: Vec::new(),
    };

    assert_eq!(
        frame.cluster_selections(&extract),
        vec![
            VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 2,
                cluster_ordinal: 1,
                page_id: 20,
                lod_level: 11,
                submission_page_id: 20,
                submission_lod_level: 11,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 2,
                entity_cluster_total_count: 3,
                lineage_depth: 4,
                frontier_rank: 7,
                resident_slot: None,
                submission_slot: Some(9),
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 3,
                cluster_ordinal: 2,
                page_id: 20,
                lod_level: 11,
                submission_page_id: 20,
                submission_lod_level: 11,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 2,
                entity_cluster_total_count: 3,
                lineage_depth: 4,
                frontier_rank: 7,
                resident_slot: None,
                submission_slot: Some(9),
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        "expected prepare-owned cluster selections to keep both cluster identity and raster submission metadata so the same internal worklist can project public selected-cluster debug state and fallback raster submissions"
    );
}
