use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::asset_editor::UiDesignerToolMode;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_designer_tool_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "designer.tool.select" => {
                self.set_ui_asset_editor_designer_tool(instance_id, UiDesignerToolMode::Select)
            }
            "designer.tool.resize_slot" => {
                self.set_ui_asset_editor_designer_tool(instance_id, UiDesignerToolMode::ResizeSlot)
            }
            "designer.tool.preview_interact" => self.set_ui_asset_editor_designer_tool(
                instance_id,
                UiDesignerToolMode::PreviewInteract,
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }

    fn set_ui_asset_editor_designer_tool(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiDesignerToolMode,
    ) -> UiAssetActionDispatch {
        UiAssetActionDispatch::handled(
            self.editor_manager
                .set_ui_asset_editor_designer_tool_mode(instance_id, mode),
        )
    }
}
