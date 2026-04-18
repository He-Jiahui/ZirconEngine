use crate::view::ViewDescriptorId;
use crate::ui::workbench::event::menu_action_binding;

use super::super::menu_action::MenuAction;
use super::super::menu_item_model::MenuItemModel;
use super::super::menu_model::MenuModel;

pub(super) fn build_help_menu() -> MenuModel {
    MenuModel {
        label: "Help".to_string(),
        items: vec![MenuItemModel {
            label: "Workbench Guide".to_string(),
            action: MenuAction::OpenView(ViewDescriptorId::new("editor.asset_browser")),
            binding: menu_action_binding(&MenuAction::OpenView(ViewDescriptorId::new(
                "editor.asset_browser",
            ))),
            shortcut: None,
            enabled: true,
        }],
    }
}
