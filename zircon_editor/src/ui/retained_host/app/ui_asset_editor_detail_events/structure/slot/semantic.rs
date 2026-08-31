use super::super::semantic_paths::slot_semantic_action_path;
use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_slot_semantic_detail(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
        value: &str,
    ) -> bool {
        match action_id {
            "slot.semantic.value.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.semantic.value",
                    value,
                );
            }
            "slot.semantic.delete" => {
                match self
                    .editor_manager
                    .delete_ui_asset_editor_selected_slot_semantic(instance_id)
                    .map(|_| ())
                {
                    Ok(()) => self.mark_presentation_dirty_for_view(instance_id),
                    Err(error) => self.set_status_line(error.to_string()),
                }
            }
            other => {
                let Some(path) = slot_semantic_action_path(other) else {
                    return false;
                };
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    &format!("slot.semantic.field.{path}"),
                    value,
                );
            }
        }

        true
    }
}
