#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MenuItemSpec {
    pub action_id: Option<String>,
    pub enabled: bool,
    pub children: Vec<MenuItemSpec>,
}

impl MenuItemSpec {
    pub(in crate::ui::slint_host::menu_pointer) fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}
