use super::prelude::*;
use super::support::{execute_pass, viewport_frame};

#[test]
fn node_and_cluster_cull_pass_splits_over_budget_child_nodes_into_enqueue_child_records() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..12)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.hierarchy_child_ids = vec![7, 70, 71];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 1,
            child_count: 2,
            cluster_start: 70,
            cluster_count: 3,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );
    assert!(
        output
            .traversal_records()
            .iter()
            .any(|record| *record
                == VirtualGeometryNodeAndClusterCullTraversalRecord {
                    op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                    child_source:
                        VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
                    instance_index: 0,
                    entity: 7,
                    cluster_array_index: 70,
                    hierarchy_node_id: Some(7),
                    node_cluster_start: 70,
                    node_cluster_count: 3,
                    child_base: 1,
                    child_count: 2,
                    traversal_index: 5,
                    cluster_budget: 1,
                    page_budget: 2,
                    forced_mip: Some(6),
                }),
        "expected an over-budget authored child node with its own children to split into a follow-up AuthoredHierarchy EnqueueChild record instead of storing all child clusters immediately"
    );
}

#[test]
fn node_and_cluster_cull_pass_consumes_child_decision_enqueue_records_into_next_traversal_wave() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..112)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            lod_level: match cluster_id {
                100 | 110 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.hierarchy_child_ids = vec![7, 70, 71];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 1,
            child_count: 2,
            cluster_start: 70,
            cluster_count: 3,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 70,
            child_base: 0,
            child_count: 0,
            cluster_start: 100,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 71,
            child_base: 0,
            child_count: 0,
            cluster_start: 110,
            cluster_count: 1,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    let child_visit_node_ids = output
        .traversal_records()
        .iter()
        .filter(|record| record.op == VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode)
        .map(|record| record.hierarchy_node_id)
        .collect::<Vec<_>>();
    assert!(
        child_visit_node_ids.contains(&Some(70)) && child_visit_node_ids.contains(&Some(71)),
        "expected follow-up child traversal wave to visit authored child nodes 70 and 71 after node 7 emitted an EnqueueChild record; got {child_visit_node_ids:?}"
    );

    let stored_child_clusters = output
        .traversal_records()
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && matches!(record.hierarchy_node_id, Some(70 | 71))
        })
        .map(|record| (record.hierarchy_node_id, record.cluster_array_index))
        .collect::<Vec<_>>();
    assert_eq!(
        stored_child_clusters,
        vec![(Some(70), 100), (Some(71), 110)],
        "expected the follow-up child traversal wave to store leaf clusters from authored nodes 70 and 71"
    );
}

#[test]
fn node_and_cluster_cull_pass_feeds_split_child_nodes_into_next_traversal_wave() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..101)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            lod_level: match cluster_id {
                100 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.hierarchy_child_ids = vec![7, 70];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 1,
            child_count: 1,
            cluster_start: 70,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 70,
            child_base: 0,
            child_count: 0,
            cluster_start: 100,
            cluster_count: 1,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 101,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert!(
        output
            .traversal_records()
            .iter()
            .any(|record| record.op == VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode
                && record.hierarchy_node_id == Some(70)
                && record.node_cluster_start == 100
                && record.node_cluster_count == 1),
        "expected a child decision EnqueueChild record for node 7 to feed a second traversal wave that visits authored grandchild node 70"
    );
    assert!(
        output
            .traversal_records()
            .iter()
            .any(|record| record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && record.hierarchy_node_id == Some(70)
                && record.cluster_array_index == 100),
        "expected the second traversal wave to resolve grandchild node 70 and store its mip-matching leaf cluster"
    );
}

