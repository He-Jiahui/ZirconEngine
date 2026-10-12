use crate::ui::binding::{EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use super::error::BuiltinHostWindowTemplateBridgeError;

const RENDER_EDITOR_MENU_COMMANDS: &[RenderEditorMenuCommand] = &[
    RenderEditorMenuCommand {
        menu_action_id: "menu.item.render.shader_editor",
        label_key: "editor.workbench.tools.shader_editor",
        extension_action_id: "workbench.extension.shader_editor.open",
    },
    RenderEditorMenuCommand {
        menu_action_id: "menu.item.render.lighting_bake",
        label_key: "editor.workbench.tools.lighting_bake",
        extension_action_id: "workbench.extension.lighting_bake.open",
    },
    RenderEditorMenuCommand {
        menu_action_id: "menu.item.render.post_process",
        label_key: "editor.workbench.tools.post_process",
        extension_action_id: "workbench.extension.post_process.open",
    },
];

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn workbench_render_editor_menu_items(&self, menu_control_id: &str) -> Vec<String> {
        let locale = self.i18n.active_locale();
        RENDER_EDITOR_MENU_COMMANDS
            .iter()
            .filter(|_| menu_control_id == "WorkbenchRenderToolsMenu")
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

    pub(crate) fn dispatch_workbench_render_editor_menu_item_state(
        &mut self,
        menu_control_id: &str,
        menu_action_id: &str,
    ) -> Result<Option<EditorUiBinding>, BuiltinHostWindowTemplateBridgeError> {
        if menu_control_id != "WorkbenchRenderToolsMenu" {
            return Ok(None);
        }
        let Some(extension_action_id) =
            render_editor_menu_extension_action(menu_control_id, menu_action_id)
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

pub(super) fn render_editor_menu_extension_action(
    menu_control_id: &str,
    menu_action_id: &str,
) -> Option<&'static str> {
    RENDER_EDITOR_MENU_COMMANDS
        .iter()
        .find(|command| {
            menu_control_id == "WorkbenchRenderToolsMenu"
                && command.menu_action_id == menu_action_id
        })
        .map(|command| command.extension_action_id)
}

struct RenderEditorMenuCommand {
    label_key: &'static str,
    menu_action_id: &'static str,
    extension_action_id: &'static str,
}

#[cfg(test)]
#[path = "../../workbench/tests/render_editor_menu.rs"]
mod tests;
