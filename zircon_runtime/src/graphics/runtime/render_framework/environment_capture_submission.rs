use crate::core::framework::render::{
    RenderEnvironmentCaptureOutputIdentity, RenderEnvironmentCapturePhase,
    RenderEnvironmentCaptureSourcePayload, RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
};
use crate::graphics::scene::{
    EnvironmentCapturePersistenceSubmission, EnvironmentCapturePersistenceSubmissionStatus,
    EnvironmentCaptureResidentOutput, EnvironmentCaptureSourceSubmission,
    EnvironmentCaptureSourceSubmissionStatus, EnvironmentCaptureSubmission,
};

use super::environment_capture_scheduler::{
    EnvironmentCapturePublication, EnvironmentCaptureScheduler, EnvironmentCaptureTransitionError,
};
use super::wgpu_render_framework::WgpuRenderFrameworkAccess;

/// Accepts at most one queued source capture after a successful viewport submission.
///
/// The caller already owns the framework operation lock. Scheduler ownership is moved
/// before renderer state is locked. Terminal settlement takes the scheduler lock first,
/// publishes or discards the physical output under renderer state, then exposes status.
pub(in crate::graphics::runtime::render_framework) fn pump_environment_capture_source_locked(
    framework: &dyn WgpuRenderFrameworkAccess,
) {
    if !settle_environment_capture_source_locked(framework) {
        return;
    }
    let Some(work_item) = framework.begin_environment_capture_work_item() else {
        return;
    };
    let handle = work_item.handle();
    let submission_result = {
        let mut state = framework.lock_state();
        if state.pending_environment_capture_submission.is_some() {
            Err(crate::graphics::GraphicsError::Asset(
                "environment capture GPU transaction owner is already retained".to_string(),
            ))
        } else {
            state
                .renderer
                .submit_environment_capture_source(work_item)
                .map(|submission| {
                    debug_assert_eq!(submission.handle(), handle);
                    state.pending_environment_capture_submission =
                        Some(EnvironmentCaptureSubmission::Capturing(submission));
                })
        }
    };

    match submission_result {
        Ok(()) => {
            if let Err(transition) = framework.advance_environment_capture_work_item(
                handle,
                // Raster capture and all HDR IBL filtering work are encoded in the
                // same command buffer by the renderer submission.
                RenderEnvironmentCapturePhase::Filtering,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            ) {
                let submission = {
                    let mut state = framework.lock_state();
                    if state
                        .pending_environment_capture_submission
                        .as_ref()
                        .is_some_and(|submission| submission.handle() == handle)
                    {
                        state.pending_environment_capture_submission.take()
                    } else {
                        None
                    }
                };
                if let Some(submission) = submission {
                    if let Some(probe_publication) = submission.probe_publication() {
                        framework
                            .lock_state()
                            .renderer
                            .cancel_environment_capture_probe(probe_publication);
                    }
                }
                let _ = framework.finish_environment_capture_work_item_failure(
                    handle,
                    format!("capture progress publication failed: {transition:?}"),
                );
            }
        }
        Err(error) => {
            let _ =
                framework.finish_environment_capture_work_item_failure(handle, error.to_string());
        }
    }
}

enum EnvironmentCaptureSettlement {
    Pending,
    StartedPersistence {
        handle: crate::core::framework::render::RenderEnvironmentCaptureHandle,
    },
    CompletedSource(EnvironmentCaptureSourceSubmission),
    CompletedPersistence(EnvironmentCapturePersistenceSubmission),
    Failed {
        submission: EnvironmentCaptureSubmission,
        diagnostic: String,
    },
}

