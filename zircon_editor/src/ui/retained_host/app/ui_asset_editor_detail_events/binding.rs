use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_binding_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "binding.add" => self
                .editor_manager
                .add_ui_asset_editor_binding(&instance_id)
                .map(|_| ()),
            "binding.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_binding(&instance_id)
                .map(|_| ()),
            "binding.id.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "binding.id",
                    value,
                );
                return;
            }
            "binding.event.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "binding.event",
                    value,
                );
                return;
            }
            "binding.route.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "binding.route",
                    value,
                );
                return;
            }
            "binding.route_target.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "binding.route_target",
                    value,
                );
                return;
            }
            "binding.action_target.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "binding.action_target",
                    value,
                );
                return;
            }
            other => {
                self.set_status_line(format!("Unknown UI asset binding action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_binding_payload_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        payload_key: &str,
        payload_value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "binding.payload.upsert" => self
                .editor_manager
                .upsert_ui_asset_editor_selected_binding_payload(
                    &instance_id,
                    payload_key,
                    payload_value,
                )
                .map(|_| ()),
            "binding.payload.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_binding_payload(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset binding payload action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_binding_payload_suggestion_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "binding.payload.suggestion.apply" => self
                .editor_manager
                .apply_ui_asset_editor_selected_binding_payload_suggestion(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset binding payload suggestion action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_binding_route_suggestion_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "binding.route.suggestion.apply" => self
                .editor_manager
                .apply_ui_asset_editor_selected_binding_route_suggestion(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset binding route suggestion action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_binding_action_suggestion_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "binding.action.suggestion.apply" => self
                .editor_manager
                .apply_ui_asset_editor_selected_binding_action_suggestion(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset binding action suggestion action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
