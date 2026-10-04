const SOURCE: &str = include_str!("../environment_capture_submission.rs");
const EXTRACT_SUBMIT: &str = include_str!("../submit_frame_extract/submit/submit.rs");
const RUNTIME_SUBMIT: &str = include_str!("../submit_frame_extract/submit/submit_runtime_frame.rs");

fn production_source() -> &'static str {
    SOURCE
        .split_once("#[cfg(test)]")
        .map(|(production, _)| production)
        .expect("environment capture pump must retain a test boundary")
}

#[test]
fn pump_moves_scheduler_work_before_locking_renderer_state() {
    let source = production_source();
    let begin = source
        .find("begin_environment_capture_work_item()")
        .expect("capture work must leave the scheduler");
    let state_lock = source[begin..]
        .find("framework.lock_state()")
        .expect("capture source must enter renderer state");

    assert!(state_lock > 0);
    assert!(source.contains("submit_environment_capture_source(work_item)"));
    assert!(source.contains("Some(EnvironmentCaptureSubmission::Capturing(submission))"));
}

#[test]
fn pump_publishes_filtering_progress_after_retaining_the_source_owner() {
    let source = production_source();
    let retain = source
        .find("Some(EnvironmentCaptureSubmission::Capturing(submission))")
        .expect("source target owner");
    let advance = source
        .find("advance_environment_capture_work_item(")
        .expect("capture progress publication");

    assert!(retain < advance);
    assert!(source.contains("RenderEnvironmentCapturePhase::Filtering"));
    assert!(source.contains("RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT"));
    assert!(source.contains("finish_environment_capture_work_item_failure("));
}

#[test]
fn both_viewport_submission_paths_pump_only_after_success() {
    for source in [EXTRACT_SUBMIT, RUNTIME_SUBMIT] {
        assert_eq!(
            source
                .matches("pump_environment_capture_source_locked(")
                .count(),
            1
        );
        assert!(source.contains("if result.is_ok()"));
    }
}

#[test]
fn settlement_is_nonblocking_and_publishes_only_successful_current_output() {
    let source = production_source();

    assert!(source.contains("settle_environment_capture_source_locked(framework)"));
    assert!(source.contains("environment_capture_submission_status(submission)"));
    assert!(source.contains("settle_environment_capture_success(framework, submission, None)"));
    assert!(source.contains("EnvironmentCapturePublication::Publish"));
    assert!(source.contains("EnvironmentCapturePublication::Discard"));
    assert!(source.contains("settle_environment_capture_work_item_success("));
    assert!(source.contains("&mut publication"));
    assert!(source.contains("environment_capture_residency"));
    assert!(source.contains("commit_environment_capture_probe"));
    assert!(source.contains("cancel_environment_capture_probe"));
    let commit = source
        .find("commit_environment_capture_probe(probe_publication)")
        .expect("array commit must be ticket-settlement owned");
    let publish = source
        .find("state.environment_capture_residency.publish(output)")
        .expect("resident output publication");
    assert!(commit < publish);
    assert!(!source.contains("Ok(false)"));
    assert!(!source.contains("finish_environment_capture_work_item_success_with_source"));
    assert!(
        !source.contains("publish_environment_capture_probe"),
        "probe-array publication must wait for its own completion-ticket owner"
    );
    assert!(!source.contains("poll_submission_completions"));
    assert!(!source.contains("wait_for_submission"));
    assert!(!source.contains("device.poll("));
}

#[test]
fn settlement_releases_transient_capture_scratch_before_taking_scheduler_lock() {
    let source = production_source();
    let settlement = source
        .split("fn settle_environment_capture_success(")
        .nth(1)
        .and_then(|source| {
            source
                .split("fn publish_environment_capture_output(")
                .next()
        })
        .expect("capture success settlement helper");
    let conversion = settlement
        .find("submission.into_resident_output()")
        .expect("transient-to-resident conversion");
    let scheduler = settlement
        .find("framework.settle_environment_capture_work_item_success(")
        .expect("scheduler publication gate");

    assert!(conversion < scheduler);
}

#[test]
fn persistence_streams_one_budgeted_batch_per_pump_before_consuming_payload() {
    let source = production_source();

    assert!(source.contains("EnvironmentCaptureSubmission::Persisting"));
    assert!(source.contains("begin_environment_capture_persistence(submission)"));
    assert!(source.contains("RenderEnvironmentCapturePhase::Persisting"));
    assert!(source.contains("EnvironmentCapturePersistenceSubmissionStatus::ReadyForNextBatch"));
    assert_eq!(
        source
            .matches("advance_environment_capture_persistence(&mut persistence)")
            .count(),
        1
    );
    assert!(source.contains("readback.into_source_rgba16f_bytes()"));
    assert!(
        source.contains("settle_environment_capture_success(framework, submission, Some(payload))")
    );
}

#[test]
fn progress_publication_failure_cancels_retained_probe_reservation() {
    let source = production_source();
    let failure_message = source
        .find("capture progress publication failed")
        .expect("progress failure path");
    let take = source[..failure_message]
        .rfind("pending_environment_capture_submission.take()")
        .expect("failed progress must release source ownership");
    let cancel = source[..failure_message]
        .rfind("cancel_environment_capture_probe(probe_publication)")
        .expect("failed progress must cancel probe reservation");
    let finish = source[..failure_message]
        .rfind("finish_environment_capture_work_item_failure(")
        .expect("failed progress must finish the scheduler item");
    assert!(take < cancel);
    assert!(cancel < finish);
}
