use std::sync::Arc;

use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    layout::{StretchMode, UiSize},
};

use crate::core::asset::AssetTypeId;
use crate::ui::retained_host::host_contract::{current_host_metrics, menu_popup_text_width};
use crate::ui::retained_host::menu_popup_contract::{
    content_measured_structured_menu_popup_width, menu_popup_content_height,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const MAIN_MENU_CONTROL_ID: &str = "WorkbenchToolbarMainMenu";

#[derive(Default)]
pub(super) struct AssetCreationMenuState {
    generation: Option<Arc<crate::core::asset::AssetCreationMenuGeneration>>,
    shortcuts: Option<MainMenuShortcutSignature>,
    authored_width: Option<f32>,
    desired_width: f32,
    #[cfg(test)]
    publish_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MainMenuShortcutSignature {
    open_project: Option<String>,
    save_project: Option<String>,
    command_palette: Option<String>,
}

impl MainMenuShortcutSignature {
    fn from_keymap(keymap: &crate::core::commands::EditorKeymap) -> Self {
        Self {
            open_project: keymap
                .chord_for_command("file.project.open")
                .map(ToString::to_string),
            save_project: keymap
                .chord_for_command("file.project.save")
                .map(ToString::to_string),
            command_palette: keymap
                .chord_for_command("editor.command.palette")
                .map(ToString::to_string),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AssetCreationMenuRequest {
    asset_type: AssetTypeId,
    template_id: String,
    target_folder: String,
}

impl AssetCreationMenuRequest {
    pub(crate) fn asset_type(&self) -> &AssetTypeId {
        &self.asset_type
    }

    pub(crate) fn template_id(&self) -> &str {
        &self.template_id
    }

    pub(crate) fn target_folder(&self) -> &str {
        &self.target_folder
    }
}

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_asset_creation_menu_state(
        &mut self,
        model: &WorkbenchViewModel,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let generation = &model.asset_creation_menu;
        let shortcuts = MainMenuShortcutSignature::from_keymap(&model.keymap);
        let item_count = if generation.entries().is_empty() {
            5
        } else {
            generation.entries().len() + 6
        };
        if self
            .asset_creation_menu
            .generation
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, generation))
            && self.asset_creation_menu.shortcuts.as_ref() == Some(&shortcuts)
        {
            return self.apply_asset_creation_menu_extent(item_count, shell_size);
        }
        let authored_width = self
            .asset_creation_menu
            .authored_width
            .unwrap_or_else(|| self.authored_main_menu_width());
        let mut menu_items = Vec::with_capacity(item_count);
        menu_items.push("Asset Browser|action=menu.item.asset_browser,icon=folder".to_string());
        if !generation.entries().is_empty() {
            menu_items.push("---".to_string());
            menu_items.extend(
                generation
                    .entries()
                    .iter()
                    .map(|entry| entry.raw_item().to_string()),
            );
        }
        menu_items.extend([
            "---".to_string(),
            main_menu_item_value(
                "Open Project",
                "action=menu.item.open_project,icon=folder",
                shortcuts.open_project.as_deref(),
            ),
            main_menu_item_value(
                "Save Project",
                "action=menu.item.save_project,icon=save",
                shortcuts.save_project.as_deref(),
            ),
            main_menu_item_value(
                "Command Palette",
                "action=menu.item.command_palette,icon=search",
                shortcuts.command_palette.as_deref(),
            ),
        ]);
        let desired_width =
            measure_main_menu_width(&menu_items, authored_width, self.presentation_scale_factor);
        self.mutate_control_property(
            MAIN_MENU_CONTROL_ID,
            "menu_items",
            UiValue::Array(menu_items.into_iter().map(UiValue::String).collect()),
        )?;
        self.asset_creation_menu.generation = Some(Arc::clone(generation));
        self.asset_creation_menu.shortcuts = Some(shortcuts);
        self.asset_creation_menu.authored_width = Some(authored_width);
        self.asset_creation_menu.desired_width = desired_width;
        #[cfg(test)]
        {
            self.asset_creation_menu.publish_count += 1;
        }
        self.apply_asset_creation_menu_extent(item_count, shell_size)
    }

    fn authored_main_menu_width(&self) -> f32 {
        self.control_node_id(MAIN_MENU_CONTROL_ID)
            .and_then(|node_id| self.template_surface.surface.tree.node(node_id))
            .map(|node| node.constraints.width.preferred.max(1.0))
            .unwrap_or(1.0)
    }

    fn apply_asset_creation_menu_extent(
        &mut self,
        item_count: usize,
        shell_size: UiSize,
    ) -> Result<(), BuiltinHostWindowTemplateBridgeError> {
        let toolbar_bottom = self
            .control_frame("WorkbenchWindowTopToolbarRegion")
            .map(|frame| frame.bottom())
            .unwrap_or(0.0);
        let available_height = (shell_size.height - toolbar_bottom).max(1.0);
        let height = menu_popup_content_height(item_count)
            .min(available_height)
            .max(1.0);
        let width = self
            .asset_creation_menu
            .desired_width
            .min(shell_size.width.max(1.0))
            .max(1.0);
        let Some(node_id) = self.control_node_id(MAIN_MENU_CONTROL_ID) else {
            return Ok(());
        };
        let changed = if let Some(node) = self.template_surface.surface.tree.node_mut(node_id) {
            let mut next_width = node.constraints.width;
            next_width.min = width;
            next_width.preferred = width;
            next_width.max = width;
            next_width.stretch_mode = StretchMode::Fixed;
            let mut next_height = node.constraints.height;
            next_height.min = height;
            next_height.preferred = height;
            next_height.max = height;
            next_height.stretch_mode = StretchMode::Fixed;
            let changed =
                node.constraints.width != next_width || node.constraints.height != next_height;
            node.constraints.width = next_width;
            node.constraints.height = next_height;
            changed
        } else {
            false
        };
        if changed {
            self.template_surface
                .surface
                .tree
                .mark_layout_dirty(node_id)?;
        }
        Ok(())
    }

    pub(crate) fn asset_creation_menu_request(
        &self,
        snapshot: &AssetWorkspaceSnapshot,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<Result<AssetCreationMenuRequest, String>> {
        if menu_control_id != MAIN_MENU_CONTROL_ID {
            return None;
        }
        let Some(generation) = self.asset_creation_menu.generation.as_ref() else {
            return None;
        };
        if !Arc::ptr_eq(generation, &snapshot.creation_menu) {
            return Some(Err(
                "asset creation menu generation changed before dispatch".to_string(),
            ));
        }
        let entry = generation.action(action_id)?;
        Some(Ok(AssetCreationMenuRequest {
            asset_type: entry.asset_type().clone(),
            template_id: entry.template_id().to_string(),
            target_folder: snapshot
                .selected_folder_id
                .clone()
                .unwrap_or_else(|| "res://".to_string()),
        }))
    }

    pub(crate) fn is_asset_creation_menu_action(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> bool {
        menu_control_id == MAIN_MENU_CONTROL_ID
            && self
                .asset_creation_menu
                .generation
                .as_ref()
                .is_some_and(|generation| generation.action(action_id).is_some())
    }

    #[cfg(test)]
    pub(crate) fn asset_creation_menu_publish_count(&self) -> usize {
        self.asset_creation_menu.publish_count
    }
}

fn main_menu_item_value(label: &str, flags: &str, shortcut: Option<&str>) -> String {
    match shortcut {
        Some(shortcut) => format!("{label}|{flags}|{shortcut}"),
        None => format!("{label}|{flags}"),
    }
}

fn measure_main_menu_width(
    menu_items: &[String],
    fallback_width: f32,
    presentation_scale_factor: f32,
) -> f32 {
    let scale_factor =
        if presentation_scale_factor.is_finite() && presentation_scale_factor > f32::EPSILON {
            presentation_scale_factor
        } else {
            1.0
        };
    let metrics = current_host_metrics();
    let adornment_reserved_width =
        (metrics.font_large + metrics.gap_m * 2.0 - metrics.input_pad[1]).max(0.0) / scale_factor;
    content_measured_structured_menu_popup_width(
        fallback_width,
        f32::MAX,
        menu_items.iter().map(String::as_str),
        adornment_reserved_width,
        |text| menu_popup_text_width(text) / scale_factor,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_menu_width_measures_label_shortcut_and_trailing_icon() {
        let items = [
            "---".to_string(),
            "Command Palette|action=menu.item.command_palette,icon=search|Ctrl+Shift+P".to_string(),
        ];

        assert!(measure_main_menu_width(&items, 190.0, 1.0) > 190.0);
    }
}