#[test]
fn node_and_cluster_cull_pass_splits_child_nodes_when_cluster_sse_exceeds_threshold() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            lod_level: match cluster_id {
                70 => 6,
                _ => 0,
            },
            screen_space_error: match cluster_id {
                70 => 2.0,
                _ => 0.0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.hierarchy_child_ids = vec![7, 70, 71];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 1,
            child_count: 2,
            cluster_start: 70,
            cluster_count: 1,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 2,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert!(
        output
            .traversal_records()
            .iter()
            .any(|record| *record
                == VirtualGeometryNodeAndClusterCullTraversalRecord {
                    op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                    child_source:
                        VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
                    instance_index: 0,
                    entity: 7,
                    cluster_array_index: 70,
                    hierarchy_node_id: Some(7),
                    node_cluster_start: 70,
                    node_cluster_count: 1,
                    child_base: 1,
                    child_count: 2,
                    traversal_index: 5,
                    cluster_budget: 1,
                    page_budget: 2,
                    forced_mip: Some(6),
                }),
        "expected an authored child node whose represented cluster exceeds the baseline SSE threshold to split even when it remains within the cluster budget"
    );
}

#[test]
fn node_and_cluster_cull_child_decision_uses_global_state_sse_threshold() {
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 1,
        page_budget: 2,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 2,
        available_page_slot_count: 1,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: Some(6),
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let mut global_state = build_node_and_cluster_cull_global_state(&frame, &cull_input, None);
    global_state.child_split_screen_space_error_threshold = 3.0;

    let clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            lod_level: match cluster_id {
                70 => 6,
                _ => 0,
            },
            screen_space_error: match cluster_id {
                70 => 2.0,
                _ => 0.0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect::<Vec<_>>();
    let hierarchy_nodes = vec![RenderVirtualGeometryHierarchyNode {
        instance_index: 0,
        node_id: 7,
        child_base: 1,
        child_count: 2,
        cluster_start: 70,
        cluster_count: 1,
    }];
    let child_visit_records = vec![VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 0,
        entity: 7,
        cluster_array_index: 70,
        hierarchy_node_id: Some(7),
        node_cluster_start: 70,
        node_cluster_count: 1,
        child_base: 1,
        child_count: 2,
        traversal_index: 4,
        cluster_budget: 1,
        page_budget: 2,
        forced_mip: Some(6),
    }];

    let decision_records =
        super::super::child_decision::build_node_and_cluster_cull_child_decision_records(
            &child_visit_records,
            &global_state,
            &frame.extract.view.camera,
            &clusters,
            &hierarchy_nodes,
            &[],
            5,
        );

    assert_eq!(
        decision_records
            .iter()
            .map(|record| (record.op, record.cluster_array_index))
            .collect::<Vec<_>>(),
        vec![(VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster, 70)],
        "expected child decision to keep/store a child node when its cluster SSE is below the policy value carried by NodeAndClusterCull global state"
    );
}

#[test]
fn node_and_cluster_cull_child_decision_limits_resident_store_records_to_cluster_budget() {
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 1,
        page_budget: 4,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 3,
        pending_page_request_count: 0,
        available_page_slot_count: 4,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: None,
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let global_state = build_node_and_cluster_cull_global_state(&frame, &cull_input, None);
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); 73];
    clusters[70] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 70,
        page_id: 10,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[71] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 71,
        page_id: 20,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[72] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 72,
        page_id: 30,
        ..RenderVirtualGeometryCluster::default()
    };
    let pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 30,
            resident: true,
            size_bytes: 4096,
        },
    ];
    let hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 3,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 71,
            cluster_count: 1,
        },
    ];
    let child_visit_records = vec![VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 0,
        entity: 7,
        cluster_array_index: 70,
        hierarchy_node_id: Some(7),
        node_cluster_start: 70,
        node_cluster_count: 3,
        child_base: 0,
        child_count: 0,
        traversal_index: 4,
        cluster_budget: 1,
        page_budget: 4,
        forced_mip: None,
    }];

    let output = super::super::child_decision::build_node_and_cluster_cull_child_decision_output(
        &child_visit_records,
        &global_state,
        &frame.extract.view.camera,
        &clusters,
        &hierarchy_nodes,
        &pages,
        5,
    );

    assert_eq!(
        output
            .traversal_records
            .iter()
            .map(|record| (record.op, record.cluster_array_index))
            .collect::<Vec<_>>(),
        vec![(VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster, 70)],
        "expected child decision StoreCluster output to respect the carried cluster budget instead of publishing every resident leaf cluster"
    );
}

