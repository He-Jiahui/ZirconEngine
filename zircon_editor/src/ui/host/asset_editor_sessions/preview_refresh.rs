use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::asset_editor::UiAssetPreviewPreset;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub fn set_ui_asset_editor_preview_preset(
        &self,
        instance_id: &ViewInstanceId,
        preview_preset: UiAssetPreviewPreset,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_preview_preset(preview_preset)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_preview_index(
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
            .select_preview_index(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)
    }

    pub fn activate_ui_asset_editor_preview_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.select_ui_asset_editor_preview_index(instance_id, index)?;
        self.open_ui_asset_editor_selected_reference(instance_id)
    }

    pub fn select_ui_asset_editor_preview_mock_property(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_preview_mock_property(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_preview_mock_subject(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_preview_mock_subject(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_preview_mock_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_preview_mock_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn select_ui_asset_editor_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_preview_mock_nested_entry(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn set_ui_asset_editor_selected_preview_mock_nested_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .set_selected_preview_mock_nested_value(value.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn upsert_ui_asset_editor_selected_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
        key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .upsert_selected_preview_mock_nested_entry(key.as_ref(), value_literal.as_ref())
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn apply_ui_asset_editor_selected_preview_mock_suggestion(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .apply_selected_preview_mock_suggestion(index)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn delete_ui_asset_editor_selected_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .delete_selected_preview_mock_nested_entry()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }

    pub fn clear_ui_asset_editor_selected_preview_mock_value(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .clear_selected_preview_mock_value()
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(changed)
    }
}
