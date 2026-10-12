use super::super::*;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_style_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) -> bool {
        match detail_id {
            "style_class" => {
                self.handle_ui_asset_style_class_detail(instance_id, action_id, primary)
            }
            "theme_source" => self.handle_ui_asset_theme_source_detail(
                instance_id,
                action_id,
                item_index,
                primary,
            ),
            "style_rule" => {
                self.handle_ui_asset_style_rule_detail(instance_id, action_id, item_index, primary)
            }
            "style_rule_declaration" => self.handle_ui_asset_style_rule_declaration_detail(
                instance_id,
                action_id,
                item_index,
                primary,
                secondary,
            ),
            "style_token" => self.handle_ui_asset_style_token_detail(
                instance_id,
                action_id,
                item_index,
                primary,
                secondary,
            ),
            _ => return false,
        }

        true
    }
}
