//! 普通视口成功提交后推进环境捕获源和读回结算；失败路径释放烘焙预约以允许重试。
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
#[path = "tests/environment_capture_submission.rs"]
mod tests;
