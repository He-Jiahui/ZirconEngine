use std::fs;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorPanePresentation, UiAssetEditorReflectionModel,
    UiAssetEditorRoute, UiAssetEditorSession,
};
use crate::ui::workbench::view::{ViewInstance, ViewInstanceId};

use super::{parse_ui_asset_document_source, preview_size_for_preset, UiAssetWorkspaceEntry};

impl EditorUiHost {
    pub fn ui_asset_editor_reflection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorReflectionModel, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let sessions = self.ui_asset_sessions.lock().unwrap();
        let entry = sessions.get(instance_id).ok_or_else(|| {
            EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
        })?;
        let mut reflection = entry.session.reflection_model();
        reflection = reflection.with_workspace_state(
            entry.has_external_conflict(),
            entry.external_conflict_summary(),
            entry.stale_import_items(),
        );
        Ok(reflection)
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
        let mut pane = entry.session.pane_presentation();
        pane.has_external_conflict = entry.has_external_conflict();
        pane.external_conflict_summary = entry.external_conflict_summary();
        pane.stale_import_items = entry.stale_import_items();
        pane.can_reload_from_disk = pane.has_external_conflict;
        pane.can_keep_local_and_save = pane.has_external_conflict;
        pane.can_open_diff_snapshot = pane.has_external_conflict;
        Ok(pane)
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

    pub fn open_ui_asset_editor_selected_theme_source(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.ensure_ui_asset_editor_session(instance_id)?;
        let selected_reference = {
            let sessions = self.ui_asset_sessions.lock().unwrap();
            let entry = sessions.get(instance_id).ok_or_else(|| {
                EditorError::UiAsset(format!("missing ui asset session {}", instance_id.0))
            })?;
            entry.session.selected_theme_source_asset_id()
        };
        let Some(selected_reference) = selected_reference else {
            return Ok(None);
        };
        self.open_ui_asset_editor_by_id(selected_reference, None)
            .map(Some)
    }

    pub(crate) fn restore_ui_asset_editor_instance(
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
                let document = parse_ui_asset_document_source(&source)
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
        let session = UiAssetEditorSession::from_source(route, source.clone(), preview_size)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        self.ui_asset_sessions.lock().unwrap().insert(
            instance.instance_id.clone(),
            UiAssetWorkspaceEntry::new(source_path, source, session),
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
