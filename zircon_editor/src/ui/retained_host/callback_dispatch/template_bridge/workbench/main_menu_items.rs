use crate::ui::binding::EditorUiBinding;

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
        let binding_id = match action_id {
            "menu.item.asset_browser" => "AssetSurface/OpenAssetBrowser",
            "menu.item.open_project" => "MenuAction/OpenProject",
            "menu.item.save_project" => "MenuAction/SaveProject",
            "menu.item.command_palette" => "CommandPalette/Commit",
            _ => return None,
        };
        self.binding_by_id(binding_id).cloned()
    }
}
