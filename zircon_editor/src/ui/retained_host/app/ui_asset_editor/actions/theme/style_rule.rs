use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_style_rule_action(
        &self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        match action_id {
            "style.rule.create" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .create_ui_asset_editor_rule_from_selection(instance_id),
            ),
            "style.rule.extract_inline" => UiAssetActionDispatch::handled(
                self.editor_manager
                    .extract_ui_asset_editor_inline_overrides_to_rule(instance_id),
            ),
            _ => UiAssetActionDispatch::Unhandled,
        }
    }
}
