use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::core::asset::DirtyExternalEffectId;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub(super) fn sync_ui_asset_editor_instance(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (title, dirty, source_revision, reported_revision) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let reflection = entry.session.reflection_model();
            (
                reflection.display_name,
                reflection.source_dirty,
                entry.session.source_revision(),
                entry.reported_dirty_source_revision,
            )
        };
        let dirty = if dirty {
            if reported_revision == Some(source_revision) {
                self.ensure_document_external_effect(
                    instance_id,
                    DirtyExternalEffectId::ui_source_buffer(),
                )?;
            } else {
                self.mark_document_external_effect(
                    instance_id,
                    DirtyExternalEffectId::ui_source_buffer(),
                )?;
                let mut sessions = self.lock_ui_asset_sessions();
                if let Some(entry) = sessions.get_mut(instance_id) {
                    if entry.session.source_revision() == source_revision {
                        entry.reported_dirty_source_revision = Some(source_revision);
                    }
                }
            }
            self.document_dirty(instance_id)?
        } else {
            self.document_dirty_if_registered(instance_id)?
                .unwrap_or(false)
        };
        let mut session = self.lock_session();
        let instance = session
            .open_view_instances
            .get_mut(instance_id)
            .ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset view {}", instance_id.0))
            })?;
        instance.title = title;
        instance.dirty = dirty;
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/sync.rs"]
mod tests;
