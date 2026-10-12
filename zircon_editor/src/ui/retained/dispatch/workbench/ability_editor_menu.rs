use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const ABILITY_EDITOR_MENU_COMMANDS: &[AbilityEditorMenuCommand] = &[
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.sequencer",
        label_key: "editor.workbench.tools.sequencer",
        extension_action_id: "workbench.extension.sequencer.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.montage_editor",
        label_key: "editor.workbench.tools.montage_editor",
        extension_action_id: "workbench.extension.montage_editor.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.blend_space",
        label_key: "editor.workbench.tools.blend_space",
        extension_action_id: "workbench.extension.blend_space.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.pose_library",
        label_key: "editor.workbench.tools.pose_library",
        extension_action_id: "workbench.extension.pose_library.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.retarget",
        label_key: "editor.workbench.tools.retarget",
        extension_action_id: "workbench.extension.retarget.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.control_rig",
        label_key: "editor.workbench.tools.control_rig",
        extension_action_id: "workbench.extension.control_rig.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.motion_matching",
        label_key: "editor.workbench.tools.motion_matching",
        extension_action_id: "workbench.extension.motion_matching.open",
    },
    AbilityEditorMenuCommand {
        menu_action_id: "menu.item.ability.animation_compression",
        label_key: "editor.workbench.tools.animation_compression",
        extension_action_id: "workbench.extension.animation_compression.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn workbench_ability_editor_menu_items(&self, menu_control_id: &str) -> Vec<String> {
        let locale = self.i18n.active_locale();
        ABILITY_EDITOR_MENU_COMMANDS
            .iter()
            .filter(|_| menu_control_id == "WorkbenchAbilityAnimationToolsMenu")
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

    pub(crate) fn dispatch_workbench_ability_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != "WorkbenchAbilityAnimationToolsMenu" {
            return Ok(None);
        }
        let Some(extension_action_id) =
            ability_editor_menu_extension_action(menu_control_id, menu_action_id)
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

pub(super) fn ability_editor_menu_extension_action(
    menu_control_id: &str,
    menu_action_id: &str,
) -> Option<&'static str> {
    ABILITY_EDITOR_MENU_COMMANDS
        .iter()
        .find(|command| {
            menu_control_id == "WorkbenchAbilityAnimationToolsMenu"
                && command.menu_action_id == menu_action_id
        })
        .map(|command| command.extension_action_id)
}

struct AbilityEditorMenuCommand {
    label_key: &'static str,
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
#[path = "tests/ability_editor_menu.rs"]
mod tests;
