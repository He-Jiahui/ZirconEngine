use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::ui_asset_editor_detail_events) fn handle_ui_asset_binding_payload_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        payload_key: &str,
        payload_value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "binding.payload.upsert" => self
                .editor_manager
                .upsert_ui_asset_editor_selected_binding_payload(
                    &instance_id,
                    payload_key,
                    payload_value,
                )
                .map(|_| ()),
            "binding.payload.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_binding_payload(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset binding payload action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
