use std::collections::{BTreeMap, BTreeSet};

use zircon_runtime::ui::tree::UiRuntimeTreeLayoutExt;
use zircon_runtime_interface::ui::{
    component::UiValue,
    layout::{StretchMode, UiSize},
};

use crate::core::asset::AssetTypeId;
use crate::ui::retained_host::menu_popup_contract::menu_popup_content_height;
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::{AssetOperationProjectionSnapshot, AssetWorkspaceSnapshot};

use super::super::popup_primitives::menu_item_action_id;
use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const MAIN_MENU_CONTROL_ID: &str = "WorkbenchToolbarMainMenu";

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
        let mut menu_items = vec!["Asset Browser|icon=folder".to_string()];
        let creation_entries = asset_creation_menu_entries(&model.asset_creation_templates);
        if !creation_entries.is_empty() {
            menu_items.push("---".to_string());
            menu_items.extend(creation_entries.into_values().map(|entry| entry.raw_item));
        }
        menu_items.extend([
            "---".to_string(),
            "Open Project|icon=folder".to_string(),
            "Save Project|icon=save".to_string(),
            "Command Palette|submenu".to_string(),
        ]);
        self.apply_asset_creation_menu_height(menu_items.len(), shell_size)?;
        self.mutate_control_property(
            MAIN_MENU_CONTROL_ID,
            "menu_items",
            UiValue::Array(menu_items.into_iter().map(UiValue::String).collect()),
        )
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
        let Some(node_id) = self.surface().tree.nodes.values().find_map(|node| {
            node.template_metadata
                .as_ref()
                .and_then(|metadata| metadata.control_id.as_deref())
                .filter(|control_id| *control_id == MAIN_MENU_CONTROL_ID)
                .map(|_| node.node_id)
        }) else {
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
        let entry = asset_creation_menu_entries(&snapshot.creation_templates).remove(action_id)?;
        let asset_type = match AssetTypeId::parse(entry.asset_type_id) {
            Ok(asset_type) => asset_type,
            Err(error) => return Some(Err(error.to_string())),
        };
        Some(Ok(AssetCreationMenuRequest {
            asset_type,
            template_id: entry.template_id,
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
        menu_control_id == MAIN_MENU_CONTROL_ID && action_id.starts_with("menu.item.create_")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AssetCreationMenuEntry {
    raw_item: String,
    asset_type_id: String,
    template_id: String,
}

fn asset_creation_menu_entries(
    templates: &[AssetOperationProjectionSnapshot],
) -> BTreeMap<String, AssetCreationMenuEntry> {
    let base_labels = templates
        .iter()
        .map(asset_creation_base_label)
        .collect::<Vec<_>>();
    let mut action_counts = BTreeMap::<String, usize>::new();
    for label in &base_labels {
        *action_counts.entry(menu_item_action_id(label)).or_default() += 1;
    }

    let mut entries = BTreeMap::new();
    let mut used_action_ids = BTreeSet::new();
    for (template, base_label) in templates.iter().zip(base_labels) {
        let base_action_id = menu_item_action_id(&base_label);
        let label = if action_counts
            .get(&base_action_id)
            .copied()
            .unwrap_or_default()
            > 1
        {
            format!(
                "{base_label} ({}/{})",
                safe_menu_label(&template.asset_type_id),
                safe_menu_label(&template.id)
            )
        } else {
            base_label
        };
        let (label, action_id) = unique_menu_label(label, &mut used_action_ids);
        entries.insert(
            action_id,
            AssetCreationMenuEntry {
                raw_item: format!("{label}|icon=plus"),
                asset_type_id: template.asset_type_id.clone(),
                template_id: template.id.clone(),
            },
        );
    }
    entries
}

fn asset_creation_base_label(template: &AssetOperationProjectionSnapshot) -> String {
    format!("Create {}", safe_menu_label(&template.display_name))
}

fn unique_menu_label(
    mut label: String,
    used_action_ids: &mut BTreeSet<String>,
) -> (String, String) {
    let base_label = label.clone();
    let mut ordinal = 2usize;
    loop {
        let action_id = menu_item_action_id(&label);
        if used_action_ids.insert(action_id.clone()) {
            return (label, action_id);
        }
        label = format!("{base_label} {ordinal}");
        ordinal += 1;
    }
}

fn safe_menu_label(value: &str) -> String {
    let label = value.replace('|', " ");
    let label = label.trim();
    if label.is_empty() {
        "Asset".to_string()
    } else {
        label.to_string()
    }
}
