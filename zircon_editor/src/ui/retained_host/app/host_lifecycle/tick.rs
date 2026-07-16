use super::super::*;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn tick(&mut self) {
        zircon_runtime::profile_frame!("editor", "retained_host_tick");
        zircon_runtime::profile_scope!("editor", "retained_host", "tick");
        self.pump_editor_job_events();
        self.poll_desktop_export_jobs();
        self.poll_desktop_export_wizard_sessions();
        self.sync_editor_job_progress();
        if let Err(error) = self.runtime.pump_runtime_event_consumers() {
            self.set_status_line(error.to_string());
        }

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

    // Retained host tick owns publishing worker job events into the editor bus.
    fn pump_editor_job_events(&self) {
        self.editor_manager.context().jobs().pump_events();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn retained_tick_owns_the_single_editor_job_event_pump_call() {
        let source = include_str!("tick.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("tick source should contain its production section");
        let pump_call = [".jobs()", ".pump_events()"].concat();
        assert_eq!(production.matches(pump_call.as_str()).count(), 1);
        let pump = production
            .find("self.pump_editor_job_events();")
            .expect("retained tick should pump editor job events");
        let export_poll = production
            .find("self.poll_desktop_export_jobs();")
            .expect("retained tick should poll export jobs");
        let wizard_poll = production
            .find("self.poll_desktop_export_wizard_sessions();")
            .expect("retained tick should poll export wizard sessions");
        let progress_sync = production
            .find("self.sync_editor_job_progress();")
            .expect("retained tick should project the unified job progress source");
        assert!(pump < export_poll);
        assert!(export_poll < wizard_poll);
        assert!(wizard_poll < progress_sync);
    }
}
