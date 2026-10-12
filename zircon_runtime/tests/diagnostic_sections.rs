use super::prelude::*;

#[test]
fn debug_readback_decode_errors_report_owning_section() {
    let node_and_cluster_cull_error =
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::NodeAndClusterCull(
            RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::InstanceSeeds {
                word_count: 1,
            },
        );
    let render_path_error = RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::RenderPath(
        RenderVirtualGeometryRenderPathWordStreamDecodeError::SelectedClusters { word_count: 1 },
    );
    let visbuffer64_error =
        RenderVirtualGeometryDebugSnapshotReadbackStreamDecodeError::VisBuffer64(
            RenderVirtualGeometryVisBuffer64ReadbackStreamDecodeError::MismatchedEntryAndValueCount {
                entry_index_count: 2,
                packed_value_count: 1,
            },
        );

    assert_eq!(
        node_and_cluster_cull_error.section(),
        RenderVirtualGeometryDebugSnapshotReadbackStreamSection::NodeAndClusterCull
    );
    assert_eq!(
        node_and_cluster_cull_error.malformed_u32_word_count(),
        Some(1)
    );
    assert_eq!(
        render_path_error.section(),
        RenderVirtualGeometryDebugSnapshotReadbackStreamSection::RenderPath
    );
    assert_eq!(render_path_error.malformed_u32_word_count(), Some(1));
    assert_eq!(
        visbuffer64_error.section(),
        RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64
    );
    assert_eq!(visbuffer64_error.malformed_u32_word_count(), None);
}

#[test]
fn section_decode_errors_report_malformed_word_counts() {
    assert_eq!(
        RenderVirtualGeometryNodeAndClusterCullWordStreamDecodeError::TraversalRecords {
            word_count: 7,
        }
        .malformed_u32_word_count(),
        7
    );
    assert_eq!(
        RenderVirtualGeometryRenderPathWordStreamDecodeError::HardwareRasterizationRecords {
            word_count: 11,
        }
        .malformed_u32_word_count(),
        11
    );
}

#[test]
fn debug_readback_report_exposes_failed_decode_section() {
    let snapshot = RenderVirtualGeometryDebugSnapshot::default();
    let clean_report = snapshot.debug_readback_stream_report();
    assert_eq!(clean_report.decode_error_section(), None);
    assert_eq!(clean_report.decode_error_section_u32_word_count(), None);

    let mut malformed_streams = snapshot.debug_readback_streams();
    malformed_streams.render_path.selected_clusters.push(99);
    let malformed_render_path_u32_words = 1;
    let report =
        RenderVirtualGeometryDebugSnapshot::report_debug_readback_streams(&malformed_streams);

    assert_eq!(
        report.decode_error_section(),
        Some(RenderVirtualGeometryDebugSnapshotReadbackStreamSection::RenderPath)
    );
    assert_eq!(
        report.decode_error_section_u32_word_count(),
        Some(malformed_render_path_u32_words)
    );
    assert_eq!(
        report.decode_error_section_byte_count(),
        Some(malformed_render_path_u32_words * 4)
    );
}

#[test]
fn debug_readback_footprint_exposes_section_budget_breakdown() {
    let footprint = RenderVirtualGeometryDebugSnapshotReadbackStreamFootprint {
        node_and_cluster_cull_u32_word_count:
            RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT,
        render_path_u32_word_count: RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT,
        visbuffer64_u32_word_count: 5,
        total_u32_word_count: RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT
            + RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT
            + 5,
        total_byte_count: (RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT
            + RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT
            + 5)
            * 4,
    };

    assert_eq!(
        footprint.section_u32_word_count(
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::NodeAndClusterCull
        ),
        RenderVirtualGeometryNodeAndClusterCullInstanceSeed::GPU_WORD_COUNT
    );
    assert_eq!(
        footprint.section_u32_word_count(
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::RenderPath
        ),
        RenderVirtualGeometrySelectedCluster::GPU_WORD_COUNT
    );
    assert_eq!(
        footprint.section_u32_word_count(
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64
        ),
        5
    );
    assert_eq!(
        footprint.section_payload_u32_word_count(
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64
        ),
        3
    );
    assert_eq!(
        footprint.section_payload_byte_count(
            RenderVirtualGeometryDebugSnapshotReadbackStreamSection::VisBuffer64
        ),
        12
    );
}
