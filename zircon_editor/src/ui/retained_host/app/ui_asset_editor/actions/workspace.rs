use super::{UiAssetActionDispatch, *};
use crate::ui::workbench::view::ViewInstanceId;

mod diff;
mod emergency;
mod history_reference;
mod local_copy;
mod save;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_workspace_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        let mut dispatch = self.dispatch_ui_asset_workspace_save_action(instance_id, action_id);
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_workspace_local_copy_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_workspace_diff_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_workspace_emergency_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch =
                self.dispatch_ui_asset_workspace_history_reference_action(instance_id, action_id);
        }

        dispatch
    }
}
