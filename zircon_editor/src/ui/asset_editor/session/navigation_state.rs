use std::cell::Ref;

use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

use super::{
    binding_inspector::{reconcile_selected_binding_index, reconcile_selected_binding_payload_key},
    hierarchy_projection::{hierarchy_node_ids, selection_for_node},
    inspector_semantics::{
        build_layout_semantic_group, build_slot_semantic_group, reconcile_selected_semantic_path,
    },
    preview_mock::reconcile_preview_mock_state,
    preview_projection::preview_node_id_for_index,
    source_sync::{
        build_source_outline_index, source_byte_offset_for_line, source_line_for_byte_offset,
        UiAssetSourceOutlineIndex,
    },
    style_inspection::build_style_inspector,
    ui_asset_editor_session::{
        UiAssetEditorSession, UiAssetEditorSessionError, UiAssetSourceCursorAnchor,
    },
    undo_stack::UiAssetEditorSourceCursorSnapshot,
};

impl UiAssetEditorSession {
    pub(super) fn source_outline_index(&self) -> Ref<'_, UiAssetSourceOutlineIndex> {
        let source_revision = self.source_buffer.revision();
        if !self
            .source_outline_cache
            .borrow()
            .is_current(source_revision)
        {
            let outline =
                build_source_outline_index(&self.last_valid_document, self.source_buffer.text());
            record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneSourceBuildCount, 1.0);
            self.source_outline_cache
                .borrow_mut()
                .replace(source_revision, outline);
        }
        Ref::map(self.source_outline_cache.borrow(), |cache| cache.index())
    }

    pub(super) fn roundtrip_source_outline_index(&self) -> Ref<'_, UiAssetSourceOutlineIndex> {
        if self.diagnostics.is_empty() {
            return self.source_outline_index();
        }

        let source_generation = self.last_valid_source_generation;
        if !self
            .last_valid_source_outline_cache
            .borrow()
            .is_current(source_generation)
        {
            let outline =
                build_source_outline_index(&self.last_valid_document, &self.last_valid_source_text);
            record_current_ui_perf_counter(UiPerfCounter::AssetEditorPaneSourceBuildCount, 1.0);
            self.last_valid_source_outline_cache
                .borrow_mut()
                .replace(source_generation, outline);
        }
        Ref::map(self.last_valid_source_outline_cache.borrow(), |cache| {
            cache.index()
        })
    }

    #[cfg(test)]
    pub(super) fn source_outline_build_count(&self) -> usize {
        self.source_outline_cache.borrow().build_count()
    }

    #[cfg(test)]
    pub(super) fn source_outline_total_build_count(&self) -> usize {
        self.source_outline_cache.borrow().build_count()
            + self.last_valid_source_outline_cache.borrow().build_count()
    }

    #[cfg(test)]
    pub(super) fn source_outline_caches_share_index(&self) -> bool {
        let source_outline_cache = self.source_outline_cache.borrow();
        let last_valid_source_outline_cache = self.last_valid_source_outline_cache.borrow();
        source_outline_cache.shares_index_with(&last_valid_source_outline_cache)
    }

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
        let node_id = self
            .source_outline_index()
            .entries()
            .get(index)
            .map(|entry| entry.node_id.clone())
            .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index })?;
        self.select_node_id(&node_id);
        self.set_source_cursor_to_selected_node_start();
        Ok(())
    }

    pub fn select_source_line(&mut self, line: usize) -> Result<(), UiAssetEditorSessionError> {
        let (node_id, line_offset) = {
            let outline = self.source_outline_index();
            let node_id = outline
                .node_id_for_line(line)
                .map(str::to_owned)
                .ok_or(UiAssetEditorSessionError::InvalidSelectionIndex { index: line })?;
            let line_offset = outline
                .entry_for_node(&node_id)
                .map(|entry| line.saturating_sub(entry.line as usize))
                .unwrap_or_default();
            (node_id, line_offset)
        };
        let byte_offset = source_byte_offset_for_line(self.source_buffer.text(), line);
        self.select_node_id(&node_id);
        self.set_source_cursor_for_selected_node_line(line_offset, byte_offset);
        Ok(())
    }

    pub fn select_source_byte_offset(
        &mut self,
        byte_offset: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let clamped = byte_offset.min(self.source_buffer.text().len());
        let line = source_line_for_byte_offset(self.source_buffer.text(), clamped);
        let Some(node_id) = self
            .source_outline_index()
            .node_id_for_line(line)
            .map(str::to_owned)
        else {
            return Ok(false);
        };
        let line_offset = self
            .source_outline_index()
            .entry_for_node(&node_id)
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
        let source_line = self
            .source_outline_index()
            .entry_for_node(node_id)
            .map(|entry| entry.line as usize);
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: node_id.to_string(),
            line_offset: 0,
        });
        if let Some(source_line) = source_line {
            self.source_cursor_byte_offset =
                source_byte_offset_for_line(self.source_buffer.text(), source_line);
        } else {
            self.source_cursor_byte_offset = self
                .source_cursor_byte_offset
                .min(self.source_buffer.text().len());
        }
    }

    fn set_source_cursor_for_selected_node_line(&mut self, line_offset: usize, byte_offset: usize) {
        let Some(node_id) = self.selection.primary_node_id.as_deref() else {
            self.source_cursor_anchor = None;
            self.source_cursor_byte_offset = 0;
            return;
        };
        let entry_range = self
            .source_outline_index()
            .entry_for_node(node_id)
            .map(|entry| (entry.line as usize, entry.end_line as usize));
        self.source_cursor_byte_offset = byte_offset.min(self.source_buffer.text().len());
        if let Some((start_line, end_line)) = entry_range {
            let max_offset = end_line.saturating_sub(start_line);
            let line_offset = line_offset.min(max_offset);
            let current_line = source_line_for_byte_offset(
                self.source_buffer.text(),
                self.source_cursor_byte_offset,
            );
            if current_line < start_line || current_line > end_line {
                self.source_cursor_byte_offset = source_byte_offset_for_line(
                    self.source_buffer.text(),
                    start_line + line_offset,
                );
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
        self.source_cursor_byte_offset = self
            .source_cursor_byte_offset
            .min(self.source_buffer.text().len());
        let entry_range = self
            .source_outline_index()
            .entry_for_node(selected_node_id)
            .map(|entry| (entry.line as usize, entry.end_line as usize));
        let Some((start_line, end_line)) = entry_range else {
            return;
        };
        let current_line =
            source_line_for_byte_offset(self.source_buffer.text(), self.source_cursor_byte_offset);
        let existing_line_offset = self
            .source_cursor_anchor
            .as_ref()
            .filter(|anchor| anchor.node_id.as_str() == selected_node_id)
            .map(|anchor| anchor.line_offset)
            .unwrap_or_default();
        let max_offset = end_line.saturating_sub(start_line);
        let inside_selected_block = current_line >= start_line && current_line <= end_line;
        let line_offset = if inside_selected_block {
            current_line.saturating_sub(start_line)
        } else {
            existing_line_offset.min(max_offset)
        };
        if !inside_selected_block {
            self.source_cursor_byte_offset =
                source_byte_offset_for_line(self.source_buffer.text(), start_line + line_offset);
        }
        self.source_cursor_anchor = Some(UiAssetSourceCursorAnchor {
            node_id: selected_node_id.to_string(),
            line_offset,
        });
    }

    fn select_node_id(&mut self, node_id: &str) {
        self.selection = selection_for_node(&self.last_valid_document, node_id);
        self.last_preview_interact_dispatch = None;
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

#[cfg(test)]
mod tests {
    use super::UiAssetEditorSession;
    use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetEditorRoute};
    use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

    const OUTLINE_CACHE_LAYOUT: &str = r#"[asset]
kind = "layout"
id = "editor.test.outline_cache"
version = 1
display_name = "Outline Cache"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
"#;

    #[test]
    fn source_generation_reuses_the_outline_across_presentation_and_navigation() {
        let route = UiAssetEditorRoute::new(
            "editor.test.outline_cache",
            UiAssetKind::Layout,
            UiAssetEditorMode::Split,
        );
        let mut session = UiAssetEditorSession::from_source(
            route,
            OUTLINE_CACHE_LAYOUT,
            UiSize::new(640.0, 360.0),
        )
        .expect("session");

        assert_eq!(session.source_outline_build_count(), 1);
        assert_eq!(session.source_outline_total_build_count(), 1);
        assert!(session.source_outline_caches_share_index());
        let initial_presentation = session.pane_presentation();
        assert_eq!(initial_presentation.source_outline_items.len(), 1);
        let root_line = session
            .source_outline_index()
            .entry_for_node("root")
            .expect("root outline entry")
            .line as usize;
        session.select_source_line(root_line).expect("source line");
        let _ = session.pane_presentation();
        assert_eq!(session.source_outline_build_count(), 1);
        assert_eq!(session.source_outline_total_build_count(), 1);

        session
            .source_buffer
            .replace(format!("{OUTLINE_CACHE_LAYOUT}\n# source generation one"));
        session
            .select_source_line(root_line)
            .expect("revised source line");
        assert_eq!(session.source_outline_build_count(), 2);
        assert_eq!(session.source_outline_total_build_count(), 2);
        assert!(!session.source_outline_caches_share_index());

        let document = session.last_valid_document.clone();
        session
            .apply_valid_document(document)
            .expect("document refresh");
        session
            .select_source_line(root_line)
            .expect("refreshed source line");
        assert_eq!(session.source_outline_build_count(), 3);
        assert_eq!(session.source_outline_total_build_count(), 3);
        assert!(session.source_outline_caches_share_index());

        session
            .source_buffer
            .replace(format!("{OUTLINE_CACHE_LAYOUT}\n# invalid source draft"));
        session.diagnostics.push("invalid source draft".to_string());
        let _ = session.pane_presentation();
        assert_eq!(session.source_outline_build_count(), 3);
        assert_eq!(session.source_outline_total_build_count(), 3);
    }

    #[test]
    fn pane_presentation_is_value_equivalent_without_a_session_mutation() {
        let route = UiAssetEditorRoute::new(
            "editor.test.presentation_equivalence",
            UiAssetKind::Layout,
            UiAssetEditorMode::Split,
        );
        let session = UiAssetEditorSession::from_source(
            route,
            OUTLINE_CACHE_LAYOUT,
            UiSize::new(640.0, 360.0),
        )
        .expect("session");

        let first = session.pane_presentation();
        let repeated = session.pane_presentation();

        assert_eq!(repeated, first);
    }
}