/// Observes statuses already advanced by the renderer's sole completion pump.
/// This function never polls the device or waits for a ticket.
fn settle_environment_capture_source_locked(framework: &dyn WgpuRenderFrameworkAccess) -> bool {
    let settlement = {
        let mut state = framework.lock_state();
        let Some(submission) = state.pending_environment_capture_submission.as_ref() else {
            return true;
        };
        match submission {
            EnvironmentCaptureSubmission::Capturing(submission) => {
                let status = state
                    .renderer
                    .environment_capture_submission_status(submission);
                match status {
                    Ok(EnvironmentCaptureSourceSubmissionStatus::Pending) => {
                        EnvironmentCaptureSettlement::Pending
                    }
                    Ok(EnvironmentCaptureSourceSubmissionStatus::Completed) => {
                        let submission = match state
                            .pending_environment_capture_submission
                            .take()
                            .expect("observed environment capture submission must remain owned")
                        {
                            EnvironmentCaptureSubmission::Capturing(submission) => submission,
                            EnvironmentCaptureSubmission::Persisting(_) => {
                                unreachable!("observed capture owner changed phase")
                            }
                        };
                        if submission.request().persistence_output_uri().is_some() {
                            let handle = submission.handle();
                            match state
                                .renderer
                                .begin_environment_capture_persistence(submission)
                            {
                                Ok(persistence) => {
                                    state.pending_environment_capture_submission =
                                        Some(EnvironmentCaptureSubmission::Persisting(persistence));
                                    EnvironmentCaptureSettlement::StartedPersistence { handle }
                                }
                                Err((submission, error)) => EnvironmentCaptureSettlement::Failed {
                                    submission: EnvironmentCaptureSubmission::Capturing(submission),
                                    diagnostic: format!(
                                        "begin environment capture source readback: {error}"
                                    ),
                                },
                            }
                        } else {
                            EnvironmentCaptureSettlement::CompletedSource(submission)
                        }
                    }
                    Ok(status @ EnvironmentCaptureSourceSubmissionStatus::Failed { .. }) => {
                        EnvironmentCaptureSettlement::Failed {
                            submission: state
                                .pending_environment_capture_submission
                                .take()
                                .expect("failed environment capture submission must remain owned"),
                            diagnostic: status
                                .failure_diagnostic()
                                .expect("failed submission status must provide a diagnostic"),
                        }
                    }
                    Err(error) => EnvironmentCaptureSettlement::Failed {
                        submission: state.pending_environment_capture_submission.take().expect(
                            "unobservable environment capture submission must remain owned",
                        ),
                        diagnostic: format!("query environment capture GPU transaction: {error}"),
                    },
                }
            }
            EnvironmentCaptureSubmission::Persisting(persistence) => {
                let status = state
                    .renderer
                    .environment_capture_persistence_status(persistence);
                match status {
                    Ok(EnvironmentCapturePersistenceSubmissionStatus::Pending) => {
                        EnvironmentCaptureSettlement::Pending
                    }
                    Ok(EnvironmentCapturePersistenceSubmissionStatus::ReadyForNextBatch) => {
                        let mut persistence = match state
                            .pending_environment_capture_submission
                            .take()
                            .expect("ready persistence submission must remain owned")
                        {
                            EnvironmentCaptureSubmission::Persisting(persistence) => persistence,
                            EnvironmentCaptureSubmission::Capturing(_) => {
                                unreachable!("ready persistence owner changed phase")
                            }
                        };
                        match state
                            .renderer
                            .advance_environment_capture_persistence(&mut persistence)
                        {
                            Ok(()) => {
                                state.pending_environment_capture_submission =
                                    Some(EnvironmentCaptureSubmission::Persisting(persistence));
                                EnvironmentCaptureSettlement::Pending
                            }
                            Err(error) => EnvironmentCaptureSettlement::Failed {
                                submission: EnvironmentCaptureSubmission::Persisting(persistence),
                                diagnostic: format!(
                                    "advance environment capture source readback: {error}"
                                ),
                            },
                        }
                    }
                    Ok(EnvironmentCapturePersistenceSubmissionStatus::Completed) => {
                        let persistence = match state
                            .pending_environment_capture_submission
                            .take()
                            .expect("completed persistence submission must remain owned")
                        {
                            EnvironmentCaptureSubmission::Persisting(persistence) => persistence,
                            EnvironmentCaptureSubmission::Capturing(_) => {
                                unreachable!("completed persistence owner changed phase")
                            }
                        };
                        EnvironmentCaptureSettlement::CompletedPersistence(persistence)
                    }
                    Ok(EnvironmentCapturePersistenceSubmissionStatus::Failed { submission }) => {
                        EnvironmentCaptureSettlement::Failed {
                            submission: state
                                .pending_environment_capture_submission
                                .take()
                                .expect("failed persistence submission must remain owned"),
                            diagnostic: format!(
                                "environment capture source readback submission failed: {submission:?}"
                            ),
                        }
                    }
                    Err(error) => EnvironmentCaptureSettlement::Failed {
                        submission: state
                            .pending_environment_capture_submission
                            .take()
                            .expect("unobservable persistence submission must remain owned"),
                        diagnostic: format!(
                            "query environment capture source readback transaction: {error}"
                        ),
                    },
                }
            }
        }
    };

    match settlement {
        EnvironmentCaptureSettlement::Pending => false,
        EnvironmentCaptureSettlement::StartedPersistence { handle } => {
            if let Err(transition) = framework.advance_environment_capture_work_item(
                handle,
                RenderEnvironmentCapturePhase::Persisting,
                RENDER_ENVIRONMENT_CAPTURE_WORK_ITEM_COUNT,
            ) {
                let submission = {
                    let mut state = framework.lock_state();
                    if state
                        .pending_environment_capture_submission
                        .as_ref()
                        .is_some_and(|submission| submission.handle() == handle)
                    {
                        state.pending_environment_capture_submission.take()
                    } else {
                        None
                    }
                };
                if let Some(submission) = submission {
                    cancel_environment_capture_probe(framework, &submission);
                }
                let _ = framework.finish_environment_capture_work_item_failure(
                    handle,
                    format!("persistence progress publication failed: {transition:?}"),
                );
                return true;
            }
            false
        }
        EnvironmentCaptureSettlement::CompletedSource(submission) => {
            let handle = submission.handle();
            match settle_environment_capture_success(framework, submission, None) {
                Ok(()) => {}
                Err(transition) => {
                    let _ = framework.finish_environment_capture_work_item_failure(
                        handle,
                        format!("capture completion publication failed: {transition:?}"),
                    );
                }
            }
            true
        }
        EnvironmentCaptureSettlement::CompletedPersistence(persistence) => {
            let handle = persistence.handle();
            let (submission, readback) = persistence.into_parts();
            let readback = match readback {
                Ok(readback) => readback,
                Err(error) => {
                    cancel_environment_capture_source_probe(framework, &submission);
                    let _ = framework.finish_environment_capture_work_item_failure(
                        handle,
                        format!("complete environment capture source readback: {error}"),
                    );
                    return true;
                }
            };
            let face_size = readback.face_size();
            let mip_count = readback.mip_count();
            let payload = RenderEnvironmentCaptureSourcePayload::new(
                handle,
                RenderEnvironmentCaptureOutputIdentity::from_request(submission.request()),
                face_size,
                mip_count,
                readback.into_source_rgba16f_bytes(),
            );
            let payload = match payload {
                Ok(payload) => payload,
                Err(error) => {
                    cancel_environment_capture_source_probe(framework, &submission);
                    let _ = framework.finish_environment_capture_work_item_failure(
                        handle,
                        format!("validate environment capture source payload: {error}"),
                    );
                    return true;
                }
            };
            match settle_environment_capture_success(framework, submission, Some(payload)) {
                Ok(()) => {}
                Err(transition) => {
                    let _ = framework.finish_environment_capture_work_item_failure(
                        handle,
                        format!("source payload publication failed: {transition:?}"),
                    );
                }
            }
            true
        }
        EnvironmentCaptureSettlement::Failed {
            submission,
            diagnostic,
        } => {
            cancel_environment_capture_probe(framework, &submission);
            let _ = framework
                .finish_environment_capture_work_item_failure(submission.handle(), diagnostic);
            true
        }
    }
}

