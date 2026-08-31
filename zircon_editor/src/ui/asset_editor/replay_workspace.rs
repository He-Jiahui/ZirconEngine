use std::collections::BTreeMap;

use crate::ui::asset_editor::UiDesignerSelectionModel;
use zircon_runtime_interface::ui::template::UiAssetDocument;

use super::undo_stack::UiAssetEditorUndoTransition;
use super::{apply_external_effects_to_asset_sources, UiAssetEditorSourceCursorSnapshot};

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorReplayWorkspace {
    pub source: String,
    pub document: UiAssetDocument,
    pub selection: UiDesignerSelectionModel,
    pub source_cursor: UiAssetEditorSourceCursorSnapshot,
    pub selected_theme_source_key: Option<String>,
    pub selected_style_rule_id: Option<String>,
    pub asset_sources: BTreeMap<String, String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorReplayWorkspaceResult {
    pub source_changed: bool,
    pub document_changed: bool,
    pub selection_changed: bool,
    pub source_cursor_changed: bool,
    pub theme_source_changed: bool,
    pub style_rule_selection_changed: bool,
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
            reuse_selection_state(&mut workspace.selection, &self.selection);
        }

        let source_cursor_changed = workspace.source_cursor != self.source_cursor;
        if source_cursor_changed {
            reuse_source_cursor(&mut workspace.source_cursor, &self.source_cursor);
        }

        let theme_source_changed =
            workspace.selected_theme_source_key != self.selected_theme_source_key;
        if theme_source_changed {
            reuse_optional_string(
                &mut workspace.selected_theme_source_key,
                &self.selected_theme_source_key,
            );
        }

        let style_rule_selection_changed =
            workspace.selected_style_rule_id != self.selected_style_rule_id;
        if style_rule_selection_changed {
            reuse_optional_string(
                &mut workspace.selected_style_rule_id,
                &self.selected_style_rule_id,
            );
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
            style_rule_selection_changed,
            asset_sources_changed,
        })
    }
}

fn reuse_selection_state(target: &mut UiDesignerSelectionModel, source: &UiDesignerSelectionModel) {
    target.primary_node_id.clone_from(&source.primary_node_id);
    target.sibling_node_ids.clone_from(&source.sibling_node_ids);
    target.parent_node_id.clone_from(&source.parent_node_id);
    target.mount.clone_from(&source.mount);
}

fn reuse_source_cursor(
    target: &mut UiAssetEditorSourceCursorSnapshot,
    source: &UiAssetEditorSourceCursorSnapshot,
) {
    target.byte_offset = source.byte_offset;
    target.anchor_node_id.clone_from(&source.anchor_node_id);
    target.line_offset = source.line_offset;
}

fn reuse_optional_string(target: &mut Option<String>, source: &Option<String>) {
    target.clone_from(source);
}

#[cfg(test)]
#[path = "replay_workspace/reused_state_tests.rs"]
mod reused_state_tests;
