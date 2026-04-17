use super::super::hybrid_gi_runtime_state::HybridGiRuntimeState;

pub(super) fn complete_pending_probes(
    runtime: &mut HybridGiRuntimeState,
    probe_ids: impl IntoIterator<Item = u32>,
    evictable_probe_ids: &[u32],
) {
    if runtime.probe_budget == 0 {
        return;
    }

    let requested_probe_ids = probe_ids
        .into_iter()
        .filter(|probe_id| runtime.pending_probes.contains(probe_id))
        .take(runtime.probe_budget)
        .collect::<Vec<_>>();

    for probe_id in requested_probe_ids {
        while runtime.resident_slots.len() >= runtime.probe_budget {
            if !runtime.evict_one(evictable_probe_ids.iter().copied()) {
                runtime
                    .evictable_probes
                    .retain(|candidate| runtime.resident_slots.contains_key(candidate));
                return;
            }
        }

        runtime.promote_to_resident(probe_id);
    }

    runtime
        .evictable_probes
        .retain(|candidate| runtime.resident_slots.contains_key(candidate));
}
