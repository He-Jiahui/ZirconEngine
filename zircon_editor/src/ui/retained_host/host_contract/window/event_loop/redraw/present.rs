use winit::event_loop::ActiveEventLoop;

use crate::core::jobs::JobId;
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::presenter::HostPresenterError;
use crate::ui::retained_host::host_contract::profiling_artifacts::{
    profile_capture_enabled, submit_present_artifacts, ProfileArtifactSubmissionError,
};
use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_current_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

use super::super::super::UiHostWindow;
use super::super::UiHostWindowEventLoop;

pub(super) fn present_redraw(
    event_loop_state: &mut UiHostWindowEventLoop,
    event_loop: &dyn ActiveEventLoop,
    redraw: HostRedrawRequest,
    scenario: UiPerfScenario,
) {
    let damage_region = redraw.damage_region().cloned();
    let _present_scenario_guard = enter_ui_perf_scenario(scenario);
    let profile_measurement_active = event_loop_state.profile_measurement_active();
    let Some(presenter) = event_loop_state.presenter.as_mut() else {
        return;
    };
    let generation = event_loop_state.host.get_host_presentation_generation();
    let presentation_cursor = generation.cursor();
    let _paint_scope = generation.enter_paint_scope();
    let presentation = generation.structure();
    let invalidation = event_loop_state.host.refresh_invalidation_diagnostics();
    let present_result = presenter.present(
        presentation,
        presentation_cursor,
        damage_region.clone(),
        invalidation,
    );
    match present_result {
        Ok(diagnostics) => {
            event_loop_state.reset_surface_present_retry_backoff();
            if let Some(backend) = event_loop_state.presenter_backend.filter(|_| {
                !event_loop_state.profile_artifact_capture_requested && profile_capture_enabled()
            }) {
                event_loop_state.profile_artifact_capture_requested = true;
                match event_loop_state.host.profile_artifact_jobs() {
                    Some(jobs) => {
                        let submitted = submit_present_artifacts(
                            &jobs,
                            &event_loop_state.host.window().size(),
                            backend,
                            || generation.materialize(),
                        );
                        if let Some(job_id) =
                            profile_artifact_submission_job_id(&event_loop_state.host, submitted)
                        {
                            event_loop_state.host.track_profile_artifact_job(job_id);
                            record_current_ui_perf_counter(UiPerfCounter::ArtifactExportCount, 1.0);
                        }
                    }
                    None => event_loop_state.host.record_host_diagnostic(
                        HostWindowDiagnosticSeverity::Warning,
                        "profile artifact export has no injected editor job system",
                    ),
                }
            }
            if profile_measurement_active {
                zircon_runtime::profile_counter!("editor", "ui.surface.submitted_count", 1_u8);
                event_loop_state.record_presented_input_batch(scenario);
            } else {
                event_loop_state.complete_profile_warmup_present();
            }
            event_loop_state
                .host
                .set_host_refresh_diagnostics_overlay(diagnostics);
            if let Err(error) = event_loop_state.host.notify_first_presented() {
                event_loop_state.host.report_fatal_failure(
                    "editor_host_window",
                    "first_present_notification",
                    format!("editor first-present notification failed: {error}"),
                    "verify the Hub startup mailbox and retry zircon_editor",
                );
                event_loop.exit();
                return;
            }
            if let Err(error) = event_loop_state.host.capture_first_presented_frame() {
                event_loop_state
                    .host
                    .record_first_presented_frame_capture_error(&error);
                event_loop_state.host.report_fatal_failure(
                    "editor_host_window",
                    "first_presented_frame_capture",
                    format!("editor first-frame capture failed: {error}"),
                    "choose a writable PNG capture path and retry zircon_editor",
                );
                event_loop.exit();
                return;
            }
            exit_after_presented_frame(
                event_loop_state.host.exit_after_first_presented_frame(),
                &event_loop_state.host,
                event_loop,
            );
        }
        Err(HostPresenterError::RetryableSurfacePresent) => {
            zircon_runtime::profile_counter!(
                "editor",
                "ui.surface.retryable_no_submit_count",
                1_u8
            );
            let retry = retry_present_request(scenario, redraw);
            event_loop_state.defer_surface_present_retry(retry, std::time::Instant::now());
        }
        Err(error) => {
            let requested = event_loop_state
                .presenter_backend
                .map(|backend| format!("presenter_backend={}", backend.label()))
                .unwrap_or_else(|| "presenter_backend=<unknown>".to_owned());
            event_loop_state.host.report_fatal_failure(
                "editor_host_window",
                requested,
                format!("presenter present failed: {error}"),
                "verify the graphics adapter and window surface, then restart zircon_editor",
            );
            event_loop.exit();
        }
    }
}

