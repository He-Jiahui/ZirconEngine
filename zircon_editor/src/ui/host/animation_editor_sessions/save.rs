use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub fn save_animation_editor(&self, instance_id: &ViewInstanceId) -> Result<(), EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        let asset_locator = {
            let mut sessions = self.lock_animation_editor_sessions();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!(
                    "missing animation editor session {}",
                    instance_id.0
                ))
            })?;
            entry
                .session
                .save()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            entry.route.asset_locator().to_string()
        };
        let _ = self.asset_manager()?.import_asset(&asset_locator);
        self.sync_animation_editor_instance(instance_id)
    }
}
