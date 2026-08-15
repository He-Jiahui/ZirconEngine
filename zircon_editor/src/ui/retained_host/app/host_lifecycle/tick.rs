use super::super::*;
use crate::ui::retained_host::ui_perf::{
    enter_ui_perf_scenario, time_ui_perf_scenario, UiPerfScenario,
};
use std::time::Instant;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn tick(&mut self) {
        zircon_runtime::profile_frame!("editor", "retained_host_tick");
        zircon_runtime::profile_scope!("editor", "retained_host", "tick");
        self.pump_editor_job_events();
        self.poll_welcome_project_probe();
        self.poll_desktop_export_jobs();
        self.poll_desktop_export_wizard_sessions();
        self.sync_editor_job_progress();
        if let Err(error) = self.runtime.pump_plugin_lifecycle_messages() {
            self.set_status_line(error);
        }
        self.runtime.update_scene_modes();
        if let Err(error) = self.ensure_hierarchy_world_watch() {
            self.set_status_line(error);
        }
        self.pump_edit_world_invalidations();
        self.consume_scene_hierarchy_fragment();
        match self.runtime.pump_runtime_event_consumers() {
            Ok(frame_demand) => self
                .ui
                .apply_runtime_frame_demand(frame_demand, Instant::now()),
            Err(error) => self.set_status_line(error.to_string()),
        }
        if let Err(error) = self.sync_plugin_template_documents_if_changed() {
            self.set_status_line(error.to_string());
        }
        self.sync_activity_notifications();
        self.sync_settings_projections();

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
        let lifecycle_pump = production
            .find("self.runtime.pump_plugin_lifecycle_messages()")
            .expect("retained tick should pump plugin lifecycle message subscriptions");
        assert!(pump < export_poll);
        assert!(export_poll < wizard_poll);
        assert!(wizard_poll < progress_sync);
        assert!(progress_sync < lifecycle_pump);
    }

    #[test]
    fn retained_tick_projects_the_unified_notification_snapshot_after_backend_polling() {
        let source = include_str!("tick.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("tick source should contain its production section");
        let backend_poll = production
            .find("self.runtime.pump_runtime_event_consumers()")
            .expect("retained tick should poll the play backend");
        let lifecycle_pump = production
            .find("self.runtime.pump_plugin_lifecycle_messages()")
            .expect("retained tick should pump plugin lifecycle message subscriptions before backend polling");
        let template_sync = production
            .find("self.sync_plugin_template_documents_if_changed()")
            .expect("retained tick should synchronize plugin templates after backend polling");
        let toast_sync = production
            .find("self.sync_activity_notifications();")
            .expect("retained tick should project the unified notification authority");
        let settings_sync = production
            .find("self.sync_settings_projections();")
            .expect("retained tick should synchronize authority-owned settings projections");
        let recompute = production
            .find("self.recompute_if_dirty();")
            .expect("retained tick should recompute invalidated presentation");
        assert!(lifecycle_pump < backend_poll);
        assert!(backend_poll < template_sync);
        assert!(template_sync < toast_sync);
        assert!(toast_sync < settings_sync);
        assert!(settings_sync < recompute);
    }

    #[test]
    fn retained_tick_consumes_edit_world_hierarchy_invalidations_before_other_runtime_consumers() {
        let source = include_str!("tick.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("tick source should contain its production section");
        let scene_modes = production
            .find("self.runtime.update_scene_modes();")
            .expect("retained tick should publish edit-world scene changes");
        let watch = production
            .find("self.ensure_hierarchy_world_watch()")
            .expect("retained tick should restore a generation-safe hierarchy watch");
        let invalidation_pump = production
            .find("self.pump_edit_world_invalidations();")
            .expect("retained tick should pump edit-world invalidations");
        let fragment = production
            .find("self.consume_scene_hierarchy_fragment();")
            .expect("retained tick should consume the newest retained hierarchy fragment");
        let runtime_consumers = production
            .find("self.runtime.pump_runtime_event_consumers()")
            .expect("retained tick should pump other runtime event consumers");
        assert!(scene_modes < watch);
        assert!(watch < invalidation_pump);
        assert!(invalidation_pump < fragment);
        assert!(fragment < runtime_consumers);
    }

    #[test]
    fn retained_tick_consumes_resize_render_work_after_the_active_recompute() {
        let source = include_str!("tick.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("tick source should contain its production section");
        let recompute = production
            .find("self.recompute_if_dirty();")
            .expect("window metrics and viewport projection recompute");
        let render = production
            .find("self.submit_render_frame_if_dirty();")
            .expect("render reason consumer");

        assert!(recompute < render);
    }
}
