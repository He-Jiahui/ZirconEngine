use zircon_runtime_interface::ui::{
    accessibility::{UiAccessibilityAction, UiAccessibilityNode, UiAccessibilityTreeSnapshot},
    dispatch::UiInputDispatchResult,
    event_ui::UiNodeId,
};

use crate::ui::surface::UiSurface;

use self::result::{
    reject_disabled_action, reject_excluded_target, reject_hidden_snapshot_target,
    reject_hidden_tree_target, reject_stale_target,
};

mod result;

pub(super) fn validate_included_target(
    snapshot: &UiAccessibilityTreeSnapshot,
    target: UiNodeId,
    action: UiAccessibilityAction,
    snapshot_node: &UiAccessibilityNode,
    mut result: UiInputDispatchResult,
) -> Result<UiInputDispatchResult, UiInputDispatchResult> {
    append_target_diagnostics(snapshot, target, &mut result);
    if snapshot_node.state.hidden {
        return Err(reject_hidden_snapshot_target(result, target));
    }
    if snapshot_node.state.disabled && action != UiAccessibilityAction::Focus {
        return Err(reject_disabled_action(result, target));
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
        return reject_stale_target(result);
    }

    append_target_diagnostics(snapshot, target, &mut result);
    if is_effectively_hidden(surface, target) {
        return reject_hidden_tree_target(result, target);
    }

    reject_excluded_target(result, target)
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
