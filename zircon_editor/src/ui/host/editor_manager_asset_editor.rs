use super::editor_error::EditorError;
use super::editor_manager::EditorManager;
use crate::ui::asset_editor::{
    UiAssetEditorMode, UiAssetEditorPanePresentation, UiAssetEditorReflectionModel,
    UiAssetPreviewPreset,
};
use crate::ui::workbench::view::ViewInstanceId;
use std::path::Path;

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

    pub fn promote_ui_asset_editor_local_theme_to_external_style_asset(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .promote_ui_asset_editor_local_theme_to_external_style_asset(instance_id)
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

    pub fn select_ui_asset_editor_theme_source(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_theme_source(instance_id, index)
    }

    pub fn detach_ui_asset_editor_selected_theme_source_to_local(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .detach_ui_asset_editor_selected_theme_source_to_local(instance_id)
    }

    pub fn clone_ui_asset_editor_selected_theme_source_to_local(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .clone_ui_asset_editor_selected_theme_source_to_local(instance_id)
    }

    pub fn prune_ui_asset_editor_duplicate_local_theme_overrides(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .prune_ui_asset_editor_duplicate_local_theme_overrides(instance_id)
    }

    pub fn apply_ui_asset_editor_all_theme_refactors(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_all_theme_refactors(instance_id)
    }

    pub fn apply_ui_asset_editor_theme_rule_helper_item(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_theme_rule_helper_item(instance_id, index)
    }

    pub fn apply_ui_asset_editor_theme_refactor_item(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_theme_refactor_item(instance_id, index)
    }

    pub fn set_ui_asset_editor_promote_theme_asset_id(
        &self,
        instance_id: &ViewInstanceId,
        asset_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_promote_theme_asset_id(instance_id, asset_id)
    }

    pub fn set_ui_asset_editor_promote_theme_document_id(
        &self,
        instance_id: &ViewInstanceId,
        document_id: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_promote_theme_document_id(instance_id, document_id)
    }

    pub fn set_ui_asset_editor_promote_theme_display_name(
        &self,
        instance_id: &ViewInstanceId,
        display_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_promote_theme_display_name(instance_id, display_name)
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

    pub fn toggle_ui_asset_editor_pseudo_state(
        &self,
        instance_id: &ViewInstanceId,
        state: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .toggle_ui_asset_editor_pseudo_state(instance_id, state)
    }

    pub fn add_ui_asset_editor_class_to_selection(
        &self,
        instance_id: &ViewInstanceId,
        class_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .add_ui_asset_editor_class_to_selection(instance_id, class_name)
    }

    pub fn remove_ui_asset_editor_class_from_selection(
        &self,
        instance_id: &ViewInstanceId,
        class_name: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .remove_ui_asset_editor_class_from_selection(instance_id, class_name)
    }

    pub fn select_ui_asset_editor_style_token(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_style_token(instance_id, index)
    }

    pub fn upsert_ui_asset_editor_style_token(
        &self,
        instance_id: &ViewInstanceId,
        token_name: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .upsert_ui_asset_editor_style_token(instance_id, token_name, value_literal)
    }

    pub fn delete_ui_asset_editor_selected_style_token(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_style_token(instance_id)
    }

    pub fn select_ui_asset_editor_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_stylesheet_rule(instance_id, index)
    }

    pub fn move_ui_asset_editor_selected_stylesheet_rule_up(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .move_ui_asset_editor_selected_stylesheet_rule_up(instance_id)
    }

    pub fn move_ui_asset_editor_selected_stylesheet_rule_down(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .move_ui_asset_editor_selected_stylesheet_rule_down(instance_id)
    }

    pub fn select_ui_asset_editor_matched_style_rule(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_matched_style_rule(instance_id, index)
    }

    pub fn rename_ui_asset_editor_selected_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
        selector: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .rename_ui_asset_editor_selected_stylesheet_rule(instance_id, selector)
    }

    pub fn select_ui_asset_editor_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_style_rule_declaration(instance_id, index)
    }

    pub fn upsert_ui_asset_editor_selected_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
        path: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .upsert_ui_asset_editor_selected_style_rule_declaration(
                instance_id,
                path,
                value_literal,
            )
    }

    pub fn delete_ui_asset_editor_selected_style_rule_declaration(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_style_rule_declaration(instance_id)
    }

    pub fn delete_ui_asset_editor_selected_stylesheet_rule(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_stylesheet_rule(instance_id)
    }

    pub fn ui_asset_editor_reflection(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorReflectionModel, EditorError> {
        self.host.ui_asset_editor_reflection(instance_id)
    }

    pub fn ui_asset_editor_pane_presentation(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<UiAssetEditorPanePresentation, EditorError> {
        self.host.ui_asset_editor_pane_presentation(instance_id)
    }

    pub fn open_ui_asset_editor_selected_reference(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .open_ui_asset_editor_selected_reference(instance_id)
    }

    pub fn open_ui_asset_editor_selected_theme_source(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .open_ui_asset_editor_selected_theme_source(instance_id)
    }

    pub fn open_ui_asset_editor(
        &self,
        path: impl AsRef<Path>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.host.open_ui_asset_editor(path, mode)
    }

    pub fn open_ui_asset_editor_by_id(
        &self,
        asset_id: impl AsRef<str>,
        mode: Option<UiAssetEditorMode>,
    ) -> Result<ViewInstanceId, EditorError> {
        self.host.open_ui_asset_editor_by_id(asset_id, mode)
    }

    pub fn set_ui_asset_editor_preview_preset(
        &self,
        instance_id: &ViewInstanceId,
        preview_preset: UiAssetPreviewPreset,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_preview_preset(instance_id, preview_preset)
    }

    pub fn select_ui_asset_editor_preview_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<(), EditorError> {
        self.host
            .select_ui_asset_editor_preview_index(instance_id, index)
    }

    pub fn activate_ui_asset_editor_preview_index(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<Option<ViewInstanceId>, EditorError> {
        self.host
            .activate_ui_asset_editor_preview_index(instance_id, index)
    }

    pub fn select_ui_asset_editor_preview_mock_property(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_preview_mock_property(instance_id, index)
    }

    pub fn select_ui_asset_editor_preview_mock_subject(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_preview_mock_subject(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_preview_mock_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_preview_mock_value(instance_id, value)
    }

    pub fn select_ui_asset_editor_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .select_ui_asset_editor_preview_mock_nested_entry(instance_id, index)
    }

    pub fn set_ui_asset_editor_selected_preview_mock_nested_value(
        &self,
        instance_id: &ViewInstanceId,
        value: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .set_ui_asset_editor_selected_preview_mock_nested_value(instance_id, value)
    }

    pub fn upsert_ui_asset_editor_selected_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
        key: impl AsRef<str>,
        value_literal: impl AsRef<str>,
    ) -> Result<bool, EditorError> {
        self.host
            .upsert_ui_asset_editor_selected_preview_mock_nested_entry(
                instance_id,
                key,
                value_literal,
            )
    }

    pub fn apply_ui_asset_editor_selected_preview_mock_suggestion(
        &self,
        instance_id: &ViewInstanceId,
        index: usize,
    ) -> Result<bool, EditorError> {
        self.host
            .apply_ui_asset_editor_selected_preview_mock_suggestion(instance_id, index)
    }

    pub fn delete_ui_asset_editor_selected_preview_mock_nested_entry(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .delete_ui_asset_editor_selected_preview_mock_nested_entry(instance_id)
    }

    pub fn clear_ui_asset_editor_selected_preview_mock_value(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .clear_ui_asset_editor_selected_preview_mock_value(instance_id)
    }

    pub fn save_ui_asset_editor(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<String, EditorError> {
        self.host.save_ui_asset_editor(instance_id)
    }
}
