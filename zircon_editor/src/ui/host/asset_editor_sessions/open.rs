use std::fs;
use std::path::Path;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute, UiAssetEditorSession};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

use super::super::project_access::normalize_ui_asset_asset_id;
use super::{
    parse_ui_asset_document_source, preview_size_for_preset, UiAssetWorkspaceEntry,
    UI_ASSET_EDITOR_DESCRIPTOR_ID,
};

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
        let source = fs::read_to_string(&source_path)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let document = parse_ui_asset_document_source(&source)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let route =
            UiAssetEditorRoute::new(asset_id, document.asset.kind, mode.unwrap_or_default());
        let preview_size = preview_size_for_preset(route.preview_preset);
        let session = UiAssetEditorSession::from_source(route, source.clone(), preview_size)
            .map_err(|error| EditorError::UiAsset(error.to_string()))?;
        let instance_id =
            self.open_view(ViewDescriptorId::new(UI_ASSET_EDITOR_DESCRIPTOR_ID), None)?;
        self.ui_asset_sessions.lock().unwrap().insert(
            instance_id.clone(),
            UiAssetWorkspaceEntry::new(source_path, source, session),
        );
        self.hydrate_ui_asset_editor_imports(&instance_id)?;
        self.sync_ui_asset_editor_instance(&instance_id)?;
        let _ = self.focus_view(&instance_id);
        Ok(instance_id)
    }
}
