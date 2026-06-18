use super::*;

mod actions;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_action(&mut self, instance_id: &str, action_id: &str) {
        self.dispatch_ui_asset_action_impl(instance_id, action_id);
    }
}
