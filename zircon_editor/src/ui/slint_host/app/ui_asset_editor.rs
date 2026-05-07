use super::*;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetPreviewPreset, UiDesignerToolMode};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};
use zircon_runtime_interface::ui::component::{
    UiComponentBindingTarget, UiComponentEvent, UiComponentEventEnvelope, UiValue,
};

impl SlintEditorHost {
    pub(super) fn dispatch_ui_asset_action(&mut self, instance_id: &str, action_id: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "save" => self
                .editor_manager
                .save_ui_asset_editor(&instance_id)
                .map(|_| ()),
            "workspace.reload_from_disk" => self
                .editor_manager
                .reload_ui_asset_editor_from_disk(&instance_id)
                .map(|_| ()),
            "workspace.keep_local_and_save" => self
                .editor_manager
                .keep_ui_asset_editor_local_and_save(&instance_id)
                .map(|_| ()),
            "workspace.save_local_copy" => match self
                .editor_manager
                .save_ui_asset_editor_local_copy_next_to_source(&instance_id)
            {
                Ok(path) => {
                    self.set_status_line(format!("Saved UI asset local copy {}", path.display()));
                    Ok(())
                }
                Err(error) => Err(error),
            },
            "workspace.diff_snapshot" => match self
                .editor_manager
                .open_ui_asset_editor_diff_snapshot(&instance_id)
            {
                Ok(Some(snapshot)) => {
                    self.set_status_line(snapshot.summary);
                    Ok(())
                }
                Ok(None) => {
                    self.set_status_line("No UI asset conflict diff available".to_string());
                    Ok(())
                }
                Err(error) => Err(error),
            },
            "emergency.reload_from_disk" => self
                .editor_manager
                .reload_ui_asset_editor_from_disk(&instance_id)
                .map(|_| ()),
            "emergency.revert_last_valid" => self
                .editor_manager
                .revert_ui_asset_editor_to_last_valid(&instance_id)
                .map(|_| ()),
            "emergency.open_asset_browser" => self
                .editor_manager
                .open_view(ViewDescriptorId::new("editor.asset_browser"), None)
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
            "theme.source.open" => self
                .editor_manager
                .open_ui_asset_editor_selected_theme_source(&instance_id)
                .map(|_| ()),
            "theme.local.promote" => self
                .editor_manager
                .promote_ui_asset_editor_local_theme_to_external_style_asset(&instance_id)
                .map(|_| ()),
            "theme.source.detach_local" => self
                .editor_manager
                .detach_ui_asset_editor_selected_theme_source_to_local(&instance_id)
                .map(|_| ()),
            "theme.source.clone_local" => self
                .editor_manager
                .clone_ui_asset_editor_selected_theme_source_to_local(&instance_id)
                .map(|_| ()),
            "theme.local.prune_duplicates" => self
                .editor_manager
                .prune_ui_asset_editor_duplicate_local_theme_overrides(&instance_id)
                .map(|_| ()),
            "preview.preset.editor_docked" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(
                    &instance_id,
                    UiAssetPreviewPreset::EditorDocked,
                )
                .map(|_| ()),
            "preview.preset.editor_floating" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(
                    &instance_id,
                    UiAssetPreviewPreset::EditorFloating,
                )
                .map(|_| ()),
            "preview.preset.game_hud" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(&instance_id, UiAssetPreviewPreset::GameHud)
                .map(|_| ()),
            "preview.preset.dialog" => self
                .editor_manager
                .set_ui_asset_editor_preview_preset(&instance_id, UiAssetPreviewPreset::Dialog)
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
            "palette.drag.drop" => self
                .editor_manager
                .drop_ui_asset_editor_selected_palette_item_at_drag_target(&instance_id)
                .map(|_| ()),
            "palette.drag.cancel" => self
                .editor_manager
                .clear_ui_asset_editor_palette_drag_target(&instance_id)
                .map(|_| ()),
            "palette.target.previous" => self
                .editor_manager
                .cycle_ui_asset_editor_palette_drag_target_candidate_previous(&instance_id)
                .map(|_| ()),
            "palette.target.next" => self
                .editor_manager
                .cycle_ui_asset_editor_palette_drag_target_candidate_next(&instance_id)
                .map(|_| ()),
            "palette.target.confirm" => self
                .editor_manager
                .confirm_ui_asset_editor_palette_target_choice(&instance_id)
                .map(|_| ()),
            "palette.target.cancel" => self
                .editor_manager
                .cancel_ui_asset_editor_palette_target_choice(&instance_id)
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
                .set_ui_asset_editor_mode(&instance_id, UiAssetEditorMode::Design),
            "mode.split" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, UiAssetEditorMode::Split),
            "mode.source" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, UiAssetEditorMode::Source),
            "mode.preview" => self
                .editor_manager
                .set_ui_asset_editor_mode(&instance_id, UiAssetEditorMode::Preview),
            "designer.tool.select" => self
                .editor_manager
                .set_ui_asset_editor_designer_tool_mode(&instance_id, UiDesignerToolMode::Select)
                .map(|_| ()),
            "designer.tool.resize_slot" => self
                .editor_manager
                .set_ui_asset_editor_designer_tool_mode(
                    &instance_id,
                    UiDesignerToolMode::ResizeSlot,
                )
                .map(|_| ()),
            "designer.tool.preview_interact" => self
                .editor_manager
                .set_ui_asset_editor_designer_tool_mode(
                    &instance_id,
                    UiDesignerToolMode::PreviewInteract,
                )
                .map(|_| ()),
            "locale.preview.authoring_fallback" => self
                .editor_manager
                .set_ui_asset_editor_locale_preview(&instance_id, "authoring-fallback")
                .map(|_| ()),
            "locale.preview.en_us" => self
                .editor_manager
                .set_ui_asset_editor_locale_preview(&instance_id, "en-US")
                .map(|_| ()),
            "locale.preview.zh_cn" => self
                .editor_manager
                .set_ui_asset_editor_locale_preview(&instance_id, "zh-CN")
                .map(|_| ()),
            other if other.starts_with("theme.source.select.") => {
                let index = other
                    .trim_start_matches("theme.source.select.")
                    .parse::<usize>();
                match index {
                    Ok(index) => self
                        .editor_manager
                        .select_ui_asset_editor_theme_source(&instance_id, index)
                        .map(|_| ()),
                    Err(_) => {
                        self.set_status_line(format!(
                            "Invalid UI asset theme source selection action {other}"
                        ));
                        return;
                    }
                }
            }
            other => {
                self.set_status_line(format!("Unknown UI asset editor action {other}"));
                return;
            }
        };

        match result {
            Ok(()) => {
                if action_id == "save" || action_id == "workspace.keep_local_and_save" {
                    self.sync_asset_workspace();
                }
                self.presentation_dirty = true;
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_detail_event(
        &mut self,
        instance_id: &str,
        detail_id: &str,
        action_id: &str,
        item_index: i32,
        primary: &str,
        secondary: &str,
    ) {
        match detail_id {
            "style_class" => {
                self.handle_ui_asset_style_class_detail(instance_id, action_id, primary)
            }
            "widget" => self.handle_ui_asset_widget_detail(instance_id, action_id, primary),
            "widget_promote" => {
                self.handle_ui_asset_widget_promote_detail(instance_id, action_id, primary)
            }
            "slot" => self.handle_ui_asset_slot_detail(instance_id, action_id, primary),
            "layout" => self.handle_ui_asset_layout_detail(instance_id, action_id, primary),
            "binding" => self.handle_ui_asset_binding_detail(instance_id, action_id, primary),
            "theme_source" => self.handle_ui_asset_theme_source_detail(
                instance_id,
                action_id,
                item_index,
                primary,
            ),
            "style_rule" => {
                self.handle_ui_asset_style_rule_detail(instance_id, action_id, item_index, primary)
            }
            "style_rule_declaration" => self.handle_ui_asset_style_rule_declaration_detail(
                instance_id,
                action_id,
                item_index,
                primary,
                secondary,
            ),
            "style_token" => self.handle_ui_asset_style_token_detail(
                instance_id,
                action_id,
                item_index,
                primary,
                secondary,
            ),
            "preview_mock" => {
                self.handle_ui_asset_preview_mock_detail(instance_id, action_id, primary)
            }
            "preview_mock_nested" => self.handle_ui_asset_preview_mock_nested_detail(
                instance_id,
                action_id,
                primary,
                secondary,
            ),
            "preview_mock_suggestion" => self.handle_ui_asset_preview_mock_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            "binding_payload" => self.handle_ui_asset_binding_payload_detail(
                instance_id,
                action_id,
                primary,
                secondary,
            ),
            "binding_payload_suggestion" => self.handle_ui_asset_binding_payload_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            "palette_drag" => {
                self.handle_ui_asset_palette_drag_detail(instance_id, action_id, primary, secondary)
            }
            "source" => {
                self.handle_ui_asset_source_detail(instance_id, action_id, item_index, primary)
            }
            "binding_route_suggestion" => self.handle_ui_asset_binding_route_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            "binding_action_suggestion" => self.handle_ui_asset_binding_action_suggestion_detail(
                instance_id,
                action_id,
                item_index,
            ),
            other => {
                self.focus_callback_source_window();
                self.set_status_line(format!("Unknown UI asset detail event {other}:{action_id}"));
            }
        }
    }

    fn handle_ui_asset_style_class_detail(
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

    fn handle_ui_asset_widget_detail(&mut self, instance_id: &str, action_id: &str, value: &str) {
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
                self.set_status_line(format!("Unknown UI asset widget action {other}"));
            }
        }
    }

    fn handle_ui_asset_widget_promote_detail(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_slot_detail(&mut self, instance_id: &str, action_id: &str, value: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "slot.mount.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.mount",
                    value,
                );
                return;
            }
            "slot.padding.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.padding",
                    value,
                );
                return;
            }
            "slot.layout.width.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.width_preferred",
                    value,
                );
                return;
            }
            "slot.layout.height.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.height_preferred",
                    value,
                );
                return;
            }
            "slot.semantic.value.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "slot.semantic.value",
                    value,
                );
                return;
            }
            "slot.semantic.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_slot_semantic(&instance_id)
                .map(|_| ()),
            other => {
                if let Some(path) = slot_semantic_action_path(other) {
                    self.dispatch_ui_asset_component_adapter_commit(
                        instance_id.0.as_str(),
                        action_id,
                        &format!("slot.semantic.field.{path}"),
                        value,
                    );
                    return;
                } else {
                    self.set_status_line(format!("Unknown UI asset slot action {other}"));
                    return;
                }
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_layout_detail(&mut self, instance_id: &str, action_id: &str, value: &str) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "layout.width.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.width_preferred",
                    value,
                );
                return;
            }
            "layout.height.preferred.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.height_preferred",
                    value,
                );
                return;
            }
            "layout.semantic.value.set" => {
                self.dispatch_ui_asset_component_adapter_commit(
                    instance_id.0.as_str(),
                    action_id,
                    "layout.semantic.value",
                    value,
                );
                return;
            }
            "layout.semantic.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_layout_semantic(&instance_id)
                .map(|_| ()),
            other => {
                if let Some(path) = layout_semantic_action_path(other) {
                    self.dispatch_ui_asset_component_adapter_commit(
                        instance_id.0.as_str(),
                        action_id,
                        &format!("layout.semantic.field.{path}"),
                        value,
                    );
                    return;
                } else {
                    self.set_status_line(format!("Unknown UI asset layout action {other}"));
                    return;
                }
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn dispatch_ui_asset_component_adapter_commit(
        &mut self,
        instance_id: &str,
        control_id: &str,
        target_path: &str,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let envelope = UiComponentEventEnvelope::new(
            "ui_asset.editor",
            control_id,
            UiComponentBindingTarget::asset_editor(instance_id.to_string(), target_path),
            UiComponentEvent::Commit {
                property: "value".to_string(),
                value: UiValue::String(value.to_string()),
            },
        )
        .with_component_id(control_id);

        match self.runtime.dispatch_ui_component_adapter_event(&envelope) {
            Ok(result) => {
                if let Some(status_text) = result.status_text {
                    self.set_status_line(status_text);
                }
                if result.changed || result.refresh_projection || !result.patches.is_empty() {
                    self.presentation_dirty = true;
                }
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_binding_detail(&mut self, instance_id: &str, action_id: &str, value: &str) {
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_theme_source_detail(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_style_rule_detail(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    pub(super) fn dispatch_ui_asset_collection_event(
        &mut self,
        instance_id: &str,
        collection_id: &str,
        event_kind: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let item_index = item_index.max(0) as usize;
        let result = match (collection_id, event_kind) {
            ("matched_style_rule", "selected") => self
                .editor_manager
                .select_ui_asset_editor_matched_style_rule(&instance_id, item_index)
                .map(|_| ()),
            ("palette", "selected") => self
                .editor_manager
                .select_ui_asset_editor_palette_index(&instance_id, item_index)
                .map(|_| ()),
            ("palette_target_candidate", "selected") => self
                .editor_manager
                .select_ui_asset_editor_palette_target_candidate(&instance_id, item_index)
                .map(|_| ()),
            ("hierarchy", "selected") => self
                .editor_manager
                .select_ui_asset_editor_hierarchy_index(&instance_id, item_index)
                .map(|_| ()),
            ("hierarchy", "activated") => self
                .editor_manager
                .activate_ui_asset_editor_hierarchy_index(&instance_id, item_index)
                .map(|_| ()),
            ("preview", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_index(&instance_id, item_index)
                .map(|_| ()),
            ("preview", "activated") => self
                .editor_manager
                .activate_ui_asset_editor_preview_index(&instance_id, item_index)
                .map(|_| ()),
            ("source_outline", "selected") => self
                .editor_manager
                .select_ui_asset_editor_source_outline_index(&instance_id, item_index)
                .map(|_| ()),
            ("preview_mock_subject", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_mock_subject(&instance_id, item_index)
                .map(|_| ()),
            ("preview_mock", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_mock_property(&instance_id, item_index)
                .map(|_| ()),
            ("preview_mock_nested", "selected") => self
                .editor_manager
                .select_ui_asset_editor_preview_mock_nested_entry(&instance_id, item_index)
                .map(|_| ()),
            ("binding", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding(&instance_id, item_index)
                .map(|_| ()),
            ("binding_event", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding_event_option(&instance_id, item_index)
                .map(|_| ()),
            ("binding_action_kind", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding_action_kind(&instance_id, item_index)
                .map(|_| ()),
            ("binding_payload", "selected") => self
                .editor_manager
                .select_ui_asset_editor_binding_payload(&instance_id, item_index)
                .map(|_| ()),
            ("slot_semantic", "selected") => self
                .editor_manager
                .select_ui_asset_editor_slot_semantic(&instance_id, item_index)
                .map(|_| ()),
            ("layout_semantic", "selected") => self
                .editor_manager
                .select_ui_asset_editor_layout_semantic(&instance_id, item_index)
                .map(|_| ()),
            _ => {
                self.set_status_line(format!(
                    "Unknown UI asset collection event {collection_id}:{event_kind}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_style_rule_declaration_detail(
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

    fn handle_ui_asset_style_token_detail(
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

    fn handle_ui_asset_source_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
        value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match action_id {
            "source.text.set" => match self
                .editor_manager
                .update_ui_asset_editor_source(&instance_id, value.to_string())
            {
                Ok(()) => self.presentation_dirty = true,
                Err(error) => self.set_status_line(error.to_string()),
            },
            "source.cursor.set" => match self
                .editor_manager
                .select_ui_asset_editor_source_byte_offset(&instance_id, item_index.max(0) as usize)
            {
                Ok(changed) => {
                    if changed {
                        self.presentation_dirty = true;
                    }
                }
                Err(error) => self.set_status_line(error.to_string()),
            },
            other => {
                self.set_status_line(format!("Unknown UI asset source action {other}"));
            }
        }
    }

    fn handle_ui_asset_palette_drag_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        primary: &str,
        secondary: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        match action_id {
            "palette.drag.hover" => {
                let surface_x = match primary.parse::<f32>() {
                    Ok(value) => value,
                    Err(error) => {
                        self.set_status_line(format!(
                            "Invalid UI asset palette drag hover x `{primary}`: {error}"
                        ));
                        return;
                    }
                };
                let surface_y = match secondary.parse::<f32>() {
                    Ok(value) => value,
                    Err(error) => {
                        self.set_status_line(format!(
                            "Invalid UI asset palette drag hover y `{secondary}`: {error}"
                        ));
                        return;
                    }
                };
                match self
                    .editor_manager
                    .update_ui_asset_editor_palette_drag_target(&instance_id, surface_x, surface_y)
                {
                    Ok(_) => self.presentation_dirty = true,
                    Err(error) => self.set_status_line(error.to_string()),
                }
            }
            other => {
                self.set_status_line(format!("Unknown UI asset palette drag action {other}"));
            }
        }
    }

    fn handle_ui_asset_preview_mock_detail(
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

    fn handle_ui_asset_preview_mock_nested_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        nested_key: &str,
        nested_value: &str,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "preview.mock.nested.value.set" => self
                .editor_manager
                .set_ui_asset_editor_selected_preview_mock_nested_value(&instance_id, nested_key)
                .map(|_| ()),
            "preview.mock.nested.upsert" => self
                .editor_manager
                .upsert_ui_asset_editor_selected_preview_mock_nested_entry(
                    &instance_id,
                    nested_key,
                    nested_value,
                )
                .map(|_| ()),
            "preview.mock.nested.delete" => self
                .editor_manager
                .delete_ui_asset_editor_selected_preview_mock_nested_entry(&instance_id)
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset preview mock nested action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_binding_payload_detail(
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

    fn handle_ui_asset_preview_mock_suggestion_detail(
        &mut self,
        instance_id: &str,
        action_id: &str,
        item_index: i32,
    ) {
        self.focus_callback_source_window();
        let instance_id = ViewInstanceId::new(instance_id);
        let result = match action_id {
            "preview.mock.suggestion.apply" => self
                .editor_manager
                .apply_ui_asset_editor_selected_preview_mock_suggestion(
                    &instance_id,
                    item_index.max(0) as usize,
                )
                .map(|_| ()),
            other => {
                self.set_status_line(format!(
                    "Unknown UI asset preview mock suggestion action {other}"
                ));
                return;
            }
        };

        match result {
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_binding_payload_suggestion_detail(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_binding_route_suggestion_detail(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }

    fn handle_ui_asset_binding_action_suggestion_detail(
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
            Ok(()) => self.presentation_dirty = true,
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}

fn slot_semantic_action_path(action_id: &str) -> Option<&'static str> {
    match action_id {
        "slot.linear.width_weight.set" => Some("layout.width.weight"),
        "slot.linear.width_stretch.set" => Some("layout.width.stretch"),
        "slot.linear.height_weight.set" => Some("layout.height.weight"),
        "slot.linear.height_stretch.set" => Some("layout.height.stretch"),
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
        "layout.box.gap.set" => Some("container.gap"),
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

#[cfg(test)]
mod tests {
    use super::{layout_semantic_action_path, slot_semantic_action_path};

    #[test]
    fn layout_semantic_action_path_maps_linear_box_gap_action() {
        assert_eq!(
            layout_semantic_action_path("layout.box.gap.set"),
            Some("container.gap")
        );
    }

    #[test]
    fn slot_semantic_action_path_maps_linear_slot_actions() {
        assert_eq!(
            slot_semantic_action_path("slot.linear.width_weight.set"),
            Some("layout.width.weight")
        );
        assert_eq!(
            slot_semantic_action_path("slot.linear.width_stretch.set"),
            Some("layout.width.stretch")
        );
        assert_eq!(
            slot_semantic_action_path("slot.linear.height_weight.set"),
            Some("layout.height.weight")
        );
        assert_eq!(
            slot_semantic_action_path("slot.linear.height_stretch.set"),
            Some("layout.height.stretch")
        );
    }
}
