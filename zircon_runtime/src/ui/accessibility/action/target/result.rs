use zircon_runtime_interface::ui::{
    accessibility::UiAccessibilityActionStatus, dispatch::UiInputDispatchResult, event_ui::UiNodeId,
};

use super::super::result::finish_unhandled;

pub(super) fn reject_hidden_snapshot_target(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "hidden_target",
        "target is hidden in the accessibility snapshot",
    )
}

pub(super) fn reject_disabled_action(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "disabled_action",
        "disabled accessibility target rejected non-focus action",
    )
}

pub(super) fn reject_stale_target(result: UiInputDispatchResult) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        None,
        UiAccessibilityActionStatus::StaleTarget,
        "stale_target",
        "target is not in the runtime UI tree",
    )
}

pub(super) fn reject_hidden_tree_target(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "hidden_target",
        "target is hidden and excluded from accessibility action dispatch",
    )
}

pub(super) fn reject_excluded_target(
    result: UiInputDispatchResult,
    target: UiNodeId,
) -> UiInputDispatchResult {
    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "excluded_target",
        "target is not included in the current accessibility snapshot",
    )
}
