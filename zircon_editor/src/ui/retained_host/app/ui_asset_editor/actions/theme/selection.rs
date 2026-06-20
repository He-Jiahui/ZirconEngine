use super::{RetainedEditorHost, UiAssetActionDispatch};
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_theme_source_selection_action(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
    ) -> UiAssetActionDispatch {
        if !action_id.starts_with("theme.source.select.") {
            return UiAssetActionDispatch::Unhandled;
        }

        let index = action_id
            .trim_start_matches("theme.source.select.")
            .parse::<usize>();
        match index {
            Ok(index) => UiAssetActionDispatch::handled(
                self.editor_manager
                    .select_ui_asset_editor_theme_source(instance_id, index),
            ),
            Err(_) => {
                self.set_status_line(format!(
                    "Invalid UI asset theme source selection action {action_id}"
                ));
                UiAssetActionDispatch::Consumed
            }
        }
    }
}
