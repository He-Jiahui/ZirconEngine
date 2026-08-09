use winit::event_loop::ActiveEventLoop;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::diagnostics::HostWindowDiagnosticSeverity;
use crate::ui::retained_host::host_contract::profiling_artifacts::{
    profile_capture_enabled, queue_present_artifacts,
};
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, record_current_ui_perf_counter, UiPerfCounter, UiPerfScenario,
};

use super::super::UiHostWindowEventLoop;

pub(super) fn present_redraw(
    event_loop_state: &mut UiHostWindowEventLoop,
    event_loop: &dyn ActiveEventLoop,
    damage_region: Option<FrameRect>,
    scenario: UiPerfScenario,
) {
    let _present_scenario_guard = enter_ui_perf_scenario(scenario);
    let Some(presenter) = event_loop_state.presenter.as_mut() else {
        return;
    };
    let generation = event_loop_state.host.get_host_presentation_generation();
    let _paint_scope = generation.enter_paint_scope();
    let presentation = generation.structure();
    let invalidation = event_loop_state.host.refresh_invalidation_diagnostics();
    #[cfg(feature = "profiling")]
    let damage_started_at = event_loop_state.pending_damage_started_at.take();
    let present_result = if event_loop_state.host.native_resize_reflow_pending() {
        presenter.present_during_native_resize(presentation, invalidation)
    } else {
        presenter.present(presentation, damage_region, invalidation)
    };
    match present_result {
        Ok(diagnostics) => {
            #[cfg(feature = "profiling")]
            if let Some(started_at) = damage_started_at {
                record_current_ui_perf_counter(
                    UiPerfCounter::DamageToSubmitUs,
                    started_at.elapsed().as_secs_f64() * 1_000_000.0,
                );
            }
            if let Some(backend) = event_loop_state.presenter_backend.filter(|_| {
                should_queue_profile_artifacts(
                    profile_capture_enabled(),
                    event_loop_state.profile_artifact_capture_requested,
                )
            }) {
                event_loop_state.profile_artifact_capture_requested = true;
                let artifact_presentation = generation.materialize();
                if queue_present_artifacts(
                    &artifact_presentation,
                    &event_loop_state.host.window().size(),
                    backend,
                ) {
                    record_current_ui_perf_counter(UiPerfCounter::ArtifactExportCount, 1.0);
                }
            }
            event_loop_state
                .host
                .set_host_refresh_diagnostics_overlay(diagnostics);
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

fn should_queue_profile_artifacts(capture_enabled: bool, already_requested: bool) -> bool {
    capture_enabled && !already_requested
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
    use super::{first_presented_frame_diagnostic, should_queue_profile_artifacts};

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
    fn profile_artifacts_are_explicit_and_one_shot() {
        assert!(!should_queue_profile_artifacts(false, false));
        assert!(should_queue_profile_artifacts(true, false));
        assert!(!should_queue_profile_artifacts(true, true));
    }
}
