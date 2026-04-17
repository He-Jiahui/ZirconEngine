use super::super::pane_empty_state_model::PaneEmptyStateModel;

pub(super) fn placeholder_empty_state() -> PaneEmptyStateModel {
    PaneEmptyStateModel {
        title: "View unavailable".to_string(),
        body: "This pane was restored from layout state but its descriptor is missing.".to_string(),
        primary_action: None,
        secondary_action: None,
        secondary_hint: None,
    }
}
