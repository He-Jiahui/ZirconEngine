use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorPanePresentation, UiAssetEditorReflectionModel,
    UiAssetPreviewPreset,
};
use crate::ui::workbench::view::ViewInstanceId;
use std::path::Path;

impl EditorManager {
    pub fn ui_asset_editor_reflection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorReflectionModel, EditorError> {
        self.host.ui_asset_editor_reflection(instance_id)
    }

    pub fn ui_asset_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorPanePresentation, EditorError> {
        self.host.ui_asset_editor_pane_presentation(instance_id)
    }

    pub fn open_ui_asset_editor_selected_reference(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .open_ui_asset_editor_selected_reference(instance_id)
    }

    pub fn open_ui_asset_editor_selected_theme_source(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .open_ui_asset_editor_selected_theme_source(instance_id)
    }

    pub fn open_ui_asset_editor(
        &self,
        path: impl AsRef<Path>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.host.open_ui_asset_editor(path, mode)
    }

    pub fn open_ui_asset_editor_by_id(
        &self,
        asset_id: impl AsRef<str>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.host.open_ui_asset_editor_by_id(asset_id, mode)
    }

    pub fn set_ui_asset_editor_preview_preset(
        &self,
        instance_id: &ViewInstanceId,
        preview_preset: UiAssetPreviewPreset,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_preview_preset(instance_id, preview_preset)
    }

    pub fn select_ui_asset_editor_preview_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.host
            .select_ui_asset_editor_preview_index(instance_id, index)
    }

    pub fn activate_ui_asset_editor_preview_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .activate_ui_asset_editor_preview_index(instance_id, index)
    }

    pub fn select_ui_asset_editor_preview_mock_property(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_preview_mock_property(instance_id, index)
    }

    pub fn select_ui_asset_editor_preview_mock_subject(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_preview_mock_subject(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_preview_mock_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_preview_mock_value(instance_id, value)
    }

    pub fn select_ui_asset_editor_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_preview_mock_nested_entry(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_preview_mock_nested_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_preview_mock_nested_value(instance_id, value)
    }

    pub fn upsert_ui_asset_editor_selected_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
        key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .upsert_ui_asset_editor_selected_preview_mock_nested_entry(
                instance_id,
                key,
                value_literal,
            )
    }

    pub fn apply_ui_asset_editor_selected_preview_mock_suggestion(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_selected_preview_mock_suggestion(instance_id, index)
    }

    pub fn delete_ui_asset_editor_selected_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_preview_mock_nested_entry(instance_id)
    }

    pub fn clear_ui_asset_editor_selected_preview_mock_value(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .clear_ui_asset_editor_selected_preview_mock_value(instance_id)
    }

    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.host.save_ui_asset_editor(instance_id)
    }
}
