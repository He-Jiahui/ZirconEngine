use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus,
    binding::UiBindingUpdateReport,
    dispatch::{UiComponentEventReport, UiInputDispatchResult},
    event_ui::UiNodeId,
    tree::UiTreeError,
};

use super::super::result::{finish_handled, finish_unhandled};
use super::fallback::default_activate_commit_event;

pub(super) fn finish_activate_widget_report(
    target: UiNodeId,
    result: UiInputDispatchResult,
    component_events: Vec<UiComponentEventReport>,
    binding_reports: Vec<UiBindingUpdateReport>,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.activate");
    result.component_events.extend(component_events);
    result.binding_reports.extend(binding_reports);
    result
}

pub(super) fn finish_activate_widget_error(
    target: UiNodeId,
    result: UiInputDispatchResult,
    error: UiTreeError,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "mutation_error",
        &format!("activate widget action failed: {error}"),
    )
}

pub(super) fn finish_activate_fallback(
    target: UiNodeId,
    result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    let mut result = finish_handled(result, target, "accessibility.activate");
    result
        .component_events
        .push(default_activate_commit_event(target));
    result
}
