use winit::event_loop::ActiveEventLoop;

use crate::core::jobs::JobId;
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::presenter::HostPresenterError;
use crate::ui::retained_host::host_contract::profiling_artifacts::{
    profile_capture_enabled, profile_dynamic_capture_enabled, submit_present_artifacts,
    ProfileArtifactSubmissionError,
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
            if let Some(backend) = event_loop_state.presenter_backend.filter(|backend| {
                profile_capture_enabled()
                    && if profile_dynamic_capture_enabled() {
                        backend.is_gpu()
                    } else {
                        !event_loop_state.profile_artifact_capture_requested
                    }
            }) {
                if !profile_dynamic_capture_enabled() {
                    event_loop_state.profile_artifact_capture_requested = true;
                }
                let capture_sequence = if profile_dynamic_capture_enabled() {
                    event_loop_state.profile_capture_sequence =
                        event_loop_state.profile_capture_sequence.saturating_add(1);
                    event_loop_state.profile_capture_sequence
                } else {
                    0
                };
                match event_loop_state.host.profile_artifact_jobs() {
                    Some(jobs) => {
                        let window = event_loop_state.host.window();
                        let window_size = window.size();
                        let submitted = submit_present_artifacts(
                            &jobs,
                            &window_size,
                            window.scale_factor(),
                            capture_sequence,
                            backend,
                            event_loop_state
                                .presenter
                                .as_ref()
                                .and_then(|presenter| presenter.submitted_text_profile()),
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
#[path = "tests/present_profile_artifact_submission_tests.rs"]
mod profile_artifact_submission_tests;

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
#[path = "tests/present.rs"]
mod tests;
