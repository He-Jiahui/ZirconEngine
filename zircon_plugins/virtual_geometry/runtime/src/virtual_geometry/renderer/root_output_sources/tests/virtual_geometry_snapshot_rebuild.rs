use super::{
    rebuild_selected_clusters_from_execution_segments, resolve_selected_clusters_for_store,
    resolve_visbuffer64_entries_for_store,
};
use zircon_runtime::core::framework::render::{
    RenderVirtualGeometryCluster, RenderVirtualGeometryDebugSnapshot,
    RenderVirtualGeometryDebugState, RenderVirtualGeometryExecutionSegment,
    RenderVirtualGeometryExecutionState, RenderVirtualGeometryExtract,
    RenderVirtualGeometryInstance, RenderVirtualGeometryPage, RenderVirtualGeometrySelectedCluster,
    RenderVirtualGeometrySelectedClusterSource, RenderVirtualGeometryVisBuffer64Entry,
    RenderVirtualGeometryVisBuffer64Source,
};
use zircon_runtime::core::math::{Transform, Vec3};

#[test]
fn rebuild_selected_clusters_from_execution_segments_drops_visibility_only_superset() {
    let entity = 77_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 0,
        clusters: vec![
            cluster(entity, 20, 200, 0, Vec3::ZERO, 9.0),
            cluster(entity, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, false), page(300, true)],
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("StoreOutputsUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        debug: extract.debug,
        selected_clusters: vec![
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity,
                cluster_id: 20,
                cluster_ordinal: 0,
                page_id: 200,
                lod_level: 0,
                state: RenderVirtualGeometryExecutionState::Missing,
            },
            RenderVirtualGeometrySelectedCluster {
                instance_index: Some(0),
                entity,
                cluster_id: 30,
                cluster_ordinal: 1,
                page_id: 300,
                lod_level: 0,
                state: RenderVirtualGeometryExecutionState::Resident,
            },
        ],
        ..RenderVirtualGeometryDebugSnapshot::default()
    };
    let execution_segments = vec![RenderVirtualGeometryExecutionSegment {
        original_index: 0,
        instance_index: Some(0),
        entity,
        stable_instance_key: 0,
        page_id: 300,
        draw_ref_index: 0,
        submission_index: Some(0),
        draw_ref_rank: Some(0),
        cluster_start_ordinal: 1,
        cluster_span_count: 1,
        cluster_total_count: 2,
        submission_slot: Some(0),
        state: RenderVirtualGeometryExecutionState::Resident,
        lineage_depth: 0,
        lod_level: 0,
        frontier_rank: 0,
    }];

    assert_eq!(
        rebuild_selected_clusters_from_execution_segments(
            &snapshot,
            Some(&extract),
            &execution_segments,
        ),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity,
            cluster_id: 30,
            cluster_ordinal: 1,
            page_id: 300,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        "expected the post-render authoritative selection to shrink from the submission-build visibility superset down to the real execution-backed cluster subset"
    );
}

#[test]
fn rebuild_selected_clusters_keeps_same_entity_instances_in_their_stable_key_domain() {
    let entity = 91_u64;
    let first_key = entity << 16;
    let second_key = first_key | 1;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 0,
        clusters: vec![
            cluster(entity, 10, 100, 0, Vec3::ZERO, 9.0),
            cluster(entity, 20, 200, 0, Vec3::ZERO, 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(100, true), page(200, true)],
        page_dependencies: Vec::new(),
        instances: vec![
            RenderVirtualGeometryInstance {
                entity,
                stable_instance_key: first_key,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 0,
                cluster_count: 1,
                page_offset: 0,
                page_count: 1,
                mesh_name: None,
                source_hint: None,
            },
            RenderVirtualGeometryInstance {
                entity,
                stable_instance_key: second_key,
                source_model: None,
                transform: Transform::default(),
                cluster_offset: 1,
                cluster_count: 1,
                page_offset: 1,
                page_count: 1,
                mesh_name: None,
                source_hint: None,
            },
        ],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        debug: extract.debug,
        ..RenderVirtualGeometryDebugSnapshot::default()
    };
    let execution_segments = vec![RenderVirtualGeometryExecutionSegment {
        original_index: 0,
        instance_index: Some(1),
        entity,
        stable_instance_key: second_key,
        page_id: 200,
        draw_ref_index: 0,
        submission_index: Some(0),
        draw_ref_rank: Some(0),
        cluster_start_ordinal: 0,
        cluster_span_count: 1,
        cluster_total_count: 1,
        submission_slot: Some(0),
        state: RenderVirtualGeometryExecutionState::Resident,
        lineage_depth: 0,
        lod_level: 0,
        frontier_rank: 0,
    }];

    assert_eq!(
        rebuild_selected_clusters_from_execution_segments(
            &snapshot,
            Some(&extract),
            &execution_segments,
        ),
        vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(1),
            entity,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }]
    );
}

