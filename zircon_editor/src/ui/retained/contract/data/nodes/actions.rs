use crate::ui::retained_host::primitives::SharedString;

#[derive(Clone, Default)]
pub(crate) struct TemplatePaneActionData {
    pub label: SharedString,
    pub action_id: SharedString,
}
