use super::*;
use crate::ui::workbench::view::ViewInstanceId;

mod fields;
mod lifecycle;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::ui_asset_editor_detail_events) fn handle_ui_asset_binding_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        if self.dispatch_ui_asset_binding_lifecycle_detail(&instance_id, action_id)
            || self.dispatch_ui_asset_binding_field_commit_detail(&instance_id, action_id, value)
        {
            return;
        }

        self.set_status_line(format!("Unknown UI asset binding action {action_id}"));
    }
}
