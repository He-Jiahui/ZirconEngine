use std::fs;
use std::path::Path;

use zircon_runtime::asset::AssetUri;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::core::asset::AssetToolkitOpenRoute;
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::asset_editor::UiAssetEditorMode;
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::super::project_access::{normalize_ui_asset_asset_id, project_asset_id_for_source_path};
use super::{
    build_ui_asset_editor_session_from_source, preview_size_for_preset,
    ui_asset_editor_route_from_source, UiAssetWorkspaceEntry, UI_ASSET_EDITOR_DESCRIPTOR_ID,
};

pub(super) const UI_ASSET_EDITOR_OPEN_OPERATION: &str = "view.editor.ui_asset.open";

fn ui_asset_editor_toolkit_route(asset_id: &str) -> Result<AssetToolkitOpenRoute, EditorError> {
    let asset_locator = AssetUri::parse(asset_id).map_err(|error| {
        EditorError::UiAsset(format!("invalid UI asset locator {asset_id}: {error}"))
    })?;
    let open_operation = EditorOperationPath::parse(UI_ASSET_EDITOR_OPEN_OPERATION)
        .expect("built-in UI asset editor operation path is valid");
    Ok(AssetToolkitOpenRoute::new(asset_locator, open_operation))
}

impl EditorUiHost {
    pub fn open_ui_asset_editor(
        &self,
        path: impl AsRef<Path>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.open_ui_asset_editor_by_id(path.as_ref().to_string_lossy(), mode)
    }

    pub fn open_ui_asset_editor_by_id(
        &self,
        asset_id: impl AsRef<str>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        let asset_id = normalize_ui_asset_asset_id(asset_id.as_ref()).to_string();
        let source_path = self.resolve_ui_asset_path(&asset_id)?;
        let persisted_asset_id = if asset_id.starts_with("res://") {
            Some(asset_id.clone())
        } else if let Some(project) = self.current_project_snapshot()? {
            Some(
                project_asset_id_for_source_path(&project, &source_path)?.ok_or_else(|| {
                    EditorError::UiAsset(format!(
                        "cannot open UI asset {} outside the active project asset roots",
                        source_path.display()
                    ))
                })?,
            )
        } else {
            None
        };
        let toolkit_route = persisted_asset_id
            .as_deref()
            .map(ui_asset_editor_toolkit_route)
            .transpose()?;
        let session_asset_id = persisted_asset_id.as_deref().unwrap_or(&asset_id);
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let route =
            ui_asset_editor_route_from_source(session_asset_id, &source, mode.unwrap_or_default())
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let route_asset_id = route.asset_id.clone();
        let preview_size = preview_size_for_preset(route.preview_preset);
        let session =
            build_ui_asset_editor_session_from_source(route, source.clone(), preview_size)
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let instance_id =
            self.open_view(ViewDescriptorId::new(UI_ASSET_EDITOR_DESCRIPTOR_ID), None)?;
        if let Some(toolkit_route) = toolkit_route {
            let payload = serde_json::to_value(toolkit_route)
                .map_err(|error| EditorError::UiAsset(error.to_string()))?;
            self.update_view_instance_metadata(&instance_id, None, None, Some(payload))?;
        }
        self.lock_ui_asset_sessions().insert(
            instance_id.clone(),
            UiAssetWorkspaceEntry::new(source_path, source, session),
        );
        self.lock_ui_asset_dependency_generation()
            .register_route(instance_id.clone(), &route_asset_id);
        self.hydrate_ui_asset_editor_imports(&instance_id)?;
        self.sync_ui_asset_editor_instance(&instance_id)?;
        self.register_ui_asset_document_toolkit(&instance_id)?;
        let _ = self.focus_view(&instance_id);
        Ok(instance_id)
    }
}
