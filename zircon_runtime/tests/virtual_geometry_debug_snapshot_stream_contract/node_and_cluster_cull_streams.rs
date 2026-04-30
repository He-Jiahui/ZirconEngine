use super::prelude::*;

#[test]
fn debug_snapshot_exports_node_and_cluster_cull_worklist_packed_word_streams() {
    let instance_seed = RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
        instance_index: 3,
        entity: 42,
        cluster_offset: 10,
        cluster_count: 4,
        page_offset: 2,
        page_count: 3,
    };
    let instance_work_item = RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_offset: 10,
        cluster_count: 4,
        page_offset: 2,
        page_count: 3,
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };
    let cluster_work_item = RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_array_index: 10,
        hierarchy_node_id: Some(7),
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };
    let child_work_item = RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
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
    let traversal_record = RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
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
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        node_and_cluster_cull_source:
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        node_and_cluster_cull_instance_seeds: vec![instance_seed.clone()],
        node_and_cluster_cull_instance_work_items: vec![instance_work_item],
        node_and_cluster_cull_cluster_work_items: vec![cluster_work_item],
        node_and_cluster_cull_child_work_items: vec![child_work_item],
        node_and_cluster_cull_traversal_records: vec![traversal_record],
        node_and_cluster_cull_hierarchy_child_ids: vec![7, 42],
        node_and_cluster_cull_page_request_ids: vec![300, 301],
        ..RenderVirtualGeometryDebugSnapshot::default()
    };

    assert_eq!(
        snapshot.node_and_cluster_cull_instance_seed_words(),
        instance_seed.packed_words().to_vec()
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_instance_work_item_words(),
        instance_work_item.packed_words().to_vec()
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_cluster_work_item_words(),
        cluster_work_item.packed_words().to_vec()
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_child_work_item_words(),
        child_work_item.packed_words().to_vec()
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_traversal_record_words(),
        traversal_record.packed_words().to_vec()
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_hierarchy_child_id_words(),
        vec![7, 42]
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_page_request_id_words(),
        vec![300, 301]
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_word_streams(),
        RenderVirtualGeometryNodeAndClusterCullWordStreams {
            source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
            global_state: None,
            dispatch_setup: None,
            launch_worklist: None,
            instance_seeds: instance_seed.packed_words().to_vec(),
            instance_work_items: instance_work_item.packed_words().to_vec(),
            cluster_work_items: cluster_work_item.packed_words().to_vec(),
            child_work_items: child_work_item.packed_words().to_vec(),
            traversal_records: traversal_record.packed_words().to_vec(),
            hierarchy_child_ids: vec![7, 42],
            page_request_ids: vec![300, 301],
        }
    );
    assert_eq!(
        snapshot.node_and_cluster_cull_decoded_streams(),
        Some(RenderVirtualGeometryNodeAndClusterCullDecodedStreams {
            source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
            global_state: None,
            dispatch_setup: None,
            launch_worklist: None,
            instance_seeds: vec![instance_seed],
            instance_work_items: vec![instance_work_item],
            cluster_work_items: vec![cluster_work_item],
            child_work_items: vec![child_work_item],
            traversal_records: vec![traversal_record],
            hierarchy_child_ids: vec![7, 42],
            page_request_ids: vec![300, 301],
        })
    );
    assert_eq!(
        snapshot.debug_readback_stream_summary(),
        Some(RenderVirtualGeometryDebugSnapshotReadbackStreamSummary {
            node_and_cluster_cull_source:
                RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
            node_and_cluster_cull_global_state_present: false,
            node_and_cluster_cull_dispatch_setup_present: false,
            node_and_cluster_cull_launch_worklist_present: false,
            node_and_cluster_cull_instance_seed_count: 1,
            node_and_cluster_cull_instance_work_item_count: 1,
            node_and_cluster_cull_cluster_work_item_count: 1,
            node_and_cluster_cull_child_work_item_count: 1,
            node_and_cluster_cull_traversal_record_count: 1,
            node_and_cluster_cull_hierarchy_child_id_count: 2,
            node_and_cluster_cull_page_request_id_count: 2,
            selected_clusters_source: RenderVirtualGeometrySelectedClusterSource::Unavailable,
            selected_cluster_count: 0,
            hardware_rasterization_source:
                RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
            hardware_rasterization_record_count: 0,
            visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::Unavailable,
            visbuffer64_clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
            visbuffer64_entry_count: 0,
        })
    );
}