fn retry_present_request(scenario: UiPerfScenario, redraw: HostRedrawRequest) -> HostRedrawRequest {
    redraw.into_present_retry(scenario)
}

fn profile_artifact_submission_job_id(
    host: &UiHostWindow,
    submission: Result<Option<JobId>, ProfileArtifactSubmissionError>,
) -> Option<JobId> {
    match submission {
        Ok(job_id) => job_id,
        Err(error) => {
            host.record_host_diagnostic(
                HostWindowDiagnosticSeverity::Warning,
                format!("profile artifact export was not submitted: {error}"),
            );
            None
        }
    }
}

#[cfg(test)]
mod profile_artifact_submission_tests {
    use super::*;

    use crate::core::jobs::JobSubmitError;
    use crate::ui::retained_host::host_contract::profiling_artifacts::ProfileOutputRootError;

    #[test]
    fn rejected_profile_artifact_submission_records_a_host_warning() {
        let host = UiHostWindow::new().expect("host window should construct");

        assert_eq!(
            profile_artifact_submission_job_id(
                &host,
                Err(ProfileArtifactSubmissionError::Job(
                    JobSubmitError::AdmissionEntryLimitExceeded { limit: 1 },
                )),
            ),
            None
        );

        let diagnostics = host.take_host_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].severity(),
            HostWindowDiagnosticSeverity::Warning
        );
        assert!(diagnostics[0]
            .message()
            .contains("profile artifact export was not submitted"));
    }

    #[test]
    fn invalid_profile_output_root_records_a_host_warning() {
        let host = UiHostWindow::new().expect("host window should construct");

        assert_eq!(
            profile_artifact_submission_job_id(
                &host,
                Err(ProfileArtifactSubmissionError::InvalidOutputRoot(
                    ProfileOutputRootError,
                )),
            ),
            None
        );

        let diagnostics = host.take_host_diagnostics();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].severity(),
            HostWindowDiagnosticSeverity::Warning
        );
        assert!(diagnostics[0]
            .message()
            .contains("outside the C: system drive"));
    }
}

fn exit_after_presented_frame(
    enabled: bool,
    host: &super::super::super::UiHostWindow,
    event_loop: &dyn ActiveEventLoop,
) {
    if let Some(diagnostic) = first_presented_frame_diagnostic(enabled) {
        host.record_host_diagnostic(HostWindowDiagnosticSeverity::Info, diagnostic);
        event_loop.exit();
    }
}

fn first_presented_frame_diagnostic(enabled: bool) -> Option<&'static str> {
    enabled.then_some("editor_first_frame_presented")
}

#[cfg(test)]
mod tests {
    use super::{first_presented_frame_diagnostic, retry_present_request};
    use crate::ui::retained_host::host_contract::data::FrameRect;
    use crate::ui::retained_host::host_contract::redraw::HostRedrawRequest;
    use crate::ui::retained_host::ui_perf::UiPerfScenario;

    #[test]
    fn first_frame_exit_emits_a_presented_frame_diagnostic() {
        assert_eq!(
            first_presented_frame_diagnostic(true),
            Some("editor_first_frame_presented")
        );
    }

    #[test]
    fn continuous_editor_does_not_emit_a_one_shot_presented_frame_diagnostic() {
        assert_eq!(first_presented_frame_diagnostic(false), None);
    }

