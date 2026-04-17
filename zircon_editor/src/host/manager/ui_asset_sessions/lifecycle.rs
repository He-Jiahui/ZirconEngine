use std::fs;

use crate::view::{ViewInstance, ViewInstanceId};
use crate::{
    EditorError, EditorManager, UiAssetEditorMode, UiAssetEditorPanePresentation,
    UiAssetEditorReflectionModel, UiAssetEditorRoute, UiAssetEditorSession,
};
use zircon_ui::UiAssetLoader;

use super::{preview_size_for_preset, UiAssetWorkspaceEntry};

impl EditorManager {
    pub fn ui_asset_editor_reflection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorReflectionModel, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        Ok(entry.session.reflection_model())
    }

    pub fn ui_asset_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorPanePresentation, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        Ok(entry.session.pane_presentation())
    }

    pub fn open_ui_asset_editor_selected_reference(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let selected_reference = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.selected_reference_asset_id()
        };
        let Some(selected_reference) = selected_reference else {
            return Ok(None);
        };
        self.open_ui_asset_editor_by_id(selected_reference, None)
            .map(Some)
    }

    pub(super) fn restore_ui_asset_editor_instance(
        &self,
        instance: &ViewInstance,
    ) -> Result<(), EditorError> {
        let route: UiAssetEditorRoute =
            if let Ok(route) = serde_json::from_value(instance.serializable_payload.clone()) {
                route
            } else if let Some(asset_id) = instance
                .serializable_payload
                .get("path")
                .and_then(|value| value.as_str())
            {
                let source_path = self.resolve_ui_asset_path(asset_id)?;
                let source = fs::read_to_string(&source_path)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                let document = UiAssetLoader::load_toml_str(&source)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                UiAssetEditorRoute::new(asset_id, document.asset.kind, UiAssetEditorMode::Design)
            } else {
                return Err(EditorError::UiAsset(format!(
                    "invalid ui asset route for {}",
                    instance.instance_id.0
                )));
            };
        let source_path = self.resolve_ui_asset_path(&route.asset_id)?;
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let preview_size = preview_size_for_preset(route.preview_preset);
        let session = UiAssetEditorSession::from_source(route, source, preview_size)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        self.ui_asset_sessions.lock().unwrap().insert(
            instance.instance_id.clone(),
            UiAssetWorkspaceEntry {
                source_path,
                session,
            },
        );
        self.hydrate_ui_asset_editor_imports(&instance.instance_id)?;
        self.sync_ui_asset_editor_instance(&instance.instance_id)
    }

    pub(super) fn ensure_ui_asset_editor_session(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<(), EditorError> {
        if self
            .ui_asset_sessions
            .lock()
            .unwrap()
            .contains_key(instance_id)
        {
            return Ok(());
        }
        let instance = self
            .session
            .lock()
            .unwrap()
            .open_view_instances
            .get(instance_id)
            .cloned()
            .ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset view {}", instance_id.0))
            })?;
        self.restore_ui_asset_editor_instance(&instance)
    }
}
