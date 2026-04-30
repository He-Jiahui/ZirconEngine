use super::prelude::*;

#[test]
fn debug_snapshot_exports_render_path_selection_and_rasterization_word_streams() {
    let selected_cluster = RenderVirtualGeometrySelectedCluster {
        instance_index: Some(2),
        entity: 0x0000_0001_0000_002a,
        cluster_id: 17,
        cluster_ordinal: 3,
        page_id: 5,
        lod_level: 1,
        state: RenderVirtualGeometryExecutionState::Resident,
    };
    let rasterization_record = RenderVirtualGeometryHardwareRasterizationRecord {
        instance_index: Some(2),
        entity: 0x0000_0001_0000_002a,
        cluster_id: 17,
        cluster_ordinal: 3,
        page_id: 5,
        lod_level: 1,
        submission_index: 9,
        submission_page_id: 11,
        submission_lod_level: 2,
        entity_cluster_start_ordinal: 30,
        entity_cluster_span_count: 4,
        entity_cluster_total_count: 64,
        lineage_depth: 6,
        frontier_rank: 12,
        resident_slot: Some(21),
        submission_slot: Some(22),
        state: RenderVirtualGeometryExecutionState::PendingUpload,
    };
    let snapshot = RenderVirtualGeometryDebugSnapshot {
        selected_clusters_source:
            RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
        hardware_rasterization_source:
            RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
        selected_clusters: vec![selected_cluster.clone()],
        hardware_rasterization_records: vec![rasterization_record.clone()],
        ..RenderVirtualGeometryDebugSnapshot::default()
    };

    assert_eq!(
        snapshot.render_path_word_streams(),
        RenderVirtualGeometryRenderPathWordStreams {
            selected_clusters_source:
                RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
            hardware_rasterization_source:
                RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
            selected_clusters: selected_cluster.packed_words().to_vec(),
            hardware_rasterization_records: rasterization_record.packed_words().to_vec(),
        }
    );
    assert_eq!(
        snapshot.render_path_decoded_streams(),
        Some(RenderVirtualGeometryRenderPathDecodedStreams {
            selected_clusters_source:
                RenderVirtualGeometrySelectedClusterSource::RenderPathExecutionSelections,
            hardware_rasterization_source:
                RenderVirtualGeometryHardwareRasterizationSource::RenderPathExecutionSelections,
            selected_clusters: vec![selected_cluster],
            hardware_rasterization_records: vec![rasterization_record],
        })
    );
    let mut malformed_word_streams = snapshot.render_path_word_streams();
    malformed_word_streams.selected_clusters.push(99);
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::decode_render_path_word_streams(
            &malformed_word_streams
        ),
        None
    );
    assert_eq!(
        RenderVirtualGeometryDebugSnapshot::try_decode_render_path_word_streams(
            &malformed_word_streams
        ),
        Err(
            RenderVirtualGeometryRenderPathWordStreamDecodeError::SelectedClusters {
                word_count: malformed_word_streams.selected_clusters.len(),
            }
        )
    );
}
