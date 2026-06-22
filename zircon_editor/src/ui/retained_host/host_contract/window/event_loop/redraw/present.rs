use winit::event_loop::ActiveEventLoop;
use zircon_runtime::diagnostic_log::write_error;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::profiling_artifacts::export_present_artifacts;
use crate::ui::retained_host::ui_perf::{enter_ui_perf_scenario, UiPerfScenario};

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
