use super::*;

impl EditorUiHost {
    pub fn undo_ui_asset_editor(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let replay = entry
            .session
            .undo_replay()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if replay.changed {
            if !replay.external_effects.is_empty() {
                let project_root = self.current_project_root()?.ok_or_else(|| {
                    EditorError::UiAsset(
                        "cannot apply ui asset undo side effects without an open project"
                            .to_string(),
                    )
                })?;
                let mut affected_asset_ids = Vec::new();
                for effect in &replay.external_effects {
                    affected_asset_ids
                        .push(self.apply_ui_asset_editor_external_effect(&project_root, effect)?);
                }
                self.refresh_ui_asset_workspace_for_changes(affected_asset_ids)?;
            }
            self.hydrate_ui_asset_editor_imports(instance_id)?;
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(replay.changed)
    }

    pub fn redo_ui_asset_editor(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let replay = entry
            .session
            .redo_replay()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if replay.changed {
            if !replay.external_effects.is_empty() {
                let project_root = self.current_project_root()?.ok_or_else(|| {
                    EditorError::UiAsset(
                        "cannot apply ui asset redo side effects without an open project"
                            .to_string(),
                    )
                })?;
                let mut affected_asset_ids = Vec::new();
                for effect in &replay.external_effects {
                    affected_asset_ids
                        .push(self.apply_ui_asset_editor_external_effect(&project_root, effect)?);
                }
                self.refresh_ui_asset_workspace_for_changes(affected_asset_ids)?;
            }
            self.hydrate_ui_asset_editor_imports(instance_id)?;
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(replay.changed)
    }

    pub fn set_ui_asset_editor_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiAssetEditorMode,
    ) -> Result<(), EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .set_mode(mode)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub fn select_ui_asset_editor_hierarchy_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .select_hierarchy_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub fn activate_ui_asset_editor_hierarchy_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.select_ui_asset_editor_hierarchy_index(instance_id, index)?;
        self.open_ui_asset_editor_selected_reference(instance_id)
    }

    pub fn select_ui_asset_editor_source_outline_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        entry
            .session
            .select_source_outline_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)
    }
}
