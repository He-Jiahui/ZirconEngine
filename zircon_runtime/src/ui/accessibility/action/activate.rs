use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityActionStatus, UiAccessibilityNode},
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use self::fallback::default_activate_commit_event;
use super::result::{finish_handled, finish_unhandled, unsupported_role_action};

mod fallback;

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
            let mut result = finish_handled(result, target, "accessibility.activate");
            result.component_events.extend(report.component_events);
            result.binding_reports.extend(report.binding_reports);
            return result;
        }
        Ok(_) => {}
        Err(error) => {
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                "mutation_error",
                &format!("activate widget action failed: {error}"),
            );
        }
    }

    let mut result = finish_handled(result, target, "accessibility.activate");
    result
        .component_events
        .push(default_activate_commit_event(target));
    result
}
