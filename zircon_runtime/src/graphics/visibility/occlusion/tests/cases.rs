use super::{
    HzbOcclusionCullReadbackStats, HzbOcclusionCullReport, HzbOcclusionIndirectArgsReadbackSummary,
};

#[test]
fn hzb_occlusion_report_preserves_readback_stats() {
    let readback_stats = HzbOcclusionCullReadbackStats::new(6, 42, 2, 18);
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
        .with_readback_stats(readback_stats);

    assert_eq!(report.readback_stats, Some(readback_stats));
}

#[test]
fn hzb_occlusion_report_preserves_workspace_churn() {
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
        .with_workspace_stats(1, 64, 2);

    assert_eq!(report.params_buffer_create_count, 1);
    assert_eq!(report.params_upload_byte_count, 64);
    assert_eq!(report.bind_group_create_count, 2);
}

#[test]
fn hzb_occlusion_report_preserves_indirect_args_readback_summary() {
    let summary = HzbOcclusionIndirectArgsReadbackSummary::new(6, 4, 2, 24);
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
        .with_indirect_args_readback(summary)
        .with_indirect_args_readback_source_frame_index(17);

    assert_eq!(report.indirect_args_readback, Some(summary));
    assert_eq!(report.indirect_args_readback_source_frame_index, Some(17));
}

#[test]
fn hzb_occlusion_report_records_delayed_stats_source_frame() {
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
        .with_readback_stats(HzbOcclusionCullReadbackStats::new(6, 42, 2, 18))
        .with_readback_stats_source_frame_index(11);

    assert_eq!(report.readback_stats_source_frame_index, Some(11));
}

#[test]
fn hzb_occlusion_report_preserves_async_readback_queue_diagnostics() {
    let report = HzbOcclusionCullReport::single_frame_reproject(6, 42, 1, 1, true)
        .with_readback_queue_diagnostics(3, 2, Some(4));

    assert_eq!(report.readback_pending_count, 3);
    assert_eq!(report.readback_dropped_count, 2);
    assert_eq!(report.readback_oldest_pending_age_frames, Some(4));
}

#[test]
fn hzb_occlusion_indirect_args_summary_saturates_totals() {
    let mut summary = HzbOcclusionIndirectArgsReadbackSummary::new(u32::MAX, u32::MAX, 1, u32::MAX);

    summary.add_assign(HzbOcclusionIndirectArgsReadbackSummary::new(1, 1, 2, 1));

    assert_eq!(summary.readback_arg_count, u32::MAX);
    assert_eq!(summary.compacted_draw_count, u32::MAX);
    assert_eq!(summary.zero_instance_arg_count, 3);
    assert_eq!(summary.remaining_instance_count, u32::MAX);
}
