use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityNode,
    component::UiValue,
    dispatch::UiInputDispatchResult,
    event_ui::{UiNodeId, UiReflectedPropertySource},
};

use crate::ui::surface::{UiPropertyMutationRequest, UiSurface};

use self::result::{
    finish_popup_dismiss_lookup_error, finish_popup_dismiss_mutation, finish_unsupported_dismiss,
};
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
            return finish_unsupported_dismiss(result, target);
        }
        Err(error) => return finish_popup_dismiss_lookup_error(result, target, error),
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
