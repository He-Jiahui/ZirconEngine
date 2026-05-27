use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityAction,
    component::{UiComponentEvent, UiValue},
    dispatch::UiComponentEventReport,
    event_ui::UiNodeId,
};

pub(super) fn adjustment_direction(action: UiAccessibilityAction) -> f64 {
    match action {
        UiAccessibilityAction::Increment => 1.0,
        UiAccessibilityAction::Decrement => -1.0,
        _ => unreachable!("adjustment_direction only handles increment/decrement"),
    }
}

pub(super) fn value_changed_event(
    target: UiNodeId,
    property: String,
    value: f64,
) -> UiComponentEventReport {
    UiComponentEventReport {
        target,
        event: UiComponentEvent::ValueChanged {
            property,
            value: UiValue::Float(value),
        },
        delivered: true,
        drag: None,
    }
}
