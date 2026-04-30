use super::prelude::*;
use super::support::{execute_pass, viewport_frame};

#[test]
fn node_and_cluster_cull_pass_carries_hierarchy_node_id_from_render_clusters_to_traversal_records()
{
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
            hierarchy_node_id: (cluster_id >= 10).then_some(900 + cluster_id),
            ..RenderVirtualGeometryCluster::default()
        })
        .collect();

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
        output.cluster_work_items()[0].hierarchy_node_id,
        Some(910),
        "expected the cluster work-item seam to preserve the asset hierarchy node id from the render extract cluster row instead of only carrying the baseline array index"
    );
    assert_eq!(
        output.traversal_records()[3],
        VirtualGeometryNodeAndClusterCullTraversalRecord {
            op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
            child_source:
                VirtualGeometryNodeAndClusterCullTraversalChildSource::FixedFanout,
            instance_index: 0,
            entity: 7,
            cluster_array_index: 11,
            hierarchy_node_id: Some(911),
            node_cluster_start: 0,
            node_cluster_count: 0,
            child_base: 44,
            child_count: 4,
            traversal_index: 3,
            cluster_budget: 1,
            page_budget: 2,
            forced_mip: Some(6),
        },
        "expected traversal records to keep the authored hierarchy node id beside the temporary fixed-fanout child range so the next slice can swap the fanout source without guessing from cluster array position"
    );
}

#[test]
fn node_and_cluster_cull_pass_uses_authored_hierarchy_child_range_for_enqueue_child() {
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
    virtual_geometry.hierarchy_nodes = vec![RenderVirtualGeometryHierarchyNode {
        instance_index: 0,
        node_id: 911,
        child_base: 120,
        child_count: 3,
        cluster_start: 50,
        cluster_count: 2,
    }];

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
        output.traversal_records()[3],
        VirtualGeometryNodeAndClusterCullTraversalRecord {
            op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
            child_source:
                VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
            instance_index: 0,
            entity: 7,
            cluster_array_index: 11,
            hierarchy_node_id: Some(911),
            node_cluster_start: 0,
            node_cluster_count: 0,
            child_base: 120,
            child_count: 3,
            traversal_index: 3,
            cluster_budget: 1,
            page_budget: 2,
            forced_mip: Some(6),
        },
        "expected authored hierarchy child ranges to replace the temporary fixed fanout whenever the cull seam can resolve the work item's hierarchy node id"
    );
}

#[test]
fn node_and_cluster_cull_pass_points_authored_children_at_child_id_table() {
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
    virtual_geometry.hierarchy_child_ids = vec![7, 42];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 2,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 3,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 42,
            child_base: 0,
            child_count: 0,
            cluster_start: 90,
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

    assert_eq!(output.hierarchy_child_ids(), &[7, 42]);
    assert!(
        output.hierarchy_child_id_buffer().is_some(),
        "expected authored child ids to materialize as a GPU-visible table for the later traversal consumer"
    );
    assert_eq!(
        output.traversal_records()[3],
        VirtualGeometryNodeAndClusterCullTraversalRecord {
            op: VirtualGeometryNodeAndClusterCullTraversalOp::EnqueueChild,
            child_source:
                VirtualGeometryNodeAndClusterCullTraversalChildSource::AuthoredHierarchy,
            instance_index: 0,
            entity: 7,
            cluster_array_index: 11,
            hierarchy_node_id: Some(911),
            node_cluster_start: 0,
            node_cluster_count: 0,
            child_base: 0,
            child_count: 2,
            traversal_index: 3,
            cluster_budget: 1,
            page_budget: 2,
            forced_mip: Some(6),
        },
        "expected authored child_base/count to address the flat child-id table so non-contiguous children like [7, 42] are not misread as a contiguous node-id range"
    );
}

