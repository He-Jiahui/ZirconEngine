use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_binding_lifecycle_detail(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> bool {
        let result = match action_id {
            "binding.add" => self
                .editor_manager
                .add_ui_asset_editor_binding(instance_id)
                .map(|_| ()),
            "binding.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_binding(instance_id)
                .map(|_| ()),
            _ => return false,
        };

        match result {
            Ok(()) => self.mark_presentation_dirty_for_view(instance_id),
            Err(error) => self.set_status_line(error.to_string()),
        }
        true
    }
}