    #[test]
    fn retryable_surface_present_requeues_the_same_present_without_a_frame_update() {
        let damage = FrameRect {
            x: 3.0,
            y: 4.0,
            width: 20.0,
            height: 12.0,
        };
        let region = retry_present_request(
            UiPerfScenario::IdleHover,
            HostRedrawRequest::region_for_scenario_with_frame_update(
                UiPerfScenario::Click,
                damage.clone(),
                true,
            ),
        );
        assert!(region.request_redraw());
        assert!(region.requires_present());
        assert!(!region.requires_frame_update());
        assert_eq!(region.damage_region(), Some(&damage));
        assert_eq!(region.scenario(), UiPerfScenario::IdleHover);

        let full = retry_present_request(
            UiPerfScenario::WindowResize,
            HostRedrawRequest::full_frame_for_scenario(UiPerfScenario::Click, true),
        );
        assert!(full.request_redraw());
        assert!(full.requires_present());
        assert!(!full.requires_frame_update());
        assert_eq!(full.damage_region(), None);
        assert_eq!(full.scenario(), UiPerfScenario::WindowResize);
    }

    #[test]
    fn retryable_surface_present_preserves_bounded_damage_pressure() {
        let redraw = HostRedrawRequest::region(FrameRect {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        })
        .merge(HostRedrawRequest::region(FrameRect {
            x: 20.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        }))
        .merge(HostRedrawRequest::region(FrameRect {
            x: 100.0,
            y: 0.0,
            width: 10.0,
            height: 10.0,
        }))
        .merge(HostRedrawRequest::region_with_frame_update(FrameRect {
            x: 32.0,
            y: 0.0,
            width: 8.0,
            height: 10.0,
        }));
        let expected_damage = redraw.damage_region().cloned();
        let expected_metrics = redraw.damage_region_metrics();

        let retry = retry_present_request(UiPerfScenario::IdleHover, redraw);

        assert!(!retry.requires_frame_update());
        assert_eq!(retry.damage_region(), expected_damage.as_ref());
        assert_eq!(retry.damage_region_metrics(), expected_metrics);
        assert_eq!(retry.scenario(), UiPerfScenario::IdleHover);
    }

    #[test]
    fn successful_present_consumes_input_batch_but_retry_retains_it() {
        let source = include_str!("present.rs");
        let success = source
            .split("Ok(diagnostics) =>")
            .nth(1)
            .and_then(|source| {
                source
                    .split("Err(HostPresenterError::RetryableSurfacePresent)")
                    .next()
            })
            .expect("successful present branch");
        let retry = source
            .split("Err(HostPresenterError::RetryableSurfacePresent) =>")
            .nth(1)
            .and_then(|source| source.split("Err(error) =>").next())
            .expect("retryable present branch");

        assert!(success.contains("record_presented_input_batch(scenario)"));
        assert!(!retry.contains("record_presented_input_batch"));
    }

    #[test]
    fn first_present_notification_is_emitted_only_from_the_successful_present_branch() {
        let source = include_str!("present.rs");
        let success = source
            .split("Ok(diagnostics) =>")
            .nth(1)
            .and_then(|source| {
                source
                    .split("Err(HostPresenterError::RetryableSurfacePresent)")
                    .next()
            })
            .expect("successful present branch");
        let retry = source
            .split("Err(HostPresenterError::RetryableSurfacePresent) =>")
            .nth(1)
            .and_then(|source| source.split("Err(error) =>").next())
            .expect("retryable present branch");

        assert!(success.contains("notify_first_presented()"));
        assert!(!retry.contains("notify_first_presented"));
    }

    #[test]
    fn warmup_exports_source_bound_geometry_before_requesting_measurement_restart() {
        let source = include_str!("present.rs");
        let success = source
            .split("Ok(diagnostics) =>")
            .nth(1)
            .and_then(|source| {
                source
                    .split("Err(HostPresenterError::RetryableSurfacePresent)")
                    .next()
            })
            .expect("successful present branch");
        let artifacts = success
            .find("submit_present_artifacts(")
            .expect("warmup must publish source-bound geometry");
        let measurement = success
            .find("if profile_measurement_active")
            .expect("successful present must gate measured counters");
        let warmup_complete = success
            .find("complete_profile_warmup_present()")
            .expect("warmup completion must request a quiescent recorder restart");

        assert!(artifacts < measurement);
        assert!(measurement < warmup_complete);
        assert!(!success.contains("reset_capture"));
        assert!(!success.contains("start_capture_from_env"));
    }
}