#[test]
fn node_and_cluster_cull_pass_stores_only_resident_child_cluster_pages() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            page_id: match cluster_id {
                70 => 10,
                71 => 20,
                _ => 0,
            },
            lod_level: match cluster_id {
                70 | 71 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
    ];
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 1,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    let child_store_records = output
        .traversal_records()
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && (70..=71).contains(&record.cluster_array_index)
        })
        .map(|record| record.cluster_array_index)
        .collect::<Vec<_>>();

    assert_eq!(
        child_store_records,
        vec![70],
        "expected child StoreCluster records to publish only clusters backed by resident pages once the frame carries a page residency table"
    );
}

#[test]
fn node_and_cluster_cull_pass_limits_page_residency_to_cull_input_page_count() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            page_id: match cluster_id {
                70 => 10,
                71 => 20,
                _ => 0,
            },
            lod_level: match cluster_id {
                70 | 71 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: true,
            size_bytes: 4096,
        },
    ];
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 4,
            instance_count: 1,
            cluster_count: 9,
            page_count: 1,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 0,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    let child_store_records = output
        .traversal_records()
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && (70..=71).contains(&record.cluster_array_index)
        })
        .map(|record| record.cluster_array_index)
        .collect::<Vec<_>>();

    assert_eq!(
        child_store_records,
        vec![70],
        "expected page_count to cap the effective page residency table before child StoreCluster decisions consume it"
    );
    assert_eq!(
        output.page_request_ids(),
        &[20],
        "expected the resident page row beyond cull_input.page_count to be ignored and reported as a missing-page request"
    );
}

#[test]
fn node_and_cluster_cull_pass_requests_missing_child_cluster_pages() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            page_id: match cluster_id {
                70 => 10,
                71 => 20,
                _ => 0,
            },
            lod_level: match cluster_id {
                70 | 71 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
    ];
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 0,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    let child_store_records = output
        .traversal_records()
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && record.hierarchy_node_id == Some(7)
        })
        .map(|record| record.cluster_array_index)
        .collect::<Vec<_>>();

    assert_eq!(
        child_store_records,
        vec![70],
        "expected child decision to store only resident child cluster pages at this pass boundary"
    );
    assert_eq!(
        output.page_request_ids(),
        &[20],
        "expected child decision to request the missing page for a visible mip-matching child cluster instead of silently dropping it"
    );
    assert_eq!(output.page_request_count(), 1);
    assert!(
        output.page_request_buffer().is_some(),
        "expected NodeAndClusterCull to materialize a pass-local page request buffer when requests exist"
    );
    assert_eq!(output.child_work_item_count(), 1);
    assert!(
        output.child_work_item_buffer().is_some(),
        "expected NodeAndClusterCull to materialize child work items for authored hierarchy expansion"
    );
}

#[test]
fn node_and_cluster_cull_child_decision_stores_resident_parent_when_child_page_is_missing() {
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 1,
        page_budget: 2,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 0,
        available_page_slot_count: 1,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: None,
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let global_state = build_node_and_cluster_cull_global_state(&frame, &cull_input, None);
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); 72];
    clusters[70] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 10,
        page_id: 10,
        lod_level: 5,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[71] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 20,
        page_id: 20,
        lod_level: 6,
        parent_cluster_id: Some(10),
        ..RenderVirtualGeometryCluster::default()
    };
    let pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
    ];
    let hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 3,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 71,
            cluster_count: 1,
        },
    ];
    let child_visit_records = vec![VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 0,
        entity: 7,
        cluster_array_index: 71,
        hierarchy_node_id: Some(7),
        node_cluster_start: 71,
        node_cluster_count: 1,
        child_base: 0,
        child_count: 0,
        traversal_index: 4,
        cluster_budget: 1,
        page_budget: 2,
        forced_mip: None,
    }];

    let output = super::super::child_decision::build_node_and_cluster_cull_child_decision_output(
        &child_visit_records,
        &global_state,
        &frame.extract.view.camera,
        &clusters,
        &hierarchy_nodes,
        &pages,
        5,
    );

    assert_eq!(
        output
            .traversal_records
            .iter()
            .map(|record| (record.op, record.cluster_array_index))
            .collect::<Vec<_>>(),
        vec![(VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster, 70)],
        "expected child decision to keep drawing the nearest resident parent cluster while the requested child page is missing"
    );
    assert_eq!(
        output.requested_page_ids,
        vec![20],
        "expected child decision to still request the missing child cluster page while using parent fallback"
    );
}

