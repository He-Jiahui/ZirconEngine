use super::super::*;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_preview_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) -> bool {
        match detail_id {
            "preview_mock" => {
                self.handle_ui_asset_preview_mock_detail(instance_id, action_id, primary)
            }
            "preview_mock_nested" => self.handle_ui_asset_preview_mock_nested_detail(
                instance_id,
                action_id,
                primary,
                secondary,
            ),
            "preview_mock_suggestion" => self.handle_ui_asset_preview_mock_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            _ => return false,
        }

        true
    }
}
