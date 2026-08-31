use std::time::Duration;

use zr_rhi::{DeviceGeneration, DeviceId};

use super::super::submission_metrics::WgpuSubmissionMetrics;

fn snapshot(
    metrics: &WgpuSubmissionMetrics,
    pending_upload_bytes: u64,
) -> super::super::submission_metrics::WgpuSubmissionMetricsSnapshot {
    metrics.snapshot(
        DeviceId::new(71),
        DeviceGeneration::initial(),
        pending_upload_bytes,
    )
}

#[test]
fn submission_metrics_snapshot_reports_upload_submission_and_completion_totals() {
    let mut metrics = WgpuSubmissionMetrics::default();

    metrics.record_resource_upload_admitted(3, 2, 176, 144);
    metrics.record_native_submission(4);
    metrics.record_completion(Duration::from_nanos(7));
    metrics.record_completion(Duration::from_nanos(11));

    let snapshot = snapshot(&metrics, 144);
    assert_eq!(snapshot.buffer_upload_batch_count(), 1);
    assert_eq!(snapshot.texture_upload_batch_count(), 1);
    assert_eq!(snapshot.buffer_write_count(), 3);
    assert_eq!(snapshot.texture_write_count(), 2);
    assert_eq!(snapshot.upload_payload_bytes(), 176);
    assert_eq!(snapshot.native_submission_count(), 1);
    assert_eq!(snapshot.submitted_ticket_count(), 4);
    assert_eq!(snapshot.completed_ticket_count(), 2);
    assert_eq!(snapshot.completed_latency_total_ns(), 18);
    assert_eq!(snapshot.completed_latency_max_ns(), 11);
    assert_eq!(snapshot.pending_upload_bytes(), 144);
    assert_eq!(snapshot.peak_pending_upload_bytes(), 144);
}

#[test]
fn submission_metrics_preserves_peak_and_tracks_upload_admission_rejections() {
    let mut metrics = WgpuSubmissionMetrics::default();

    metrics.record_resource_upload_admitted(0, 1, 32, 96);
    metrics.record_resource_upload_admitted(1, 0, 16, 48);
    metrics.record_resource_upload_rejected(false, true);
    metrics.record_resource_upload_rejected(false, true);
    metrics.record_resource_upload_rejected(true, false);

    let snapshot = snapshot(&metrics, 48);
    assert_eq!(snapshot.pending_upload_bytes(), 48);
    assert_eq!(snapshot.peak_pending_upload_bytes(), 96);
    assert_eq!(snapshot.buffer_upload_rejection_count(), 1);
    assert_eq!(snapshot.texture_upload_rejection_count(), 2);
}

#[test]
fn submission_metrics_delta_keeps_counters_and_sampling_values_distinct() {
    let mut metrics = WgpuSubmissionMetrics::default();

    metrics.record_resource_upload_admitted(2, 0, 48, 48);
    let baseline = snapshot(&metrics, 48);

    metrics.record_resource_upload_admitted(0, 3, 80, 96);
    metrics.record_native_submission(2);
    metrics.record_completion(Duration::from_nanos(13));
    metrics.record_resource_upload_rejected(false, true);
    let delta = metrics
        .snapshot(DeviceId::new(71), DeviceGeneration::initial(), 16)
        .delta_since(baseline)
        .expect("monotonic snapshots should produce a measurement interval");

    assert_eq!(delta.buffer_upload_batch_count(), 0);
    assert_eq!(delta.texture_upload_batch_count(), 1);
    assert_eq!(delta.buffer_write_count(), 0);
    assert_eq!(delta.texture_write_count(), 3);
    assert_eq!(delta.upload_payload_bytes(), 80);
    assert_eq!(delta.native_submission_count(), 1);
    assert_eq!(delta.submitted_ticket_count(), 2);
    assert_eq!(delta.completed_ticket_count(), 1);
    assert_eq!(delta.completed_latency_total_ns(), 13);
    assert_eq!(delta.pending_upload_bytes(), 16);
    assert_eq!(delta.lifetime_peak_pending_upload_bytes(), 96);
    assert_eq!(delta.lifetime_max_completion_latency_ns(), 13);
    assert_eq!(delta.buffer_upload_rejection_count(), 0);
    assert_eq!(delta.texture_upload_rejection_count(), 1);
}

#[test]
fn submission_metrics_delta_rejects_a_counter_reset() {
    let mut previous_service_metrics = WgpuSubmissionMetrics::default();
    previous_service_metrics.record_resource_upload_admitted(1, 0, 16, 16);
    let previous = snapshot(&previous_service_metrics, 16);

    let replacement_service_metrics = WgpuSubmissionMetrics::default();
    assert!(replacement_service_metrics
        .snapshot(DeviceId::new(71), DeviceGeneration::initial(), 0)
        .delta_since(previous)
        .is_none());
}

#[test]
fn submission_metrics_delta_rejects_a_different_device_generation() {
    let mut metrics = WgpuSubmissionMetrics::default();
    metrics.record_native_submission(1);
    let baseline = snapshot(&metrics, 0);
    metrics.record_completion(Duration::from_nanos(9));

    let replacement_generation = metrics.snapshot(DeviceId::new(71), DeviceGeneration::new(2), 0);
    assert!(replacement_generation.delta_since(baseline).is_none());
}
