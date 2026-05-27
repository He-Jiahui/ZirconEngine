use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus, binding::UiBindingUpdateStatus,
    dispatch::UiInputDispatchResult, event_ui::UiNodeId, tree::UiTreeError,
};

use crate::ui::surface::UiSurface;

use super::super::result::{finish_handled, finish_unhandled};
use super::binding::{append_scroll_binding_report, scroll_state_offset};

pub(super) fn finish_scroll_to_mutation(
    surface: &UiSurface,
    target: UiNodeId,
    requested_offset: f32,
    previous_offset: f32,
    result: UiInputDispatchResult,
    report: Result<bool, UiTreeError>,
) -> UiInputDispatchResult {
    match report {
        Ok(true) => {
            finish_accepted_scroll_to(surface, target, requested_offset, previous_offset, result)
        }
        Ok(false) => finish_unchanged_scroll_to(surface, target, previous_offset, result),
        Err(error) => finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "mutation_error",
            &format!("scroll to action failed: {error}"),
        ),
    }
}

fn finish_accepted_scroll_to(
    surface: &UiSurface,
    target: UiNodeId,
    requested_offset: f32,
    previous_offset: f32,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.scroll_to");
    let next_offset = scroll_state_offset(surface, target).unwrap_or(requested_offset);
    append_scroll_binding_report(
        surface,
        &mut result,
        target,
        previous_offset,
        next_offset,
        UiBindingUpdateStatus::Applied,
    );
    result
}

fn finish_unchanged_scroll_to(
    surface: &UiSurface,
    target: UiNodeId,
    previous_offset: f32,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.scroll_to");
    result
        .diagnostics
        .notes
        .push("accessibility_scroll_unchanged".to_string());
    append_scroll_binding_report(
        surface,
        &mut result,
        target,
        previous_offset,
        previous_offset,
        UiBindingUpdateStatus::Unchanged,
    );
    result
}
