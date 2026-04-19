use super::super::HybridGiRuntimeState;

pub(super) fn complete_pending_probes(
    runtime: &mut HybridGiRuntimeState,
    probe_ids: impl IntoIterator<Item = u32>,
    evictable_probe_ids: &[u32],
) {
    if runtime.probe_budget == 0 {
        return;
    }

    let mut requested_probe_ids = Vec::new();
    let mut seen_probe_ids = std::collections::BTreeSet::new();
    for probe_id in probe_ids {
        if !runtime.pending_probes.contains(&probe_id) || !seen_probe_ids.insert(probe_id) {
            continue;
        }
        requested_probe_ids.push(probe_id);
        if requested_probe_ids.len() >= runtime.probe_budget {
            break;
        }
    }

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
        runtime.current_requested_probe_ids.remove(&probe_id);
    }

    runtime
        .evictable_probes
        .retain(|candidate| runtime.resident_slots.contains_key(candidate));
}
