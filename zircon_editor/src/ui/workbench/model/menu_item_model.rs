use crate::core::editor_event::MenuAction;
use crate::ui::binding::EditorUiBinding;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItemModel {
    pub label: String,
    pub action: MenuAction,
    pub binding: EditorUiBinding,
    pub shortcut: Option<String>,
    pub enabled: bool,
}
