use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityNode},
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use self::result::{
    finish_activate_fallback, finish_activate_widget_error, finish_activate_widget_report,
};
use super::result::unsupported_role_action;

mod fallback;
mod result;

pub(super) fn dispatch_activate(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Activate)
    {
        return unsupported_role_action(result, target, "target does not expose activate action");
    }

    match surface.apply_default_keyboard_component_action(target) {
        Ok(report) if report.handled => {
            return finish_activate_widget_report(
                target,
                result,
                report.component_events,
                report.binding_reports,
            )
        }
        Ok(_) => {}
        Err(error) => return finish_activate_widget_error(target, result, error),
    }

    finish_activate_fallback(target, result)
}
