use zircon_runtime_interface::ui::{
    accessibility::{
        UiA11yRole, UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode,
    },
    binding::UiBindingSourceKind,
    dispatch::UiInputDispatchResult,
};

use crate::ui::surface::UiSurface;

use super::result::unsupported_role_action;

use self::adjustment::adjustment_direction;
use self::result::finish_adjust_value_mutation;

mod adjustment;
mod result;

pub(super) fn dispatch_adjust_value(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let target = request.target;
    if !snapshot_node.actions.contains(&request.action) {
        return unsupported_role_action(result, target, "target does not expose adjust action");
    }
    if snapshot_node.role != UiA11yRole::Slider {
        return unsupported_role_action(result, target, "adjust value requires slider role");
    }
    let direction = adjustment_direction(request.action);
    let report = surface.mutate_default_range_step_value_with_source_kind(
        target,
        direction,
        UiBindingSourceKind::AccessibilityAction,
    );
    finish_adjust_value_mutation(target, result, report)
}
