use super::{UiAssetActionDispatch, *};
use crate::ui::workbench::view::ViewInstanceId;

mod pseudo_state;
mod selection;
mod source;
mod style_rule;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_theme_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        let mut dispatch = self.dispatch_ui_asset_theme_source_action(instance_id, action_id);
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_style_rule_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_pseudo_state_action(instance_id, action_id);
        }
        if matches!(dispatch, UiAssetActionDispatch::Unhandled) {
            dispatch = self.dispatch_ui_asset_theme_source_selection_action(instance_id, action_id);
        }

        dispatch
    }
}
