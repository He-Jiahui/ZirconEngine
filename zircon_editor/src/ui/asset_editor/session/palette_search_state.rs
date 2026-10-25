use super::ui_asset_editor_session::{UiAssetEditorSession, UiAssetEditorSessionError};
use crate::ui::asset_editor::palette::UiAssetPaletteSearchSnapshot;

impl UiAssetEditorSession {
    /// Navigation only: querying never writes the document or adds an undo entry.
    pub(crate) fn set_palette_search_query(&mut self, query: &str) -> bool {
        if !self.palette_catalog.set_search_query(query) {
            return false;
        }
        if self
            .selected_palette_index
            .is_some_and(|index| !self.palette_catalog.search_contains_entry(index))
        {
            self.selected_palette_index = None;
            self.selected_palette_entry = None;
        }
        self.clear_palette_drag_state();
        true
    }

    /// A focused editor may type several queries while its original source remains current.
    /// Host focus/window checks are required in addition to this document proof.
    pub(crate) fn set_palette_search_query_from_snapshot(
        &mut self,
        snapshot: &UiAssetPaletteSearchSnapshot,
        query: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if !snapshot.is_current_source(&self.route.asset_id, &self.source_buffer) {
            return Err(UiAssetEditorSessionError::RetiredPaletteSearch);
        }
        Ok(self.set_palette_search_query(query))
    }

    pub(crate) fn set_palette_search_page_from_snapshot(
        &mut self,
        snapshot: &UiAssetPaletteSearchSnapshot,
        start: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if !snapshot.is_current_source(&self.route.asset_id, &self.source_buffer)
            || !self.palette_catalog.is_current_search_snapshot(snapshot)
        {
            return Err(UiAssetEditorSessionError::RetiredPaletteSearch);
        }
        Ok(self.set_palette_visible_start(start))
    }

    pub(crate) fn palette_search_snapshot(&self) -> UiAssetPaletteSearchSnapshot {
        self.palette_catalog.search_snapshot(
            &self.route.asset_id,
            &self.source_buffer,
            self.selected_palette_index,
            self.diagnostics.is_empty() && self.source_buffer.text() == self.last_valid_source_text,
        )
    }

    pub(crate) fn select_palette_search_row(
        &mut self,
        snapshot: &UiAssetPaletteSearchSnapshot,
        row: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if !snapshot.is_current_source(&self.route.asset_id, &self.source_buffer)
            || !self.palette_catalog.is_current_search_snapshot(snapshot)
        {
            return Err(UiAssetEditorSessionError::RetiredPaletteSearch);
        }
        self.ensure_editable_source()?;
        if self.source_buffer.text() != self.last_valid_source_text {
            return Err(UiAssetEditorSessionError::InvalidSourceBuffer);
        }
        let index = snapshot
            .catalog_index_for_row(row)
            .ok_or(UiAssetEditorSessionError::InvalidPaletteSearchRow { row })?;
        self.select_palette_index(index)
    }
}

#[cfg(test)]
#[path = "tests/palette_search_state.rs"]
mod tests;
