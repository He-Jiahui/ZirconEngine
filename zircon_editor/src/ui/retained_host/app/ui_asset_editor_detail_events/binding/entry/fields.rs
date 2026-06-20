use super::super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_binding_field_commit_detail(
        &mut self,
        instance_id: &ViewInstanceId,
        action_id: &str,
        value: &str,
    ) -> bool {
        let target_path = match action_id {
            "binding.id.set" => "binding.id",
            "binding.event.set" => "binding.event",
            "binding.route.set" => "binding.route",
            "binding.route_target.set" => "binding.route_target",
            "binding.action_target.set" => "binding.action_target",
            _ => return false,
        };

        self.dispatch_ui_asset_component_adapter_commit(
            instance_id.0.as_str(),
            action_id,
            target_path,
            value,
        );
        true
    }
}
