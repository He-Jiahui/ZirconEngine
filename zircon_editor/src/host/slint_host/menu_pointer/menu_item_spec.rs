#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::host::slint_host::menu_pointer) struct MenuItemSpec {
    pub action_id: Option<String>,
    pub enabled: bool,
}
