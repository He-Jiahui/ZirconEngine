use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityNode},
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
    focus::{UiFocusChangeReason, UiFocusVisible, UiFocusVisibleReason},
};

use crate::ui::surface::UiSurface;

use self::result::{finish_focus_rejected, finish_focus_success};
use super::result::unsupported_role_action;

mod result;

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
        Ok(_) => finish_focus_success(result, target),
        Err(error) => finish_focus_rejected(result, target, error),
    }
}
