use super::{
    command::UiAssetEditorTreeEdit,
    hierarchy_projection::selection_for_node,
    palette_drop::{
        build_palette_insert_plan,
        resolve_palette_drag_target as resolve_palette_drag_target_for_preview,
        UiAssetPaletteDragResolution, UiAssetPaletteDragTarget, UiAssetPaletteInsertPlan,
    },
    palette_target_chooser::{reconcile_palette_target_chooser, UiAssetPaletteTargetChooser},
    preview_projection::build_preview_projection,
    tree_editing::{
        build_palette_entries, insert_palette_item_with_placement,
        move_selected_node as tree_move_selected_node,
        reparent_selected_node as tree_reparent_selected_node, PaletteInsertMode,
        UiTreeMoveDirection, UiTreeReparentDirection,
    },
    ui_asset_editor_session::{
        move_direction_label, palette_insert_mode_label, reparent_direction_label,
        UiAssetEditorSession, UiAssetEditorSessionError,
    },
};

impl UiAssetEditorSession {
    pub fn select_palette_index(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        if index >= palette_entries.len() {
            return Err(UiAssetEditorSessionError::InvalidPaletteIndex { index });
        }
        let changed = self.selected_palette_index != Some(index);
        self.selected_palette_index = Some(index);
        if changed {
            self.clear_palette_drag_state();
        }
        Ok(changed)
    }

    pub fn insert_selected_palette_item_as_child(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.insert_selected_palette_item(PaletteInsertMode::Child)
    }

    pub fn insert_selected_palette_item_after_selection(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.insert_selected_palette_item(PaletteInsertMode::After)
    }

    pub fn update_palette_drag_target(
        &mut self,
        surface_x: f32,
        surface_y: f32,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let next = reconcile_palette_target_chooser(
            self.palette_target_chooser.as_ref(),
            self.resolve_palette_drag_target(surface_x, surface_y),
        );
        if self.palette_target_chooser == next {
            return Ok(false);
        }
        self.palette_target_chooser = next;
        Ok(true)
    }

    pub fn clear_palette_drag_target(&mut self) -> bool {
        let changed = self.palette_target_chooser.is_some();
        self.clear_palette_drag_state();
        changed
    }

    pub fn cycle_palette_drag_target_candidate_next(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.cycle_palette_drag_target_candidate(1)
    }

    pub fn cycle_palette_drag_target_candidate_previous(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.cycle_palette_drag_target_candidate(-1)
    }

    fn cycle_palette_drag_target_candidate(
        &mut self,
        direction: isize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(chooser) = self.palette_target_chooser.as_mut() else {
            return Ok(false);
        };
        let resolution = chooser.resolution_mut();
        if resolution.candidates.len() <= 1 {
            return Ok(false);
        }

        let candidate_count = resolution.candidates.len() as isize;
        let current = resolution.selected_index as isize;
        let next = (current + direction).rem_euclid(candidate_count) as usize;
        if next == resolution.selected_index {
            return Ok(false);
        }
        resolution.selected_index = next;
        chooser.set_manual_selection(true);
        Ok(true)
    }

    pub fn select_palette_target_candidate(
        &mut self,
        index: usize,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(chooser) = self.palette_target_chooser.as_mut() else {
            return Ok(false);
        };
        if index >= chooser.resolution().candidates.len() {
            return Err(UiAssetEditorSessionError::InvalidSelectionIndex { index });
        }
        Ok(chooser.select_candidate(index))
    }

    pub(super) fn selected_insert_target_node_id(&self) -> Option<&str> {
        self.selection
            .primary_node_id
            .as_deref()
            .or_else(|| self.last_valid_document.root_node_id())
    }

    pub fn drop_selected_palette_item_at_palette_drag_target(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        if let Some(chooser) = self.palette_target_chooser.as_mut() {
            if chooser.arm_sticky() {
                return Ok(true);
            }
        }
        self.confirm_palette_target_choice()
    }

    pub fn confirm_palette_target_choice(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let Some(target) = self.selected_palette_drag_target().cloned() else {
            return Ok(false);
        };
        let changed = self.insert_selected_palette_item_with_plan(&target.plan)?;
        self.clear_palette_drag_state();
        Ok(changed)
    }

    pub fn cancel_palette_target_choice(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        let changed = self.palette_target_chooser.is_some();
        self.clear_palette_drag_state();
        Ok(changed)
    }

