use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_source_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match action_id {
            "source.text.set" => match self
                .editor_manager
                .update_ui_asset_editor_source(&instance_id, value.to_string())
            {
                Ok(()) => self.mark_presentation_dirty_for_view(&instance_id),
                Err(error) => self.set_status_line(error.to_string()),
            },
            "source.cursor.set" => match self
                .editor_manager
                .select_ui_asset_editor_source_byte_offset(&instance_id, item_index.max(0) as usize)
            {
                Ok(changed) => {
                    if changed {
                        self.mark_presentation_dirty_for_view(&instance_id);
                    }
                }
                Err(error) => self.set_status_line(error.to_string()),
            },
            other => {
                self.set_status_line(format!("Unknown UI asset source action {other}"));
            }
        }
    }
}
