use crate::core::editor_event::MenuAction;
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::{DockCommand, EditorUiBinding, EditorUiBindingPayload, EditorUiEventKind};
use crate::ui::template_runtime::builtin::builtin_template_bindings;
use crate::ui::workbench::event::{editor_operation_binding, menu_action_binding};
use zircon_runtime::scene::components::NodeKind;

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;

const MAIN_MENU_CONTROL_ID: &str = "WorkbenchToolbarMainMenu";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn main_menu_item_binding(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<EditorUiBinding> {
        if menu_control_id == MAIN_MENU_CONTROL_ID {
            return self.toolbar_main_menu_item_binding(action_id);
        }
        let action = match (menu_control_id, action_id) {
            ("UnrealFileMenuPopup", "menu.item.new_scene") => MenuAction::CreateScene,
            ("UnrealFileMenuPopup", "menu.item.open_scene") => MenuAction::OpenScene,
            ("UnrealFileMenuPopup", "file.project.save") => MenuAction::SaveProject,
            ("UnrealEditMenuPopup", "workbench.history.undo") => MenuAction::Undo,
            ("UnrealEditMenuPopup", "workbench.history.redo") => MenuAction::Redo,
            ("UnrealWindowMenuPopup", "workbench.layout.save") => MenuAction::SaveLayout,
            ("UnrealWindowMenuPopup", "workbench.layout.reset") => MenuAction::ResetLayout,
            ("UnrealWindowMenuPopup", "LoadPreset.current") => {
                return Some(EditorUiBinding::new(
                    "DockCommand",
                    "LoadPreset",
                    EditorUiEventKind::Click,
                    EditorUiBindingPayload::dock_command(DockCommand::LoadPreset {
                        name: "current".to_string(),
                    }),
                ));
            }
            ("UnrealModesMenuPopup", "workbench.scene.mode.select") => {
                return builtin_template_bindings().get("Tool/Select").cloned();
            }
            ("UnrealAddMenuPopup", "workbench.scene.node.create.cube") => {
                MenuAction::CreateNode(NodeKind::Cube)
            }
            _ => return None,
        };
        Some(menu_action_binding(&action))
    }

    fn toolbar_main_menu_item_binding(&self, action_id: &str) -> Option<EditorUiBinding> {
        match action_id {
            "menu.item.asset_browser" => builtin_template_bindings()
                .get("AssetSurface/OpenAssetBrowser")
                .cloned(),
            "menu.item.open_project" => Some(menu_action_binding(&MenuAction::OpenProject)),
            "menu.item.new_scene" => Some(menu_action_binding(&MenuAction::CreateScene)),
            "menu.item.open_scene" => Some(menu_action_binding(&MenuAction::OpenScene)),
            "menu.item.save_project" => Some(menu_action_binding(&MenuAction::SaveProject)),
            "menu.item.command_palette" => Some(editor_operation_binding(
                &EditorOperationPath::parse("editor.command.palette")
                    .expect("the built-in command palette operation path is valid"),
            )),
            _ => None,
        }
    }
}

#[cfg(test)]
#[path = "../../workbench/tests/main_menu_items.rs"]
mod tests;
