use super::super::*;
use super::routing::is_build_export_surface_action;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_pane_surface_control_clicked(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        if let Some(result) =
            self.dispatch_componentized_workbench_surface_control(control_id, action_id)
        {
            self.apply_dispatch_result(result);
            return;
        }
        if control_id == "ModulePluginAction" {
            self.dispatch_module_plugin_action(action_id);
            return;
        }
        if is_build_export_surface_action(control_id, action_id) {
            self.dispatch_build_export_surface_action(control_id, action_id);
            return;
        }
        if control_id == profiling::PERFORMANCE_TIMELINE_ACTION_CONTROL_ID {
            self.dispatch_performance_timeline_action(action_id);
            return;
        }
        let Some(result) = callback_dispatch::dispatch_builtin_pane_surface_control(
            &self.runtime,
            &self.pane_surface_bridge,
            control_id,
            UiEventKind::Click,
            vec![UiBindingValue::string(action_id)],
        ) else {
            if let Some(result) =
                callback_dispatch::dispatch_builtin_template_binding(&self.runtime, action_id)
            {
                self.apply_dispatch_result(result);
                return;
            }
            if !action_id.is_empty() {
                self.apply_dispatch_result(callback_dispatch::dispatch_menu_action(
                    &self.runtime,
                    action_id,
                ));
                return;
            }
            self.set_status_line(format!("Unknown pane surface control {control_id}"));
            return;
        };

        self.apply_dispatch_result(result);
    }
}
