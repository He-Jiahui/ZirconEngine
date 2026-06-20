use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::ui_asset_editor_detail_events) fn handle_ui_asset_preview_mock_suggestion_detail(
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
