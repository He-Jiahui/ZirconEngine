use super::menu_item_model::MenuItemModel;

#[derive(Clone, Debug, PartialEq)]
pub struct MenuModel {
    pub label: String,
    pub items: Vec<MenuItemModel>,
}
