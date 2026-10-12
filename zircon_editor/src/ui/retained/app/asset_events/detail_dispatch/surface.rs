use super::super::*;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_surface_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) -> bool {
        match detail_id {
            "widget" => self.handle_ui_asset_widget_detail(instance_id, action_id, primary),
            "widget_promote" => {
                self.handle_ui_asset_widget_promote_detail(instance_id, action_id, primary)
            }
            "slot" => self.handle_ui_asset_slot_detail(instance_id, action_id, primary),
            "layout" => self.handle_ui_asset_layout_detail(instance_id, action_id, primary),
            "palette_drag" => {
                self.handle_ui_asset_palette_drag_detail(instance_id, action_id, primary, secondary)
            }
            "source" => {
                self.handle_ui_asset_source_detail(instance_id, action_id, item_index, primary)
            }
            _ => return false,
        }

        true
    }
}
