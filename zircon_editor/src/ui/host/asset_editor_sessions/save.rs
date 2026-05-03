use std::fs;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::project_access::normalize_ui_asset_asset_id;

impl EditorUiHost {
    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (saved, asset_id, source_path) = {
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let saved = entry
                .session
                .save_to_canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (
                saved,
                entry.session.route().asset_id.clone(),
                entry.source_path.clone(),
            )
        };
        fs::write(&source_path, &saved).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        {
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.update_disk_baseline(saved.clone());
            entry.conflict = None;
            entry.diff_snapshot = None;
        }
        if asset_id.starts_with("res://") {
            let normalized = normalize_ui_asset_asset_id(&asset_id).to_string();
            let _ = self.asset_manager()?.import_asset(&normalized);
            self.refresh_ui_asset_workspace_for_changes(vec![normalized])?;
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(saved)
    }
}
