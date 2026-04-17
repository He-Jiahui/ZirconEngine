use zircon_editor_ui::UiDesignerSelectionModel;
use zircon_ui::UiAssetDocument;

use super::command::{UiAssetEditorTreeEdit, UiAssetEditorTreeEditKind};
use super::document_diff::UiAssetDocumentDiff;

#[derive(Clone, Debug, PartialEq)]
pub struct UiAssetEditorUndoSnapshot {
    pub source: String,
    pub selection: UiDesignerSelectionModel,
    pub document: Option<UiAssetEditorUndoDocumentReplay>,
}

impl UiAssetEditorUndoSnapshot {
    pub fn apply_to_document(&self, document: &mut UiAssetDocument) -> Result<bool, &'static str> {
        self.document
            .as_ref()
            .map(|replay| replay.apply_to_document(document))
            .unwrap_or(Ok(false))
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
    before: UiAssetEditorUndoSnapshot,
    after: UiAssetEditorUndoSnapshot,
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
        before_document: Option<UiAssetDocument>,
        after_source: String,
        after_selection: UiDesignerSelectionModel,
        after_document: Option<UiAssetDocument>,
    ) {
        self.undo_stack.push(SourceEditEntry {
            label: label.into(),
            tree_edit,
            before: UiAssetEditorUndoSnapshot {
                source: before_source,
                selection: before_selection,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(after_document, before_document),
                    ),
                    _ => None,
                },
            },
            after: UiAssetEditorUndoSnapshot {
                source: after_source,
                selection: after_selection,
                document: match (before_document.as_ref(), after_document.as_ref()) {
                    (Some(before_document), Some(after_document)) => Some(
                        UiAssetEditorUndoDocumentReplay::between(before_document, after_document),
                    ),
                    _ => None,
                },
            },
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<UiAssetEditorUndoSnapshot> {
        let entry = self.undo_stack.pop()?;
        let snapshot = entry.before.clone();
        self.redo_stack.push(entry);
        Some(snapshot)
    }

    pub fn redo(&mut self) -> Option<UiAssetEditorUndoSnapshot> {
        let entry = self.redo_stack.pop()?;
        let snapshot = entry.after.clone();
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
}
