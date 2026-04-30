use std::collections::BTreeSet;

use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn apply_gpu_cache_entries(&mut self, cache_entries: &[(u32, u32)]) {
        let mut unique_cache_entries = Vec::new();
        let mut seen_probe_ids = BTreeSet::new();
        for (probe_id, slot) in cache_entries {
            if !self.probe_scene_data().contains_key(probe_id) {
                continue;
            }

            if !seen_probe_ids.insert(*probe_id) {
                continue;
            }
            unique_cache_entries.push((*probe_id, *slot));
        }

        let resident_probe_ids = self.resident_probe_ids().collect::<Vec<_>>();
        let gpu_resident_probes = unique_cache_entries
            .iter()
            .map(|(probe_id, _)| *probe_id)
            .collect::<BTreeSet<_>>();

        for probe_id in resident_probe_ids {
            if !gpu_resident_probes.contains(&probe_id) {
                self.evict_one([probe_id]);
            }
        }

        for (probe_id, slot) in unique_cache_entries {
            self.promote_to_resident_in_slot(probe_id, slot);
        }

        self.retain_resident_evictable_probes();
    }
}
