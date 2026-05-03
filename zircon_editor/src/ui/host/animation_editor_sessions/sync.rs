use serde_json::json;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub(super) fn sync_animation_editor_instance(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        let (title, dirty, payload) = {
            let sessions = self.lock_animation_editor_sessions();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing animation editor session {}",
                    instance_id.0
                ))
            })?;
            (
                entry.session.display_name(),
                entry.session.is_dirty(),
                json!({ "path": entry.session.asset_path() }),
            )
        };
        self.update_view_instance_metadata(instance_id, Some(title), Some(dirty), Some(payload))
    }
}
