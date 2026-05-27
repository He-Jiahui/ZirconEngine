use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
};

pub(super) fn default_activate_commit_event(target: UiNodeId) -> UiComponentEventReport {
    UiComponentEventReport {
        target,
        event: UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        delivered: true,
        drag: None,
    }
}
