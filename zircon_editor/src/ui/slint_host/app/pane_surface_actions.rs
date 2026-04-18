use super::*;

impl SlintEditorHost {
    pub(super) fn dispatch_pane_surface_control_clicked(
        &mut self,
        control_id: &str,
        action_id: &str,
    ) {
        self.focus_callback_source_window();
        let Some(result) = callback_dispatch::dispatch_builtin_pane_surface_control(
            &self.runtime,
            &self.pane_surface_bridge,
            control_id,
            UiEventKind::Click,
            vec![UiBindingValue::string(action_id)],
        ) else {
            self.set_status_line(format!("Unknown pane surface control {control_id}"));
            return;
        };

        self.apply_dispatch_result(result);
    }
}
