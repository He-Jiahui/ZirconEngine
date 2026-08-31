use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const ABILITY_EDITOR_MENU_COMMANDS: &[AbilityEditorMenuCommand] = &[
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.sequencer",
        extension_action_id: "workbench.extension.sequencer.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.montage_editor",
        extension_action_id: "workbench.extension.montage_editor.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.blend_space",
        extension_action_id: "workbench.extension.blend_space.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.pose_library",
        extension_action_id: "workbench.extension.pose_library.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.retarget",
        extension_action_id: "workbench.extension.retarget.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.control_rig",
        extension_action_id: "workbench.extension.control_rig.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.motion_matching",
        extension_action_id: "workbench.extension.motion_matching.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.animation_compression",
        extension_action_id: "workbench.extension.animation_compression.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn dispatch_workbench_ability_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != "WorkbenchAbilityAnimationToolsMenu" {
            return Ok(None);
        }
        let Some(command) = ABILITY_EDITOR_MENU_COMMANDS
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

struct AbilityEditorMenuCommand {
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn ability_editor_menu_actions_are_complete_and_unique() {
        assert_eq!(8, ABILITY_EDITOR_MENU_COMMANDS.len());
        assert_eq!(
            ABILITY_EDITOR_MENU_COMMANDS.len(),
            ABILITY_EDITOR_MENU_COMMANDS
                .iter()
                .map(|command| command.menu_action_id)
                .collect::<HashSet<_>>()
                .len()
        );
        assert!(ABILITY_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.menu_action_id.starts_with("menu.item.ability.")));
        assert!(ABILITY_EDITOR_MENU_COMMANDS.iter().all(|command| command
            .extension_action_id
            .starts_with("workbench.extension.")));
        assert!(ABILITY_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.extension_action_id.ends_with(".open")));
    }
}
