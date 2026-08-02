use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::core::asset::DirtyExternalEffectId;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub(super) fn sync_ui_asset_editor_instance(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (title, dirty) = {
            let sessions = self.lock_ui_asset_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let reflection = entry.session.reflection_model();
            (reflection.display_name, reflection.source_dirty)
        };
        let dirty = if dirty {
            self.ensure_document_external_effect(
                instance_id,
                DirtyExternalEffectId::ui_source_buffer(),
            )?;
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
mod tests {
    #[test]
    fn syncing_instance_builds_one_reflection_model() {
        let source = include_str!("sync.rs");
        let reflection_build = ["session.", "reflection_model()"].concat();

        assert_eq!(source.matches(&reflection_build).count(), 1);
    }
}