#[test]
fn node_and_cluster_cull_pass_keeps_page_requests_when_upload_slots_are_full() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            page_id: match cluster_id {
                70 => 20,
                71 => 30,
                _ => 0,
            },
            lod_level: match cluster_id {
                70 | 71 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.pages = vec![
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 30,
            resident: false,
            size_bytes: 4096,
        },
    ];
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 4,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 0,
            available_page_slot_count: 0,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(
        output.page_request_ids(),
        &[20, 30],
        "expected NodeAndClusterCull to preserve visible missing-page feedback even when runtime upload slots are full"
    );
    assert_eq!(output.page_request_count(), 2);
}

#[test]
fn node_and_cluster_cull_child_decision_stores_shared_resident_parent_fallback_once() {
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 4,
        page_budget: 4,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 0,
        available_page_slot_count: 4,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: None,
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let global_state = build_node_and_cluster_cull_global_state(&frame, &cull_input, None);
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); 73];
    clusters[70] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 10,
        page_id: 10,
        lod_level: 5,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[71] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 20,
        page_id: 20,
        lod_level: 6,
        parent_cluster_id: Some(10),
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[72] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 30,
        page_id: 30,
        lod_level: 6,
        parent_cluster_id: Some(10),
        ..RenderVirtualGeometryCluster::default()
    };
    let pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 30,
            resident: false,
            size_bytes: 4096,
        },
    ];
    let child_visit_records = vec![VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 0,
        entity: 7,
        cluster_array_index: 71,
        hierarchy_node_id: Some(7),
        node_cluster_start: 71,
        node_cluster_count: 2,
        child_base: 0,
        child_count: 0,
        traversal_index: 4,
        cluster_budget: 4,
        page_budget: 4,
        forced_mip: None,
    }];

    let output = super::super::child_decision::build_node_and_cluster_cull_child_decision_output(
        &child_visit_records,
        &global_state,
        &frame.extract.view.camera,
        &clusters,
        &[],
        &pages,
        5,
    );

    assert_eq!(
        output
            .traversal_records
            .iter()
            .map(|record| (record.op, record.cluster_array_index))
            .collect::<Vec<_>>(),
        vec![(VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster, 70)],
        "expected two missing child clusters that share a resident parent to store that fallback parent only once"
    );
    assert_eq!(
        output.requested_page_ids,
        vec![20, 30],
        "expected both missing child pages to remain visible to the runtime request path"
    );
}

