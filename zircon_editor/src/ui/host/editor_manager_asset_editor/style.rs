use super::super::editor_error::EditorError;
use super::super::editor_manager::EditorManager;
use crate::ui::workbench::view::ViewInstanceId;

impl EditorManager {
    pub fn promote_ui_asset_editor_local_theme_to_external_style_asset(
        &self,
        instance_id: &ViewInstanceId,
    ) -> Result<bool, EditorError> {
        self.host
            .promote_ui_asset_editor_local_theme_to_external_style_asset(instance_id)
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
}
