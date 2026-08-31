use std::fs;

use crate::ui::asset_editor::UiAssetEditorSession;
use crate::ui::host::editor_error::EditorError;
use crate::ui::host::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

use super::super::{ui_asset_source_digest, UiAssetDiffSnapshot};
use super::normalize::rebuild_ui_asset_session_from_source;

impl EditorUiHost {
    pub fn reload_ui_asset_editor_from_disk(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (source_path, route) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            (entry.source_path.clone(), entry.session.route().clone())
        };
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let session = rebuild_ui_asset_session_from_source(route, source.clone())?;
        self.replace_ui_asset_session_from_disk(instance_id, session, source)?;
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(true)
    }

    pub fn keep_ui_asset_editor_local_and_save(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.save_ui_asset_editor(instance_id)
    }

    pub fn open_ui_asset_editor_diff_snapshot(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<UiAssetDiffSnapshot>, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        if entry.diff_snapshot.is_none() {
            entry.diff_snapshot = entry.conflict.as_ref().map(UiAssetDiffSnapshot::from);
        }
        Ok(entry.diff_snapshot.clone())
    }

    pub fn revert_ui_asset_editor_to_last_valid(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let changed = {
            let mut sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry
                .session
                .revert_source_to_last_valid()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?
        };
        if changed {
            self.hydrate_ui_asset_editor_imports(instance_id)?;
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub(super) fn replace_ui_asset_session_from_disk(
        &self,
        instance_id: &ViewInstanceId,
        session: UiAssetEditorSession,
        source: String,
    ) -> Result<(), EditorError> {
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry.session = session;
        entry.disk_source_digest = ui_asset_source_digest(&source);
        entry.disk_source = source;
        entry.conflict = None;
        entry.diff_snapshot = None;
        entry.stale_imports.clear();
        Ok(())
    }
}
