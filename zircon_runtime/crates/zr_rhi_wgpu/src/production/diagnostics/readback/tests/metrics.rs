use super::*;
use zr_rhi::{DiagnosticReadbackBudget, DiagnosticReadbackKind, DiagnosticReadbackTracker};

fn owner() -> (DeviceId, DeviceGeneration) {
    (DeviceId::new(9), DeviceGeneration::new(4))
}

#[test]
fn metrics_delta_rejects_a_foreign_device_generation() {
    let (device_id, generation) = owner();
    let baseline = WgpuDiagnosticReadbackMetrics::new(device_id, generation).snapshot(0, 0, 0);
    let foreign =
        WgpuDiagnosticReadbackMetrics::new(device_id, DeviceGeneration::new(5)).snapshot(0, 0, 0);

    assert!(foreign.delta_since(baseline).is_none());
}

#[test]
fn metrics_keep_monotonic_totals_separate_from_current_gauges() {
    let (device_id, generation) = owner();
    let mut tracker =
        DiagnosticReadbackTracker::new(device_id, generation, DiagnosticReadbackBudget::default());
    tracker.begin_frame(1).expect("diagnostic frame");
    let request = tracker
        .admit(DiagnosticReadbackKind::Buffer, 16)
        .expect("request admission");
    let admission = DiagnosticReadbackAdmission::Admitted(request);
    let mut metrics = WgpuDiagnosticReadbackMetrics::new(device_id, generation);
    let baseline = metrics.snapshot(0, 0, 0);
    metrics.begin_frame();
    metrics.record_admission(admission, 16);
    metrics.record_submitted_batch(1, 16);
    metrics.record_map_started();
    metrics.record_map_completed();
    let receipt = tracker
        .terminalize(request, DiagnosticReadbackTerminal::Succeeded)
        .expect("terminal receipt");
    metrics.record_terminal(receipt);
    metrics.release_in_flight_batch(1, 16);
    metrics.record_delivery_drained(16);
    let snapshot = metrics.snapshot(0, 0, 0);
    let delta = snapshot.delta_since(baseline).expect("same owner interval");

    assert_eq!(delta.admitted_request_count(), 1);
    assert_eq!(delta.succeeded_request_count(), 1);
    assert_eq!(delta.drained_delivery_bytes(), 16);
    assert_eq!(delta.in_flight_batch_count(), 0);
    assert_eq!(delta.in_flight_bytes(), 0);
    assert_eq!(delta.lifetime_peak_in_flight_bytes(), 16);
}
