use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_pseudo_state_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "style.state.hover" => self.toggle_ui_asset_editor_pseudo_state(instance_id, "hover"),
            "style.state.focus" => self.toggle_ui_asset_editor_pseudo_state(instance_id, "focus"),
            "style.state.pressed" => {
                self.toggle_ui_asset_editor_pseudo_state(instance_id, "pressed")
            }
            "style.state.disabled" => {
                self.toggle_ui_asset_editor_pseudo_state(instance_id, "disabled")
            }
            "style.state.selected" => {
                self.toggle_ui_asset_editor_pseudo_state(instance_id, "selected")
            }
            _ => UiAssetActionDispatch::Unhandled,
        }
    }

    fn toggle_ui_asset_editor_pseudo_state(
        &self,
        instance_id: &ViewInstanceId,
        state: &str,
    ) -> UiAssetActionDispatch {
        UiAssetActionDispatch::handled(
            self.editor_manager
                .toggle_ui_asset_editor_pseudo_state(instance_id, state),
        )
    }
}
