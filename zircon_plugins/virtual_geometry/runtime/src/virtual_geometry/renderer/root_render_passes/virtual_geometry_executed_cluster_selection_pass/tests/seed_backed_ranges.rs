use super::support::{
    cluster_ordering, clusters_by_id, node_and_cluster_cull_pass_output_from_launch_worklist,
};
use super::*;

#[test]
fn seed_backed_execution_selection_expands_all_clusters_in_seed_range_and_page_residency() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                hierarchy_node_id: None,
                page_id: 200,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                hierarchy_node_id: None,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedBackedUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(200_u32, true), (300_u32, false)]);
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
            2,
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
                lod_level: 1,
                submission_page_id: 200,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 2,
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
                lod_level: 0,
                submission_page_id: 300,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 2,
                lineage_depth: 0,
                frontier_rank: 0,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::PendingUpload,
            },
        ],
        "expected the seed-backed execution selection path to expand the full cluster range in one instance seed and derive execution state from page residency so later VisBuffer64/HardwareRasterization passes can bind a real multi-cluster seam before full BVH traversal exists"
    );
}

#[test]
fn seed_backed_execution_selection_derives_lineage_depth_from_parent_chain() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 3,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                hierarchy_node_id: None,
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
                hierarchy_node_id: None,
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
                hierarchy_node_id: None,
                page_id: 400,
                lod_level: 0,
                parent_cluster_id: Some(30),
                bounds_center: Vec3::new(2.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some("SeedBackedLineageDepthUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(200_u32, false), (300_u32, false)]);
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
            3,
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
                lod_level: 2,
                submission_page_id: 200,
                submission_lod_level: 2,
                entity_cluster_start_ordinal: 0,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
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
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 1,
                submission_page_id: 300,
                submission_lod_level: 1,
                entity_cluster_start_ordinal: 1,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 1,
                frontier_rank: 1,
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
                lod_level: 0,
                submission_page_id: 400,
                submission_lod_level: 0,
                entity_cluster_start_ordinal: 2,
                entity_cluster_span_count: 1,
                entity_cluster_total_count: 3,
                lineage_depth: 2,
                frontier_rank: 2,
                resident_slot: None,
                submission_slot: None,
                state: VirtualGeometryPrepareClusterState::Missing,
            },
        ],
        "expected the seed-backed execution selection path to derive lineage_depth from the parent cluster chain so downstream hardware-rasterization startup metadata matches the existing visibility-plan lineage semantics instead of hardcoding every seed-expanded cluster to depth zero"
    );
}

#[test]
fn seed_backed_execution_selection_keeps_instance_local_cluster_ordinal_for_subset_seed_ranges() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                hierarchy_node_id: None,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                hierarchy_node_id: None,
                page_id: 200,
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
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 1,
            cluster_count: 1,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedBackedSubsetOrderingUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(200_u32, true), (300_u32, false)]);
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
            1,
            1,
            None,
        ),
        vec![VirtualGeometryClusterSelection {
            submission_index: 0,
            instance_index: Some(0),
            entity,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            submission_page_id: 200,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 1,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: VirtualGeometryPrepareClusterState::Resident,
        }],
        "expected the seed-backed execution selection path to preserve the stable instance-local cluster ordinal even when the current root seed covers only a subset of the source slice, so downstream raster slice metadata does not reuse the raw extract offset as though it were already the submission ordinal"
    );
}

