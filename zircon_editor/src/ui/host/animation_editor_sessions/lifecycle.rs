use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::animation_editor::{AnimationEditorPanePresentation, AnimationEditorSession};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

use super::AnimationEditorWorkspaceEntry;

impl EditorUiHost {
    pub fn animation_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<AnimationEditorPanePresentation, EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        let sessions = self.lock_animation_editor_sessions();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!(
                "missing animation editor session {}",
                instance_id.0
            ))
        })?;
        Ok(entry.session.pane_presentation())
    }

    pub(crate) fn restore_animation_editor_instance(
        &self,
        instance: &ViewInstance,
    ) -> Result<(), EditorError> {
        let source_path = instance
            .serializable_payload
            .get("path")
            .and_then(|value| value.as_str())
            .ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "invalid animation editor route for {}",
                    instance.instance_id.0
                ))
            })?;
        let session = AnimationEditorSession::from_path(std::path::Path::new(source_path))
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        self.lock_animation_editor_sessions().insert(
            instance.instance_id.clone(),
            AnimationEditorWorkspaceEntry {
                source_path: std::path::PathBuf::from(source_path),
                session,
            },
        );
        self.sync_animation_editor_instance(&instance.instance_id)
    }

    pub(super) fn ensure_animation_editor_session(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        if self
            .lock_animation_editor_sessions()
            .contains_key(instance_id)
        {
            return Ok(());
        }
        let instance = self
            .lock_session()
            .open_view_instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                EditorError::UiAsset(format!("missing animation editor view {}", instance_id.0))
            })?;
        self.restore_animation_editor_instance(&instance)
    }
}
