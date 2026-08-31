use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const ASSET_EDITOR_MENU_COMMANDS: &[AssetEditorMenuCommand] = &[
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.terrain_editor",
        extension_action_id: "workbench.extension.terrain_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.foliage_editor",
        extension_action_id: "workbench.extension.foliage_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.level_streaming",
        extension_action_id: "workbench.extension.level_streaming.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.level_variant",
        extension_action_id: "workbench.extension.level_variant.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.prefab_editor",
        extension_action_id: "workbench.extension.prefab_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.scatter_editor",
        extension_action_id: "workbench.extension.scatter_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.volume_editor",
        extension_action_id: "workbench.extension.volume_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.weather_editor",
        extension_action_id: "workbench.extension.weather_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.spawn_rules",
        extension_action_id: "workbench.extension.spawn_rules.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.world_state",
        extension_action_id: "workbench.extension.world_state.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.collision_proxy",
        extension_action_id: "workbench.extension.collision_proxy.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.physics_collision",
        extension_action_id: "workbench.extension.physics_collision.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.navmesh_ai",
        extension_action_id: "workbench.extension.navmesh_ai.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.lobby_editor",
        extension_action_id: "workbench.extension.lobby_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.matchmaking_editor",
        extension_action_id: "workbench.extension.matchmaking_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.data_table",
        extension_action_id: "workbench.extension.data_table.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.source_control",
        extension_action_id: "workbench.extension.source_control.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.build_export",
        extension_action_id: "workbench.extension.build_export.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.automation_report",
        extension_action_id: "workbench.extension.automation_report.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.project_overview",
        extension_action_id: "workbench.extension.project_overview.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.plugin_manager",
        extension_action_id: "workbench.extension.plugin_manager.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.save_data",
        extension_action_id: "workbench.extension.save_data.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn dispatch_workbench_asset_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        let Some(command) = ASSET_EDITOR_MENU_COMMANDS.iter().find(|command| {
            command.menu_control_id == menu_control_id && command.menu_action_id == menu_action_id
        }) else {
            return Ok(None);
        };

        self.apply_reference_menu_action(menu_control_id, command.extension_action_id)?;
        Ok(Some(EditorUiBinding::new(
            menu_control_id,
            menu_action_id,
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action(command.extension_action_id),
        )))
    }
}

struct AssetEditorMenuCommand {
    menu_control_id: &'static str,
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn asset_editor_menu_actions_are_complete_and_unique() {
        assert_eq!(22, ASSET_EDITOR_MENU_COMMANDS.len());
        assert_eq!(
            ASSET_EDITOR_MENU_COMMANDS.len(),
            ASSET_EDITOR_MENU_COMMANDS
                .iter()
                .map(|command| (command.menu_control_id, command.menu_action_id))
                .collect::<HashSet<_>>()
                .len()
        );
        assert!(ASSET_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.menu_action_id.starts_with("menu.item.assets.")));
        assert!(ASSET_EDITOR_MENU_COMMANDS.iter().all(|command| command
            .extension_action_id
            .starts_with("workbench.extension.")));
        assert!(ASSET_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.extension_action_id.ends_with(".open")));
    }
}
