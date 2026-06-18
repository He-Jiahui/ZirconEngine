use super::{
    RenderVirtualGeometryClusterSelectionInputSource, RenderVirtualGeometryCullInputSnapshot,
    RenderVirtualGeometryNodeAndClusterCullChildWorkItem,
    RenderVirtualGeometryNodeAndClusterCullClusterWorkItem,
    RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot,
    RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot,
    RenderVirtualGeometryNodeAndClusterCullInstanceSeed,
    RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem,
    RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot,
    RenderVirtualGeometryNodeAndClusterCullTraversalChildSource,
    RenderVirtualGeometryNodeAndClusterCullTraversalOp,
    RenderVirtualGeometryNodeAndClusterCullTraversalRecord,
};
use crate::core::framework::render::RenderVirtualGeometryDebugState;

#[test]
fn cull_input_snapshot_roundtrips_through_gpu_word_layout() {
    let snapshot = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 12,
        page_budget: 7,
        instance_count: 3,
        cluster_count: 42,
        page_count: 9,
        visible_entity_count: 2,
        visible_cluster_count: 17,
        resident_page_count: 5,
        pending_page_request_count: 4,
        available_page_slot_count: 6,
        evictable_page_count: 1,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(10),
            freeze_cull: true,
            visualize_bvh: true,
            visualize_visbuffer: false,
            print_leaf_clusters: true,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };

    let words = snapshot.packed_words();
    let decoded = RenderVirtualGeometryCullInputSnapshot::from_packed_words(&words)
        .expect("expected cull-input snapshot to decode from its stable GPU word layout");

    assert_eq!(
            decoded, snapshot,
            "expected the future NaniteGlobalStateBuffer-compatible word layout to round-trip every authored budget/debug/provenance field without host-side reinterpretation"
        );
}

#[test]
fn node_and_cluster_cull_global_state_roundtrips_through_gpu_word_layout() {
    let snapshot = RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
        cull_input: RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 12,
            page_budget: 7,
            instance_count: 3,
            cluster_count: 42,
            page_count: 9,
            visible_entity_count: 2,
            visible_cluster_count: 17,
            resident_page_count: 5,
            pending_page_request_count: 4,
            available_page_slot_count: 6,
            evictable_page_count: 1,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(10),
                freeze_cull: true,
                visualize_bvh: true,
                visualize_visbuffer: false,
                print_leaf_clusters: true,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        },
        viewport_size: [1920, 1080],
        camera_translation: [1.25, -2.5, 3.75],
        child_split_screen_space_error_threshold: 0.375,
        child_frustum_culling_enabled: true,
        view_proj: [
            [1.0, 2.0, 3.0, 4.0],
            [5.0, 6.0, 7.0, 8.0],
            [9.0, 10.0, 11.0, 12.0],
            [13.0, 14.0, 15.0, 16.0],
        ],
        previous_camera_translation: [-1.25, 2.5, -3.75],
        previous_view_proj: [
            [17.0, 18.0, 19.0, 20.0],
            [21.0, 22.0, 23.0, 24.0],
            [25.0, 26.0, 27.0, 28.0],
            [29.0, 30.0, 31.0, 32.0],
        ],
    };

    let words = snapshot.packed_words();
    let decoded =
        RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot::from_packed_words(&words)
            .expect("expected node-and-cluster-cull global state to decode");

    assert_eq!(
            decoded, snapshot,
            "expected the NodeAndClusterCull global-state word layout to round-trip cull input, viewport, camera origin, and view-projection data without host-side reinterpretation"
        );
}

#[test]
fn node_and_cluster_cull_instance_seed_roundtrips_through_gpu_word_layout() {
    let seed = RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
        instance_index: 3,
        entity: 42,
        cluster_offset: 10,
        cluster_count: 4,
        page_offset: 7,
        page_count: 2,
    };

    let words = seed.packed_words();
    let decoded = RenderVirtualGeometryNodeAndClusterCullInstanceSeed::from_packed_words(&words)
        .expect("expected node-and-cluster-cull instance seed to decode");

    assert_eq!(
            decoded, seed,
            "expected the NodeAndClusterCull instance-seed word layout to round-trip the per-instance root worklist contract without host-side reinterpretation"
        );
}

#[test]
fn node_and_cluster_cull_instance_work_item_roundtrips_through_gpu_word_layout() {
    let work_item = RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_offset: 10,
        cluster_count: 4,
        page_offset: 7,
        page_count: 2,
        cluster_budget: 12,
        page_budget: 7,
        forced_mip: Some(10),
    };

    let words = work_item.packed_words();
    let decoded =
        RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem::from_packed_words(&words)
            .expect("expected node-and-cluster-cull instance work item to decode");

    assert_eq!(
            decoded, work_item,
            "expected the NodeAndClusterCull instance-work-item word layout to round-trip the first compute-stub output contract so the renderer-owned GPU buffer and baseline pass can share one typed per-instance seam"
        );
}

