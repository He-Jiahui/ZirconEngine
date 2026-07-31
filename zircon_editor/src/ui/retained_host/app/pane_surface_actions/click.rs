use super::super::*;
use super::routing::is_build_export_surface_action;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_pointer_component_event(
        &mut self,
        event: UiPointerComponentEvent,
    ) {
        self.focus_callback_source_window();
        let Some(template_action) = event.template_action else {
            return;
        };
        self.apply_dispatch_result(callback_dispatch::dispatch_template_action(
            &self.runtime,
            &template_action,
        ));
    }

    pub(in crate::ui::retained_host::app) fn dispatch_template_table_row_selected(
        &mut self,
        pane_id: &str,
        control_id: &str,
        source_index: i32,
        identity_kind: &str,
        identity_text: &str,
    ) {
        self.focus_callback_source_window();
        let selected = self.builtin_template_runtime.select_template_table_row(
            pane_id,
            control_id,
            source_index,
            identity_kind,
            identity_text,
        ) || self.component_showcase_runtime.select_template_table_row(
            pane_id,
            control_id,
            source_index,
            identity_kind,
            identity_text,
        );
        if selected {
            self.mark_presentation_dirty();
        }
    }

    pub(in crate::ui::retained_host::app) fn dispatch_pane_surface_control_clicked(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        let template_action_result = {
            let runtime = &self.runtime;
            self.builtin_template_runtime
                .dispatch_template_action_for_token(action_id, |action| {
                    callback_dispatch::dispatch_template_action(runtime, action)
                })
        }
        .or_else(|| {
            let runtime = &self.runtime;
            self.component_showcase_runtime
                .dispatch_template_action_for_token(action_id, |action| {
                    callback_dispatch::dispatch_template_action(runtime, action)
                })
        });
        if let Some(result) = template_action_result {
            self.apply_dispatch_result(result);
            return;
        }
        if self
            .builtin_template_runtime
            .is_template_action_token(action_id)
            || self
                .component_showcase_runtime
                .is_template_action_token(action_id)
        {
            return;
        }
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
