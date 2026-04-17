use std::collections::BTreeSet;

use super::super::hybrid_gi_runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn apply_gpu_cache_entries(&mut self, cache_entries: &[(u32, u32)]) {
        let resident_probe_ids = self.resident_slots.keys().copied().collect::<Vec<_>>();
        let gpu_resident_probes = cache_entries
            .iter()
            .map(|(probe_id, _)| *probe_id)
            .collect::<BTreeSet<_>>();

        for probe_id in resident_probe_ids {
            if !gpu_resident_probes.contains(&probe_id) {
                self.evict_one([probe_id]);
            }
        }

        for (probe_id, slot) in cache_entries {
            self.promote_to_resident_in_slot(*probe_id, *slot);
        }

        self.evictable_probes
            .retain(|probe_id| self.resident_slots.contains_key(probe_id));
    }
}
