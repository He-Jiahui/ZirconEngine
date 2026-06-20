use super::super::super::ui_asset_editor_detail_routes::widget_prop_state_target_path;
use super::super::*;

impl RetainedEditorHost {
    pub(in crate::ui::retained_host::app::ui_asset_editor_detail_events) fn handle_ui_asset_widget_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        match action_id {
            "widget.control_id.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id,
                    action_id,
                    "widget.control_id",
                    value,
                );
            }
            "widget.text.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id,
                    action_id,
                    "widget.text",
                    value,
                );
            }
            "component.root_class_policy.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id,
                    action_id,
                    "component.root_class_policy",
                    value,
                );
            }
            other => {
                if let Some(target_path) = widget_prop_state_target_path(other) {
                    self.dispatch_ui_asset_component_adapter_commit(
                        instance_id,
                        action_id,
                        &target_path,
                        value,
                    );
                } else {
                    self.set_status_line(format!("Unknown UI asset widget action {other}"));
                }
            }
        }
    }
}
