use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_workspace_save_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "save" => UiAssetActionDispatch::handled(
                self.editor_manager.save_ui_asset_editor(instance_id),
            ),
            "workspace.reload_from_disk" | "emergency.reload_from_disk" => {
                UiAssetActionDispatch::handled(
                    self.editor_manager
                        .reload_ui_asset_editor_from_disk(instance_id),
                )
            }
            "workspace.keep_local_and_save" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .keep_ui_asset_editor_local_and_save(instance_id),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
