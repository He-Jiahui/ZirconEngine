use std::cmp::Reverse;

use crate::hybrid_gi::HybridGiPrepareUpdateRequest;

use super::super::HybridGiRuntimeState;

const TRACE_SUPPORT_SORT_SCALE: f32 = 1024.0;

pub(super) fn collect_pending_updates(
    runtime: &HybridGiRuntimeState,
) -> Vec<HybridGiPrepareUpdateRequest> {
    let mut pending_updates = runtime
        .pending_update_requests()
        .iter()
        .filter(|update| !has_pending_ancestor_update(runtime, update.probe_id()))
        .cloned()
        .collect::<Vec<_>>();
    pending_updates.sort_by_cached_key(|update| {
        (
            Reverse(lineage_trace_support_sort_key(runtime, update.probe_id())),
            Reverse(resident_descendant_count(runtime, update.probe_id())),
            Reverse(descendant_count(runtime, update.probe_id())),
            probe_depth(runtime, update.probe_id()),
            update.generation(),
            update.probe_id(),
        )
    });

    pending_updates
        .into_iter()
        .map(|update| HybridGiPrepareUpdateRequest {
            probe_id: update.probe_id(),
            ray_budget: update.ray_budget(),
            generation: update.generation(),
        })
        .collect::<Vec<_>>()
}

fn has_pending_ancestor_update(runtime: &HybridGiRuntimeState, probe_id: u32) -> bool {
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = std::collections::BTreeSet::from([probe_id]);

    while let Some(parent_probe_id) = runtime
        .probe_parent_probes()
        .get(&current_probe_id)
        .copied()
    {
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        if runtime.has_pending_probe(parent_probe_id) {
            return true;
        }
        current_probe_id = parent_probe_id;
    }

    false
}

fn resident_descendant_count(runtime: &HybridGiRuntimeState, probe_id: u32) -> usize {
    runtime
        .probe_descendant_ids(probe_id)
        .into_iter()
        .filter(|descendant_probe_id| runtime.has_resident_probe(*descendant_probe_id))
        .count()
}

fn descendant_count(runtime: &HybridGiRuntimeState, probe_id: u32) -> usize {
    runtime.probe_descendant_ids(probe_id).len()
}

fn probe_depth(runtime: &HybridGiRuntimeState, probe_id: u32) -> usize {
    let mut depth = 0usize;
    let mut current_probe_id = probe_id;
    let mut visited_probe_ids = std::collections::BTreeSet::from([probe_id]);

    while let Some(parent_probe_id) = runtime
        .probe_parent_probes()
        .get(&current_probe_id)
        .copied()
    {
        if !visited_probe_ids.insert(parent_probe_id) {
            break;
        }
        depth += 1;
        current_probe_id = parent_probe_id;
    }

    depth
}

fn lineage_trace_support_sort_key(runtime: &HybridGiRuntimeState, probe_id: u32) -> u32 {
    (lineage_trace_support_score(runtime, probe_id) * TRACE_SUPPORT_SORT_SCALE).round() as u32
}

fn lineage_trace_support_score(runtime: &HybridGiRuntimeState, probe_id: u32) -> f32 {
    runtime.effective_lineage_trace_support_score(probe_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hybrid_gi::HybridGiProbeUpdateRequest;

    #[test]
    fn cached_pending_update_sort_preserves_priority_order() {
        let mut runtime = HybridGiRuntimeState::default();
        runtime.push_pending_update_request(HybridGiProbeUpdateRequest::new(30, 3, 2));
        runtime.push_pending_update_request(HybridGiProbeUpdateRequest::new(20, 2, 1));
        runtime.push_pending_update_request(HybridGiProbeUpdateRequest::new(10, 1, 1));

        let pending_updates = collect_pending_updates(&runtime);
        let projected = pending_updates
            .iter()
            .map(|update| (update.probe_id, update.ray_budget, update.generation))
            .collect::<Vec<_>>();

        assert_eq!(projected, vec![(10, 1, 1), (20, 2, 1), (30, 3, 2)]);
    }
}
