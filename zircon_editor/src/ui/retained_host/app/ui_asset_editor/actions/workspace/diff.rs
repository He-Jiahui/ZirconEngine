use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_workspace_diff_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        if action_id != "workspace.diff_snapshot" {
            return UiAssetActionDispatch::Unhandled;
        }

        match self
            .editor_manager
            .open_ui_asset_editor_diff_snapshot(instance_id)
        {
            Ok(Some(snapshot)) => {
                self.set_status_line(snapshot.summary);
                UiAssetActionDispatch::Handled(Ok(()))
            }
            Ok(None) => {
                self.set_status_line("No UI asset conflict diff available".to_string());
                UiAssetActionDispatch::Handled(Ok(()))
            }
            Err(error) => UiAssetActionDispatch::Handled(Err(error)),
        }
    }
}
