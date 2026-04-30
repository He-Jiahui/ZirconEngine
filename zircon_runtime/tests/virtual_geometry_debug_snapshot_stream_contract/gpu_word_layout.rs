use super::prelude::*;

#[test]
fn node_and_cluster_cull_cluster_work_item_roundtrips_through_public_gpu_word_layout() {
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
        .expect("cluster work item words should decode");

    assert_eq!(
        words.len(),
        RenderVirtualGeometryNodeAndClusterCullClusterWorkItem::GPU_WORD_COUNT
    );
    assert_eq!(decoded, work_item);
}

#[test]
fn node_and_cluster_cull_child_work_item_roundtrips_through_public_gpu_word_layout() {
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
        .expect("child work item words should decode");

    assert_eq!(
        words.len(),
        RenderVirtualGeometryNodeAndClusterCullChildWorkItem::GPU_WORD_COUNT
    );
    assert_eq!(decoded, work_item);
}

#[test]
fn node_and_cluster_cull_traversal_record_roundtrips_through_public_gpu_word_layout() {
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
        .expect("traversal record words should decode");

    assert_eq!(
        words.len(),
        RenderVirtualGeometryNodeAndClusterCullTraversalRecord::GPU_WORD_COUNT
    );
    assert_eq!(decoded, record);
}
