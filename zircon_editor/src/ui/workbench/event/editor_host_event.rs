use crate::core::editor_event::MenuAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditorHostEvent {
    Menu(MenuAction),
}
