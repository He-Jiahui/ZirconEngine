#[derive(Clone, Debug, PartialEq, Eq)]
struct SourceEditEntry {
    before_source: String,
    after_source: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiAssetEditorUndoStack {
    undo_stack: Vec<SourceEditEntry>,
    redo_stack: Vec<SourceEditEntry>,
}

impl UiAssetEditorUndoStack {
    pub fn push_source_edit(&mut self, before_source: String, after_source: String) {
        self.undo_stack.push(SourceEditEntry {
            before_source,
            after_source,
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) -> Option<String> {
        let entry = self.undo_stack.pop()?;
        let source = entry.before_source.clone();
        self.redo_stack.push(entry);
        Some(source)
    }

    pub fn redo(&mut self) -> Option<String> {
        let entry = self.redo_stack.pop()?;
        let source = entry.after_source.clone();
        self.undo_stack.push(entry);
        Some(source)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
