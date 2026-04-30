use super::prelude::*;

#[test]
fn debug_snapshot_exports_full_chain_readback_stream_bundle() {
    let instance_seed = RenderVirtualGeometryNodeAndClusterCullInstanceSeed {
        instance_index: 5,
        entity: 77,
        cluster_offset: 12,
        cluster_count: 3,
        page_offset: 2,
        page_count: 1,
    };
    let selected_cluster = RenderVirtualGeometrySelectedCluster {
        instance_index: Some(5),
        entity: 77,
        cluster_id: 12,
        cluster_ordinal: 2,
        page_id: 2,
        lod_level: 1,
        state: RenderVirtualGeometryExecutionState::Resident,
    };
    let visbuffer_entry =
        RenderVirtualGeometryVisBuffer64Entry::from_selected_cluster(3, &selected_cluster);
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        node_and_cluster_cull_source:
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        node_and_cluster_cull_instance_seeds: vec![instance_seed],
        selected_clusters_source:
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        selected_clusters: vec![selected_cluster.clone()],
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        visbuffer64_clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        visbuffer64_entries: vec![visbuffer_entry.clone()],
        ..RenderVirtualGeometryDebugSnapshot::default()
    };

    assert_eq!(
        snapshot.debug_readback_streams(),
        RenderVirtualGeometryDebugSnapshotReadbackStreams {
            node_and_cluster_cull: RenderVirtualGeometryNodeAndClusterCullWordStreams {
                source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
                global_state: None,
                dispatch_setup: None,
                launch_worklist: None,
                instance_seeds: instance_seed.packed_words().to_vec(),
                instance_work_items: Vec::new(),
                cluster_work_items: Vec::new(),
                child_work_items: Vec::new(),
                traversal_records: Vec::new(),
                hierarchy_child_ids: Vec::new(),
                page_request_ids: Vec::new(),
            },
            render_path: RenderVirtualGeometryRenderPathWordStreams {
                selected_clusters_source:
                    RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
                hardware_rasterization_source:
                    RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
                selected_clusters: selected_cluster.packed_words().to_vec(),
                hardware_rasterization_records: Vec::new(),
            },
            visbuffer64: RenderVirtualGeometryVisBuffer64ReadbackStream {
                source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
                clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
                entry_indices: vec![3],
                packed_values: vec![visbuffer_entry.packed_value],
            },
        }
    );
    let readback_streams = snapshot.debug_readback_streams();
    assert!(readback_streams.has_payload());
    assert!(readback_streams.node_and_cluster_cull.has_payload());
    assert!(readback_streams.render_path.has_payload());
    assert!(readback_streams.visbuffer64.has_payload());
    let node_u32_words = RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT;
    let render_path_u32_words = RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT;
    let visbuffer64_u32_words = 5;
    let visbuffer64_payload_u32_words = 3;
    let total_u32_words = node_u32_words + render_path_u32_words + visbuffer64_u32_words;
    let payload_u32_words = node_u32_words + render_path_u32_words + visbuffer64_payload_u32_words;
    assert_eq!(
        readback_streams
            .node_and_cluster_cull
            .payload_u32_word_count(),
        node_u32_words
    );
    assert_eq!(
        readback_streams.render_path.payload_u32_word_count(),
        render_path_u32_words
    );
    assert_eq!(
        readback_streams.visbuffer64.payload_u32_word_count(),
        visbuffer64_payload_u32_words
    );
    assert_eq!(readback_streams.payload_u32_word_count(), payload_u32_words);
    assert_eq!(readback_streams.payload_byte_count(), payload_u32_words * 4);
    assert_eq!(
        snapshot.debug_readback_stream_footprint(),
        RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
            node_and_cluster_cull_u32_word_count: node_u32_words,
            render_path_u32_word_count: render_path_u32_words,
            visbuffer64_u32_word_count: visbuffer64_u32_words,
            total_u32_word_count: total_u32_words,
            total_byte_count: total_u32_words * 4,
        }
    );
    assert_eq!(
        snapshot
            .debug_readback_stream_footprint()
            .payload_u32_word_count(),
        payload_u32_words
    );
    assert_eq!(
        snapshot
            .debug_readback_stream_footprint()
            .payload_byte_count(),
        payload_u32_words * 4
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::summarize_debug_readback_stream_footprint(
            &readback_streams
        ),
        snapshot.debug_readback_stream_footprint()
    );
    assert_eq!(
        snapshot.debug_decoded_streams(),
        Some(RenderVirtualGeometryDebugSnapshotDecodedStreams {
            node_and_cluster_cull: RenderVirtualGeometryNodeAndClusterCullDecodedStreams {
                source: RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
                global_state: None,
                dispatch_setup: None,
                launch_worklist: None,
                instance_seeds: vec![instance_seed],
                instance_work_items: Vec::new(),
                cluster_work_items: Vec::new(),
                child_work_items: Vec::new(),
                traversal_records: Vec::new(),
                hierarchy_child_ids: Vec::new(),
                page_request_ids: Vec::new(),
            },
            render_path: RenderVirtualGeometryRenderPathDecodedStreams {
                selected_clusters_source:
                    RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
                hardware_rasterization_source:
                    RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
                selected_clusters: vec![selected_cluster],
                hardware_rasterization_records: Vec::new(),
            },
            visbuffer64: RenderVirtualGeometryVisBuffer64DecodedStream {
                source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
                clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
                entries: vec![RenderVirtualGeometryVisBuffer64Entry {
                    entry_index: 3,
                    packed_value: visbuffer_entry.packed_value,
                    instance_index: Some(5),
                    entity: 0,
                    cluster_id: 12,
                    page_id: 2,
                    lod_level: 1,
                    state: RenderVirtualGeometryExecutionState::Resident,
                }],
            },
        })
    );
    assert_eq!(
        snapshot.try_debug_decoded_streams(),
        Ok(snapshot
            .debug_decoded_streams()
            .expect("snapshot decoded streams should remain valid"))
    );
    let expected_summary = RenderVirtualGeometryDebugSnapshotReadbackStreamSummary {
        node_and_cluster_cull_source:
            RenderVirtualGeometryNodeAndClusterCullSource::RenderPathCullInput,
        node_and_cluster_cull_global_state_present: false,
        node_and_cluster_cull_dispatch_setup_present: false,
        node_and_cluster_cull_launch_worklist_present: false,
        node_and_cluster_cull_instance_seed_count: 1,
        node_and_cluster_cull_instance_work_item_count: 0,
        node_and_cluster_cull_cluster_work_item_count: 0,
        node_and_cluster_cull_child_work_item_count: 0,
        node_and_cluster_cull_traversal_record_count: 0,
        node_and_cluster_cull_hierarchy_child_id_count: 0,
        node_and_cluster_cull_page_request_id_count: 0,
        selected_clusters_source:
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        selected_cluster_count: 1,
        hardware_rasterization_source:
            RenderVirtualGeometryHardwareRasterizationSource::Unavailable,
        hardware_rasterization_record_count: 0,
        visbuffer64_source: RenderVirtualGeometryVisBuffer64Source::RenderPathExecutionSelections,
        visbuffer64_clear_value: RenderVirtualGeometryVisBuffer64Entry::CLEAR_VALUE,
        visbuffer64_entry_count: 1,
    };
    assert_eq!(
        snapshot.debug_readback_stream_summary(),
        Some(expected_summary.clone())
    );
    assert_eq!(
        snapshot.debug_readback_stream_report(),
        RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
            footprint: snapshot.debug_readback_stream_footprint(),
            summary: Some(expected_summary.clone()),
            decode_error: None,
        }
    );
    assert!(expected_summary.has_payload());
    assert!(snapshot
        .debug_readback_stream_report()
        .has_decoded_payload());
    assert_eq!(
        snapshot
            .debug_readback_stream_report()
            .payload_u32_word_count(),
        payload_u32_words
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::report_debug_readback_streams(&readback_streams),
        RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
            footprint: snapshot.debug_readback_stream_footprint(),
            summary: Some(expected_summary),
            decode_error: None,
        }
    );
    assert_eq!(
        snapshot.try_debug_readback_stream_summary(),
        Ok(snapshot
            .debug_readback_stream_summary()
            .expect("snapshot summary should remain valid"))
    );
    let invalid_state_value = 3_u64 << 62;
    let mut malformed_streams = snapshot.debug_readback_streams();
    malformed_streams.visbuffer64.packed_values[0] = invalid_state_value;
    let expected_decode_error =
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::VisBuffer64(
            RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError::InvalidPackedState {
                entry_index: 3,
                packed_value: invalid_state_value,
            },
        );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::try_decode_debug_readback_streams(&malformed_streams),
        Err(expected_decode_error)
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::try_summarize_debug_readback_streams(
            &malformed_streams
        ),
        Err(expected_decode_error)
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::report_debug_readback_streams(&malformed_streams),
        RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
            footprint:
                RenderVirtualGeometryDebugSnapshot::summarize_debug_readback_stream_footprint(
                    &malformed_streams
                ),
            summary: None,
            decode_error: Some(expected_decode_error),
        }
    );
    let mut malformed_footprint_streams = snapshot.debug_readback_streams();
    malformed_footprint_streams
        .render_path
        .selected_clusters
        .push(99);
    let malformed_total_u32_words = total_u32_words + 1;
    let malformed_footprint = RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
        node_and_cluster_cull_u32_word_count: node_u32_words,
        render_path_u32_word_count: render_path_u32_words + 1,
        visbuffer64_u32_word_count: visbuffer64_u32_words,
        total_u32_word_count: malformed_total_u32_words,
        total_byte_count: malformed_total_u32_words * 4,
    };
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::summarize_debug_readback_stream_footprint(
            &malformed_footprint_streams
        ),
        malformed_footprint
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::report_debug_readback_streams(
            &malformed_footprint_streams
        ),
        RenderVirtualGeometryDebugSnapshotReadbackStreamReport {
            footprint: malformed_footprint,
            summary: None,
            decode_error: Some(
                RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::RenderPath(
                    RenderVirtualGeometryRenderPathWordStreamDecodeError::SelectedClusters {
                        word_count: render_path_u32_words + 1,
                    }
                )
            ),
        }
    );
}

