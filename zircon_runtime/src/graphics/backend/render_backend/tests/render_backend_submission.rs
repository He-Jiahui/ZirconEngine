#[test]
fn raw_backend_submission_routes_only_through_the_wgpu_render_device() {
    let source = include_str!("../render_backend_submission.rs");

    assert!(source.contains("submit_native_recording_packet(recorder.finish()?)"));
    assert!(source.contains("enqueue_native_buffer_upload_batch(batch)"));
    assert!(source.contains("enqueue_native_texture_upload_batch(batch)"));
    assert!(source.contains("enqueue_native_recording_packet(recorder.finish()?)"));
    assert!(source.contains(".poll_submissions()"));
    assert!(source.contains("self.render_device.submission_metrics()"));
    assert!(source.contains(".append_submission_statuses(tickets, statuses)"));
    assert!(source.contains(".settle_abandoned_native_submissions(tickets)"));
    assert!(!source.contains("submission_coordinator"));
    assert!(!source.contains("queue.submit"));
}

#[test]
fn frame_submission_metrics_are_derived_without_flushing_or_polling() {
    let source = include_str!("../render_backend_submission.rs");
    let sampler = source
        .split("pub(crate) fn frame_submission_metrics_since")
        .nth(1)
        .and_then(|source| source.split("pub(crate) fn submission_status").next())
        .expect("frame submission metrics sampler");

    assert!(sampler.contains("self.submission_metrics().delta_since(baseline)"));
    assert!(sampler.contains("RenderFrameSubmissionMetrics::new("));
    assert!(!sampler.contains("flush_submissions"));
    assert!(!sampler.contains("poll_submissions"));
    assert!(!sampler.contains("queue.submit"));
}

#[test]
fn explicit_diagnostic_drain_is_bounded_and_uses_the_single_completion_pump() {
    let source = include_str!("../render_backend_submission.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(source.contains("PRODUCT_DIAGNOSTIC_CAPTURE_TIMEOUT"));
    assert!(source.contains("let poll_receipt = self.poll_submission_completions()?"));
    assert!(source.contains("observe_poll(poll_receipt)?"));
    assert!(source.contains("metrics.in_flight_request_count() == 0"));
    assert!(source.contains("metrics.retained_delivery_count() == 0"));
    assert!(source.contains("DiagnosticReadbackTimedOut"));
    assert!(!source.contains("wait_indefinitely"));
    assert!(!source.contains("self.device.poll("));
}

#[test]
fn surface_submission_uses_the_same_device_owned_scene_packet() {
    let source = include_str!("../render_backend_submission.rs");

    assert!(source.contains(".submit_native_recording_packet_with_frame_diagnostics_and_surface("));
    assert!(source.contains("surface_target,"));
    assert!(!source.contains("queue.submit"));
}

#[test]
fn rejected_frame_producer_tickets_are_settled_at_the_backend_boundary() {
    let source = include_str!("../render_backend_submission.rs");
    let record = source
        .find("transaction.record_pre_scene_submission(producer, ticket)")
        .expect("backend helper must delegate producer recording");
    let settle = source
        .find("self.settle_rejected_pre_scene_submission(ticket, error)")
        .expect("backend helper must settle a rejected ticket");
    let settle_impl = source
        .find("self.settle_abandoned_submissions(&[ticket])")
        .expect("rejected ticket settlement must use the backend owner");

    assert!(record < settle);
    assert!(settle < settle_impl);
    assert!(source.contains("record_pre_scene_resource_submission("));
    assert!(source.contains("FrameProducerRegistrationFailed"));
}