#[test]
fn seed_backed_execution_selection_collection_uses_node_and_cluster_cull_seed_range_as_the_authoritative_submission_slice(
) {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 3,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                hierarchy_node_id: None,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::new(1.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                hierarchy_node_id: None,
                page_id: 200,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 10,
                hierarchy_node_id: None,
                page_id: 100,
                lod_level: 2,
                parent_cluster_id: None,
                bounds_center: Vec3::new(-1.0, 0.0, 0.0),
                bounds_radius: 0.5,
                screen_space_error: 0.75,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![RenderVirtualGeometryPage {
            page_id: 200,
            resident: true,
            size_bytes: 2048,
        }],
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 3,
            page_offset: 0,
            page_count: 3,
            mesh_name: Some("SeedBackedAuthoritativeSeedSliceUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };

    let collection = collect_execution_cluster_selection_collection_from_root_seeds(
        Some(&extract),
        &node_and_cluster_cull_pass_output_from_launch_worklist(
            1,
            3,
            vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: 0,
                entity,
                cluster_offset: 1,
                cluster_count: 1,
                page_offset: 0,
                page_count: 3,
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
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            submission_page_id: 200,
            submission_lod_level: 1,
            entity_cluster_start_ordinal: 0,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 1,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: VirtualGeometryPrepareClusterState::Resident,
        }],
        "expected the seed-backed baseline seam to treat NodeAndClusterCull.instance_seeds as the authoritative submission slice, so a later split seed over a larger extract instance still resets ordinal/start/total metadata to that seed-local worklist instead of drifting back to the broader extract slice"
    );
    assert_eq!(
        collection.selected_clusters(),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 1,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the selected-cluster publication on the root-seed baseline path to use the same authoritative seed-local ordinal as the submission metadata so later VisBuffer64 and hardware-raster seams stay aligned once NodeAndClusterCull starts emitting narrower seed ranges than the original extract instances"
    );
}

#[test]
fn seed_backed_execution_selection_collection_requires_explicit_instance_work_item_contract() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![RenderVirtualGeometryCluster {
            entity,
            cluster_id: 20,
            hierarchy_node_id: None,
            page_id: 200,
            lod_level: 1,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.5,
            screen_space_error: 0.5,
        }],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![RenderVirtualGeometryPage {
            page_id: 200,
            resident: true,
            size_bytes: 2048,
        }],
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 1,
            page_offset: 0,
            page_count: 1,
            mesh_name: Some("SeedBackedRequiresLaunchWorklistUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let mut node_and_cluster_cull_pass = node_and_cluster_cull_pass_output_from_launch_worklist(
        1,
        1,
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 0,
            entity,
            cluster_offset: 0,
            cluster_count: 1,
            page_offset: 0,
            page_count: 1,
        }],
        RenderVirtualGeometryDebugState::default(),
    );
    node_and_cluster_cull_pass.clear_instance_work_items_for_test();

    let collection = collect_execution_cluster_selection_collection_from_root_seeds(
        Some(&extract),
        &node_and_cluster_cull_pass,
    );

    assert!(
        collection.selections().is_empty() && collection.selected_clusters().is_empty(),
        "expected the seed-backed execution selection path to require the explicit NodeAndClusterCull instance-work-item contract instead of silently reconstructing execution work from launch setup or the broader extract"
    );
}

#[test]
fn seed_backed_execution_selection_collection_requires_explicit_cluster_work_item_contract() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 1,
        page_budget: 1,
        clusters: vec![RenderVirtualGeometryCluster {
            entity,
            cluster_id: 20,
            hierarchy_node_id: None,
            page_id: 200,
            lod_level: 1,
            parent_cluster_id: None,
            bounds_center: Vec3::ZERO,
            bounds_radius: 0.5,
            screen_space_error: 0.5,
        }],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![RenderVirtualGeometryPage {
            page_id: 200,
            resident: true,
            size_bytes: 2048,
        }],
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 1,
            page_offset: 0,
            page_count: 1,
            mesh_name: Some("SeedBackedRequiresClusterWorkItemsUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let mut node_and_cluster_cull_pass = node_and_cluster_cull_pass_output_from_launch_worklist(
        1,
        1,
        vec![RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
            instance_index: 0,
            entity,
            cluster_offset: 0,
            cluster_count: 1,
            page_offset: 0,
            page_count: 1,
        }],
        RenderVirtualGeometryDebugState::default(),
    );
    node_and_cluster_cull_pass.clear_cluster_work_items_for_test();

    let collection = collect_execution_cluster_selection_collection_from_root_seeds(
        Some(&extract),
        &node_and_cluster_cull_pass,
    );

    assert!(
        collection.selections().is_empty() && collection.selected_clusters().is_empty(),
        "expected the seed-backed execution selection path to require the explicit NodeAndClusterCull cluster-work-item seam instead of silently reconstructing cluster candidates from broader instance ranges"
    );
}

#[test]
fn seed_backed_execution_selection_respects_forced_mip() {
    let entity = 42_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 4,
        page_budget: 2,
        clusters: vec![
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 20,
                hierarchy_node_id: None,
                page_id: 200,
                lod_level: 1,
                parent_cluster_id: None,
                bounds_center: Vec3::ZERO,
                bounds_radius: 0.5,
                screen_space_error: 0.5,
            },
            RenderVirtualGeometryCluster {
                entity,
                cluster_id: 30,
                hierarchy_node_id: None,
                page_id: 300,
                lod_level: 0,
                parent_cluster_id: None,
                bounds_center: Vec3::X,
                bounds_radius: 0.5,
                screen_space_error: 0.25,
            },
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: Vec::new(),
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("SeedBackedForcedMipUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let page_residency = HashMap::from([(200_u32, true), (300_u32, false)]);
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
            2,
            Some(0),
        ),
        vec![VirtualGeometryClusterSelection {
            submission_index: 0,
            instance_index: Some(0),
            entity,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            submission_page_id: 300,
            submission_lod_level: 0,
            entity_cluster_start_ordinal: 1,
            entity_cluster_span_count: 1,
            entity_cluster_total_count: 2,
            lineage_depth: 0,
            frontier_rank: 0,
            resident_slot: None,
            submission_slot: None,
            state: VirtualGeometryPrepareClusterState::PendingUpload,
        }],
        "expected the seed-backed execution selection path to honor forced_mip while expanding a seed range so render-path execution selection stays aligned with the manual mip override before real BVH traversal lands"
    );
}