#[test]
fn debug_snapshot_decodes_node_and_cluster_cull_worklist_packed_word_streams() {
    let instance_seed = RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
        instance_index: 3,
        entity: 42,
        cluster_offset: 10,
        cluster_count: 4,
        page_offset: 2,
        page_count: 3,
    };
    let instance_work_item = RenderVirtualGeometryNodeAndClusterCullInstanceWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_offset: 10,
        cluster_count: 4,
        page_offset: 2,
        page_count: 3,
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };
    let cluster_work_item = RenderVirtualGeometryNodeAndClusterCullClusterWorkItem {
        instance_index: 3,
        entity: 42,
        cluster_array_index: 10,
        hierarchy_node_id: Some(7),
        cluster_budget: 12,
        page_budget: 5,
        forced_mip: Some(2),
    };
    let child_work_item = RenderVirtualGeometryNodeAndClusterCullChildWorkItem {
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
    let traversal_record = RenderVirtualGeometryNodeAndClusterCullTraversalRecord {
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

    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_instance_seed_words(
            &instance_seed.packed_words()
        ),
        Some(vec![instance_seed])
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_instance_work_item_words(
            &instance_work_item.packed_words()
        ),
        Some(vec![instance_work_item])
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_cluster_work_item_words(
            &cluster_work_item.packed_words()
        ),
        Some(vec![cluster_work_item])
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_child_work_item_words(
            &child_work_item.packed_words()
        ),
        Some(vec![child_work_item])
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_traversal_record_words(
            &traversal_record.packed_words()
        ),
        Some(vec![traversal_record])
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_child_work_item_words(&[
            1, 2, 3
        ]),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_hierarchy_child_id_words(
            &[7, 42]
        ),
        vec![7, 42]
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_page_request_id_words(&[
            300, 301
        ]),
        vec![300, 301]
    );
    let word_streams = RenderVirtualGeometryNodeAndClusterCullWordStreams {
        source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        global_state: None,
        dispatch_setup: None,
        launch_worklist: None,
        instance_seeds: instance_seed.packed_words().to_vec(),
        instance_work_items: instance_work_item.packed_words().to_vec(),
        cluster_work_items: cluster_work_item.packed_words().to_vec(),
        child_work_items: child_work_item.packed_words().to_vec(),
        traversal_records: traversal_record.packed_words().to_vec(),
        hierarchy_child_ids: vec![7, 42],
        page_request_ids: vec![300, 301],
    };

    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_word_streams(
            &word_streams
        ),
        Some(RenderVirtualGeometryNodeAndClusterCullDecodedStreams {
            source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
            global_state: None,
            dispatch_setup: None,
            launch_worklist: None,
            instance_seeds: vec![instance_seed],
            instance_work_items: vec![instance_work_item],
            cluster_work_items: vec![cluster_work_item],
            child_work_items: vec![child_work_item],
            traversal_records: vec![traversal_record],
            hierarchy_child_ids: vec![7, 42],
            page_request_ids: vec![300, 301],
        })
    );
    let mut malformed_word_streams = word_streams;
    malformed_word_streams.child_work_items.push(99);
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_node_and_cluster_cull_word_streams(
            &malformed_word_streams
        ),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::try_decode_node_and_cluster_cull_word_streams(
            &malformed_word_streams
        ),
        Err(
            RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::ChildWorkItems {
                word_count: malformed_word_streams.child_work_items.len(),
            }
        )
    );
}
