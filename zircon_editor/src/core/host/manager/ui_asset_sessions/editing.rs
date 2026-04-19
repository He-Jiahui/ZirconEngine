mod binding;
mod inspector;
mod navigation;
mod node_ops;
mod palette;
mod source;
mod style;

use std::fs;
use std::path::{Path, PathBuf};

use crate::default_constraints_for_content;
use crate::view::{PreferredHost, ViewDescriptor, ViewDescriptorId, ViewInstanceId, ViewKind};
use crate::{
    EditorError, EditorManager, UiAssetEditorCommand, UiAssetEditorExternalEffect,
    UiAssetEditorMode, UiAssetEditorSession, UiAssetPreviewPreset, UiSize, ViewContentKind,
};
use zircon_asset::assets::{UiStyleAsset, UiWidgetAsset};

use super::super::project_access::normalize_ui_asset_asset_id;
use super::super::ui_asset_promotion::{
    resolve_external_style_target, resolve_external_widget_target,
};

pub(crate) const UI_ASSET_EDITOR_DESCRIPTOR_ID: &str = "editor.ui_asset";

pub(crate) struct UiAssetWorkspaceEntry {
    pub(crate) source_path: PathBuf,
    pub(crate) session: UiAssetEditorSession,
}

fn ui_asset_effect_source_path(project_root: &Path, asset_id: &str) -> PathBuf {
    let relative = asset_id.strip_prefix("res://").unwrap_or(asset_id);
    project_root.join("assets").join(relative)
}

pub(crate) fn ui_asset_editor_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(UI_ASSET_EDITOR_DESCRIPTOR_ID),
        ViewKind::ActivityWindow,
        "UI Asset Editor",
    )
    .with_multi_instance(true)
    .with_preferred_host(PreferredHost::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiAssetEditor,
    ))
    .with_icon_key("ui-asset")
}

impl EditorManager {
    fn apply_ui_asset_editor_external_effect(
        &self,
        project_root: &Path,
        effect: &UiAssetEditorExternalEffect,
    ) -> Result<(), EditorError> {
        match effect {
            UiAssetEditorExternalEffect::UpsertAssetSource { asset_id, source }
            | UiAssetEditorExternalEffect::RestoreAssetSource { asset_id, source } => {
                let source_path = ui_asset_effect_source_path(project_root, asset_id);
                if let Some(parent) = source_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                }
                fs::write(&source_path, source)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                let normalized = normalize_ui_asset_asset_id(asset_id).to_string();
                let _ = self.asset_manager()?.import_asset(&normalized);
                Ok(())
            }
            UiAssetEditorExternalEffect::RemoveAssetSource { asset_id } => {
                let source_path = ui_asset_effect_source_path(project_root, asset_id);
                if source_path.exists() {
                    fs::remove_file(&source_path)
                        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                }
                let _ = self.asset_manager()?.reimport_all();
                Ok(())
            }
        }
    }
}

pub(crate) fn preview_size_for_preset(preview_preset: UiAssetPreviewPreset) -> UiSize {
    match preview_preset {
        UiAssetPreviewPreset::EditorDocked => UiSize::new(1280.0, 720.0),
        UiAssetPreviewPreset::EditorFloating => UiSize::new(1100.0, 780.0),
        UiAssetPreviewPreset::GameHud => UiSize::new(1920.0, 1080.0),
        UiAssetPreviewPreset::Dialog => UiSize::new(640.0, 480.0),
    }
}
