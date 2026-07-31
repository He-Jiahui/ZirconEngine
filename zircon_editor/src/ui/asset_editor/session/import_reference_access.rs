use super::UiAssetEditorSession;

impl UiAssetEditorSession {
    pub(crate) fn import_reference_count(&self) -> usize {
        self.last_valid_document
            .imports
            .widgets
            .len()
            .saturating_add(self.last_valid_document.imports.styles.len())
    }

    pub(crate) fn import_reference_at(&self, index: usize) -> Option<&str> {
        self.last_valid_document
            .imports
            .widgets
            .get(index)
            .or_else(|| {
                index
                    .checked_sub(self.last_valid_document.imports.widgets.len())
                    .and_then(|style_index| {
                        self.last_valid_document.imports.styles.get(style_index)
                    })
            })
            .map(String::as_str)
    }
}
