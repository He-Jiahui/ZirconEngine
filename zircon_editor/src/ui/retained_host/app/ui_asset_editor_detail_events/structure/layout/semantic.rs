use super::super::semantic_paths::layout_semantic_action_path;
use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_layout_semantic_detail(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
        value: &str,
    ) -> bool {
        match action_id {
            "layout.semantic.value.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.semantic.value",
                    value,
                );
            }
            "layout.semantic.delete" => {
                match self
                    .editor_manager
                    .delete_ui_asset_editor_selected_layout_semantic(instance_id)
                    .map(|_| ())
                {
                    Ok(()) => self.mark_presentation_dirty(),
                    Err(error) => self.set_status_line(error.to_string()),
                }
            }
            other => {
                let Some(path) = layout_semantic_action_path(other) else {
                    return false;
                };
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    &format!("layout.semantic.field.{path}"),
                    value,
                );
            }
        }

        true
    }
}
