use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const HUD_EDITOR_MENU_COMMANDS: &[HudEditorMenuCommand] = &[
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.console_diagnostics",
        label_key: "editor.workbench.tools.console_diagnostics",
        extension_action_id: "workbench.extension.console_diagnostics.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.runtime_diagnostics",
        label_key: "editor.workbench.tools.runtime_diagnostics",
        extension_action_id: "workbench.extension.runtime_diagnostics.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.telemetry_dashboard",
        label_key: "editor.workbench.tools.telemetry_dashboard",
        extension_action_id: "workbench.extension.telemetry_dashboard.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.performance",
        label_key: "editor.workbench.tools.performance",
        extension_action_id: "workbench.extension.performance.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.font_atlas",
        label_key: "editor.workbench.tools.font_atlas",
        extension_action_id: "workbench.extension.font_atlas.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.menu_flow",
        label_key: "editor.workbench.tools.menu_flow",
        extension_action_id: "workbench.extension.menu_flow.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.accessibility_audit",
        label_key: "editor.workbench.tools.accessibility_audit",
        extension_action_id: "workbench.extension.accessibility_audit.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.icon_library",
        label_key: "editor.workbench.tools.icon_library",
        extension_action_id: "workbench.extension.icon_library.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.ui_binding",
        label_key: "editor.workbench.tools.ui_binding",
        extension_action_id: "workbench.extension.ui_binding.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.ui_asset_editor",
        label_key: "editor.workbench.tools.ui_asset_editor",
        extension_action_id: "workbench.extension.ui_asset_editor.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn workbench_hud_editor_menu_items(&self, menu_control_id: &str) -> Vec<String> {
        let locale = self.i18n.active_locale();
        HUD_EDITOR_MENU_COMMANDS
            .iter()
            .filter(|_| menu_control_id == "WorkbenchHudToolsMenu")
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

    pub(crate) fn dispatch_workbench_hud_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != "WorkbenchHudToolsMenu" {
            return Ok(None);
        }
        let Some(extension_action_id) =
            hud_editor_menu_extension_action(menu_control_id, menu_action_id)
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

pub(super) fn hud_editor_menu_extension_action(
    menu_control_id: &str,
    menu_action_id: &str,
) -> Option<&'static str> {
    HUD_EDITOR_MENU_COMMANDS
        .iter()
        .find(|command| {
            menu_control_id == "WorkbenchHudToolsMenu" && command.menu_action_id == menu_action_id
        })
        .map(|command| command.extension_action_id)
}

struct HudEditorMenuCommand {
    label_key: &'static str,
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
#[path = "../../workbench/tests/hud_editor_menu.rs"]
mod tests;
