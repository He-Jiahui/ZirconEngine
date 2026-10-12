use super::{UiAssetActionDispatch, *};
use crate::ui::workbench::view::ViewInstanceId;

#[path = "../../actions/mode_preview/designer_tool.rs"]
mod designer_tool;
#[path = "../../actions/mode_preview/editor_mode.rs"]
mod editor_mode;
#[path = "mode_preview/locale.rs"]
mod locale;
#[path = "../../actions/mode_preview/preview_preset.rs"]
mod preview_preset;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_mode_preview_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        let mut dispatch = self.dispatch_ui_asset_preview_preset_action(instance_id, action_id);
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_editor_mode_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_designer_tool_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_locale_preview_action(instance_id, action_id);
        }

        dispatch
    }
}
