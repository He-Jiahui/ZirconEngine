use super::support::{
    cluster_ordering, clusters_by_id, node_and_cluster_cull_pass_output_from_launch_worklist,
};
use super::*;

#[test]
fn seed_backed_execution_selection_collection_applies_cluster_budget_after_stable_selected_cluster_order(
) {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 40,
                hierarchy_node_id: None,
                page_id: 400,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                hierarchy_node_id: None,
                page_id: 300,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedBackedBudgetOrderUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };

    let collection = collect_execution_cluster_selection_collection_from_root_seeds(
        Some(&extract),
        &node_and_cluster_cull_pass_output_from_launch_worklist(
            1,
            2,
            vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: 0,
                entity,
                cluster_offset: 0,
                cluster_count: 2,
                page_offset: 0,
                page_count: 2,
            }],
            RenderVirtualGeometryDebugState::default(),
        ),
    );

    assert_eq!(
        collection.selections(),
        vec![VirtualGeometryClusterSelection {
            submission_index: 0,
            instance_index: Some(0),
            entity,
            cluster_id: 30,
            cluster_ordinal: 0,
            page_id: 300,
            lod_level: 1,
            submission_page_id: 300,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: VirtualGeometryPrepareClusterState::Missing,
        }],
        "expected cluster_budget clamping to happen after the root-seed baseline seam reaches stable selected-cluster ordering, so an unsorted extract still keeps the ordinal-0 cluster instead of the first raw slice entry"
    );
    assert_eq!(
        collection.selected_clusters(),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity,
            cluster_id: 30,
            cluster_ordinal: 0,
            page_id: 300,
            lod_level: 1,
            state: RenderVirtualGeometryExecutionState::Missing,
        }],
        "expected selected-cluster publication to see cluster-budget clamping only after stable ordering has been established for the root-seed baseline path"
    );
}

#[test]
fn seed_backed_execution_selection_derives_frontier_rank_from_first_unresolved_page_occurrence() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 8,
        page_budget: 4,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                hierarchy_node_id: None,
                page_id: 200,
                lod_level: 3,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 1.0,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                hierarchy_node_id: None,
                page_id: 300,
                lod_level: 2,
                parent_cluster_id: None,
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.75,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 40,
                hierarchy_node_id: None,
                page_id: 400,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::new(2.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 50,
                hierarchy_node_id: None,
                page_id: 500,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::new(3.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 4,
            page_offset: 0,
            page_count: 4,
            mesh_name: Some("SeedBackedFrontierRankUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(200_u32, true), (300_u32, false), (500_u32, false)]);
    let clusters_by_id = clusters_by_id(&extract);
    let cluster_ordering = cluster_ordering(&extract);

    assert_eq!(
        build_seed_backed_execution_selections(
            &extract,
            &clusters_by_id,
            &cluster_ordering,
            &page_residency,
            &mut HashSet::new(),
            0,
            entity,
            0,
            4,
            None,
        ),
        vec![
            VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 3,
                submission_page_id: 200,
                submission_lod_level: 3,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::Resident,
            },
            VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 2,
                submission_page_id: 300,
                submission_lod_level: 2,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
            VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 40,
                cluster_ordinal: 2,
                page_id: 400,
                lod_level: 1,
                submission_page_id: 400,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 1,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::Missing,
            },
            VirtualGeometryClusterSelection {
                submission_index: 0,
                instance_index: Some(0),
                entity,
                cluster_id: 50,
                cluster_ordinal: 3,
                page_id: 500,
                lod_level: 0,
                submission_page_id: 500,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 3,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 4,
                lineage_depth: 0,
                frontier_rank: 2,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        "expected the seed-backed execution selection path to approximate frontier order from the first occurrence of each unresolved page in the expanded worklist, so later raster/debug seams can stop reporting every seed-expanded cluster as frontier rank zero before true VisitNode traversal exists"
    );
}
