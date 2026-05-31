use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus, dispatch::UiInputDispatchResult,
    event_ui::UiNodeId, tree::UiTreeError,
};

use super::super::result::{finish_handled, finish_unhandled};

pub(super) fn finish_focus_success(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_handled(result, target, "accessibility.focus")
}

pub(super) fn finish_focus_rejected(
    result: UiInputDispatchResult,
    target: UiNodeId,
    error: UiTreeError,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "focus_rejected",
        &format!("focus target rejected by runtime focus API: {error}"),
    )
}