#[test]
fn node_and_cluster_cull_child_decision_labels_parent_fallback_with_parent_hierarchy_id() {
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 2,
        page_budget: 2,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 1,
        pending_page_request_count: 0,
        available_page_slot_count: 2,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: None,
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let global_state = build_node_and_cluster_cull_global_state(&frame, &cull_input, None);
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); 72];
    clusters[70] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 10,
        hierarchy_node_id: Some(3),
        page_id: 10,
        lod_level: 5,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[71] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 20,
        hierarchy_node_id: Some(7),
        page_id: 20,
        lod_level: 6,
        parent_cluster_id: Some(10),
        ..RenderVirtualGeometryCluster::default()
    };
    let pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
    ];
    let hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 3,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 71,
            cluster_count: 1,
        },
    ];
    let child_visit_records = vec![VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 0,
        entity: 7,
        cluster_array_index: 71,
        hierarchy_node_id: Some(7),
        node_cluster_start: 71,
        node_cluster_count: 1,
        child_base: 0,
        child_count: 0,
        traversal_index: 4,
        cluster_budget: 2,
        page_budget: 2,
        forced_mip: None,
    }];

    let output = super::super::child_decision::build_node_and_cluster_cull_child_decision_output(
        &child_visit_records,
        &global_state,
        &frame.extract.view.camera,
        &clusters,
        &hierarchy_nodes,
        &pages,
        5,
    );

    assert_eq!(
        output
            .traversal_records
            .iter()
            .map(|record| {
                (
                    record.cluster_array_index,
                    record.hierarchy_node_id,
                    record.node_cluster_start,
                    record.node_cluster_count,
                )
            })
            .collect::<Vec<_>>(),
        vec![(70, Some(3), 70, 1)],
        "expected missing-page parent fallback StoreCluster records to keep the fallback parent's hierarchy node range instead of the child visit range"
    );
}

#[test]
fn node_and_cluster_cull_child_decision_limits_distinct_parent_fallbacks_by_cluster_budget() {
    let frame = viewport_frame(UVec2::new(96, 64));
    let cull_input = RenderVirtualGeometryCullInputSnapshot {
        cluster_budget: 1,
        page_budget: 4,
        instance_count: 1,
        cluster_count: 9,
        page_count: 5,
        visible_entity_count: 1,
        visible_cluster_count: 7,
        resident_page_count: 2,
        pending_page_request_count: 0,
        available_page_slot_count: 4,
        evictable_page_count: 0,
        debug: RenderVirtualGeometryDebugState {
            forced_mip: None,
            freeze_cull: true,
            visualize_bvh: false,
            visualize_visbuffer: true,
            print_leaf_clusters: false,
        },
        cluster_selection_input_source:
            RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
    };
    let global_state = build_node_and_cluster_cull_global_state(&frame, &cull_input, None);
    let mut clusters = vec![RenderVirtualGeometryCluster::default(); 74];
    clusters[70] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 10,
        hierarchy_node_id: Some(3),
        page_id: 10,
        lod_level: 5,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[71] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 20,
        hierarchy_node_id: Some(7),
        page_id: 20,
        lod_level: 6,
        parent_cluster_id: Some(10),
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[72] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 30,
        hierarchy_node_id: Some(4),
        page_id: 30,
        lod_level: 5,
        ..RenderVirtualGeometryCluster::default()
    };
    clusters[73] = RenderVirtualGeometryCluster {
        entity: 7,
        cluster_id: 40,
        hierarchy_node_id: Some(7),
        page_id: 40,
        lod_level: 6,
        parent_cluster_id: Some(30),
        ..RenderVirtualGeometryCluster::default()
    };
    let pages = vec![
        RenderVirtualGeometryPage {
            page_id: 10,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 30,
            resident: true,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 40,
            resident: false,
            size_bytes: 4096,
        },
    ];
    let hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 3,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 4,
            child_base: 0,
            child_count: 0,
            cluster_start: 72,
            cluster_count: 1,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 71,
            cluster_count: 3,
        },
    ];
    let child_visit_records = vec![VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
        instance_index: 0,
        entity: 7,
        cluster_array_index: 71,
        hierarchy_node_id: Some(7),
        node_cluster_start: 71,
        node_cluster_count: 3,
        child_base: 0,
        child_count: 0,
        traversal_index: 4,
        cluster_budget: 1,
        page_budget: 4,
        forced_mip: None,
    }];

    let output = super::super::child_decision::build_node_and_cluster_cull_child_decision_output(
        &child_visit_records,
        &global_state,
        &frame.extract.view.camera,
        &clusters,
        &hierarchy_nodes,
        &pages,
        5,
    );

    assert_eq!(
        output
            .traversal_records
            .iter()
            .map(|record| record.cluster_array_index)
            .collect::<Vec<_>>(),
        vec![70],
        "expected parent fallback StoreCluster records emitted from one child visit to respect that visit's cluster_budget even when the fallbacks carry distinct parent hierarchy metadata"
    );
    assert_eq!(
        output.requested_page_ids,
        vec![20, 40],
        "expected request feedback to keep both missing pages even when fallback StoreCluster output is budget-clamped"
    );
}

