use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityActionStatus, UiAccessibilityNode},
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use crate::ui::surface::UiSurface;

use super::result::{finish_handled, finish_unhandled, unsupported_role_action};

pub(super) fn dispatch_focus(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::Focus)
    {
        return unsupported_role_action(result, target, "target does not expose focus action");
    }

    match surface.focus_node_with_reason(
        target,
        UiFocusChangeReason::Programmatic,
        UiFocusVisible::visible(UiFocusVisibleReason::Programmatic),
    ) {
        Ok(_) => finish_handled(result, target, "accessibility.focus"),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "focus_rejected",
            &format!("focus target rejected by runtime focus API: {error}"),
        ),
    }
}
