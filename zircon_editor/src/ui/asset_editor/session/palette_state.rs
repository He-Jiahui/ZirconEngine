use super::{
    command::UiAssetEditorTreeEdit,
    hierarchy_projection::selection_for_node,
    palette::{insert_palette_item_with_placement, PaletteInsertMode},
    palette_drop::{
        build_palette_insert_plan,
        resolve_palette_drag_target as resolve_palette_drag_target_for_preview,
        UiAssetPaletteDragResolution, UiAssetPaletteDragTarget, UiAssetPaletteInsertPlan,
    },
    palette_target_chooser::{reconcile_palette_target_chooser, UiAssetPaletteTargetChooser},
    preview_projection::build_preview_hit_index,
    tree_editing::{
        move_selected_node as tree_move_selected_node,
        reparent_selected_node as tree_reparent_selected_node, UiTreeMoveDirection,
        UiTreeReparentDirection,
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
        let entry = self
            .palette_catalog
            .entry(index)
            .cloned()
            .ok_or(UiAssetEditorSessionError::InvalidPaletteIndex { index })?;
        let changed = self.selected_palette_index != Some(index);
        self.selected_palette_index = Some(index);
        self.selected_palette_entry = Some(entry);
        if changed {
            self.clear_palette_drag_state();
        }
        Ok(changed)
    }

    #[cfg(test)]
    pub(super) fn palette_catalog_build_count(&self) -> usize {
        self.palette_catalog_build_count
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
        let next_resolution = self.resolve_palette_drag_target(surface_x, surface_y);
        let previous = self.palette_target_chooser.take();
        let (next, changed) = reconcile_palette_target_chooser(previous, next_resolution);
        self.palette_target_chooser = next;
        Ok(changed)
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
        &mut self,
        surface_x: f32,
        surface_y: f32,
    ) -> Option<UiAssetPaletteDragResolution> {
        if !self.diagnostics.is_empty() {
            return None;
        }
        if self.selected_palette_entry.is_none() {
            return None;
        }
        self.ensure_preview_hit_index();
        let entry = self.selected_palette_entry.as_ref()?;
        let hit_index = self.preview_hit_index.as_ref()?;
        resolve_palette_drag_target_for_preview(
            &self.last_valid_document,
            entry,
            self.palette_catalog.reference_imports(),
            hit_index,
            surface_x,
            surface_y,
        )
    }

    fn ensure_preview_hit_index(&mut self) {
        if self.preview_hit_index.is_none() {
            #[cfg(test)]
            {
                self.preview_hit_index_build_count += 1;
            }
            self.preview_hit_index =
                build_preview_hit_index(&self.last_valid_document, self.preview_host.as_ref());
        }
    }

    #[cfg(test)]
    pub(super) fn preview_hit_index_build_count(&self) -> usize {
        self.preview_hit_index_build_count
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
        let Some(entry) = self.selected_palette_entry.clone() else {
            return Ok(false);
        };
        let Some(plan) = build_palette_insert_plan(
            &self.last_valid_document,
            &entry,
            target_node_id,
            mode,
            self.palette_catalog.reference_imports(),
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
        let Some(entry) = self.selected_palette_entry.clone() else {
            return Ok(false);
        };
        let mut document = self.last_valid_document.clone();
        let Some(node_id) = insert_palette_item_with_placement(
            &mut document,
            &plan.node_id,
            &entry,
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

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::UiAssetEditorSession;
    use crate::ui::asset_editor::{
        UiAssetEditorMode, UiAssetEditorRoute, UiAssetPaletteEntryKind, UiAssetPreviewPreset,
    };
    use zircon_runtime::ui::v2::UiV2AssetLoader;
    use zircon_runtime_interface::ui::{layout::UiSize, template::UiAssetKind};

    const PREVIEW_HIT_INDEX_LAYOUT: &str = r#"
[asset]
kind = "layout"
id = "editor.test.preview_hit_index"
version = 1
display_name = "Preview Hit Index"

[root]
node = "root"

[nodes.root]
kind = "native"
type = "VerticalBox"
control_id = "Root"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" }, container = { kind = "VerticalBox", gap = 8.0 } }
children = [{ child = "status" }]

[nodes.status]
kind = "native"
type = "Label"
control_id = "StatusLabel"
props = { text = "Ready" }
layout = { width = { stretch = "Stretch" }, height = { min = 24.0, preferred = 24.0, max = 24.0, stretch = "Fixed" } }
"#;

    const V2_PALETTE_VIEW: &str = r#"
[asset]
kind = "view"
id = "editor.test.palette_catalog.v2"
version = 2
display_name = "Palette Catalog V2"

[root]
node = "root"

[nodes.root]
component = "VerticalGroup"
"#;

    const V2_PALETTE_COMPONENT: &str = r#"
[asset]
kind = "component"
id = "editor.test.palette_catalog.component"
version = 2
display_name = "Palette Catalog Component"

[components.ImportedWidget]
root = "imported_root"

[nodes.imported_root]
component = "Text"
props = { text = "Imported" }
"#;

    #[test]
    fn palette_drag_reuses_the_hit_index_until_preview_or_document_rebuild() {
        let route = UiAssetEditorRoute::new(
            "editor.test.preview_hit_index",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        );
        let mut session = UiAssetEditorSession::from_source(
            route,
            PREVIEW_HIT_INDEX_LAYOUT,
            UiSize::new(640.0, 360.0),
        )
        .expect("session");
        session
            .select_palette_index(0)
            .expect("native palette entry");

        session
            .update_palette_drag_target(16.0, 16.0)
            .expect("first palette drag resolution");
        let first_target = session
            .selected_palette_drag_target()
            .cloned()
            .expect("first palette drag target");
        assert_eq!(session.preview_hit_index_build_count(), 1);

        session
            .update_palette_drag_target(16.0, 16.0)
            .expect("stable palette drag resolution");
        assert_eq!(session.preview_hit_index_build_count(), 1);
        assert_eq!(
            session.selected_palette_drag_target(),
            Some(&first_target),
            "stable hover preserves the same drag target semantics"
        );

        session.rebuild_preview_snapshot().expect("preview rebuild");
        assert!(session.preview_hit_index.is_none());
        session
            .update_palette_drag_target(16.0, 16.0)
            .expect("drag after preview rebuild");
        assert_eq!(session.preview_hit_index_build_count(), 2);

        session
            .set_preview_preset(UiAssetPreviewPreset::Dialog)
            .expect("preview preset");
        assert!(session.preview_hit_index.is_none());
        session
            .update_palette_drag_target(16.0, 16.0)
            .expect("drag after preview resize");
        assert_eq!(session.preview_hit_index_build_count(), 3);

        let document = session.last_valid_document.clone();
        session
            .apply_valid_document(document)
            .expect("document replacement");
        assert!(session.preview_hit_index.is_none());
        session
            .update_palette_drag_target(16.0, 16.0)
            .expect("drag after document replacement");
        assert_eq!(session.preview_hit_index_build_count(), 4);
    }

    #[test]
    fn palette_catalog_is_reused_until_the_document_generation_changes() {
        let route = UiAssetEditorRoute::new(
            "editor.test.palette_catalog",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        );
        let mut session = UiAssetEditorSession::from_source(
            route,
            PREVIEW_HIT_INDEX_LAYOUT,
            UiSize::new(640.0, 360.0),
        )
        .expect("session");

        assert_eq!(session.palette_catalog_build_count(), 1);
        let first_presentation = session.pane_presentation();
        assert!(!first_presentation.palette_items.is_empty());
        session.select_palette_index(0).expect("palette selection");
        let repeated_presentation = session.pane_presentation();
        assert_eq!(
            repeated_presentation.palette_items,
            first_presentation.palette_items
        );
        assert_eq!(session.palette_catalog_build_count(), 1);

        let mut imported_widget = session.last_valid_document.clone();
        imported_widget.asset.kind = UiAssetKind::Widget;
        session
            .register_widget_import(
                "res://ui/widgets/imported.zui#ImportedWidget",
                imported_widget,
            )
            .expect("widget import");
        assert_eq!(session.palette_catalog_build_count(), 2);
        assert!(session
            .pane_presentation()
            .palette_items
            .iter()
            .any(|item| item == "Reference / ImportedWidget"));

        let mut imported_style = session.last_valid_document.clone();
        imported_style.asset.kind = UiAssetKind::Style;
        session
            .register_style_import("res://ui/styles/imported.zss", imported_style)
            .expect("style import");
        assert_eq!(session.palette_catalog_build_count(), 2);

        let imported_index = session.palette_catalog.entries().len() - 1;
        session
            .select_palette_index(imported_index)
            .expect("imported palette selection");
        assert!(session
            .selected_palette_entry
            .as_ref()
            .expect("selected imported palette entry")
            .label
            .starts_with("Reference / "));
        session
            .replace_resolved_imports(
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
            )
            .expect("clear imports");
        assert_eq!(session.palette_catalog_build_count(), 3);
        assert!(session
            .selected_palette_entry
            .as_ref()
            .expect("reconciled palette entry")
            .label
            .starts_with("Native / "));

        let document = session.last_valid_document.clone();
        session
            .apply_valid_document(document)
            .expect("document refresh");
        assert_eq!(session.palette_catalog_build_count(), 4);
    }

    #[test]
    fn v2_widget_import_refreshes_the_palette_catalog() {
        let route = UiAssetEditorRoute::new(
            "editor.test.palette_catalog.v2",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        );
        let mut session =
            UiAssetEditorSession::from_v2_source(route, V2_PALETTE_VIEW, UiSize::new(640.0, 360.0))
                .expect("v2 session");
        let component = UiV2AssetLoader::load_toml_str(V2_PALETTE_COMPONENT).expect("v2 component");

        assert_eq!(session.palette_catalog_build_count(), 1);
        session
            .register_v2_widget_import("res://ui/widgets/imported_widget.zui", component)
            .expect("v2 widget import");

        assert_eq!(session.palette_catalog_build_count(), 2);
        assert!(session
            .pane_presentation()
            .palette_items
            .iter()
            .any(|item| item == "Reference / ImportedWidget"));
        let imported_index = session
            .palette_catalog
            .entries()
            .iter()
            .position(|entry| entry.label == "Reference / ImportedWidget")
            .expect("v2 palette reference");
        assert!(session
            .palette_catalog
            .reference_imports()
            .contains_key("res://ui/widgets/imported_widget.zui#ImportedWidget"));
        session
            .select_palette_index(imported_index)
            .expect("select v2 palette reference");
        assert!(session.pane_presentation().can_insert_child);
        assert!(session
            .insert_selected_palette_item_as_child()
            .expect("insert v2 palette reference"));
    }

    #[test]
    fn palette_reference_selection_survives_a_lexically_earlier_import() {
        let route = UiAssetEditorRoute::new(
            "editor.test.palette_catalog.v2.selection",
            UiAssetKind::Layout,
            UiAssetEditorMode::Design,
        );
        let mut session =
            UiAssetEditorSession::from_v2_source(route, V2_PALETTE_VIEW, UiSize::new(640.0, 360.0))
                .expect("v2 session");
        let component = UiV2AssetLoader::load_toml_str(V2_PALETTE_COMPONENT).expect("v2 component");
        let selected_reference = "res://ui/widgets/z_last.zui#ImportedWidget";

        session
            .register_v2_widget_import(selected_reference, component.clone())
            .expect("register selected reference");
        let selected_index = session
            .palette_catalog
            .entries()
            .iter()
            .position(|entry| {
                matches!(
                    &entry.kind,
                    UiAssetPaletteEntryKind::Reference { component_ref }
                        if component_ref == selected_reference
                )
            })
            .expect("selected reference index");
        session
            .select_palette_index(selected_index)
            .expect("select reference B");

        session
            .register_v2_widget_import("res://ui/widgets/a_first.zui#ImportedWidget", component)
            .expect("register lexically earlier reference");

        assert!(matches!(
            session
                .selected_palette_entry
                .as_ref()
                .map(|entry| &entry.kind),
            Some(UiAssetPaletteEntryKind::Reference { component_ref })
                if component_ref == selected_reference
        ));
    }
}
