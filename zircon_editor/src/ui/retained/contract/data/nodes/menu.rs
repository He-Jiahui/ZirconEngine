use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneMenuItemData {
    pub raw: SharedString,
    pub action_id: SharedString,
    pub label: SharedString,
    pub shortcut: SharedString,
    pub checked: bool,
    pub disabled: bool,
    pub separator: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub loading: bool,
}
