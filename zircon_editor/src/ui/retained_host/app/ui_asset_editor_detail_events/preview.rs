use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_preview_mock_detail(
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
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_preview_mock_nested_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        nested_key: &str,
        nested_value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "preview.mock.nested.value.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_preview_mock_nested_value(&instance_id, nested_key)
                .map(|_| ()),
            "preview.mock.nested.upsert" => self
                .editor_manager
                .upsert_ui_asset_editor_selected_preview_mock_nested_entry(
                    &instance_id,
                    nested_key,
                    nested_value,
                )
                .map(|_| ()),
            "preview.mock.nested.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_preview_mock_nested_entry(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset preview mock nested action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_preview_mock_suggestion_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "preview.mock.suggestion.apply" => self
                .editor_manager
                .apply_ui_asset_editor_selected_preview_mock_suggestion(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset preview mock suggestion action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
