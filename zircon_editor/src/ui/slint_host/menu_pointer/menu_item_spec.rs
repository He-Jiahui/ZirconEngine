#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MenuItemSpec {
    pub action_id: Option<String>,
    pub enabled: bool,
}
