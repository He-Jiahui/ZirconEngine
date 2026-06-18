use super::*;
use crate::ui::workbench::view::ViewInstanceId;

impl RetainedEditorHost {
    pub(super) fn handle_ui_asset_style_class_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        class_name: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "style.class.add" => self
                .editor_manager
                .add_ui_asset_editor_class_to_selection(&instance_id, class_name)
                .map(|_| ()),
            "style.class.remove" => self
                .editor_manager
                .remove_ui_asset_editor_class_from_selection(&instance_id, class_name)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset style class action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_theme_source_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "theme.promote.asset_id.set" => self
                .editor_manager
                .set_ui_asset_editor_promote_theme_asset_id(&instance_id, value)
                .map(|_| ()),
            "theme.promote.document_id.set" => self
                .editor_manager
                .set_ui_asset_editor_promote_theme_document_id(&instance_id, value)
                .map(|_| ()),
            "theme.promote.display_name.set" => self
                .editor_manager
                .set_ui_asset_editor_promote_theme_display_name(&instance_id, value)
                .map(|_| ()),
            "theme.rule_helper.apply" => self
                .editor_manager
                .apply_ui_asset_editor_theme_rule_helper_item(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            "theme.refactor.apply" => self
                .editor_manager
                .apply_ui_asset_editor_theme_refactor_item(&instance_id, item_index.max(0) as usize)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset theme source action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_style_rule_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        selector: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "style.rule.select" => self
                .editor_manager
                .select_ui_asset_editor_stylesheet_rule(&instance_id, item_index.max(0) as usize)
                .map(|_| ()),
            "style.rule.rename" => self
                .editor_manager
                .rename_ui_asset_editor_selected_stylesheet_rule(&instance_id, selector)
                .map(|_| ()),
            "style.rule.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_stylesheet_rule(&instance_id)
                .map(|_| ()),
            "style.rule.move_up" => self
                .editor_manager
                .move_ui_asset_editor_selected_stylesheet_rule_up(&instance_id)
                .map(|_| ()),
            "style.rule.move_down" => self
                .editor_manager
                .move_ui_asset_editor_selected_stylesheet_rule_down(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset style rule action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_style_rule_declaration_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        declaration_path: &str,
        declaration_value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "style.rule.declaration.select" => self
                .editor_manager
                .select_ui_asset_editor_style_rule_declaration(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            "style.rule.declaration.upsert" => self
                .editor_manager
                .upsert_ui_asset_editor_selected_style_rule_declaration(
                    &instance_id,
                    declaration_path,
                    declaration_value,
                )
                .map(|_| ()),
            "style.rule.declaration.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_style_rule_declaration(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset style rule declaration action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn handle_ui_asset_style_token_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        token_name: &str,
        token_value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "style.token.select" => self
                .editor_manager
                .select_ui_asset_editor_style_token(&instance_id, item_index.max(0) as usize)
                .map(|_| ()),
            "style.token.upsert" => self
                .editor_manager
                .upsert_ui_asset_editor_style_token(&instance_id, token_name, token_value)
                .map(|_| ()),
            "style.token.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_style_token(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset style token action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.mark_presentation_dirty(),
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
