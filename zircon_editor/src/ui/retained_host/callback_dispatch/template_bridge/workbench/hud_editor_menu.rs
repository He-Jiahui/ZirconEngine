use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const HUD_EDITOR_MENU_COMMANDS: &[HudEditorMenuCommand] = &[
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.console_diagnostics",
        extension_action_id: "workbench.extension.console_diagnostics.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.runtime_diagnostics",
        extension_action_id: "workbench.extension.runtime_diagnostics.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.telemetry_dashboard",
        extension_action_id: "workbench.extension.telemetry_dashboard.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.performance",
        extension_action_id: "workbench.extension.performance.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.font_atlas",
        extension_action_id: "workbench.extension.font_atlas.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.menu_flow",
        extension_action_id: "workbench.extension.menu_flow.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.accessibility_audit",
        extension_action_id: "workbench.extension.accessibility_audit.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.icon_library",
        extension_action_id: "workbench.extension.icon_library.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.ui_binding",
        extension_action_id: "workbench.extension.ui_binding.open",
    },
    HudEditorMenuCommand {
        menu_action_id: "menu.item.hud.ui_asset_editor",
        extension_action_id: "workbench.extension.ui_asset_editor.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn dispatch_workbench_hud_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != "WorkbenchHudToolsMenu" {
            return Ok(None);
        }
        let Some(command) = HUD_EDITOR_MENU_COMMANDS
            .iter()
            .find(|command| command.menu_action_id == menu_action_id)
        else {
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

struct HudEditorMenuCommand {
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn hud_editor_menu_actions_are_complete_and_unique() {
        assert_eq!(10, HUD_EDITOR_MENU_COMMANDS.len());
        assert_eq!(
            HUD_EDITOR_MENU_COMMANDS.len(),
            HUD_EDITOR_MENU_COMMANDS
                .iter()
                .map(|command| command.menu_action_id)
                .collect::<HashSet<_>>()
                .len()
        );
        assert!(HUD_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.menu_action_id.starts_with("menu.item.hud.")));
        assert!(HUD_EDITOR_MENU_COMMANDS.iter().all(|command| command
            .extension_action_id
            .starts_with("workbench.extension.")));
        assert!(HUD_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.extension_action_id.ends_with(".open")));
    }
}