fn settle_environment_capture_success(
    framework: &dyn WgpuRenderFrameworkAccess,
    submission: EnvironmentCaptureSourceSubmission,
    source_payload: Option<RenderEnvironmentCaptureSourcePayload>,
) -> Result<(), EnvironmentCaptureTransitionError> {
    let handle = submission.handle();
    let mut pending_output = Some(submission.into_resident_output());
    let mut publication = |disposition, scheduler: &EnvironmentCaptureScheduler| {
        debug_assert!(scheduler
            .poll(handle)
            .is_ok_and(|status| !status.phase().is_terminal()));
        let output = pending_output
            .take()
            .expect("environment capture publication callback must run once");
        match disposition {
            EnvironmentCapturePublication::Publish => {
                publish_environment_capture_output(framework, output);
            }
            EnvironmentCapturePublication::Discard => {
                cancel_environment_capture_resident_probe(framework, &output);
            }
        }
    };
    let result = framework.settle_environment_capture_work_item_success(
        handle,
        source_payload,
        &mut publication,
    );
    drop(publication);
    if result.is_err() {
        if let Some(output) = pending_output.take() {
            cancel_environment_capture_resident_probe(framework, &output);
        }
    }
    result
}

fn publish_environment_capture_output(
    framework: &dyn WgpuRenderFrameworkAccess,
    output: EnvironmentCaptureResidentOutput,
) {
    let probe_publication = output.probe_publication();
    let mut state = framework.lock_state();
    if let Some(probe_publication) = probe_publication {
        state
            .renderer
            .commit_environment_capture_probe(probe_publication);
    }
    state.environment_capture_residency.publish(output);
}

fn cancel_environment_capture_resident_probe(
    framework: &dyn WgpuRenderFrameworkAccess,
    output: &EnvironmentCaptureResidentOutput,
) {
    if let Some(probe_publication) = output.probe_publication() {
        framework
            .lock_state()
            .renderer
            .cancel_environment_capture_probe(probe_publication);
    }
}

fn cancel_environment_capture_probe(
    framework: &dyn WgpuRenderFrameworkAccess,
    submission: &EnvironmentCaptureSubmission,
) {
    if let Some(probe_publication) = submission.probe_publication() {
        framework
            .lock_state()
            .renderer
            .cancel_environment_capture_probe(probe_publication);
    }
}

fn cancel_environment_capture_source_probe(
    framework: &dyn WgpuRenderFrameworkAccess,
    submission: &EnvironmentCaptureSourceSubmission,
) {
    if let Some(probe_publication) = submission.probe_publication() {
        framework
            .lock_state()
            .renderer
            .cancel_environment_capture_probe(probe_publication);
    }
}

#[cfg(test)]
mod tests {
    const SOURCE: &str = include_str!("environment_capture_submission.rs");
    const EXTRACT_SUBMIT: &str = include_str!("submit_frame_extract/submit/submit.rs");
    const RUNTIME_SUBMIT: &str =
        include_str!("submit_frame_extract/submit/submit_runtime_frame.rs");

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
        assert!(source
            .contains("settle_environment_capture_success(framework, submission, Some(payload))"));
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
}
