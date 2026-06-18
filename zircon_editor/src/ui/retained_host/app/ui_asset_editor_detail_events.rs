use super::*;

mod binding;
mod collection;
mod component_adapter;
mod palette;
mod preview;
mod source;
mod structure;
mod style;
mod widget;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) {
        match detail_id {
            "style_class" => {
                self.handle_ui_asset_style_class_detail(instance_id, action_id, primary)
            }
            "widget" => self.handle_ui_asset_widget_detail(instance_id, action_id, primary),
            "widget_promote" => {
                self.handle_ui_asset_widget_promote_detail(instance_id, action_id, primary)
            }
            "slot" => self.handle_ui_asset_slot_detail(instance_id, action_id, primary),
            "layout" => self.handle_ui_asset_layout_detail(instance_id, action_id, primary),
            "binding" => self.handle_ui_asset_binding_detail(instance_id, action_id, primary),
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
            "palette_drag" => {
                self.handle_ui_asset_palette_drag_detail(instance_id, action_id, primary, secondary)
            }
            "source" => {
                self.handle_ui_asset_source_detail(instance_id, action_id, item_index, primary)
            }
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
            other => {
                self.focus_callback_source_window();
                self.set_status_line(format!("Unknown UI asset detail event {other}:{action_id}"));
            }
        }
    }
}
