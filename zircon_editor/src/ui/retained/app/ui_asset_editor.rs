use super::*;

#[path = "ui_asset_editor/actions.rs"]
mod actions;
pub(in crate::ui::retained_host::app) use actions::PendingCanvasAnchorPopup;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_action(&mut self, instance_id: &str, action_id: &str) {
        self.dispatch_ui_asset_action_impl(instance_id, action_id);
    }
}
