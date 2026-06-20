use super::super::*;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_binding_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) -> bool {
        match detail_id {
            "binding" => self.handle_ui_asset_binding_detail(instance_id, action_id, primary),
            "binding_payload" => self.handle_ui_asset_binding_payload_detail(
                instance_id,
                action_id,
                primary,
                secondary,
            ),
            "binding_payload_suggestion" => self.handle_ui_asset_binding_payload_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            "binding_route_suggestion" => self.handle_ui_asset_binding_route_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            "binding_action_suggestion" => self.handle_ui_asset_binding_action_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            _ => return false,
        }

        true
    }
}
