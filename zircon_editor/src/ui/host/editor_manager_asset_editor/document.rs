use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use crate::ui::asset_editor::UiAssetEditorMode;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn undo_ui_asset_editor(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.undo_ui_asset_editor(instance_id)
    }

    pub fn redo_ui_asset_editor(&self, instance_id: &ViewInstanceId) -> Result<bool, EditorError> {
        self.host.redo_ui_asset_editor(instance_id)
    }

    pub fn set_ui_asset_editor_mode(
        &self,
        instance_id: &ViewInstanceId,
        mode: UiAssetEditorMode,
    ) -> Result<(), EditorError> {
        self.host.set_ui_asset_editor_mode(instance_id, mode)
    }

    pub fn select_ui_asset_editor_hierarchy_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.host
            .select_ui_asset_editor_hierarchy_index(instance_id, index)
    }

    pub fn activate_ui_asset_editor_hierarchy_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .activate_ui_asset_editor_hierarchy_index(instance_id, index)
    }

    pub fn select_ui_asset_editor_source_outline_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.host
            .select_ui_asset_editor_source_outline_index(instance_id, index)
    }

    pub fn convert_ui_asset_editor_selected_node_to_reference(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .convert_ui_asset_editor_selected_node_to_reference(instance_id)
    }

    pub fn extract_ui_asset_editor_selected_node_to_component(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .extract_ui_asset_editor_selected_node_to_component(instance_id)
    }

    pub fn promote_ui_asset_editor_selected_component_to_external_widget(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .promote_ui_asset_editor_selected_component_to_external_widget(instance_id)
    }

    pub fn move_ui_asset_editor_selected_node_up(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host.move_ui_asset_editor_selected_node_up(instance_id)
    }

    pub fn move_ui_asset_editor_selected_node_down(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .move_ui_asset_editor_selected_node_down(instance_id)
    }

    pub fn reparent_ui_asset_editor_selected_node_into_previous(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .reparent_ui_asset_editor_selected_node_into_previous(instance_id)
    }

    pub fn reparent_ui_asset_editor_selected_node_into_next(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .reparent_ui_asset_editor_selected_node_into_next(instance_id)
    }

    pub fn reparent_ui_asset_editor_selected_node_outdent(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .reparent_ui_asset_editor_selected_node_outdent(instance_id)
    }

    pub fn wrap_ui_asset_editor_selected_node(
        &self,
        instance_id: &ViewInstanceId,
        widget_type: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .wrap_ui_asset_editor_selected_node(instance_id, widget_type)
    }

    pub fn unwrap_ui_asset_editor_selected_node(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host.unwrap_ui_asset_editor_selected_node(instance_id)
    }

    pub fn select_ui_asset_editor_palette_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_palette_index(instance_id, index)
    }

    pub fn update_ui_asset_editor_palette_drag_target(
        &self,
        instance_id: &ViewInstanceId,
        surface_x: f32,
        surface_y: f32,
    ) -> Result<bool, EditorError> {
        self.host
            .update_ui_asset_editor_palette_drag_target(instance_id, surface_x, surface_y)
    }

    pub fn clear_ui_asset_editor_palette_drag_target(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .clear_ui_asset_editor_palette_drag_target(instance_id)
    }

    pub fn cycle_ui_asset_editor_palette_drag_target_candidate_next(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .cycle_ui_asset_editor_palette_drag_target_candidate_next(instance_id)
    }

    pub fn cycle_ui_asset_editor_palette_drag_target_candidate_previous(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .cycle_ui_asset_editor_palette_drag_target_candidate_previous(instance_id)
    }

    pub fn select_ui_asset_editor_palette_target_candidate(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_palette_target_candidate(instance_id, index)
    }

    pub fn confirm_ui_asset_editor_palette_target_choice(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .confirm_ui_asset_editor_palette_target_choice(instance_id)
    }

    pub fn cancel_ui_asset_editor_palette_target_choice(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .cancel_ui_asset_editor_palette_target_choice(instance_id)
    }

    pub fn drop_ui_asset_editor_selected_palette_item_at_drag_target(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .drop_ui_asset_editor_selected_palette_item_at_drag_target(instance_id)
    }

    pub fn insert_ui_asset_editor_selected_palette_item_as_child(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .insert_ui_asset_editor_selected_palette_item_as_child(instance_id)
    }

    pub fn insert_ui_asset_editor_selected_palette_item_after_selection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .insert_ui_asset_editor_selected_palette_item_after_selection(instance_id)
    }

    pub fn select_ui_asset_editor_source_byte_offset(
        &self,
        instance_id: &ViewInstanceId,
        byte_offset: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_source_byte_offset(instance_id, byte_offset)
    }

    pub fn update_ui_asset_editor_source(
        &self,
        instance_id: &ViewInstanceId,
        next_source: impl Into<String>,
    ) -> Result<(), EditorError> {
        self.host
            .update_ui_asset_editor_source(instance_id, next_source)
    }

    pub fn create_ui_asset_editor_rule_from_selection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .create_ui_asset_editor_rule_from_selection(instance_id)
    }

    pub fn extract_ui_asset_editor_inline_overrides_to_rule(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .extract_ui_asset_editor_inline_overrides_to_rule(instance_id)
    }
}