    fn resolve_palette_drag_target(
        &self,
        surface_x: f32,
        surface_y: f32,
    ) -> Option<UiAssetPaletteDragResolution> {
        if !self.diagnostics.is_empty() {
            return None;
        }
        let Some(preview_host) = self.preview_host.as_ref() else {
            return None;
        };
        let Some(selected_palette_index) = self.selected_palette_index else {
            return None;
        };
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        let entry = palette_entries.get(selected_palette_index)?;
        let projection = build_preview_projection(
            &self.last_valid_document,
            Some(preview_host),
            &self.selection,
        );
        resolve_palette_drag_target_for_preview(
            &self.last_valid_document,
            entry,
            &self.compiler_imports.widgets,
            &projection,
            surface_x,
            surface_y,
        )
    }

    pub(super) fn selected_palette_drag_target(&self) -> Option<&UiAssetPaletteDragTarget> {
        self.palette_target_chooser
            .as_ref()
            .and_then(UiAssetPaletteTargetChooser::selected_target)
    }

    pub(super) fn clear_palette_drag_state(&mut self) {
        self.palette_target_chooser = None;
    }

    pub fn move_selected_node_up(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_node(UiTreeMoveDirection::Up)
    }

    pub fn move_selected_node_down(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.move_selected_node(UiTreeMoveDirection::Down)
    }

    pub fn reparent_selected_node_into_previous(
        &mut self,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.reparent_selected_node(UiTreeReparentDirection::IntoPrevious)
    }

    pub fn reparent_selected_node_into_next(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.reparent_selected_node(UiTreeReparentDirection::IntoNext)
    }

    pub fn reparent_selected_node_outdent(&mut self) -> Result<bool, UiAssetEditorSessionError> {
        self.reparent_selected_node(UiTreeReparentDirection::Outdent)
    }

    fn insert_selected_palette_item(
        &mut self,
        mode: PaletteInsertMode,
    ) -> Result<bool, UiAssetEditorSessionError> {
        let Some(node_id) = self.selected_insert_target_node_id().map(str::to_string) else {
            return Ok(false);
        };
        self.insert_selected_palette_item_at_target(mode, &node_id)
    }

    fn insert_selected_palette_item_at_target(
        &mut self,
        mode: PaletteInsertMode,
        target_node_id: &str,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_palette_index) = self.selected_palette_index else {
            return Ok(false);
        };
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        let Some(entry) = palette_entries.get(selected_palette_index) else {
            return Ok(false);
        };
        let Some(plan) = build_palette_insert_plan(
            &self.last_valid_document,
            entry,
            target_node_id,
            mode,
            &self.compiler_imports.widgets,
            None,
        ) else {
            return Ok(false);
        };
        self.insert_selected_palette_item_with_plan(&plan)
    }

    fn insert_selected_palette_item_with_plan(
        &mut self,
        plan: &UiAssetPaletteInsertPlan,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(selected_palette_index) = self.selected_palette_index else {
            return Ok(false);
        };
        let palette_entries =
            build_palette_entries(&self.last_valid_document, &self.compiler_imports.widgets);
        let Some(entry) = palette_entries.get(selected_palette_index) else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(node_id) = insert_palette_item_with_placement(
            &mut document,
            &plan.node_id,
            entry,
            plan.mode,
            &plan.placement,
        ) else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &node_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::InsertPaletteItem {
                node_id,
                parent_node_id: selection.parent_node_id.clone(),
                palette_item_label: entry.label.clone(),
                insert_mode: palette_insert_mode_label(plan.mode).to_string(),
            },
            "Insert Palette Item",
            selection,
        )?;
        Ok(true)
    }

    fn move_selected_node(
        &mut self,
        direction: UiTreeMoveDirection,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let Some(node_id) = self.selection.primary_node_id.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        if !tree_move_selected_node(&mut document, &self.selection, direction) {
            return Ok(false);
        }
        self.apply_document_edit_with_tree_edit(
            document,
            UiAssetEditorTreeEdit::MoveNode {
                node_id,
                direction: move_direction_label(direction).to_string(),
            },
            "Move Node",
        )?;
        Ok(true)
    }

    fn reparent_selected_node(
        &mut self,
        direction: UiTreeReparentDirection,
    ) -> Result<bool, UiAssetEditorSessionError> {
        self.ensure_editable_source()?;
        let mut document = self.last_valid_document.clone();
        let Some(node_id) = tree_reparent_selected_node(&mut document, &self.selection, direction)
        else {
            return Ok(false);
        };
        let selection = selection_for_node(&document, &node_id);
        self.apply_document_edit_with_tree_edit_and_selection(
            document.clone(),
            UiAssetEditorTreeEdit::ReparentNode {
                node_id,
                parent_node_id: selection.parent_node_id.clone(),
                direction: reparent_direction_label(direction).to_string(),
            },
            "Reparent Node",
            selection,
        )?;
        Ok(true)
    }
}
