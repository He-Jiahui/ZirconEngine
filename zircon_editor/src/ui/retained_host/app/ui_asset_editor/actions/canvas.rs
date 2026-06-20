use super::{UiAssetActionDispatch, *};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_canvas_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "canvas.move.up" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .move_ui_asset_editor_selected_node_up(instance_id),
            ),
            "canvas.move.down" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .move_ui_asset_editor_selected_node_down(instance_id),
            ),
            "canvas.reparent.into_previous" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .reparent_ui_asset_editor_selected_node_into_previous(instance_id),
            ),
            "canvas.reparent.into_next" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .reparent_ui_asset_editor_selected_node_into_next(instance_id),
            ),
            "canvas.reparent.outdent" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .reparent_ui_asset_editor_selected_node_outdent(instance_id),
            ),
            "canvas.convert.reference" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .convert_ui_asset_editor_selected_node_to_reference(instance_id),
            ),
            "canvas.extract.component" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .extract_ui_asset_editor_selected_node_to_component(instance_id),
            ),
            "canvas.promote.widget" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .promote_ui_asset_editor_selected_component_to_external_widget(instance_id),
            ),
            "canvas.wrap.vertical_box" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .wrap_ui_asset_editor_selected_node(instance_id, "VerticalBox"),
            ),
            "canvas.unwrap" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .unwrap_ui_asset_editor_selected_node(instance_id),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
