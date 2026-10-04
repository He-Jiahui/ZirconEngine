use super::super::pane_empty_state_model::PaneEmptyStateModel;

/// 恢复布局缺少描述符时保留占位说明，避免把残留标签当作可用视图。
pub(super) fn placeholder_empty_state() -> PaneEmptyStateModel {
    PaneEmptyStateModel {
        title: "View unavailable".to_string(),
        body: "This pane was restored from layout state but its descriptor is missing.".to_string(),
        primary_action: None,
        secondary_action: None,
        secondary_hint: None,
    }
}
