use super::*;
use crate::ViewInstanceId;

impl SlintEditorHost {
    pub(super) fn dispatch_ui_asset_action(&mut self, instance_id: &str, action_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "save" => self
                .editor_manager
                .save_ui_asset_editor(&instance_id)
                .map(|_| ()),
            "undo" => self
                .editor_manager
                .undo_ui_asset_editor(&instance_id)
                .map(|_| ()),
            "redo" => self
                .editor_manager
                .redo_ui_asset_editor(&instance_id)
                .map(|_| ()),
            "style.rule.create" => self
                .editor_manager
                .create_ui_asset_editor_rule_from_selection(&instance_id)
                .map(|_| ()),
            "style.rule.extract_inline" => self
                .editor_manager
                .extract_ui_asset_editor_inline_overrides_to_rule(&instance_id)
                .map(|_| ()),
            "style.state.hover" => self
                .editor_manager
                .toggle_ui_asset_editor_pseudo_state(&instance_id, "hover")
                .map(|_| ()),
            "style.state.focus" => self
                .editor_manager
                .toggle_ui_asset_editor_pseudo_state(&instance_id, "focus")
                .map(|_| ()),
            "style.state.pressed" => self
                .editor_manager
                .toggle_ui_asset_editor_pseudo_state(&instance_id, "pressed")
                .map(|_| ()),
            "style.state.disabled" => self
                .editor_manager
                .toggle_ui_asset_editor_pseudo_state(&instance_id, "disabled")
                .map(|_| ()),
            "style.state.selected" => self
                .editor_manager
                .toggle_ui_asset_editor_pseudo_state(&instance_id, "selected")
                .map(|_| ()),
            "mode.design" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, crate::UiAssetEditorMode::Design),
            "mode.split" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, crate::UiAssetEditorMode::Split),
            "mode.source" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, crate::UiAssetEditorMode::Source),
            "mode.preview" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, crate::UiAssetEditorMode::Preview),
            other => {
                self.set_status_line(format!("Unknown UI asset editor action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => {
                if action_id == "save" {
                    self.sync_asset_workspace();
                }
                self.presentation_dirty = true;
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_style_class_action(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_inspector_widget_action(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "widget.control_id.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_widget_control_id(&instance_id, value)
                .map(|_| ()),
            "widget.text.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_widget_text_property(&instance_id, value)
                .map(|_| ()),
            "slot.mount.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_slot_mount(&instance_id, value)
                .map(|_| ()),
            "slot.padding.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_slot_padding(&instance_id, value)
                .map(|_| ()),
            "slot.layout.width.preferred.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_slot_width_preferred(&instance_id, value)
                .map(|_| ()),
            "slot.layout.height.preferred.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_slot_height_preferred(&instance_id, value)
                .map(|_| ()),
            "layout.width.preferred.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_layout_width_preferred(&instance_id, value)
                .map(|_| ()),
            "layout.height.preferred.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_layout_height_preferred(&instance_id, value)
                .map(|_| ()),
            "binding.add" => self
                .editor_manager
                .add_ui_asset_editor_binding(&instance_id)
                .map(|_| ()),
            "binding.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_binding(&instance_id)
                .map(|_| ()),
            "binding.id.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_binding_id(&instance_id, value)
                .map(|_| ()),
            "binding.event.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_binding_event(&instance_id, value)
                .map(|_| ()),
            "binding.route.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_binding_route(&instance_id, value)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset inspector widget action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_style_rule_action(
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
            other => {
                self.set_status_line(format!("Unknown UI asset style rule action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_matched_style_rule_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_matched_style_rule(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_style_rule_declaration_action(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_style_token_action(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_source_edited(&mut self, instance_id: &str, value: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .update_ui_asset_editor_source(&instance_id, value.to_string())
        {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_hierarchy_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_hierarchy_index(&instance_id, item_index.max(0) as usize)
        {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_preview_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_preview_index(&instance_id, item_index.max(0) as usize)
        {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_binding_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_binding(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
