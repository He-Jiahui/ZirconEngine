use std::sync::Arc;

use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    layout::{StretchMode, UiSize},
};

use crate::core::asset::AssetTypeId;
use crate::ui::retained_host::menu_popup_contract::menu_popup_content_height;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::AssetWorkspaceSnapshot;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const MAIN_MENU_CONTROL_ID: &str = "WorkbenchToolbarMainMenu";

#[derive(Default)]
pub(super) struct AssetCreationMenuState {
    generation: Option<Arc<crate::core::asset::AssetCreationMenuGeneration>>,
    #[cfg(test)]
    publish_count: usize,
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
        let item_count = if generation.entries().is_empty() {
            5
        } else {
            generation.entries().len() + 6
        };
        self.apply_asset_creation_menu_height(item_count, shell_size)?;
        if self
            .asset_creation_menu
            .generation
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, generation))
        {
            return Ok(());
        }
        let mut menu_items = Vec::with_capacity(item_count);
        menu_items.push(UiValue::String("Asset Browser|icon=folder".to_string()));
        if !generation.entries().is_empty() {
            menu_items.push(UiValue::String("---".to_string()));
            menu_items.extend(
                generation
                    .entries()
                    .iter()
                    .map(|entry| UiValue::String(entry.raw_item().to_string())),
            );
        }
        menu_items.extend([
            UiValue::String("---".to_string()),
            UiValue::String("Open Project|icon=folder".to_string()),
            UiValue::String("Save Project|icon=save".to_string()),
            UiValue::String("Command Palette|submenu".to_string()),
        ]);
        self.mutate_control_property(
            MAIN_MENU_CONTROL_ID,
            "menu_items",
            UiValue::Array(menu_items),
        )?;
        self.asset_creation_menu.generation = Some(Arc::clone(generation));
        #[cfg(test)]
        {
            self.asset_creation_menu.publish_count += 1;
        }
        Ok(())
    }

    fn apply_asset_creation_menu_height(
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
        let Some(node_id) = self.control_node_id(MAIN_MENU_CONTROL_ID) else {
            return Ok(());
        };
        let changed = if let Some(node) = self.template_surface.surface.tree.node_mut(node_id) {
            let mut next = node.constraints.height;
            next.min = height;
            next.preferred = height;
            next.max = height;
            next.stretch_mode = StretchMode::Fixed;
            let changed = node.constraints.height != next;
            node.constraints.height = next;
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
