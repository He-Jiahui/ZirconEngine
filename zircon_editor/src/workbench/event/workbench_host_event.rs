use super::super::model::MenuAction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkbenchHostEvent {
    Menu(MenuAction),
}
