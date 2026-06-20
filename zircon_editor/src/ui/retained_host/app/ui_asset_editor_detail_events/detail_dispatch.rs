use super::*;

mod binding;
mod preview;
mod style;
mod surface;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app) fn dispatch_ui_asset_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) {
        if self.dispatch_ui_asset_surface_detail_event(
            instance_id,
            detail_id,
            action_id,
            item_index,
            primary,
            secondary,
        ) || self.dispatch_ui_asset_style_detail_event(
            instance_id,
            detail_id,
            action_id,
            item_index,
            primary,
            secondary,
        ) || self.dispatch_ui_asset_preview_detail_event(
            instance_id,
            detail_id,
            action_id,
            item_index,
            primary,
            secondary,
        ) || self.dispatch_ui_asset_binding_detail_event(
            instance_id,
            detail_id,
            action_id,
            item_index,
            primary,
            secondary,
        ) {
            return;
        }

        self.focus_callback_source_window();
        self.set_status_line(format!(
            "Unknown UI asset detail event {detail_id}:{action_id}"
        ));
    }
}
