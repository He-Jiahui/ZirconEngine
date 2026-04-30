use super::prelude::*;
use super::support::{execute_pass, viewport_frame};

#[test]
fn node_and_cluster_cull_cluster_work_item_roundtrips_through_gpu_word_layout() {
    let work_item = VirtualGeometryNodeAndClusterCullClusterWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_array_index: 17,
        hierarchy_node_id: Some(321),
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(9),
    };

    let words = work_item.packed_words();
    let decoded = VirtualGeometryNodeAndClusterCullClusterWorkItem::from_packed_words(&words)
        .expect("expected node-and-cluster-cull cluster work item to decode");

    assert_eq!(
        decoded, work_item,
        "expected the internal NodeAndClusterCull cluster-work-item layout to round-trip the first per-cluster queue seam so renderer-owned buffers and future traversal kernels can share one typed contract"
    );
}

#[test]
fn node_and_cluster_cull_pass_publishes_visit_node_and_store_cluster_records_from_cluster_work_items(
) {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
    let output = execute_pass(
        &backend,
        true,
        &frame,
        Some(&RenderVirtualGeometryCullInputSnapshot {
            cluster_budget: 4,
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

    assert_eq!(
        output.traversal_records(),
        &[
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 10,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 0,
                cluster_budget: 4,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 10,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 1,
                cluster_budget: 4,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 11,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 2,
                cluster_budget: 4,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 11,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 3,
                cluster_budget: 4,
                page_budget: 2,
                forced_mip: Some(6),
            },
        ],
        "expected the first cull-side consumer below cluster work items to publish explicit VisitNode and StoreCluster traversal records before later GPU hierarchy logic replaces the fixed-fanout expansion"
    );
}

#[test]
fn node_and_cluster_cull_pass_gates_store_cluster_records_by_cluster_budget() {
    let backend = RenderBackend::new_offscreen().expect("backend should initialize");
    let frame = viewport_frame(UVec2::new(96, 64));
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

    assert_eq!(
        output.traversal_records(),
        &[
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 10,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 0,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 10,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 1,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 11,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 0,
                child_count: 0,
                traversal_index: 2,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
                child_source:
                    VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 11,
                hierarchy_node_id: None,
                node_cluster_start: 0,
                node_cluster_count: 0,
                child_base: 44,
                child_count: 4,
                traversal_index: 3,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
        ],
        "expected the first traversal-side consumer to preserve VisitNode audit rows for over-budget candidates and publish an explicit child-work marker instead of pretending the candidate was stored"
    );
    assert_eq!(output.traversal_record_count(), 4);
}

#[test]
fn node_and_cluster_cull_enqueue_child_record_roundtrips_through_gpu_word_layout() {
    let record = VirtualGeometryNodeAndClusterCullTraversalRecord {
        op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
        child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
        instance_index: 3,
        entity: 42,
        cluster_array_index: 17,
        hierarchy_node_id: Some(321),
        node_cluster_start: 0,
        node_cluster_count: 0,
        child_base: 68,
        child_count: 4,
        traversal_index: 5,
        cluster_budget: 1,
        page_budget: 2,
        forced_mip: Some(9),
    };

    let words = record.packed_words();
    let decoded = VirtualGeometryNodeAndClusterCullTraversalRecord::from_packed_words(&words)
        .expect("expected enqueue-child traversal record to decode");

    assert_eq!(
        decoded, record,
        "expected the EnqueueChild traversal op to survive the stable GPU word layout so renderer-owned readback can distinguish child work from stored cluster output"
    );
}
