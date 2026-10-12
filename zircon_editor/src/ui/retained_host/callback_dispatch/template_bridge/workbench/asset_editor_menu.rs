use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const ASSET_EDITOR_MENU_COMMANDS: &[AssetEditorMenuCommand] = &[
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.terrain_editor",
        label_key: "editor.workbench.tools.terrain_editor",
        extension_action_id: "workbench.extension.terrain_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.foliage_editor",
        label_key: "editor.workbench.tools.foliage_editor",
        extension_action_id: "workbench.extension.foliage_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.level_streaming",
        label_key: "editor.workbench.tools.level_streaming",
        extension_action_id: "workbench.extension.level_streaming.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.level_variant",
        label_key: "editor.workbench.tools.level_variant",
        extension_action_id: "workbench.extension.level_variant.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.prefab_editor",
        label_key: "editor.workbench.tools.prefab_editor",
        extension_action_id: "workbench.extension.prefab_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.scatter_editor",
        label_key: "editor.workbench.tools.scatter_editor",
        extension_action_id: "workbench.extension.scatter_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.volume_editor",
        label_key: "editor.workbench.tools.volume_editor",
        extension_action_id: "workbench.extension.volume_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsWorldToolsMenu",
        menu_action_id: "menu.item.assets.weather_editor",
        label_key: "editor.workbench.tools.weather_editor",
        extension_action_id: "workbench.extension.weather_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.spawn_rules",
        label_key: "editor.workbench.tools.spawn_rules",
        extension_action_id: "workbench.extension.spawn_rules.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.world_state",
        label_key: "editor.workbench.tools.world_state",
        extension_action_id: "workbench.extension.world_state.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.collision_proxy",
        label_key: "editor.workbench.tools.collision_proxy",
        extension_action_id: "workbench.extension.collision_proxy.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.physics_collision",
        label_key: "editor.workbench.tools.physics_collision",
        extension_action_id: "workbench.extension.physics_collision.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.navmesh_ai",
        label_key: "editor.workbench.tools.navmesh_ai",
        extension_action_id: "workbench.extension.navmesh_ai.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.lobby_editor",
        label_key: "editor.workbench.tools.lobby_editor",
        extension_action_id: "workbench.extension.lobby_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsGameplayToolsMenu",
        menu_action_id: "menu.item.assets.matchmaking_editor",
        label_key: "editor.workbench.tools.matchmaking_editor",
        extension_action_id: "workbench.extension.matchmaking_editor.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.data_table",
        label_key: "editor.workbench.tools.data_table",
        extension_action_id: "workbench.extension.data_table.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.source_control",
        label_key: "editor.workbench.tools.source_control",
        extension_action_id: "workbench.extension.source_control.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.build_export",
        label_key: "editor.workbench.tools.build_export",
        extension_action_id: "workbench.extension.build_export.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.automation_report",
        label_key: "editor.workbench.tools.automation_report",
        extension_action_id: "workbench.extension.automation_report.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.project_overview",
        label_key: "editor.workbench.tools.project_overview",
        extension_action_id: "workbench.extension.project_overview.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.plugin_manager",
        label_key: "editor.workbench.tools.plugin_manager",
        extension_action_id: "workbench.extension.plugin_manager.open",
    },
    AssetEditorMenuCommand {
        menu_control_id: "WorkbenchAssetsProductionToolsMenu",
        menu_action_id: "menu.item.assets.save_data",
        label_key: "editor.workbench.tools.save_data",
        extension_action_id: "workbench.extension.save_data.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn workbench_asset_editor_menu_items(&self, menu_control_id: &str) -> Vec<String> {
        let locale = self.i18n.active_locale();
        ASSET_EDITOR_MENU_COMMANDS
            .iter()
            .filter(|command| command.menu_control_id == menu_control_id)
            .map(|command| {
                let unavailable =
                    super::extension_module_navigation::workbench_extension_workspace_document(
                        command.extension_action_id,
                    )
                    .is_none();
                format!(
                    "{}|action={}{}",
                    self.i18n.translate_for_locale(&locale, command.label_key),
                    command.menu_action_id,
                    if unavailable { ",disabled" } else { "" }
                )
            })
            .collect()
    }

    pub(crate) fn dispatch_workbench_asset_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        let Some(extension_action_id) =
            asset_editor_menu_extension_action(menu_control_id, menu_action_id)
        else {
            return Ok(None);
        };

        self.apply_reference_menu_action(menu_control_id, extension_action_id)?;
        Ok(Some(EditorUiBinding::new(
            menu_control_id,
            menu_action_id,
            EditorUiEventKind::Click,
            EditorUiBindingPayload::menu_action(extension_action_id),
        )))
    }
}

pub(super) fn asset_editor_menu_extension_action(
    menu_control_id: &str,
    menu_action_id: &str,
) -> Option<&'static str> {
    ASSET_EDITOR_MENU_COMMANDS
        .iter()
        .find(|command| {
            command.menu_control_id == menu_control_id && command.menu_action_id == menu_action_id
        })
        .map(|command| command.extension_action_id)
}

struct AssetEditorMenuCommand {
    label_key: &'static str,
    menu_control_id: &'static str,
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
#[path = "tests/asset_editor_menu.rs"]
mod tests;
