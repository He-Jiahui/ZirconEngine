use super::super::ui_asset_editor_detail_routes::widget_prop_state_target_path;
use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_widget_detail(
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

    pub(super) fn handle_ui_asset_widget_promote_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "promote.asset_id.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_promote_widget_asset_id(&instance_id, value)
                .map(|_| ()),
            "promote.component_name.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_promote_widget_component_name(&instance_id, value)
                .map(|_| ()),
            "promote.document_id.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_promote_widget_document_id(&instance_id, value)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset widget promote action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
