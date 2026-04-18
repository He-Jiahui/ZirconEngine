use super::menu_model::MenuModel;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuBarModel {
    pub menus: Vec<MenuModel>,
}
