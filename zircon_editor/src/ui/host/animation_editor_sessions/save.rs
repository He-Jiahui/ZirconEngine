use std::path::Path;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorUiHost {
    pub fn save_animation_editor(&self, instance_id: &ViewInstanceId) -> Result<(), EditorError> {
        self.ensure_animation_editor_session(instance_id)?;
        let source_path = {
            let mut sessions = self.animation_editor_sessions.lock().unwrap();
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
            entry.source_path.clone()
        };
        if let Some(asset_id) = self.animation_asset_id_for_source_path(&source_path)? {
            let _ = self.asset_manager()?.import_asset(&asset_id);
        }
        self.sync_animation_editor_instance(instance_id)
    }

    fn animation_asset_id_for_source_path(
        &self,
        source_path: &Path,
    ) -> Result<Option<String>, EditorError> {
        let Some(project_root) = self.current_project_root()? else {
            return Ok(None);
        };
        let assets_root = project_root.join("assets");
        let Ok(relative) = source_path.strip_prefix(&assets_root) else {
            return Ok(None);
        };
        let normalized = relative
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        Ok(Some(format!("res://{normalized}")))
    }
}
