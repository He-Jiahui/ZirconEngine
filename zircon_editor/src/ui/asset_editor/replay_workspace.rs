use std::collections::BTreeMap;

use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime::ui::template::UiAssetDocument;

use super::undo_stack::UiAssetEditorUndoTransition;
use super::{apply_external_effects_to_asset_sources, UiAssetEditorSourceCursorSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorReplayWorkspace {
    pub source: String,
    pub document: UiAssetDocument,
    pub selection: UiDesignerSelectionModel,
    pub source_cursor: UiAssetEditorSourceCursorSnapshot,
    pub selected_theme_source_key: Option<String>,
    pub asset_sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorReplayWorkspaceResult {
    pub source_changed: bool,
    pub document_changed: bool,
    pub selection_changed: bool,
    pub source_cursor_changed: bool,
    pub theme_source_changed: bool,
    pub asset_sources_changed: bool,
}

impl UiAssetEditorUndoTransition {
    pub fn apply_to_workspace(
        &self,
        workspace: &mut UiAssetEditorReplayWorkspace,
    ) -> Result<UiAssetEditorReplayWorkspaceResult, &'static str> {
        let source_changed = self.apply_to_source(&mut workspace.source)?;
        let document_changed = self.apply_to_document(&mut workspace.document)?;
        let selection_changed = workspace.selection != self.selection;
        if selection_changed {
            workspace.selection = self.selection.clone();
        }

        let source_cursor_changed = workspace.source_cursor != self.source_cursor;
        if source_cursor_changed {
            workspace.source_cursor = self.source_cursor.clone();
        }

        let theme_source_changed =
            workspace.selected_theme_source_key != self.selected_theme_source_key;
        if theme_source_changed {
            workspace.selected_theme_source_key = self.selected_theme_source_key.clone();
        }

        let asset_sources_changed = apply_external_effects_to_asset_sources(
            &mut workspace.asset_sources,
            &self.external_effects,
        );

        Ok(UiAssetEditorReplayWorkspaceResult {
            source_changed,
            document_changed,
            selection_changed,
            source_cursor_changed,
            theme_source_changed,
            asset_sources_changed,
        })
    }
}
