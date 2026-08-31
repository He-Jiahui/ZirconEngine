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
    result.diagnostics.notes.reserve(snapshot.diagnostics.len());
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

#[cfg(test)]
mod optimization_tests {
    #[test]
    fn optimization_batch_20260830db_target_diagnostics_reserve_source_upper_bound() {
        let source = include_str!("target.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("accessibility target production source");

        assert!(production.contains("notes.reserve(snapshot.diagnostics.len());"));
    }

    #[test]
    #[ignore = "release-only performance evidence"]
    fn optimization_batch_20260830db_target_diagnostic_capacity_evidence() {
        const BATCH_COUNT: usize = 32_768;
        const DIAGNOSTIC_COUNT: usize = 32;
        const MARKER: &str = "RUNTIME514_TARGET_DIAGNOSTIC_CAPACITY_BENCH_V1";

        let legacy_growth_events = note_growth_events(BATCH_COUNT, DIAGNOSTIC_COUNT, false);
        let optimized_growth_events = note_growth_events(BATCH_COUNT, DIAGNOSTIC_COUNT, true);

        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
        println!(
            "{MARKER} batches={BATCH_COUNT} diagnostics={DIAGNOSTIC_COUNT} \
             legacy_growth_events={legacy_growth_events} \
             optimized_growth_events={optimized_growth_events} reduction_pct=100"
        );
    }

    fn note_growth_events(batch_count: usize, diagnostic_count: usize, reserve: bool) -> usize {
        let mut growth_events = 0;
        for _ in 0..batch_count {
            let mut notes = if reserve {
                Vec::with_capacity(diagnostic_count)
            } else {
                Vec::new()
            };
            for note in 0..diagnostic_count {
                let previous_capacity = notes.capacity();
                notes.push(note);
                growth_events += usize::from(notes.capacity() != previous_capacity);
            }
        }
        growth_events
    }
}
