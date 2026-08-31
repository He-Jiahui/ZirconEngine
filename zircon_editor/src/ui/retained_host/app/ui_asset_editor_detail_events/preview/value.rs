use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::ui_asset_editor_detail_events) fn handle_ui_asset_preview_mock_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "preview.mock.value.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_preview_mock_value(&instance_id, value)
                .map(|_| ()),
            "preview.mock.clear" => self
                .editor_manager
                .clear_ui_asset_editor_selected_preview_mock_value(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset preview mock action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty_for_view(&instance_id),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
