use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::{write_error, write_log};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::profiling_artifacts::export_present_artifacts;
use crate::ui::retained_host::ui_perf::{UiPerfScenario, enter_ui_perf_scenario};

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
    let presentation = event_loop_state.host.get_host_presentation();
    let invalidation = event_loop_state.host.refresh_invalidation_diagnostics();
    match presenter.present(&presentation, damage_region, invalidation) {
        Ok(diagnostics) => {
            if let Some(backend) = event_loop_state.presenter_backend {
                export_present_artifacts(
                    &presentation,
                    &event_loop_state.host.window().size(),
                    backend,
                );
            }
            event_loop_state
                .host
                .set_host_refresh_diagnostics_overlay(diagnostics);
            if let Err(error) = event_loop_state.host.capture_first_presented_frame() {
                event_loop_state
                    .host
                    .record_first_presented_frame_capture_error(&error);
                write_error(
                    "editor_host_window",
                    format!("editor first-frame capture failed: {error}"),
                );
                event_loop.exit();
                return;
            }
            exit_after_presented_frame(
                event_loop_state.host.exit_after_first_presented_frame(),
                event_loop,
            );
        }
        Err(error) => {
            write_error(
                "editor_host_window",
                format!("presenter present failed: {error}"),
            );
            event_loop.exit();
        }
    }
}

fn exit_after_presented_frame(enabled: bool, event_loop: &dyn ActiveEventLoop) {
    if let Some(diagnostic) = first_presented_frame_diagnostic(enabled) {
        write_log("editor_host_window", diagnostic);
        event_loop.exit();
    }
}

fn first_presented_frame_diagnostic(enabled: bool) -> Option<&'static str> {
    enabled.then_some("editor_first_frame_presented")
}

#[cfg(test)]
mod tests {
    use super::first_presented_frame_diagnostic;

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
}
