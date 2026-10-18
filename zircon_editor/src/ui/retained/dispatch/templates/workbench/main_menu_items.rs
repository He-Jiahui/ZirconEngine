use crate::core::editor_event::MenuAction;
use crate::core::editor_operation::EditorOperationPath;
use crate::ui::binding::EditorUiBinding;
use crate::ui::template_runtime::builtin::builtin_template_bindings;
use crate::ui::workbench::event::{editor_operation_binding, menu_action_binding};

use super::componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;

#[derive(serde::Deserialize)]
struct CommandMenuContract {
    menus: std::collections::BTreeMap<String, Vec<String>>,
    triggers: std::collections::BTreeMap<String, String>,
}

fn command_menu_contract() -> &'static CommandMenuContract {
    static CONTRACT: std::sync::OnceLock<CommandMenuContract> = std::sync::OnceLock::new();
    CONTRACT.get_or_init(|| toml::from_str(include_str!(
        "../../../../../../assets/ui/editor/unreal/menus.contract.toml"
    )).expect("built-in command menu contract must be valid"))
}

fn find_menu_command<'a>(
    items: &'a [crate::core::commands::MenuItemModel],
    id: &str,
) -> Option<&'a crate::core::commands::MenuItemModel> {
    items.iter().find_map(|item| {
        if item.operation_path.as_ref().is_some_and(|path| path.as_str() == id) {
            Some(item)
        } else {
            find_menu_command(&item.children, id)
        }
    })
}

const MAIN_MENU_CONTROL_ID: &str = "WorkbenchToolbarMainMenu";

impl BuiltinWorkbenchWindowTemplateSurfaceBridge {
    pub(super) fn apply_command_menu_state(
        &mut self,
        model: &crate::ui::workbench::model::WorkbenchViewModel,
    ) -> Result<(), super::error::BuiltinHostWindowTemplateBridgeError> {
        self.apply_command_menu_bar(&model.menu_bar)
    }

    fn apply_command_menu_bar(
        &mut self,
        menu_bar: &crate::core::commands::MenuBarModel,
    ) -> Result<(), super::error::BuiltinHostWindowTemplateBridgeError> {
        use zircon_runtime_interface::ui::component::UiValue;
        for (control, commands) in &command_menu_contract().menus {
            if !self.has_control(control) { continue; }
            let items: Vec<UiValue> = commands.iter().filter_map(|id| {
                let item = menu_bar.menus.iter()
                    .find_map(|menu| find_menu_command(&menu.items, id))?;
                // The popup encoding reserves these separators.
                let label = item.label.replace(['|', '\n', '\r'], " ");
                let disabled = if item.enabled { "" } else { ",disabled" };
                Some(UiValue::String(format!("{label}|action={id}{disabled}|{}",
                    item.shortcut.as_deref().unwrap_or_default())))
            }).collect();
            if let Some(trigger) = command_menu_contract().triggers.get(control) {
                self.set_visible(trigger, !items.is_empty())?;
            }
            self.mutate_control_property(control, "menu_items", UiValue::Array(items))?;
        }
        Ok(())
    }

    pub(crate) fn main_menu_item_binding(
        &self,
        menu_control_id: &str,
        action_id: &str,
    ) -> Option<EditorUiBinding> {
        if menu_control_id == MAIN_MENU_CONTROL_ID {
            return self.toolbar_main_menu_item_binding(action_id);
        }
        let contract = command_menu_contract();
        let commands = contract.menus.get(menu_control_id)?;
        if !commands.iter().any(|command| command == action_id)
            || self.popup_menu_item_is_selectable(menu_control_id, action_id) != Some(true)
        {
            return None;
        }
        Some(editor_operation_binding(&EditorOperationPath::parse(action_id).ok()?))
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
