use zircon_editor_ui::EditorUiBinding;

use super::menu_action::MenuAction;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemModel {
    pub label: String,
    pub action: MenuAction,
    pub binding: EditorUiBinding,
    pub shortcut: Option<String>,
    pub enabled: bool,
}