#[test]
fn node_and_cluster_cull_pass_expands_authored_child_ids_into_child_work_items() {
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
    virtual_geometry.hierarchy_child_ids = vec![7, 42];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 2,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 3,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 42,
            child_base: 0,
            child_count: 0,
            cluster_start: 90,
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

    assert_eq!(output.child_work_item_count(), 2);
    assert_eq!(
        output.child_work_items(),
        &[
            VirtualGeometryNodeAndClusterCullChildWorkItem {
                instance_index: 0,
                entity: 7,
                parent_cluster_array_index: 11,
                parent_hierarchy_node_id: Some(911),
                child_node_id: 7,
                child_table_index: 0,
                traversal_index: 3,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullChildWorkItem {
                instance_index: 0,
                entity: 7,
                parent_cluster_array_index: 11,
                parent_hierarchy_node_id: Some(911),
                child_node_id: 42,
                child_table_index: 1,
                traversal_index: 3,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
        ],
        "expected authored EnqueueChild traversal ranges to expand through the flat child-id table into persistent child-node work rows instead of staying as a marker only"
    );
    assert!(
        output.child_work_item_buffer().is_some(),
        "expected child-node work rows to materialize as a GPU-visible buffer for the next persistent traversal slice"
    );
}

#[test]
fn node_and_cluster_cull_child_work_item_roundtrips_through_gpu_word_layout() {
    let work_item = VirtualGeometryNodeAndClusterCullChildWorkItem {
        instance_index: 3,
        entity: 42,
        parent_cluster_array_index: 17,
        parent_hierarchy_node_id: Some(321),
        child_node_id: 654,
        child_table_index: 8,
        traversal_index: 5,
        cluster_budget: 12,
        page_budget: 7,
        forced_mip: Some(9),
    };

    let words = work_item.packed_words();
    let decoded = VirtualGeometryNodeAndClusterCullChildWorkItem::from_packed_words(&words)
        .expect("expected node-and-cluster-cull child work item to decode");

    assert_eq!(
        decoded, work_item,
        "expected the child-node work-item layout to round-trip so renderer-owned buffers and the later persistent traversal kernel share one typed contract"
    );
}

#[test]
fn node_and_cluster_cull_pass_consumes_child_work_items_into_child_visit_records() {
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
    virtual_geometry.hierarchy_child_ids = vec![7, 42];
    virtual_geometry.hierarchy_nodes = vec![
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 911,
            child_base: 0,
            child_count: 2,
            cluster_start: 50,
            cluster_count: 2,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 7,
            child_base: 0,
            child_count: 0,
            cluster_start: 70,
            cluster_count: 3,
        },
        RenderVirtualGeometryHierarchyNode {
            instance_index: 0,
            node_id: 42,
            child_base: 0,
            child_count: 0,
            cluster_start: 90,
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

    assert_eq!(
        &output.traversal_records()[4..],
        &[
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::VisitNode,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 11,
                hierarchy_node_id: Some(7),
                node_cluster_start: 70,
                node_cluster_count: 3,
                child_base: 0,
                child_count: 0,
                traversal_index: 4,
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
                hierarchy_node_id: Some(42),
                node_cluster_start: 90,
                node_cluster_count: 1,
                child_base: 0,
                child_count: 0,
                traversal_index: 5,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 70,
                hierarchy_node_id: Some(7),
                node_cluster_start: 70,
                node_cluster_count: 3,
                child_base: 0,
                child_count: 0,
                traversal_index: 6,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
            VirtualGeometryNodeAndClusterCullTraversalRecord {
                op: VirtualGeometryNodeAndClusterCullTraversalOp::StoreCluster,
                child_source: VirtualGeometryNodeAndClusterCullTraversalChildSource::None,
                instance_index: 0,
                entity: 7,
                cluster_array_index: 90,
                hierarchy_node_id: Some(42),
                node_cluster_start: 90,
                node_cluster_count: 1,
                child_base: 0,
                child_count: 0,
                traversal_index: 7,
                cluster_budget: 1,
                page_budget: 2,
                forced_mip: Some(6),
            },
        ],
        "expected persistent authored child-node work rows to feed follow-up VisitNode traversal records, while StoreCluster child outputs stay bounded by each child node budget"
    );
}
