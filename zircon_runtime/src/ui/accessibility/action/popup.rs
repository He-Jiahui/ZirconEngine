use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityActionStatus, UiAccessibilityNode},
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::{UiNodeId, UiReflectedPropertySource},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiSurface};

use super::result::{finish_unhandled, finish_unhandled_with_note};

use self::result::finish_popup_dismiss_mutation;
use self::tooltip::{dispatch_tooltip_dismiss, tooltip_dismissal_target};

mod result;
mod tooltip;

pub(super) fn dispatch_dismiss(
    surface: &mut UiSurface,
    target: UiNodeId,
    snapshot_node: &UiAccessibilityNode,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let dismiss_target = match surface.default_popup_dismissal_target(target) {
        Ok(Some(target)) => target,
        Ok(None) => {
            if let Some(tooltip_id) = tooltip_dismissal_target(surface, snapshot_node) {
                return dispatch_tooltip_dismiss(surface, target, tooltip_id, result);
            }
            return unsupported_dismiss(result, target);
        }
        Err(error) => {
            return finish_unhandled(
                result,
                Some(target),
                UiAccessibilityActionStatus::Rejected,
                "mutation_error",
                &format!("dismiss popup lookup failed: {error}"),
            );
        }
    };
    let (popup_id, property) = dismiss_target;
    let report = surface.mutate_property(
        UiPropertyMutationRequest::accessibility_action(
            popup_id,
            property.clone(),
            UiValue::Bool(false),
        )
        .with_source(UiReflectedPropertySource::RuntimeState),
    );
    finish_popup_dismiss_mutation(popup_id, result, report)
}

fn unsupported_dismiss(result: UiInputDispatchResult, target: UiNodeId) -> UiInputDispatchResult {
    finish_unhandled_with_note(
        result,
        Some(target),
        UiAccessibilityActionStatus::Unsupported,
        "unsupported_role_action",
        "accessibility dismiss requires popup id",
        "accessibility dismiss requires popup id",
    )
}
