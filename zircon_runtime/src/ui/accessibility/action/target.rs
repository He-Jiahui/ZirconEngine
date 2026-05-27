use zircon_runtime_interface::ui::{
    accessibility::{
        UiAccessibilityAction, UiAccessibilityActionStatus, UiAccessibilityNode,
        UiAccessibilityTreeSnapshot,
    },
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use super::result::finish_unhandled;

pub(super) fn validate_included_target(
    snapshot: &UiAccessibilityTreeSnapshot,
    target: UiNodeId,
    action: UiAccessibilityAction,
    snapshot_node: &UiAccessibilityNode,
    mut result: UiInputDispatchResult,
) -> Result<UiInputDispatchResult, UiInputDispatchResult> {
    append_target_diagnostics(snapshot, target, &mut result);
    if snapshot_node.state.hidden {
        return Err(finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "hidden_target",
            "target is hidden in the accessibility snapshot",
        ));
    }
    if snapshot_node.state.disabled && action != UiAccessibilityAction::Focus {
        return Err(finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "disabled_action",
            "disabled accessibility target rejected non-focus action",
        ));
    }

    Ok(result)
}

pub(super) fn reject_missing_target(
    surface: &UiSurface,
    snapshot: &UiAccessibilityTreeSnapshot,
    target: UiNodeId,
    mut result: UiInputDispatchResult,
) -> UiInputDispatchResult {
    if !surface.tree.nodes.contains_key(&target) {
        return finish_unhandled(
            result,
            None,
            UiAccessibilityActionStatus::StaleTarget,
            "stale_target",
            "target is not in the runtime UI tree",
        );
    }

    append_target_diagnostics(snapshot, target, &mut result);
    if is_effectively_hidden(surface, target) {
        return finish_unhandled(
            result,
            Some(target),
            UiAccessibilityActionStatus::Rejected,
            "hidden_target",
            "target is hidden and excluded from accessibility action dispatch",
        );
    }

    finish_unhandled(
        result,
        Some(target),
        UiAccessibilityActionStatus::Rejected,
        "excluded_target",
        "target is not included in the current accessibility snapshot",
    )
}

pub(super) fn append_target_diagnostics(
    snapshot: &UiAccessibilityTreeSnapshot,
    target: UiNodeId,
    result: &mut UiInputDispatchResult,
) {
    result.diagnostics.notes.extend(
        snapshot
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.node_id == Some(target))
            .map(|diagnostic| format!("accessibility_diagnostic={:?}", diagnostic.code)),
    );
}

fn is_effectively_hidden(surface: &UiSurface, target: UiNodeId) -> bool {
    let mut current = Some(target);
    while let Some(node_id) = current {
        let Some(node) = surface.tree.nodes.get(&node_id) else {
            return false;
        };
        if !node.is_render_visible() {
            return true;
        }
        current = node.parent;
    }
    false
}
