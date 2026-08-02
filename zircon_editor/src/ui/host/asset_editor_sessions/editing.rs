mod binding;
mod inspector;
mod navigation;
mod node_ops;
mod palette;
mod source;
mod style;

use std::fs;
use std::path::PathBuf;

use zircon_runtime::asset::project::ProjectManager;

use super::super::editor_error::EditorError;
use super::super::editor_ui_host::EditorUiHost;
use crate::core::commands::DocumentKind;
use crate::ui::asset_editor::{
    UiAssetEditorCommand, UiAssetEditorExternalEffect, UiAssetEditorMode, UiAssetPreviewPreset,
    UiDesignerToolMode,
};
use crate::ui::workbench::autolayout::default_constraints_for_content;
use crate::ui::workbench::snapshot::ViewContentKind;
use crate::ui::workbench::view::{
    ActivityWindowTemplateSpec, ViewDescriptor, ViewDescriptorId, ViewInstanceId, ViewKind,
    WorkbenchSlot,
};
use zircon_runtime_interface::ui::layout::UiSize;

use super::super::project_access::{normalize_ui_asset_asset_id, resolve_project_asset_write_path};
use super::super::ui_asset_promotion::{
    resolve_external_style_target, resolve_external_widget_target,
};

pub(crate) const UI_ASSET_EDITOR_DESCRIPTOR_ID: &str = "editor.ui_asset";

fn ui_asset_effect_source_path(
    project: &ProjectManager,
    asset_id: &str,
) -> Result<PathBuf, EditorError> {
    resolve_project_asset_write_path(project, asset_id)
}

pub(crate) fn ui_asset_editor_view_descriptor() -> ViewDescriptor {
    ViewDescriptor::new(
        ViewDescriptorId::new(UI_ASSET_EDITOR_DESCRIPTOR_ID),
        ViewKind::ActivityWindow,
        "UI Asset Editor",
    )
    .with_document_kind(DocumentKind::ui_asset())
    .with_multi_instance(true)
    .with_workbench_slot(WorkbenchSlot::DocumentCenter)
    .with_default_constraints(default_constraints_for_content(
        ViewContentKind::UiAssetEditor,
    ))
    .with_activity_window_template(ActivityWindowTemplateSpec::new(
        "res://ui/editor/windows/ui_layout_editor_window.zui",
    ))
    .with_icon_key("ui-asset")
}

impl EditorUiHost {
    fn apply_ui_asset_editor_external_effect(
        &self,
        project: &ProjectManager,
        effect: &UiAssetEditorExternalEffect,
    ) -> Result<String, EditorError> {
        match effect {
            UiAssetEditorExternalEffect::UpsertAssetSource { asset_id, source }
            | UiAssetEditorExternalEffect::RestoreAssetSource { asset_id, source } => {
                let source_path = ui_asset_effect_source_path(project, asset_id)?;
                if let Some(parent) = source_path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                }
                fs::write(&source_path, source)
                    .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                let normalized = normalize_ui_asset_asset_id(asset_id).to_string();
                let _ = self.asset_manager()?.import_asset(&normalized);
                Ok(normalized)
            }
            UiAssetEditorExternalEffect::RemoveAssetSource { asset_id } => {
                let source_path = ui_asset_effect_source_path(project, asset_id)?;
                if source_path.exists() {
                    fs::remove_file(&source_path)
                        .map_err(|error| EditorError::UiAsset(error.to_string()))?;
                }
                let _ = self.asset_manager()?.reimport_all();
                Ok(normalize_ui_asset_asset_id(asset_id).to_string())
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
