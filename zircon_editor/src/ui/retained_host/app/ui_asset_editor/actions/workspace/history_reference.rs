use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_workspace_history_reference_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "undo" => UiAssetActionDispatch::handled(
                self.editor_manager.undo_ui_asset_editor(instance_id),
            ),
            "redo" => UiAssetActionDispatch::handled(
                self.editor_manager.redo_ui_asset_editor(instance_id),
            ),
            "reference.open" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .open_ui_asset_editor_selected_reference(instance_id),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
