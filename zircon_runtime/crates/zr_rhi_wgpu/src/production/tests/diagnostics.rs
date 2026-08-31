use zr_rhi::{
    DiagnosticQueryPlan, DiagnosticReadbackBudget, DiagnosticReadbackTerminal, RenderDevice,
    RenderDeviceFeature, RenderDeviceRequestPolicy, RenderQueueClass, SubmissionStatus,
};

use super::super::{WgpuDiagnosticQueryDelivery, WgpuRenderDevice};
use super::{production_test_device, production_test_device_with_policy};

#[test]
fn optional_query_scope_without_requested_native_features_keeps_the_packet_renderable() {
    let Some(device) = production_test_device() else {
        return;
    };
    let mut plan = DiagnosticQueryPlan::for_frame(73, DiagnosticReadbackBudget::default());
    let pass = plan.register_pass().unwrap();
    let timestamp = plan.reserve_timestamp_scope(pass).unwrap();
    let scope = plan.pass_scope(Some(timestamp), None).unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "optional-query-scope")
        .unwrap();
    command_list.begin_compute_pass_with_diagnostics("optional-query", scope);
    command_list.end_compute_pass();
    let packet = device
        .create_submission_packet_with_diagnostic_query_plan(
            RenderQueueClass::Compute,
            vec![command_list],
            plan,
        )
        .unwrap();

    let ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    super::wait_for_submission(&device, ticket);
    assert_eq!(
        device.submission_status(ticket).unwrap(),
        SubmissionStatus::Completed
    );

    let delivery = device
        .take_diagnostic_query_delivery()
        .expect("unavailable optional diagnostics must remain observable");
    assert_eq!(delivery.frame_index, 73);
    assert_eq!(delivery.terminal, DiagnosticReadbackTerminal::Unavailable);
    assert_eq!(delivery.pass_results, None);
}

#[test]
fn query_lifecycle_can_overlap_copy_admission_without_a_submit_or_poll_owner() {
    let query_source = include_str!("../diagnostics/query.rs");
    let service_source = include_str!("../diagnostics/readback/service.rs");
    let device_diagnostics_source = include_str!("../device/diagnostics.rs");

    assert!(query_source.contains("tracker: DiagnosticReadbackTracker,"));
    assert!(!query_source.contains("tracker: &mut DiagnosticReadbackTracker"));
    assert!(service_source.contains("query_service: WgpuDiagnosticQueryService"));
    assert!(!service_source.contains("collect_completed(&mut self.tracker"));
    assert!(!query_source.contains("queue.submit("));
    assert!(!query_source.contains("device.poll("));
    assert!(!query_source.contains("wait_indefinitely"));
    assert!(device_diagnostics_source.contains("pub fn submission_metrics"));
    assert!(device_diagnostics_source.contains("self.submissions.metrics_snapshot()"));
}

#[test]
fn standalone_diagnostic_readback_submit_flushes_through_the_device_owner() {
    let source = include_str!("../device/diagnostics.rs");
    let start = source
        .find("pub fn submit_and_flush_diagnostic_readback_frame(")
        .expect("standalone diagnostic submit owner");
    let end = source[start..]
        .find("/// Cancels requests admitted into the active diagnostic frame")
        .map(|offset| start + offset)
        .expect("standalone diagnostic submit owner boundary");
    let owner = &source[start..end];

    assert!(owner.contains("self.submit_diagnostic_readback_frame(label)?"));
    assert!(owner.contains("self.flush_submissions()"));
    assert!(owner.contains("self.cancel_submission(frame.submission())"));
    assert!(!owner.contains("queue.submit("));
    assert!(!owner.contains("device.poll("));
}

#[test]
fn native_query_recording_reserves_before_passes_and_binds_only_at_scene_enqueue() {
    let query_source = include_str!("../diagnostics/query.rs");
    let device_diagnostics_source = include_str!("../device/diagnostics.rs");
    let native_recording_source = include_str!("../device/native_recording.rs");

    assert!(query_source.contains("pub struct WgpuNativeDiagnosticQueryRecorder"));
    assert!(query_source.contains("pub(crate) fn begin_native_frame("));
    assert!(query_source.contains("pub(crate) fn prepare_native_frame("));
    assert!(query_source.contains("pub(crate) fn bind_native_frame("));
    assert!(query_source.contains("encode_resolve_into(encoder)"));
    assert!(!query_source.contains("queue.submit("));
    assert!(!query_source.contains("device.poll("));

    assert!(device_diagnostics_source.contains("pub fn begin_native_diagnostic_query_frame("));
    assert!(device_diagnostics_source.contains("pub fn prepare_native_diagnostic_query_frame("));
    assert!(native_recording_source
        .contains("pub fn enqueue_native_recording_packet_with_frame_diagnostics("));
    assert!(native_recording_source.contains("bind_native_query_frame(ticket, frame)"));
    assert!(native_recording_source
        .find("bind_native_query_frame(ticket, frame)")
        .is_some_and(|bind| native_recording_source
            .find("self.submissions.commit_packet(ticket, command_buffers)")
            .is_some_and(|commit| bind < commit)));
}

