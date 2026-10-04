use crate::core::editor_event::MenuAction;
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::EditorUiBinding;
use crate::ui::template_runtime::builtin::builtin_template_bindings;
use crate::ui::workbench::event::{editor_operation_binding, menu_action_binding};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;

const MAIN_MENU_CONTROL_ID: &str = "WorkbenchToolbarMainMenu";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(crate) fn main_menu_item_binding(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<EditorUiBinding> {
        if menu_control_id != MAIN_MENU_CONTROL_ID {
            return None;
        }
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
#[path = "tests/main_menu_items.rs"]
mod tests;