#[test]
fn node_and_cluster_cull_pass_limits_page_requests_to_remaining_page_budget() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            page_id: match cluster_id {
                70 => 20,
                71 => 30,
                _ => 0,
            },
            lod_level: match cluster_id {
                70 | 71 => 6,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.pages = vec![
        RenderVirtualGeometryPage {
            page_id: 20,
            resident: false,
            size_bytes: 4096,
        },
        RenderVirtualGeometryPage {
            page_id: 30,
            resident: false,
            size_bytes: 4096,
        },
    ];
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 4,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 3,
            available_page_slot_count: 4,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    assert_eq!(
        output.page_request_ids(),
        &[20],
        "expected NodeAndClusterCull to leave room for already-pending page requests when applying page_budget to newly emitted requests"
    );
    assert_eq!(output.page_request_count(), 1);
}

#[test]
fn node_and_cluster_cull_pass_stores_only_forced_mip_child_clusters() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            lod_level: match cluster_id {
                70 => 6,
                71 => 7,
                _ => 0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 1,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    let child_store_records = output
        .traversal_records()
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && record.hierarchy_node_id == Some(7)
        })
        .map(|record| record.cluster_array_index)
        .collect::<Vec<_>>();

    assert_eq!(
        child_store_records,
        vec![70],
        "expected forced_mip to keep only child StoreCluster rows whose render cluster lod_level matches the requested mip"
    );
}

#[test]
fn node_and_cluster_cull_pass_stores_only_frustum_visible_child_clusters() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let mut frame = viewport_frame(UVec2::new(96, 64));
    let virtual_geometry = frame
        .extract
        .geometry
        .virtual_geometry
        .as_mut()
        .expect("expected VG extract");
    virtual_geometry.clusters = (0..72)
        .map(|cluster_id| RenderVirtualGeometryCluster {
            cluster_id,
            hierarchy_node_id: (cluster_id == 11).then_some(911),
            lod_level: match cluster_id {
                70 | 71 => 6,
                _ => 0,
            },
            bounds_center: match cluster_id {
                70 => Vec3::new(3.0, 4.0, 0.0),
                71 => Vec3::new(1000.0, 4.0, 0.0),
                _ => Vec3::ZERO,
            },
            bounds_radius: match cluster_id {
                70 | 71 => 0.5,
                _ => 0.0,
            },
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();
    virtual_geometry.hierarchy_child_ids = vec![7];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 1,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 2,
        },
    ];

    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 1,
            page_budget: 2,
            instance_count: 1,
            cluster_count: 9,
            page_count: 5,
            visible_entity_count: 1,
            visible_cluster_count: 7,
            resident_page_count: 1,
            pending_page_request_count: 1,
            available_page_slot_count: 1,
            evictable_page_count: 0,
            debug: RenderVirtualGeometryDebugState {
                forced_mip: Some(6),
                freeze_cull: true,
                visualize_bvh: false,
                visualize_visbuffer: true,
                print_leaf_clusters: false,
            },
            cluster_selection_input_source:
                RenderVirtualGeometryClusterSelectionInputSource::PrepareOnDemand,
        }),
    );

    let child_store_records = output
        .traversal_records()
        .iter()
        .filter(|record| {
            record.op == VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster
                && record.hierarchy_node_id == Some(7)
        })
        .map(|record| record.cluster_array_index)
        .collect::<Vec<_>>();

    assert_eq!(
        child_store_records,
        vec![70],
        "expected child StoreCluster records to skip clusters whose bounds sphere is outside the active camera frustum"
    );
}