#[test]
fn timestamp_enabled_device_resolves_one_packet_bound_query_frame() {
    let policy = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(RenderDeviceFeature::GpuTimestamp);
    let Some(device) =
        production_test_device_with_policy(zr_rhi::GpuMemoryBudget::reference_1080p_mid(), &policy)
    else {
        return;
    };
    if !device.caps().supports_gpu_timestamp {
        return;
    }

    let mut plan = DiagnosticQueryPlan::for_frame(74, DiagnosticReadbackBudget::default());
    let pass = plan.register_pass().unwrap();
    let timestamp = plan.reserve_timestamp_scope(pass).unwrap();
    let scope = plan.pass_scope(Some(timestamp), None).unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "timestamp-query-scope")
        .unwrap();
    command_list.begin_compute_pass_with_diagnostics("timestamp-query", scope);
    command_list.end_compute_pass();
    let packet = device
        .create_submission_packet_with_diagnostic_query_plan(
            RenderQueueClass::Compute,
            vec![command_list],
            plan,
        )
        .unwrap();

    let ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    super::wait_for_submission(&device, ticket);
    let delivery = wait_for_query_delivery(&device);
    assert_eq!(delivery.frame_index, 74);
    assert_eq!(delivery.terminal, DiagnosticReadbackTerminal::Succeeded);
    assert_eq!(delivery.pass_results.as_ref().map(Vec::len), Some(1));
    assert!(delivery.timestamp_period_ns > 0.0);
}

#[test]
fn pipeline_statistics_enabled_device_resolves_one_query_index_to_all_counters() {
    let policy = RenderDeviceRequestPolicy::mvp_baseline()
        .with_optional_feature(RenderDeviceFeature::PipelineStatistics);
    let Some(device) =
        production_test_device_with_policy(zr_rhi::GpuMemoryBudget::reference_1080p_mid(), &policy)
    else {
        return;
    };
    if !device.caps().supports_pipeline_statistics_query {
        return;
    }

    let mut plan = DiagnosticQueryPlan::for_frame(75, DiagnosticReadbackBudget::default());
    let pass = plan.register_pass().unwrap();
    let pipeline_statistics = plan.reserve_pipeline_statistics_scope(pass).unwrap();
    assert_eq!(pipeline_statistics.query_index(), 0);
    assert_eq!(plan.pipeline_statistics_query_count(), 1);
    assert_eq!(plan.pipeline_statistics_result_value_count(), 5);
    let scope = plan.pass_scope(None, Some(pipeline_statistics)).unwrap();
    let mut command_list = device
        .create_command_list(RenderQueueClass::Compute, "pipeline-statistics-query-scope")
        .unwrap();
    command_list.begin_compute_pass_with_diagnostics("pipeline-statistics-query", scope);
    command_list.end_compute_pass();
    let packet = device
        .create_submission_packet_with_diagnostic_query_plan(
            RenderQueueClass::Compute,
            vec![command_list],
            plan,
        )
        .unwrap();

    let ticket = device.enqueue_submission_packet(packet).unwrap();
    assert_eq!(device.flush_submissions().unwrap(), 1);
    super::wait_for_submission(&device, ticket);
    let delivery = wait_for_query_delivery(&device);
    assert_eq!(delivery.frame_index, 75);
    assert_eq!(delivery.terminal, DiagnosticReadbackTerminal::Succeeded);
    assert_eq!(delivery.pass_results.as_ref().map(Vec::len), Some(1));
}

fn wait_for_query_delivery(device: &WgpuRenderDevice) -> WgpuDiagnosticQueryDelivery {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        device.poll_submissions().unwrap();
        if let Some(delivery) = device.take_diagnostic_query_delivery() {
            return delivery;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "diagnostic query map timed out"
        );
        std::thread::yield_now();
    }
}