#[test]
fn resolve_selected_clusters_for_store_prefers_pass_owned_selected_clusters() {
    let entity = 77_u64;
    let extract = RenderVirtualGeometryExtract {
        cluster_budget: 2,
        page_budget: 0,
        clusters: vec![
            cluster(entity, 20, 200, 0, Vec3::ZERO, 9.0),
            cluster(entity, 30, 300, 0, Vec3::new(0.1, 0.0, 0.0), 8.0),
        ],
        hierarchy_nodes: Vec::new(),
        hierarchy_child_ids: Vec::new(),
        pages: vec![page(200, true), page(300, true)],
        page_dependencies: Vec::new(),
        instances: vec![RenderVirtualGeometryInstance {
            entity,
            stable_instance_key: 0,
            source_model: None,
            transform: Transform::default(),
            cluster_offset: 0,
            cluster_count: 2,
            page_offset: 0,
            page_count: 2,
            mesh_name: Some("StoreOutputsExplicitSelectionUnitTestMesh".to_string()),
            source_hint: Some("unit-test".to_string()),
        }],
        debug: RenderVirtualGeometryDebugState::default(),
    };
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        instances: extract.instances.clone(),
        debug: extract.debug,
        selected_clusters: vec![RenderVirtualGeometrySelectedCluster {
            instance_index: Some(0),
            entity,
            cluster_id: 20,
            cluster_ordinal: 0,
            page_id: 200,
            lod_level: 0,
            state: RenderVirtualGeometryExecutionState::Resident,
        }],
        ..RenderVirtualGeometryDebugSnapshot::default()
    };
    let execution_segments = vec![RenderVirtualGeometryExecutionSegment {
        original_index: 0,
        instance_index: Some(0),
        entity,
        stable_instance_key: 0,
        page_id: 300,
        draw_ref_index: 0,
        submission_index: Some(0),
        draw_ref_rank: Some(0),
        cluster_start_ordinal: 1,
        cluster_span_count: 1,
        cluster_total_count: 2,
        submission_slot: Some(0),
        state: RenderVirtualGeometryExecutionState::Resident,
        lineage_depth: 0,
        lod_level: 0,
        frontier_rank: 0,
    }];
    let explicit_selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
        instance_index: Some(0),
        entity,
        cluster_id: 20,
        cluster_ordinal: 0,
        page_id: 200,
        lod_level: 0,
        state: RenderVirtualGeometryExecutionState::Resident,
    }];

    assert_eq!(
        resolve_selected_clusters_for_store(
            &snapshot,
            Some(&extract),
            &execution_segments,
            &explicit_selected_clusters,
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        ),
        explicit_selected_clusters,
        "expected store_last_runtime_outputs to trust the executed selected-cluster pass output directly once that seam produced an authoritative render-path selection instead of re-deriving a second cluster list from execution segments"
    );
}

#[test]
fn resolve_visbuffer64_entries_for_store_prefers_pass_owned_entries() {
    let selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
        instance_index: Some(0),
        entity: 77,
        cluster_id: 30,
        cluster_ordinal: 1,
        page_id: 300,
        lod_level: 0,
        state: RenderVirtualGeometryExecutionState::Resident,
    }];
    let pass_owned_entries = vec![RenderVirtualGeometryVisBuffer64Entry {
        entry_index: 0,
        packed_value: RenderVirtualGeometryVisBuffer64Entry::packed_value_for(
            Some(9),
            20,
            200,
            1,
            RenderVirtualGeometryExecutionState::PendingUpload,
        ),
        instance_index: Some(9),
        entity: 999,
        cluster_id: 20,
        page_id: 200,
        lod_level: 1,
        state: RenderVirtualGeometryExecutionState::PendingUpload,
    }];

    assert_eq!(
        resolve_visbuffer64_entries_for_store(
            &selected_clusters,
            &pass_owned_entries,
            RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        ),
        pass_owned_entries,
        "expected store_last_runtime_outputs to trust the render-path VisBuffer64 pass output directly once that seam exists instead of rebuilding a second logical entry stream from selected clusters"
    );
}

#[test]
fn resolve_visbuffer64_entries_for_store_rebuilds_when_pass_entries_are_missing() {
    let selected_clusters = vec![RenderVirtualGeometrySelectedCluster {
        instance_index: Some(0),
        entity: 77,
        cluster_id: 30,
        cluster_ordinal: 1,
        page_id: 300,
        lod_level: 0,
        state: RenderVirtualGeometryExecutionState::Resident,
    }];

    assert_eq!(
        resolve_visbuffer64_entries_for_store(
            &selected_clusters,
            &[],
            RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        ),
        vec![
            RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(
                0,
                &selected_clusters[0],
            )
        ],
        "expected store_last_runtime_outputs to fall back to execution-backed selected-cluster rebuild when the render path claims execution ownership but the pass-owned VisBuffer64 entry stream is still empty"
    );
}

fn cluster(
    entity: u64,
    cluster_id: u32,
    page_id: u32,
    lod_level: u8,
    bounds_center: Vec3,
    screen_space_error: f32,
) -> RenderVirtualGeometryCluster {
    RenderVirtualGeometryCluster {
        entity,
        cluster_id,
        hierarchy_node_id: None,
        page_id,
        lod_level,
        parent_cluster_id: None,
        bounds_center,
        bounds_radius: 0.5,
        screen_space_error,
    }
}

fn page(page_id: u32, resident: bool) -> RenderVirtualGeometryPage {
    RenderVirtualGeometryPage {
        page_id,
        resident,
        size_bytes: 4096,
    }
}
