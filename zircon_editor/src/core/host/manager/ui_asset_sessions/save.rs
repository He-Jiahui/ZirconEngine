use std::fs;

use crate::view::ViewInstanceId;
use crate::{EditorError, EditorManager};

use super::super::project_access::normalize_ui_asset_asset_id;

impl EditorManager {
    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let (saved, asset_id, source_path) = {
            let mut sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get_mut(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            let saved = entry
                .session
                .save_to_canonical_source()
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            (
                saved,
                entry.session.route().asset_id.clone(),
                entry.source_path.clone(),
            )
        };
        fs::write(&source_path, &saved).map_err(|error| EditorError::UiAsset(error.to_string()))?;
        if asset_id.starts_with("res://") {
            let normalized = normalize_ui_asset_asset_id(&asset_id).to_string();
            let _ = self.asset_manager()?.import_asset(&normalized);
        }
        self.hydrate_ui_asset_editor_imports(instance_id)?;
        self.sync_ui_asset_editor_instance(instance_id)?;
        Ok(saved)
    }
}
