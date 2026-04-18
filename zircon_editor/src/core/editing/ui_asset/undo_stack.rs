use crate::ui::UiDesignerSelectionModel;
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
    pub undo: Vec<UiAssetEditorExternalEffect>,
    pub redo: Vec<UiAssetEditorExternalEffect>,
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
    pub selected_theme_source_key: Option<String>,
    pub document: Option<UiAssetEditorUndoDocumentReplay>,
    pub external_effects: Vec<UiAssetEditorExternalEffect>,
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
        before_selected_theme_source_key: Option<String>,
        before_document: Option<UiAssetDocument>,
        after_source: String,
        after_selection: UiDesignerSelectionModel,
        after_source_cursor: UiAssetEditorSourceCursorSnapshot,
        after_selected_theme_source_key: Option<String>,
        after_document: Option<UiAssetDocument>,
        external_effects: UiAssetEditorUndoExternalEffects,
    ) {
        self.undo_stack.push(SourceEditEntry {
            label: label.into(),
            tree_edit,
            undo: UiAssetEditorUndoTransition {
                selection: before_selection,
                source_cursor: before_source_cursor,
                selected_theme_source_key: before_selected_theme_source_key,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(after_document, before_document),
                    ),
                    _ => None,
                },
                external_effects: external_effects.undo,
                source: UiAssetEditorSourceReplay::between(&after_source, &before_source),
            },
            redo: UiAssetEditorUndoTransition {
                selection: after_selection,
                source_cursor: after_source_cursor,
                selected_theme_source_key: after_selected_theme_source_key,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(before_document, after_document),
                    ),
                    _ => None,
                },
                external_effects: external_effects.redo,
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

    pub fn next_undo_label(&self) -> Option<String> {
        self.undo_stack.last().map(|entry| entry.label.clone())
    }

    pub fn next_redo_label(&self) -> Option<String> {
        self.redo_stack.last().map(|entry| entry.label.clone())
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

    pub fn next_undo_inverse_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.undo_stack
            .last()
            .and_then(|entry| entry.tree_edit.as_ref())
            .and_then(inverse_tree_edit)
    }

    pub fn next_redo_inverse_tree_edit(&self) -> Option<UiAssetEditorTreeEdit> {
        self.redo_stack
            .last()
            .and_then(|entry| entry.tree_edit.as_ref())
            .and_then(inverse_tree_edit)
    }

    pub fn next_undo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.next_undo_external_effects().into_iter().next()
    }

    pub fn next_redo_external_effect(&self) -> Option<UiAssetEditorExternalEffect> {
        self.next_redo_external_effects().into_iter().next()
    }

    pub fn next_undo_external_effects(&self) -> Vec<UiAssetEditorExternalEffect> {
        self.undo_stack
            .last()
            .map(|entry| entry.undo.external_effects.clone())
            .unwrap_or_default()
    }

    pub fn next_redo_external_effects(&self) -> Vec<UiAssetEditorExternalEffect> {
        self.redo_stack
            .last()
            .map(|entry| entry.redo.external_effects.clone())
            .unwrap_or_default()
    }
}

fn inverse_tree_edit(edit: &UiAssetEditorTreeEdit) -> Option<UiAssetEditorTreeEdit> {
    match edit {
        UiAssetEditorTreeEdit::MoveNode { node_id, direction } => {
            Some(UiAssetEditorTreeEdit::MoveNode {
                node_id: node_id.clone(),
                direction: inverse_move_direction(direction)?,
            })
        }
        UiAssetEditorTreeEdit::ReparentNode {
            node_id,
            parent_node_id,
            direction,
        } => Some(UiAssetEditorTreeEdit::ReparentNode {
            node_id: node_id.clone(),
            parent_node_id: parent_node_id.clone(),
            direction: inverse_reparent_direction(direction)?,
        }),
        UiAssetEditorTreeEdit::WrapNode {
            node_id,
            wrapper_node_id,
            ..
        } => Some(UiAssetEditorTreeEdit::UnwrapNode {
            wrapper_node_id: wrapper_node_id.clone(),
            child_node_id: node_id.clone(),
        }),
        _ => None,
    }
}

fn inverse_move_direction(direction: &str) -> Option<String> {
    match direction.to_ascii_lowercase().as_str() {
        "up" => Some(preserve_direction_case(direction, "Down", "down")),
        "down" => Some(preserve_direction_case(direction, "Up", "up")),
        _ => None,
    }
}

fn inverse_reparent_direction(direction: &str) -> Option<String> {
    match direction.to_ascii_lowercase().as_str() {
        "into_previous" | "into_next" => {
            Some(preserve_direction_case(direction, "Outdent", "outdent"))
        }
        _ => None,
    }
}

fn preserve_direction_case(direction: &str, title_case: &str, lower_case: &str) -> String {
    if direction
        .chars()
        .next()
        .map(|ch| ch.is_ascii_uppercase())
        .unwrap_or(false)
    {
        title_case.to_string()
    } else {
        lower_case.to_string()
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
