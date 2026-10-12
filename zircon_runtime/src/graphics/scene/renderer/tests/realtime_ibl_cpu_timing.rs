use super::{
    RealtimeIblCpuTimingCollector, RealtimeIblCpuTimingReport,
    REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY,
};

fn report(capture_epoch: u64, frame_number: u64) -> RealtimeIblCpuTimingReport {
    RealtimeIblCpuTimingReport {
        profile_capture_epoch: capture_epoch,
        frame_number,
        ..RealtimeIblCpuTimingReport::default()
    }
}

#[test]
fn cpu_timing_collector_evicts_oldest_reports_and_exposes_overwrite_count() {
    let mut collector = RealtimeIblCpuTimingCollector::default();
    for frame_number in 0..=REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY as u64 {
        collector.record_completed(report(7, frame_number));
    }

    let reports = collector.take_completed();
    assert_eq!(reports.len(), REALTIME_IBL_CPU_TIMING_REPORT_CAPACITY);
    assert_eq!(reports.first().map(|report| report.frame_number), Some(1));
    assert_eq!(reports.last().map(|report| report.frame_number), Some(256));
    assert_eq!(
        reports.last().map(|report| report.overwritten_report_count),
        Some(1)
    );
}

#[test]
fn cpu_timing_collector_never_mixes_profile_capture_epochs() {
    let mut collector = RealtimeIblCpuTimingCollector::default();
    collector.record_completed(report(7, 1));
    collector.record_completed(report(8, 2));

    let reports = collector.take_completed();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].profile_capture_epoch, 8);
    assert_eq!(reports[0].frame_number, 2);
    assert_eq!(reports[0].overwritten_report_count, 0);
}

#[test]
fn cpu_timing_collector_discards_prior_samples_when_a_new_capture_starts() {
    let mut collector = RealtimeIblCpuTimingCollector::default();
    collector.record_completed(report(7, 1));
    collector.synchronize_capture_epoch(Some(8));

    assert!(collector.take_completed().is_empty());
}
