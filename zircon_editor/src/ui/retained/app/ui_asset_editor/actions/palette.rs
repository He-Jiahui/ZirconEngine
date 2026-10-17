use super::{UiAssetActionDispatch, *};
use crate::ui::host::EditorError;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_palette_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "palette.insert.child" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.insert_ui_asset_editor_selected_palette_item_as_child(instance_id)
                    }),
            ),
            "palette.insert.after" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.insert_ui_asset_editor_selected_palette_item_after_selection(
                            instance_id,
                        )
                    }),
            ),
            "palette.drag.drop" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager
                            .drop_ui_asset_editor_selected_palette_item_at_drag_target(instance_id)
                    }),
            ),
            "palette.drag.cancel" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.clear_ui_asset_editor_palette_drag_target(instance_id)
                    }),
            ),
            "palette.target.previous" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.cycle_ui_asset_editor_palette_drag_target_candidate_previous(
                            instance_id,
                        )
                    }),
            ),
            "palette.target.next" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager
                            .cycle_ui_asset_editor_palette_drag_target_candidate_next(instance_id)
                    }),
            ),
            "palette.target.confirm" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.confirm_ui_asset_editor_palette_target_choice(instance_id)
                    }),
            ),
            "palette.target.cancel" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .enter()
                    .map_err(EditorError::from)
                    .and_then(|manager| {
                        manager.cancel_ui_asset_editor_palette_target_choice(instance_id)
                    }),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