#[test]
fn node_and_cluster_cull_cluster_work_item_roundtrips_through_gpu_word_layout() {
    let work_item = RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_array_index: 10,
        hierarchy_node_id: Some(7),
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };

    let words = work_item.packed_words();
    let decoded = RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::from_packed_words(&words)
        .expect("expected node-and-cluster-cull cluster work item to decode");

    assert_eq!(
            decoded, work_item,
            "expected the public NodeAndClusterCull cluster-work-item word layout to round-trip the per-cluster traversal input contract used by renderer-owned buffers and debug snapshots"
        );
}

#[test]
fn node_and_cluster_cull_child_work_item_roundtrips_through_gpu_word_layout() {
    let work_item = RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
        instance_index: 3,
        entity: 42,
        parent_cluster_array_index: 10,
        parent_hierarchy_node_id: Some(7),
        child_node_id: 70,
        child_table_index: 2,
        traversal_index: 9,
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };

    let words = work_item.packed_words();
    let decoded = RenderVirtualGeometryNodeAndClusterCullChildWorkItem::from_packed_words(&words)
        .expect("expected node-and-cluster-cull child work item to decode");

    assert_eq!(
            decoded, work_item,
            "expected the public NodeAndClusterCull child-work-item word layout to round-trip authored child traversal input without private renderer-side reinterpretation"
        );
}

#[test]
fn node_and_cluster_cull_traversal_record_roundtrips_through_gpu_word_layout() {
    let record = RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
        op: RenderVirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
        child_source:
            RenderVirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 3,
        entity: 42,
        cluster_array_index: 10,
        hierarchy_node_id: Some(7),
        node_cluster_start: 70,
        node_cluster_count: 4,
        child_base: 2,
        child_count: 3,
        traversal_index: 9,
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };

    let words = record.packed_words();
    let decoded = RenderVirtualGeometryNodeAndClusterCullTraversalRecord::from_packed_words(&words)
        .expect("expected node-and-cluster-cull traversal record to decode");

    assert_eq!(
            decoded, record,
            "expected the public NodeAndClusterCull traversal-record word layout to round-trip VisitNode/StoreCluster/EnqueueChild decisions without private renderer-side reinterpretation"
        );
}

#[test]
fn node_and_cluster_cull_launch_worklist_roundtrips_through_gpu_word_layout() {
    let snapshot = RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot {
        global_state: RenderVirtualGeometryNodeAndClusterCullGlobalStateSnapshot {
            cull_input: RenderVirtualGeometryCullInputSnapshot {
                cluster_budget: 12,
                page_budget: 7,
                instance_count: 3,
                cluster_count: 42,
                page_count: 9,
                visible_entity_count: 2,
                visible_cluster_count: 17,
                resident_page_count: 5,
                pending_page_request_count: 4,
                available_page_slot_count: 6,
                evictable_page_count: 1,
                debug: RenderVirtualGeometryDebugState {
                    forced_mip: Some(10),
                    freeze_cull: true,
                    visualize_bvh: true,
                    visualize_visbuffer: false,
                    print_leaf_clusters: true,
                },
                cluster_selection_input_source:
                    RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
            },
            viewport_size: [1920, 1080],
            camera_translation: [1.25, -2.5, 3.75],
            child_split_screen_space_error_threshold: 0.375,
            child_frustum_culling_enabled: true,
            view_proj: [
                [1.0, 2.0, 3.0, 4.0],
                [5.0, 6.0, 7.0, 8.0],
                [9.0, 10.0, 11.0, 12.0],
                [13.0, 14.0, 15.0, 16.0],
            ],
            previous_camera_translation: [-1.25, 2.5, -3.75],
            previous_view_proj: [
                [17.0, 18.0, 19.0, 20.0],
                [21.0, 22.0, 23.0, 24.0],
                [25.0, 26.0, 27.0, 28.0],
                [29.0, 30.0, 31.0, 32.0],
            ],
        },
        dispatch_setup: RenderVirtualGeometryNodeAndClusterCullDispatchSetupSnapshot {
            instance_seed_count: 2,
            cluster_budget: 12,
            page_budget: 7,
            workgroup_size: 64,
            dispatch_group_count: [1, 1, 1],
        },
        instance_seeds: vec![
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: 0,
                entity: 42,
                cluster_offset: 10,
                cluster_count: 4,
                page_offset: 7,
                page_count: 2,
            },
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
                instance_index: 1,
                entity: 99,
                cluster_offset: 20,
                cluster_count: 8,
                page_offset: 11,
                page_count: 3,
            },
        ],
    };

    let words = snapshot.packed_words();
    let decoded =
        RenderVirtualGeometryNodeAndClusterCullLaunchWorklistSnapshot::from_packed_words(&words)
            .expect("expected node-and-cluster-cull launch worklist to decode");

    assert_eq!(
            decoded, snapshot,
            "expected the NodeAndClusterCull launch-worklist word layout to round-trip the combined global state, dispatch setup, and root seeds so the renderer-owned GPU buffer can stay the single baseline compute-stub contract"
        );
}
