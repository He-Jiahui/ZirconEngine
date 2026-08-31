use std::collections::HashSet;

use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(crate) fn apply_gpu_cache_entries(&mut self, cache_entries: &[(u32, u32)]) {
        let mut unique_cache_entries = Vec::with_capacity(cache_entries.len());
        let mut gpu_resident_probe_ids = HashSet::with_capacity(cache_entries.len());
        for (probe_id, slot) in cache_entries {
            if !self.has_live_gpu_feedback_probe(*probe_id) {
                continue;
            }

            if !gpu_resident_probe_ids.insert(*probe_id) {
                continue;
            }
            unique_cache_entries.push((*probe_id, *slot));
        }

        let resident_probe_ids = self.resident_probe_ids().collect::<Vec<_>>();
        for probe_id in resident_probe_ids {
            if !gpu_resident_probe_ids.contains(&probe_id) {
                self.evict_one([probe_id]);
            }
        }

        for (probe_id, slot) in unique_cache_entries {
            self.promote_to_resident_in_slot(probe_id, slot);
        }

        self.retain_resident_evictable_probes();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zircon_runtime::core::math::Vec3;

    #[test]
    fn runtime98_gpu_cache_membership_preserves_first_slot_and_evicts_absent_residents() {
        let mut state = HybridGiRuntimeState::default();
        state.seed_runtime_probe_scene_data_for_test([
            (1, Vec3::ZERO, 1.0, None, 64),
            (2, Vec3::ZERO, 1.0, None, 64),
            (3, Vec3::ZERO, 1.0, None, 64),
            (4, Vec3::ZERO, 1.0, None, 64),
        ]);
        state.insert_resident_probe_slot(1, 1);

        state.apply_gpu_cache_entries(&[(2, 7), (2, 9), (3, 8), (4, 8), (99, 11)]);

        assert_eq!(state.probe_slot(1), None);
        assert_eq!(state.probe_slot(2), Some(7));
        assert_eq!(state.probe_slot(3), None);
        assert_eq!(state.probe_slot(4), Some(8));
        assert_eq!(state.probe_slot(99), None);
    }
}