#[test]
fn debug_snapshot_readback_payload_check_ignores_visbuffer_clear_word() {
    let snapshot = RenderVirtualGeometryDebugSnapshot::default();
    let readback_streams = snapshot.debug_readback_streams();

    assert_eq!(readback_streams.u32_word_count(), 2);
    assert_eq!(readback_streams.byte_count(), 8);
    assert_eq!(readback_streams.payload_u32_word_count(), 0);
    assert_eq!(readback_streams.payload_byte_count(), 0);
    assert!(readback_streams.is_empty());
    assert!(!readback_streams.has_payload());
    assert!(readback_streams.node_and_cluster_cull.is_empty());
    assert!(readback_streams.render_path.is_empty());
    assert!(readback_streams.visbuffer64.is_empty());
    assert_eq!(readback_streams.visbuffer64.payload_u32_word_count(), 0);

    let summary = snapshot
        .debug_readback_stream_summary()
        .expect("empty snapshot should still decode into a summary");
    assert!(summary.is_empty());
    assert!(!summary.has_payload());

    let report = snapshot.debug_readback_stream_report();
    assert!(report.is_decodable());
    assert!(!report.has_decoded_payload());
    assert_eq!(report.payload_u32_word_count(), 0);
    assert_eq!(report.payload_byte_count(), 0);
}
