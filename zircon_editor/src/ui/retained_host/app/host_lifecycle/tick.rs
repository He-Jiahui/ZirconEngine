use super::super::*;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn tick(&mut self) {
        zircon_runtime::profile_frame!("editor", "retained_host_tick");
        zircon_runtime::profile_scope!("editor", "retained_host", "tick");
        self.poll_desktop_export_jobs();
        self.poll_desktop_export_wizard_sessions();

        {
            let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::AssetRefresh);
            let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::AssetRefresh);
            if let Err(error) = self.refresh_project_assets() {
                self.set_status_line(error);
            }
        }

        {
            let frame_scenario = self.pending_ui_perf_scenario.take();
            let _frame_scenario_guard = frame_scenario.map(enter_ui_perf_scenario);
            if let Some(scenario) = frame_scenario {
                self.ui.mark_completed_frame_update_scenario(scenario);
            }

            self.sync_shell_size();
            self.recompute_if_dirty();
            self.submit_render_frame_if_dirty();
        }

        {
            let _ui_perf_scenario = enter_ui_perf_scenario(UiPerfScenario::ViewportImage);
            let _ui_perf_timer = time_ui_perf_scenario(UiPerfScenario::ViewportImage);
            self.poll_viewport_image_for_native_host();
        }
        if let Some(error) = self.viewport.take_error() {
            self.set_status_line(error);
            self.recompute_if_dirty();
        }
    }

    pub(in crate::ui::retained_host::app) fn refresh_ui(&mut self) {
        self.recompute_if_dirty();
    }

    pub(in crate::ui::retained_host::app) fn use_committed_pointer_layout(&self) {
        // Pointer routing must stay on the last committed bridge frames. Dirty
        // presentation/layout state is consumed by tick/refresh instead of
        // rebuilding the whole editor tree inside native pointer callbacks.
        self.publish_refresh_invalidation_diagnostics();
    }
}
