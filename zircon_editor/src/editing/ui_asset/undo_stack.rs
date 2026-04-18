use zircon_editor_ui::UiDesignerSelectionModel;
use zircon_ui::UiAssetDocument;

use super::command::{UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind};
use super::document_diff::UiAssetDocumentDiff;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiAssetEditorExternalEffect {
    UpsertAssetSource { asset_id: String, source: String },
    RemoveAssetSource { asset_id: String },
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorUndoExternalEffects {
    pub undo: Option<UiAssetEditorExternalEffect>,
    pub redo: Option<UiAssetEditorExternalEffect>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorSourceCursorSnapshot {
    pub byte_offset: usize,
    pub anchor_node_id: Option<String>,
    pub line_offset: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorUndoTransition {
    pub selection: UiDesignerSelectionModel,
    pub source_cursor: UiAssetEditorSourceCursorSnapshot,
    pub document: Option<UiAssetEditorUndoDocumentReplay>,
    pub external_effect: Option<UiAssetEditorExternalEffect>,
    source: UiAssetEditorSourceReplay,
}

impl UiAssetEditorUndoTransition {
    pub fn apply_to_source(&self, source: &mut String) -> Result<bool, &'static str> {
        self.source.apply_to(source)
    }

    pub fn apply_to_document(&self, document: &mut UiAssetDocument) -> Result<bool, &'static str> {
        self.document
            .as_ref()
            .map(|replay| replay.apply_to_document(document))
            .unwrap_or(Ok(false))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct UiAssetEditorSourceReplay {
    replace_start: usize,
    replace_len: usize,
    insert: String,
}

impl UiAssetEditorSourceReplay {
    fn between(current: &str, target: &str) -> Self {
        let prefix_len = common_prefix_len(current, target);
        let current_suffix = &current[prefix_len..];
        let target_suffix = &target[prefix_len..];
        let suffix_len = common_suffix_len(current_suffix, target_suffix);
        let current_end = current.len().saturating_sub(suffix_len);
        let target_end = target.len().saturating_sub(suffix_len);
        Self {
            replace_start: prefix_len,
            replace_len: current_end.saturating_sub(prefix_len),
            insert: target[prefix_len..target_end].to_string(),
        }
    }

    fn apply_to(&self, source: &mut String) -> Result<bool, &'static str> {
        let replace_end = self
            .replace_start
            .checked_add(self.replace_len)
            .ok_or("invalid source replay")?;
        if replace_end > source.len()
            || !source.is_char_boundary(self.replace_start)
            || !source.is_char_boundary(replace_end)
        {
            return Err("invalid source replay");
        }
        let changed = source[self.replace_start..replace_end] != self.insert;
        source.replace_range(self.replace_start..replace_end, &self.insert);
        Ok(changed)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorUndoDocumentReplay {
    diff: UiAssetDocumentDiff,
}

impl UiAssetEditorUndoDocumentReplay {
    fn between(current: &UiAssetDocument, target: &UiAssetDocument) -> Self {
        Self {
            diff: UiAssetDocumentDiff::between(current, target),
        }
    }

    pub fn apply_to_document(&self, document: &mut UiAssetDocument) -> Result<bool, &'static str> {
        Ok(self.diff.apply_to(document))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct SourceEditEntry {
    label: String,
    tree_edit: Option<UiAssetEditorTreeEdit>,
    undo: UiAssetEditorUndoTransition,
    redo: UiAssetEditorUndoTransition,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiAssetEditorUndoStack {
    undo_stack: Vec<SourceEditEntry>,
    redo_stack: Vec<SourceEditEntry>,
}

impl UiAssetEditorUndoStack {
    pub fn push_edit(
        &mut self,
        label: impl Into<String>,
        tree_edit: Option<UiAssetEditorTreeEdit>,
        before_source: String,
        before_selection: UiDesignerSelectionModel,
        before_source_cursor: UiAssetEditorSourceCursorSnapshot,
        before_document: Option<UiAssetDocument>,
        after_source: String,
        after_selection: UiDesignerSelectionModel,
        after_source_cursor: UiAssetEditorSourceCursorSnapshot,
        after_document: Option<UiAssetDocument>,
        external_effects: UiAssetEditorUndoExternalEffects,
    ) {
        self.undo_stack.push(SourceEditEntry {
            label: label.into(),
            tree_edit,
            undo: UiAssetEditorUndoTransition {
                selection: before_selection,
                source_cursor: before_source_cursor,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(after_document, before_document),
                    ),
                    _ => None,
                },
                external_effect: external_effects.undo,
                source: UiAssetEditorSourceReplay::between(&after_source, &before_source),
            },
            redo: UiAssetEditorUndoTransition {
                selection: after_selection,
                source_cursor: after_source_cursor,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(before_document, after_document),
                    ),
                    _ => None,
                },
                external_effect: external_effects.redo,
                source: UiAssetEditorSourceReplay::between(&before_source, &after_source),
            },
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<UiAssetEditorUndoTransition> {
        let entry = self.undo_stack.pop()?;
        let snapshot = entry.undo.clone();
        self.redo_stack.push(entry);
        Some(snapshot)
    }

    pub fn redo(&mut self) -> Option<UiAssetEditorUndoTransition> {
        let entry = self.redo_stack.pop()?;
        let snapshot = entry.redo.clone();
        self.undo_stack.push(entry);
        Some(snapshot)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn next_undo_tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.tree_edit.as_ref().map(UiAssetEditorTreeEdit::kind))
    }

    pub fn next_redo_tree_edit_kind(&self) -> Option<UiAssetEditorTreeEditKind> {
        self.redo_stack
            .last()
            .and_then(|entry| entry.tree_edit.as_ref().map(UiAssetEditorTreeEdit::kind))
    }

    pub fn next_undo_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.tree_edit.clone())
    }

    pub fn next_redo_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.redo_stack
            .last()
            .and_then(|entry| entry.tree_edit.clone())
    }

    pub fn next_undo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.undo.external_effect.clone())
    }

    pub fn next_redo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.redo_stack
            .last()
            .and_then(|entry| entry.redo.external_effect.clone())
    }
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
