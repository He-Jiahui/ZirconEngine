use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneOptionData {
    pub id: SharedString,
    pub label: SharedString,
    pub description: SharedString,
    pub tone: SharedString,
    pub selected: bool,
    pub disabled: bool,
    pub special: bool,
    pub unread: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
    pub loading: bool,
    pub matched: bool,
}
