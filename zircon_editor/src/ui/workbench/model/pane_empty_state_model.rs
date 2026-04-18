use super::pane_action_model::PaneActionModel;

#[derive(Clone, Debug, PartialEq)]
pub struct PaneEmptyStateModel {
    pub title: String,
    pub body: String,
    pub primary_action: Option<PaneActionModel>,
    pub secondary_action: Option<PaneActionModel>,
    pub secondary_hint: Option<String>,
}
