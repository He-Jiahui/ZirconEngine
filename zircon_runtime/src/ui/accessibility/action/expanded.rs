use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityActionRequest, UiAccessibilityNode},
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::UiReflectedPropertySource,
};

use crate::ui::surface::{UiPropertyMutationRequest, UiSurface};

use super::result::unsupported_role_action;

use self::result::finish_expanded_mutation;
use self::target::{expandable_action_target, expanded_component_event};

mod result;
mod target;

pub(super) fn dispatch_expanded_state(
    surface: &mut UiSurface,
    request: &UiAccessibilityActionRequest,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
    expanded: bool,
) -> UiInputDispatchResult {
    let target = request.target;
    let action = if expanded {
        UiAccessibilityAction::Expand
    } else {
        UiAccessibilityAction::Collapse
    };
    if !snapshot_node.actions.contains(&action) {
        return unsupported_role_action(
            result,
            target,
            if expanded {
                "target does not expose expand action"
            } else {
                "target does not expose collapse action"
            },
        );
    }
    if snapshot_node.state.expanded.is_none() {
        return unsupported_role_action(result, target, "target has no expandable state");
    }
    let Some(expandable) = expandable_action_target(surface, target) else {
        return unsupported_role_action(
            result,
            target,
            "expand/collapse requires disclosure or popup widget behavior",
        );
    };
    let phase = if expanded {
        "accessibility.expand"
    } else {
        "accessibility.collapse"
    };

    let report = surface.mutate_property(
        UiPropertyMutationRequest::accessibility_action(
            target,
            expandable.property.clone(),
            UiValue::Bool(expanded),
        )
        .with_source(UiReflectedPropertySource::RuntimeState),
    );
    finish_expanded_mutation(
        target,
        phase,
        expanded_component_event(expandable.kind, expanded),
        result,
        report,
    )
}
