use crate::ui::asset_editor::{
    UiAssetEditorRoute, UiDesignerSelectionModel, UiStyleInspectorReflectionModel,
};
use thiserror::Error;
use zircon_runtime::ui::template::{UiAssetError, UiCompiledDocument, UiTemplateBuildError};
use zircon_runtime::ui::tree::UiTreeError;
use zircon_runtime::ui::{template::UiAssetDocument, template::UiAssetKind};

use super::{
    command::UiAssetEditorTreeEdit,
    hierarchy_projection::selection_for_node,
    palette_target_chooser::UiAssetPaletteTargetChooser,
    preview_host::UiAssetPreviewHost,
    preview_mock::UiAssetPreviewMockState,
    session_state::UiAssetCompilerImports,
    source_buffer::UiAssetSourceBuffer,
    tree_editing::{
        unwrap_selected_node, wrap_selected_node, PaletteInsertMode, UiTreeMoveDirection,
        UiTreeReparentDirection,
    },
    undo_stack::{UiAssetEditorExternalEffect, UiAssetEditorUndoStack},
};

#[derive(Debug, Error)]
pub enum UiAssetEditorSessionError {
    #[error(transparent)]
    Asset(#[from] UiAssetError),
    #[error(transparent)]
    Build(#[from] UiTemplateBuildError),
    #[error(transparent)]
    Layout(#[from] UiTreeError),
    #[error("expected ui asset kind {expected:?} but document was {actual:?}")]
    UnexpectedKind {
        expected: UiAssetKind,
        actual: UiAssetKind,
    },
    #[error("cannot serialize an invalid ui asset source buffer")]
    InvalidSourceBuffer,
    #[error("ui asset selection index {index} is out of range")]
    InvalidSelectionIndex { index: usize },
    #[error("ui asset preview index {index} did not map to a selectable node")]
    InvalidPreviewIndex { index: usize },
    #[error("ui asset stylesheet rule index {index} is out of range")]
    InvalidStyleRuleIndex { index: usize },
    #[error("ui asset matched style rule index {index} is out of range")]
    InvalidMatchedStyleRuleIndex { index: usize },
    #[error("ui asset stylesheet rule declaration index {index} is out of range")]
    InvalidStyleRuleDeclarationIndex { index: usize },
    #[error("ui asset style token index {index} is out of range")]
    InvalidStyleTokenIndex { index: usize },
    #[error("ui asset binding index {index} is out of range")]
    InvalidBindingIndex { index: usize },
    #[error("ui asset palette index {index} is out of range")]
    InvalidPaletteIndex { index: usize },
    #[error("ui asset stylesheet selector is invalid: {selector}")]
    InvalidStyleSelector { selector: String },
    #[error("ui asset stylesheet declaration path is invalid: {path}")]
    InvalidStyleDeclarationPath { path: String },
    #[error("ui asset inspector field {field} expects a numeric literal, received: {value}")]
    InvalidInspectorNumericLiteral { field: &'static str, value: String },
    #[error("ui asset binding event is invalid: {value}")]
    InvalidBindingEvent { value: String },
    #[error("ui asset preview mock value is invalid: {message}")]
    InvalidPreviewMockValue { message: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorReplayResult {
    pub changed: bool,
    pub label: String,
    pub external_effects: Vec<UiAssetEditorExternalEffect>,
}

pub struct UiAssetEditorSession {
    pub(super) route: UiAssetEditorRoute,
    pub(super) source_buffer: UiAssetSourceBuffer,
    pub(super) last_valid_source_text: String,
    pub(super) last_valid_document: UiAssetDocument,
    pub(super) last_valid_compiled: Option<UiCompiledDocument>,
    pub(super) preview_host: Option<UiAssetPreviewHost>,
    pub(super) undo_stack: UiAssetEditorUndoStack,
    pub(super) diagnostics: Vec<String>,
    pub(super) source_cursor_byte_offset: usize,
    pub(super) source_cursor_anchor: Option<UiAssetSourceCursorAnchor>,
    pub(super) selection: UiDesignerSelectionModel,
    pub(super) style_inspector: UiStyleInspectorReflectionModel,
    pub(super) selected_style_rule_index: Option<usize>,
    pub(super) selected_matched_style_rule_index: Option<usize>,
    pub(super) selected_style_rule_declaration_path: Option<String>,
    pub(super) selected_style_token_name: Option<String>,
    pub(super) selected_theme_source_key: Option<String>,
    pub(super) selected_binding_index: Option<usize>,
    pub(super) selected_binding_payload_key: Option<String>,
    pub(super) selected_slot_semantic_path: Option<String>,
    pub(super) selected_layout_semantic_path: Option<String>,
    pub(super) selected_palette_index: Option<usize>,
    pub(super) palette_target_chooser: Option<UiAssetPaletteTargetChooser>,
    pub(super) selected_promote_source_component_name: Option<String>,
    pub(super) selected_promote_widget_asset_id: Option<String>,
    pub(super) selected_promote_widget_component_name: Option<String>,
    pub(super) selected_promote_widget_document_id: Option<String>,
    pub(super) selected_promote_theme_asset_id: Option<String>,
    pub(super) selected_promote_theme_document_id: Option<String>,
    pub(super) selected_promote_theme_display_name: Option<String>,
    pub(super) preview_mock_state: UiAssetPreviewMockState,
    pub(super) compiler_imports: UiAssetCompilerImports,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct UiAssetSourceCursorAnchor {
    pub(super) node_id: String,
    pub(super) line_offset: usize,
}

impl UiAssetEditorSession {
    pub fn wrap_selected_node_with(
        &mut self,
        widget_type: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(wrapper_id) = wrap_selected_node(&mut document, &self.selection, widget_type)
        else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &wrapper_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::WrapNode {
                node_id,
                wrapper_node_id: wrapper_id,
                wrapper_widget_type: widget_type.to_string(),
            },
            "Wrap Node",
            selection,
        )?;
        Ok(true)
    }

    pub fn unwrap_selected_node(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(wrapper_node_id) = self.selection.primary_node_id.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(child_id) = unwrap_selected_node(&mut document, &self.selection) else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &child_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::UnwrapNode {
                wrapper_node_id,
                child_node_id: child_id,
            },
            "Unwrap Node",
            selection,
        )?;
        Ok(true)
    }

    pub(super) fn ensure_editable_source(&self) -> Result<(), UiAssetEditorSessionError> {
        if self.diagnostics.is_empty() {
            Ok(())
        } else {
            Err(UiAssetEditorSessionError::InvalidSourceBuffer)
        }
    }

    pub(super) fn roundtrip_source_text(&self) -> &str {
        if self.diagnostics.is_empty() {
            self.source_buffer.text()
        } else {
            &self.last_valid_source_text
        }
    }
}

pub(super) fn serialize_document(
    document: &UiAssetDocument,
) -> Result<String, UiAssetEditorSessionError> {
    toml::to_string_pretty(document)
        .map_err(|error| UiAssetError::ParseToml(error.to_string()).into())
}

pub(super) fn remap_source_byte_offset(current: &str, next: &str, byte_offset: usize) -> usize {
    let byte_offset = byte_offset.min(current.len());
    let prefix_len = common_prefix_len(current, next);
    let current_suffix = &current[prefix_len..];
    let next_suffix = &next[prefix_len..];
    let suffix_len = common_suffix_len(current_suffix, next_suffix);
    let current_replace_end = current.len().saturating_sub(suffix_len);
    let next_replace_end = next.len().saturating_sub(suffix_len);
    if byte_offset <= prefix_len {
        return byte_offset;
    }
    if byte_offset >= current_replace_end {
        return next_replace_end + byte_offset.saturating_sub(current_replace_end);
    }
    next_replace_end
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left_char, right_char)| left_char == right_char)
        .map(|(character, _)| character.len_utf8())
        .sum()
}

fn common_suffix_len(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left_char, right_char)| left_char == right_char)
        .map(|(character, _)| character.len_utf8())
        .sum()
}

pub(super) fn palette_insert_mode_label(mode: PaletteInsertMode) -> &'static str {
    match mode {
        PaletteInsertMode::Child => "child",
        PaletteInsertMode::After => "after_selection",
    }
}

pub(super) fn move_direction_label(direction: UiTreeMoveDirection) -> &'static str {
    match direction {
        UiTreeMoveDirection::Up => "up",
        UiTreeMoveDirection::Down => "down",
    }
}

pub(super) fn reparent_direction_label(direction: UiTreeReparentDirection) -> &'static str {
    match direction {
        UiTreeReparentDirection::IntoPrevious => "into_previous",
        UiTreeReparentDirection::IntoNext => "into_next",
        UiTreeReparentDirection::Outdent => "outdent",
    }
}
