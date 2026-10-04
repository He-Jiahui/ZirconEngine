use super::*;
use crate::core::asset::DirtyExternalEffectId;

impl EditorUiHost {
    pub fn select_ui_asset_editor_source_byte_offset(
        &self,
        instance_id: &ViewInstanceId,
        byte_offset: usize,
    ) -> Result<bool, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let changed = entry
            .session
            .select_source_byte_offset(byte_offset)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        drop(sessions);
        if changed {
            self.sync_ui_asset_editor_instance(instance_id)?;
        }
        Ok(changed)
    }

    pub fn update_ui_asset_editor_source(
        &self,
        instance_id: &ViewInstanceId,
        next_source: impl Into<String>,
    ) -> Result<(), EditorError> {
        let next_source = next_source.into();
        self.ensure_ui_asset_editor_session(instance_id)?;
        let _edit = self.begin_document_edit(instance_id)?;
        let mut sessions = self.lock_ui_asset_sessions();
        let entry = sessions.get_mut(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let before_revision = entry.session.source_revision();
        let applied = entry
            .session
            .apply_command(UiAssetEditorCommand::edit_source(next_source));
        let source_revision = entry.session.source_revision();
        let source_dirty = entry.session.source_buffer().is_dirty();
        drop(sessions);
        if source_dirty {
            if source_revision != before_revision {
                self.mark_document_external_effect(
                    instance_id,
                    DirtyExternalEffectId::ui_source_buffer(),
                )?;
                if let Some(entry) = self.lock_ui_asset_sessions().get_mut(instance_id) {
                    if entry.session.source_revision() == source_revision {
                        entry.reported_dirty_source_revision = Some(source_revision);
                    }
                }
            } else {
                self.ensure_document_external_effect(
                    instance_id,
                    DirtyExternalEffectId::ui_source_buffer(),
                )?;
            }
        }
        applied.map_err(|error| EditorError::UiAsset(error.to_string()))?;
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)
    }
}
