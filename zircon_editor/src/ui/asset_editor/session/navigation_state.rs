use super::{
    binding_inspector::{reconcile_selected_binding_index, reconcile_selected_binding_payload_key},
    hierarchy_projection::{hierarchy_node_ids, selection_for_node},
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group, reconcile_selected_semantic_path,
    },
    preview_mock::reconcile_preview_mock_state,
    preview_projection::preview_node_id_for_index,
    source_sync::{
        build_source_outline, source_byte_offset_for_line, source_line_for_byte_offset,
        source_outline_entry_for_node, source_outline_node_id_for_line,
    },
    style_inspection::build_style_inspector,
    ui_asset_editor_session::{
        UiAssetEditorSession, UiAssetEditorSessionError, UiAssetSourceCursorAnchor,
    },
    undo_stack::UiAssetEditorSourceCursorSnapshot,
};

impl UiAssetEditorSession {
    pub fn select_hierarchy_index(
        &mut self,
        index: usize,
    ) -> Result<(), UiAssetEditorSessionError> {
        let node_id = hierarchy_node_ids(&self.last_valid_document)
            .into_iter()
            .nth(index)
            .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index })?;
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_preview_index(&mut self, index: usize) -> Result<(), UiAssetEditorSessionError> {
        let Some(preview_host) = self.preview_host.as_ref() else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        let Some(node_id) =
            preview_node_id_for_index(&self.last_valid_document, preview_host, index)
        else {
            return Err(UiAssetEditorSessionError::InvalidPreviewIndex { index });
        };
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_source_outline_index(
        &mut self,
        index: usize,
    ) -> Result<(), UiAssetEditorSessionError> {
        let node_id = build_source_outline(&self.last_valid_document, self.source_buffer.text())
            .into_iter()
            .nth(index)
            .map(|entry| entry.node_id)
            .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index })?;
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_source_line(&mut self, line: usize) -> Result<(), UiAssetEditorSessionError> {
        let node_id = source_outline_node_id_for_line(
            &self.last_valid_document,
            self.source_buffer.text(),
            line,
        )
        .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index: line })?;
        let line_offset = source_outline_entry_for_node(self.source_buffer.text(), &node_id)
            .map(|entry| line.saturating_sub(entry.line as usize))
            .unwrap_or_default();
        self.select_node_id(&node_id);
        self.set_source_cursor_for_selected_node_line(
            line_offset,
            source_byte_offset_for_line(self.source_buffer.text(), line),
        );
        Ok(())
    }

    pub fn select_source_byte_offset(
        &mut self,
        byte_offset: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let clamped = byte_offset.min(self.source_buffer.text().len());
        let line = source_line_for_byte_offset(self.source_buffer.text(), clamped);
        let Some(node_id) = source_outline_node_id_for_line(
            &self.last_valid_document,
            self.source_buffer.text(),
            line,
        ) else {
            return Ok(false);
        };
        let line_offset = source_outline_entry_for_node(self.source_buffer.text(), &node_id)
            .map(|entry| line.saturating_sub(entry.line as usize))
            .unwrap_or_default();
        let selection_changed = self.selection.primary_node_id.as_deref() != Some(node_id.as_str());
        let cursor_changed = self.source_cursor_byte_offset != clamped
            || self
                .source_cursor_anchor
                .as_ref()
                .map(|anchor| {
                    anchor.node_id.as_str() != node_id.as_str() || anchor.line_offset != line_offset
                })
                .unwrap_or(true);
        if !selection_changed && !cursor_changed {
            return Ok(false);
        }
        if selection_changed {
            self.select_node_id(&node_id);
        }
        self.set_source_cursor_for_selected_node_line(line_offset, clamped);
        Ok(true)
    }

    pub(super) fn set_source_cursor_to_selected_node_start(&mut self) {
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: node_id.to_string(),
            line_offset: 0,
        });
        let source = self.source_buffer.text().to_string();
        if let Some(entry) = source_outline_entry_for_node(&source, node_id) {
            self.source_cursor_byte_offset =
                source_byte_offset_for_line(&source, entry.line as usize);
        } else {
            self.source_cursor_byte_offset = self.source_cursor_byte_offset.min(source.len());
        }
    }

    fn set_source_cursor_for_selected_node_line(&mut self, line_offset: usize, byte_offset: usize) {
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        let source = self.source_buffer.text().to_string();
        self.source_cursor_byte_offset = byte_offset.min(source.len());
        if let Some(entry) = source_outline_entry_for_node(&source, node_id) {
            let max_offset = (entry.end_line - entry.line).max(0) as usize;
            let line_offset = line_offset.min(max_offset);
            let current_line = source_line_for_byte_offset(&source, self.source_cursor_byte_offset);
            if current_line < entry.line as usize || current_line > entry.end_line as usize {
                self.source_cursor_byte_offset =
                    source_byte_offset_for_line(&source, entry.line as usize + line_offset);
            }
            self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
                node_id: node_id.to_string(),
                line_offset,
            });
        } else {
            self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
                node_id: node_id.to_string(),
                line_offset,
            });
        }
    }

    pub(super) fn selected_source_line_offset(&self) -> Option<usize> {
        let selected_node_id = self.selection.primary_node_id.as_deref()?;
        self.source_cursor_anchor
            .as_ref()
            .filter(|anchor| anchor.node_id.as_str() == selected_node_id)
            .map(|anchor| anchor.line_offset)
    }

    pub(super) fn source_cursor_snapshot(&self) -> UiAssetEditorSourceCursorSnapshot {
        UiAssetEditorSourceCursorSnapshot {
            byte_offset: self.source_cursor_byte_offset,
            anchor_node_id: self
                .source_cursor_anchor
                .as_ref()
                .map(|anchor| anchor.node_id.clone()),
            line_offset: self
                .source_cursor_anchor
                .as_ref()
                .map(|anchor| anchor.line_offset)
                .unwrap_or_default(),
        }
    }

    pub(super) fn restore_source_cursor_snapshot(
        &mut self,
        snapshot: &UiAssetEditorSourceCursorSnapshot,
    ) {
        let source_len = self.source_buffer.text().len();
        self.source_cursor_byte_offset = snapshot.byte_offset.min(source_len);
        self.source_cursor_anchor =
            snapshot
                .anchor_node_id
                .as_ref()
                .map(|node_id| UiAssetSourceCursorAnchor {
                    node_id: node_id.clone(),
                    line_offset: snapshot.line_offset,
                });
    }

    pub(super) fn reconcile_source_cursor_state(&mut self) {
        let Some(selected_node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        let source = self.source_buffer.text().to_string();
        self.source_cursor_byte_offset = self.source_cursor_byte_offset.min(source.len());
        let Some(entry) = source_outline_entry_for_node(&source, selected_node_id) else {
            return;
        };
        let current_line = source_line_for_byte_offset(&source, self.source_cursor_byte_offset);
        let existing_line_offset = self
            .source_cursor_anchor
            .as_ref()
            .filter(|anchor| anchor.node_id.as_str() == selected_node_id)
            .map(|anchor| anchor.line_offset)
            .unwrap_or_default();
        let max_offset = (entry.end_line - entry.line).max(0) as usize;
        let inside_selected_block =
            current_line >= entry.line as usize && current_line <= entry.end_line as usize;
        let line_offset = if inside_selected_block {
            current_line.saturating_sub(entry.line as usize)
        } else {
            existing_line_offset.min(max_offset)
        };
        if !inside_selected_block {
            self.source_cursor_byte_offset =
                source_byte_offset_for_line(&source, entry.line as usize + line_offset);
        }
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: selected_node_id.to_string(),
            line_offset,
        });
    }

    fn select_node_id(&mut self, node_id: &str) {
        self.selection = selection_for_node(&self.last_valid_document, node_id);
        self.clear_palette_drag_state();
        self.reconcile_promote_widget_draft();
        reconcile_preview_mock_state(
            &self.last_valid_document,
            &self.selection,
            &mut self.preview_mock_state,
        );
        self.style_inspector = build_style_inspector(
            &self.last_valid_document,
            &self.selection,
            &self.compiler_imports,
            &self.style_inspector.active_pseudo_states,
        );
        self.selected_binding_index = reconcile_selected_binding_index(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
        );
        self.selected_binding_payload_key = reconcile_selected_binding_payload_key(
            &self.last_valid_document,
            &self.selection,
            self.selected_binding_index,
            self.selected_binding_payload_key.as_deref(),
        );
        self.selected_slot_semantic_path = reconcile_selected_semantic_path(
            &build_slot_semantic_group(&self.last_valid_document, &self.selection).entries,
            self.selected_slot_semantic_path.as_deref(),
        );
        self.selected_layout_semantic_path = reconcile_selected_semantic_path(
            &build_layout_semantic_group(&self.last_valid_document, &self.selection).entries,
            self.selected_layout_semantic_path.as_deref(),
        );
        self.selected_matched_style_rule_index = None;
    }
}
