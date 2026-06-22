use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use crate::ui::asset_editor::UiDesignerToolMode;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn set_ui_asset_editor_selected_widget_control_id(
        &self,
        instance_id: &ViewInstanceId,
        control_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_widget_control_id(instance_id, control_id)
    }

    pub fn set_ui_asset_editor_selected_widget_text_property(
        &self,
        instance_id: &ViewInstanceId,
        text: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_widget_text_property(instance_id, text)
    }

    pub fn set_ui_asset_editor_selected_widget_prop_literal(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_widget_prop_literal(instance_id, path, literal)
    }

    pub fn set_ui_asset_editor_selected_widget_state_literal(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_widget_state_literal(instance_id, path, literal)
    }

    pub fn set_ui_asset_editor_selected_component_root_class_policy(
        &self,
        instance_id: &ViewInstanceId,
        policy: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_component_root_class_policy(instance_id, policy)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_asset_id(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_promote_widget_asset_id(instance_id, asset_id)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_component_name(
        &self,
        instance_id: &ViewInstanceId,
        component_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_promote_widget_component_name(instance_id, component_name)
    }

    pub fn set_ui_asset_editor_selected_promote_widget_document_id(
        &self,
        instance_id: &ViewInstanceId,
        document_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_promote_widget_document_id(instance_id, document_id)
    }

    pub fn set_ui_asset_editor_selected_slot_mount(
        &self,
        instance_id: &ViewInstanceId,
        mount: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_slot_mount(instance_id, mount)
    }

    pub fn set_ui_asset_editor_selected_slot_padding(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_slot_padding(instance_id, literal)
    }

    pub fn set_ui_asset_editor_selected_slot_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_slot_width_preferred(instance_id, literal)
    }

    pub fn set_ui_asset_editor_selected_slot_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_slot_height_preferred(instance_id, literal)
    }

    pub fn set_ui_asset_editor_designer_tool_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiDesignerToolMode,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_designer_tool_mode(instance_id, mode)
    }

    pub fn set_ui_asset_editor_locale_preview(
        &self,
        instance_id: &ViewInstanceId,
        locale: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_locale_preview(instance_id, locale)
    }

    pub fn resize_ui_asset_editor_selected_slot_preferred_size(
        &self,
        instance_id: &ViewInstanceId,
        width: f32,
        height: f32,
    ) -> Result<bool, EditorError> {
        self.host
            .resize_ui_asset_editor_selected_slot_preferred_size(instance_id, width, height)
    }

    pub fn set_ui_asset_editor_selected_layout_width_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_layout_width_preferred(instance_id, literal)
    }

    pub fn set_ui_asset_editor_selected_layout_height_preferred(
        &self,
        instance_id: &ViewInstanceId,
        literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_layout_height_preferred(instance_id, literal)
    }

    pub fn select_ui_asset_editor_slot_semantic(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_slot_semantic(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_slot_semantic_value(instance_id, value)
    }

    pub fn set_ui_asset_editor_selected_slot_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_slot_semantic_field(instance_id, path, value)
    }

    pub fn delete_ui_asset_editor_selected_slot_semantic(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_slot_semantic(instance_id)
    }

    pub fn select_ui_asset_editor_layout_semantic(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_layout_semantic(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_layout_semantic_value(instance_id, value)
    }

    pub fn set_ui_asset_editor_selected_layout_semantic_field(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_layout_semantic_field(instance_id, path, value)
    }

    pub fn delete_ui_asset_editor_selected_layout_semantic(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_layout_semantic(instance_id)
    }
}
