#[cfg(test)]
use super::asset_editor_sessions::UiAssetDiffSnapshot;
use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn reload_ui_asset_editor_from_disk(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host.reload_ui_asset_editor_from_disk(instance_id)
    }

    pub fn keep_ui_asset_editor_local_and_save(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.host.keep_ui_asset_editor_local_and_save(instance_id)
    }

    #[cfg(test)]
    pub(crate) fn open_ui_asset_editor_diff_snapshot(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<UiAssetDiffSnapshot>, EditorError> {
        self.host.open_ui_asset_editor_diff_snapshot(instance_id)
    }

    pub fn refresh_ui_asset_workspace_for_changes(
        &self,
        changed_asset_ids: impl IntoIterator<Item = String>,
    ) -> Result<(), EditorError> {
        self.host
            .refresh_ui_asset_workspace_for_changes(changed_asset_ids)
    }

    pub fn poll_ui_asset_workspace_watcher(&self) -> Result<Vec<String>, EditorError> {
        self.host.poll_ui_asset_workspace_watcher()
    }
}
