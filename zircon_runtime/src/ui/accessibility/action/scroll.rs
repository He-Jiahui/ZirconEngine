use zircon_runtime_interface::ui::{
    accessibility::{
        UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityActionStatus,
        UiAccessibilityNode,
    },
    dispatch::UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;
use crate::ui::tree::UiRuntimeTreeScrollExt;

use super::result::{finish_unhandled, unsupported_role_action};

use self::binding::scroll_state_offset;
use self::payload::scroll_to_offset;
use self::result::finish_scroll_to_mutation;

mod binding;
mod payload;
mod result;

pub(super) fn dispatch_scroll_to(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node
        .actions
        .contains(&UiAccessibilityAction::ScrollTo)
    {
        return unsupported_role_action(result, target, "target does not expose scroll action");
    }
    let Some(offset) = scroll_to_offset(surface, target, request) else {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "missing_value",
            "scroll to action requires value or numeric_value",
        );
    };
    let previous_offset = scroll_state_offset(surface, target).unwrap_or_default();
    let report = surface.tree.set_scroll_offset(target, offset as f32);

    finish_scroll_to_mutation(
        surface,
        target,
        offset as f32,
        previous_offset,
        result,
        report,
    )
}
