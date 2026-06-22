use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn select_ui_asset_editor_binding(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host.select_ui_asset_editor_binding(instance_id, index)
    }

    pub fn add_ui_asset_editor_binding(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host.add_ui_asset_editor_binding(instance_id)
    }

    pub fn select_ui_asset_editor_binding_event_option(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_binding_event_option(instance_id, index)
    }

    pub fn delete_ui_asset_editor_selected_binding(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_binding(instance_id)
    }

    pub fn set_ui_asset_editor_selected_binding_id(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_binding_id(instance_id, value)
    }

    pub fn set_ui_asset_editor_selected_binding_event(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_binding_event(instance_id, value)
    }

    pub fn select_ui_asset_editor_binding_action_kind(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_binding_action_kind(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_binding_route(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_binding_route(instance_id, value)
    }

    pub fn set_ui_asset_editor_selected_binding_route_target(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_binding_route_target(instance_id, value)
    }

    pub fn set_ui_asset_editor_selected_binding_action_target(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_binding_action_target(instance_id, value)
    }

    pub fn apply_ui_asset_editor_selected_binding_route_suggestion(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_selected_binding_route_suggestion(instance_id, index)
    }

    pub fn apply_ui_asset_editor_selected_binding_action_suggestion(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_selected_binding_action_suggestion(instance_id, index)
    }

    pub fn select_ui_asset_editor_binding_payload(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_binding_payload(instance_id, index)
    }

    pub fn upsert_ui_asset_editor_selected_binding_payload(
        &self,
        instance_id: &ViewInstanceId,
        payload_key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host.upsert_ui_asset_editor_selected_binding_payload(
            instance_id,
            payload_key,
            value_literal,
        )
    }

    pub fn delete_ui_asset_editor_selected_binding_payload(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_binding_payload(instance_id)
    }

    pub fn apply_ui_asset_editor_selected_binding_payload_suggestion(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_selected_binding_payload_suggestion(instance_id, index)
    }
}
