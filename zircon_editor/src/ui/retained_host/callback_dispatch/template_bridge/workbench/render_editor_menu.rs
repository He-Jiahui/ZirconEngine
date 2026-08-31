use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const RENDER_EDITOR_MENU_COMMANDS: &[RenderEditorMenuCommand] = &[
    RenderEditorMenuCommand {
        menu_action_id: "menu.item.render.shader_editor",
        extension_action_id: "workbench.extension.shader_editor.open",
    },
    RenderEditorMenuCommand {
        menu_action_id: "menu.item.render.lighting_bake",
        extension_action_id: "workbench.extension.lighting_bake.open",
    },
    RenderEditorMenuCommand {
        menu_action_id: "menu.item.render.post_process",
        extension_action_id: "workbench.extension.post_process.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn dispatch_workbench_render_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != "WorkbenchRenderToolsMenu" {
            return Ok(None);
        }
        let Some(command) = RENDER_EDITOR_MENU_COMMANDS
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

struct RenderEditorMenuCommand {
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn render_editor_menu_actions_are_complete_and_unique() {
        assert_eq!(3, RENDER_EDITOR_MENU_COMMANDS.len());
        assert_eq!(
            RENDER_EDITOR_MENU_COMMANDS.len(),
            RENDER_EDITOR_MENU_COMMANDS
                .iter()
                .map(|command| command.menu_action_id)
                .collect::<HashSet<_>>()
                .len()
        );
        assert!(RENDER_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.menu_action_id.starts_with("menu.item.render.")));
        assert!(RENDER_EDITOR_MENU_COMMANDS.iter().all(|command| command
            .extension_action_id
            .starts_with("workbench.extension.")));
        assert!(RENDER_EDITOR_MENU_COMMANDS
            .iter()
            .all(|command| command.extension_action_id.ends_with(".open")));
    }
}
