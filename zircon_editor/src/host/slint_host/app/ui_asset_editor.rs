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
            "reference.open" => self
                .editor_manager
                .open_ui_asset_editor_selected_reference(&instance_id)
                .map(|_| ()),
            "preview.preset.editor_docked" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(
                    &instance_id,
                    crate::UiAssetPreviewPreset::EditorDocked,
                )
                .map(|_| ()),
            "preview.preset.editor_floating" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(
                    &instance_id,
                    crate::UiAssetPreviewPreset::EditorFloating,
                )
                .map(|_| ()),
            "preview.preset.game_hud" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(
                    &instance_id,
                    crate::UiAssetPreviewPreset::GameHud,
                )
                .map(|_| ()),
            "preview.preset.dialog" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(
                    &instance_id,
                    crate::UiAssetPreviewPreset::Dialog,
                )
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
            "palette.insert.child" => self
                .editor_manager
                .insert_ui_asset_editor_selected_palette_item_as_child(&instance_id)
                .map(|_| ()),
            "palette.insert.after" => self
                .editor_manager
                .insert_ui_asset_editor_selected_palette_item_after_selection(&instance_id)
                .map(|_| ()),
            "palette.target.previous" => self
                .editor_manager
                .cycle_ui_asset_editor_palette_drag_target_candidate_previous(&instance_id)
                .map(|_| ()),
            "palette.target.next" => self
                .editor_manager
                .cycle_ui_asset_editor_palette_drag_target_candidate_next(&instance_id)
                .map(|_| ()),
            "canvas.move.up" => self
                .editor_manager
                .move_ui_asset_editor_selected_node_up(&instance_id)
                .map(|_| ()),
            "canvas.move.down" => self
                .editor_manager
                .move_ui_asset_editor_selected_node_down(&instance_id)
                .map(|_| ()),
            "canvas.reparent.into_previous" => self
                .editor_manager
                .reparent_ui_asset_editor_selected_node_into_previous(&instance_id)
                .map(|_| ()),
            "canvas.reparent.into_next" => self
                .editor_manager
                .reparent_ui_asset_editor_selected_node_into_next(&instance_id)
                .map(|_| ()),
            "canvas.reparent.outdent" => self
                .editor_manager
                .reparent_ui_asset_editor_selected_node_outdent(&instance_id)
                .map(|_| ()),
            "canvas.convert.reference" => self
                .editor_manager
                .convert_ui_asset_editor_selected_node_to_reference(&instance_id)
                .map(|_| ()),
            "canvas.extract.component" => self
                .editor_manager
                .extract_ui_asset_editor_selected_node_to_component(&instance_id)
                .map(|_| ()),
            "canvas.promote.widget" => self
                .editor_manager
                .promote_ui_asset_editor_selected_component_to_external_widget(&instance_id)
                .map(|_| ()),
            "canvas.wrap.vertical_box" => self
                .editor_manager
                .wrap_ui_asset_editor_selected_node(&instance_id, "VerticalBox")
                .map(|_| ()),
            "canvas.unwrap" => self
                .editor_manager
                .unwrap_ui_asset_editor_selected_node(&instance_id)
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
            "slot.semantic.value.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_slot_semantic_value(&instance_id, value)
                .map(|_| ()),
            "slot.semantic.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_slot_semantic(&instance_id)
                .map(|_| ()),
            "layout.width.preferred.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_layout_width_preferred(&instance_id, value)
                .map(|_| ()),
            "layout.height.preferred.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_layout_height_preferred(&instance_id, value)
                .map(|_| ()),
            "layout.semantic.value.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_layout_semantic_value(&instance_id, value)
                .map(|_| ()),
            "layout.semantic.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_layout_semantic(&instance_id)
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
            "binding.route_target.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_binding_route_target(&instance_id, value)
                .map(|_| ()),
            "binding.action_target.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_binding_action_target(&instance_id, value)
                .map(|_| ()),
            other => {
                if let Some(path) = slot_semantic_action_path(other) {
                    self.editor_manager
                        .set_ui_asset_editor_selected_slot_semantic_field(&instance_id, path, value)
                        .map(|_| ())
                } else if let Some(path) = layout_semantic_action_path(other) {
                    self.editor_manager
                        .set_ui_asset_editor_selected_layout_semantic_field(
                            &instance_id,
                            path,
                            value,
                        )
                        .map(|_| ())
                } else {
                    self.set_status_line(format!(
                        "Unknown UI asset inspector widget action {other}"
                    ));
                    return;
                }
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

    pub(super) fn dispatch_ui_asset_palette_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_palette_index(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_palette_drag_hover(
        &mut self,
        instance_id: &str,
        surface_x: f32,
        surface_y: f32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .update_ui_asset_editor_palette_drag_target(&instance_id, surface_x, surface_y)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_palette_drag_drop(&mut self, instance_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .drop_ui_asset_editor_selected_palette_item_at_drag_target(&instance_id)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_palette_drag_cancel(&mut self, instance_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .clear_ui_asset_editor_palette_drag_target(&instance_id)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_palette_target_candidate_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_palette_target_candidate(
                &instance_id,
                item_index.max(0) as usize,
            ) {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_palette_target_confirm(&mut self, instance_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .confirm_ui_asset_editor_palette_target_choice(&instance_id)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_palette_target_cancel(&mut self, instance_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .cancel_ui_asset_editor_palette_target_choice(&instance_id)
        {
            Ok(_) => self.presentation_dirty = true,
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

    pub(super) fn dispatch_ui_asset_hierarchy_activated(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .activate_ui_asset_editor_hierarchy_index(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
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

    pub(super) fn dispatch_ui_asset_source_outline_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_source_outline_index(&instance_id, item_index.max(0) as usize)
        {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_preview_activated(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .activate_ui_asset_editor_preview_index(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_preview_mock_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_preview_mock_property(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_preview_mock_action(
        &mut self,
        instance_id: &str,
        action_id: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "preview.mock.value.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_preview_mock_value(&instance_id, value)
                .map(|_| ()),
            "preview.mock.clear" => self
                .editor_manager
                .clear_ui_asset_editor_selected_preview_mock_value(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!("Unknown UI asset preview mock action {other}"));
                return;
            }
        };

        match result {
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

    pub(super) fn dispatch_ui_asset_binding_event_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_binding_event_option(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_binding_action_kind_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_binding_action_kind(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_binding_payload_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_binding_payload(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_binding_payload_action(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_slot_semantic_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_slot_semantic(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_layout_semantic_selected(
        &mut self,
        instance_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match self
            .editor_manager
            .select_ui_asset_editor_layout_semantic(&instance_id, item_index.max(0) as usize)
        {
            Ok(_) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn slot_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "slot.overlay.anchor_x.set" => Some("layout.anchor.x"),
        "slot.overlay.anchor_y.set" => Some("layout.anchor.y"),
        "slot.overlay.pivot_x.set" => Some("layout.pivot.x"),
        "slot.overlay.pivot_y.set" => Some("layout.pivot.y"),
        "slot.overlay.position_x.set" => Some("layout.position.x"),
        "slot.overlay.position_y.set" => Some("layout.position.y"),
        "slot.overlay.z_index.set" => Some("layout.z_index"),
        "slot.grid.row.set" => Some("row"),
        "slot.grid.column.set" => Some("column"),
        "slot.grid.row_span.set" => Some("row_span"),
        "slot.grid.column_span.set" => Some("column_span"),
        "slot.flow.break_before.set" => Some("break_before"),
        "slot.flow.alignment.set" => Some("alignment"),
        _ => None,
    }
}

fn layout_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "layout.scroll.axis.set" => Some("container.axis"),
        "layout.scroll.gap.set" => Some("container.gap"),
        "layout.scroll.scrollbar_visibility.set" => Some("container.scrollbar_visibility"),
        "layout.scroll.virtualization.item_extent.set" => {
            Some("container.virtualization.item_extent")
        }
        "layout.scroll.virtualization.overscan.set" => Some("container.virtualization.overscan"),
        "layout.scroll.clip.set" => Some("clip"),
        _ => None,
    }
}
