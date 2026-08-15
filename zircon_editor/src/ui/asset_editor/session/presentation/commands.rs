use zircon_runtime_interface::ui::template::UiAssetDocument;

use super::super::{
    palette::{can_convert_selected_node_to_reference, PaletteInsertMode},
    palette_drop::can_insert_palette_item_for_node as can_insert_palette_item_at_node,
    promote_widget::can_promote_selected_component_to_external_widget,
    style_inspection::selected_node_has_inline_overrides,
    tree_editing::{
        can_extract_selected_node_to_component, move_selected_node,
        reparent_selected_node as tree_reparent_selected_node, unwrap_selected_node,
        wrap_selected_node, UiTreeMoveDirection, UiTreeReparentDirection,
    },
    ui_asset_editor_session::UiAssetEditorSession,
};
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(super) struct UiAssetCommandAvailability {
    pub(super) can_create_rule: bool,
    pub(super) can_extract_rule: bool,
    pub(super) can_insert_child: bool,
    pub(super) can_insert_after: bool,
    pub(super) can_move_up: bool,
    pub(super) can_move_down: bool,
    pub(super) can_reparent_into_previous: bool,
    pub(super) can_reparent_into_next: bool,
    pub(super) can_reparent_outdent: bool,
    pub(super) can_open_reference: bool,
    pub(super) can_convert_to_reference: bool,
    pub(super) can_extract_component: bool,
    pub(super) can_promote_to_external_widget: bool,
    pub(super) can_wrap_in_vertical_box: bool,
    pub(super) can_unwrap: bool,
}

impl UiAssetEditorSession {
    pub(super) fn command_availability(
        &self,
        has_selected_node_selector: bool,
    ) -> UiAssetCommandAvailability {
        zircon_runtime::profile_scope!(
            "editor",
            "asset_editor.presentation",
            "command_availability",
        );
        let diagnostics_empty = self.diagnostics.is_empty();
        let palette_entries = self.palette_catalog.entries();
        let can_create_rule =
            diagnostics_empty && has_selected_node_selector && self.preview_host.is_some();
        let can_extract_rule = can_create_rule
            && selected_node_has_inline_overrides(&self.last_valid_document, &self.selection);
        let can_insert_child = diagnostics_empty
            && self
                .selected_insert_target_node_id()
                .and_then(|node_id| {
                    self.selected_palette_index
                        .and_then(|index| palette_entries.get(index).map(|entry| (node_id, entry)))
                })
                .is_some_and(|(node_id, entry)| {
                    can_insert_palette_item_at_node(
                        &self.last_valid_document,
                        entry,
                        node_id,
                        PaletteInsertMode::Child,
                        self.palette_catalog.reference_imports(),
                    )
                });
        let can_insert_after = diagnostics_empty
            && self
                .selected_insert_target_node_id()
                .and_then(|node_id| {
                    self.selected_palette_index
                        .and_then(|index| palette_entries.get(index).map(|entry| (node_id, entry)))
                })
                .is_some_and(|(node_id, entry)| {
                    can_insert_palette_item_at_node(
                        &self.last_valid_document,
                        entry,
                        node_id,
                        PaletteInsertMode::After,
                        self.palette_catalog.reference_imports(),
                    )
                });
        let can_move_up = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                move_selected_node(document, &self.selection, UiTreeMoveDirection::Up)
            });
        let can_move_down = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                move_selected_node(document, &self.selection, UiTreeMoveDirection::Down)
            });
        let can_reparent_into_previous = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                tree_reparent_selected_node(
                    document,
                    &self.selection,
                    UiTreeReparentDirection::IntoPrevious,
                )
                .is_some()
            });
        let can_reparent_into_next = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                tree_reparent_selected_node(
                    document,
                    &self.selection,
                    UiTreeReparentDirection::IntoNext,
                )
                .is_some()
            });
        let can_reparent_outdent = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                tree_reparent_selected_node(
                    document,
                    &self.selection,
                    UiTreeReparentDirection::Outdent,
                )
                .is_some()
            });
        let can_open_reference = self.selected_reference_asset_id().is_some();
        let can_convert_to_reference = self
            .selected_palette_index
            .and_then(|index| palette_entries.get(index))
            .is_some_and(|entry| {
                can_convert_selected_node_to_reference(
                    &self.last_valid_document,
                    &self.selection,
                    entry,
                    self.palette_catalog.reference_imports(),
                )
            });
        let can_extract_component = diagnostics_empty
            && can_extract_selected_node_to_component(&self.last_valid_document, &self.selection);
        let can_promote_to_external_widget = diagnostics_empty
            && can_promote_selected_component_to_external_widget(
                &self.last_valid_document,
                &self.selection,
            );
        let can_wrap_in_vertical_box = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                wrap_selected_node(document, &self.selection, "VerticalBox").is_some()
            });
        let can_unwrap = diagnostics_empty
            && can_apply_tree_document_edit(&self.last_valid_document, |document| {
                unwrap_selected_node(document, &self.selection).is_some()
            });
        record_current_ui_perf_counter(
            UiPerfCounter::AssetEditorPaneCommandAvailabilityBuildCount,
            1.0,
        );
        UiAssetCommandAvailability {
            can_create_rule,
            can_extract_rule,
            can_insert_child,
            can_insert_after,
            can_move_up,
            can_move_down,
            can_reparent_into_previous,
            can_reparent_into_next,
            can_reparent_outdent,
            can_open_reference,
            can_convert_to_reference,
            can_extract_component,
            can_promote_to_external_widget,
            can_wrap_in_vertical_box,
            can_unwrap,
        }
    }
}

fn can_apply_tree_document_edit(
    document: &UiAssetDocument,
    edit: impl FnOnce(&mut UiAssetDocument) -> bool,
) -> bool {
    let mut document = document.clone();
    edit(&mut document)
}
