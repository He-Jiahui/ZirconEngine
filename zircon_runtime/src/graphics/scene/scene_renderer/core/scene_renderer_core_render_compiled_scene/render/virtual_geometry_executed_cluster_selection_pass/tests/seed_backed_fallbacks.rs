use super::support::{cluster_ordering, clusters_by_id};
use super::*;

#[test]
fn seed_backed_execution_selection_falls_back_to_nearest_resident_parent_cluster() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                page_id: 200,
                lod_level: 2,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.75,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                page_id: 300,
                lod_level: 1,
                parent_cluster_id: Some(20),
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 40,
                page_id: 400,
                lod_level: 0,
                parent_cluster_id: Some(30),
                bounds_center: Vec3::new(2.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
        ],
        pages: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some("SeedCompatResidentParentFallbackUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(200_u32, true), (300_u32, true), (400_u32, false)]);
    let clusters_by_id = clusters_by_id(&extract);
    let cluster_ordering = cluster_ordering(&extract);

    assert_eq!(
        build_seed_backed_execution_selection_records(
            &extract,
            &clusters_by_id,
            &cluster_ordering,
            &page_residency,
            &mut HashSet::new(),
            0,
            entity,
            2,
            1,
            None,
        ),
        vec![SeedBackedExecutionSelectionRecord {
            selection: VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 40,
                cluster_ordinal: 2,
                page_id: 400,
                lod_level: 0,
                submission_page_id: 400,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 2,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            selected_cluster: RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 1,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        }],
        "expected resident-parent fallback to preserve the original child submission metadata while publishing the nearest resident ancestor on the selected-cluster seam, so later raster/debug passes can distinguish requested child work from the fallback cluster that will actually draw"
    );
}

#[test]
fn seed_backed_execution_selection_keeps_selected_cluster_order_when_later_child_overwrites_fallback_metadata(
) {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                page_id: 300,
                lod_level: 2,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.75,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 40,
                page_id: 400,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 50,
                page_id: 500,
                lod_level: 0,
                parent_cluster_id: Some(30),
                bounds_center: Vec3::new(2.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
        ],
        pages: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some("SeedCompatDuplicateFallbackUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(300_u32, true), (400_u32, true), (500_u32, false)]);
    let clusters_by_id = clusters_by_id(&extract);
    let cluster_ordering = cluster_ordering(&extract);

    assert_eq!(
        build_seed_backed_execution_selection_records(
            &extract,
            &clusters_by_id,
            &cluster_ordering,
            &page_residency,
            &mut HashSet::new(),
            0,
            entity,
            0,
            3,
            None,
        ),
        vec![
            SeedBackedExecutionSelectionRecord {
                selection: VirtualGeometryClusterSelection {
                    submission_index: 0,
                    instance_index: Some(0),
                    entity,
                    cluster_id: 50,
                    cluster_ordinal: 2,
                    page_id: 500,
                    lod_level: 0,
                    submission_page_id: 500,
                    submission_lod_level: 0,
                    entity_cluster_start_ordinal: 2,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 3,
                    lineage_depth: 1,
                    frontier_rank: 0,
                    resident_slot: None,
                    submission_slot: None,
                    state: VirtualGeometryPrepareClusterState::PendingUpload,
                },
                selected_cluster: RenderVirtualGeometrySelectedCluster {
                    instance_index: Some(0),
                    entity,
                    cluster_id: 30,
                    cluster_ordinal: 0,
                    page_id: 300,
                    lod_level: 2,
                    state: RenderVirtualGeometryExecutionState::Resident,
                },
            },
            SeedBackedExecutionSelectionRecord {
                selection: VirtualGeometryClusterSelection {
                    submission_index: 0,
                    instance_index: Some(0),
                    entity,
                    cluster_id: 40,
                    cluster_ordinal: 1,
                    page_id: 400,
                    lod_level: 1,
                    submission_page_id: 400,
                    submission_lod_level: 1,
                    entity_cluster_start_ordinal: 1,
                    entity_cluster_span_count: 1,
                    entity_cluster_total_count: 3,
                    lineage_depth: 0,
                    frontier_rank: 0,
                    resident_slot: None,
                    submission_slot: None,
                    state: VirtualGeometryPrepareClusterState::Resident,
                },
                selected_cluster: RenderVirtualGeometrySelectedCluster {
                    instance_index: Some(0),
                    entity,
                    cluster_id: 40,
                    cluster_ordinal: 1,
                    page_id: 400,
                    lod_level: 1,
                    state: RenderVirtualGeometryExecutionState::Resident,
                },
            },
        ],
        "expected duplicate resident-parent fallback to keep the resolved selected-cluster order stable while overwriting only the startup metadata with the later child request that collapsed onto that same resident cluster"
    );
}
