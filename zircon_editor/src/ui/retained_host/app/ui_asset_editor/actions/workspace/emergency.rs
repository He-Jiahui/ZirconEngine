use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::host::EditorError;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_workspace_emergency_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "emergency.revert_last_valid" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| manager.revert_ui_asset_editor_to_last_valid(instance_id)),
            ),
            "emergency.open_asset_browser" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.open_view(ViewDescriptorId::new("editor.asset_browser"), None)
                    }),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
