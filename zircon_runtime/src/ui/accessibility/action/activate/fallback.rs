use zircon_runtime_interface::ui::{
    component::{UiComponentEvent, UiValue},
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
};

// 无组件默认处理器时向下游提供一次语义激活通知；此事件本身不修改节点的 retained 属性。
pub(super) fn default_activate_commit_event(target: UiNodeId) -> UiComponentEventReport {
    UiComponentEventReport {
        target,
        event: UiComponentEvent::Commit {
            property: "activated".to_string(),
            value: UiValue::Bool(true),
        },
        delivered: true,
        drag: None,
        template_action: None,
    }
}
