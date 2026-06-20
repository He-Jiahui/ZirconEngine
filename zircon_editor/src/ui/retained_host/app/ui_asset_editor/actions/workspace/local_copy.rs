use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_workspace_local_copy_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        if action_id != "workspace.save_local_copy" {
            return UiAssetActionDispatch::Unhandled;
        }

        match self
            .editor_manager
            .save_ui_asset_editor_local_copy_next_to_source(instance_id)
        {
            Ok(path) => {
                self.set_status_line(format!("Saved UI asset local copy {}", path.display()));
                UiAssetActionDispatch::Handled(Ok(()))
            }
            Err(error) => UiAssetActionDispatch::Handled(Err(error)),
        }
    }
}
