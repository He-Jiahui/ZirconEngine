use super::super::*;
use crate::ui::asset_editor::{UiAssetEditorMode, UiAssetPreviewPreset, UiDesignerToolMode};
use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};

impl RetainedEditorHost {
    pub(super) fn dispatch_ui_asset_action_impl(&mut self, instance_id: &str, action_id: &str) {
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
                self.mark_presentation_dirty();
            }
            Err(error) => self.set_status_line(error.to_string()),
        }
    }
}
