use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi) fn evict_one(
        &mut self,
        probe_ids: impl IntoIterator<Item = u32>,
    ) -> bool {
        for probe_id in probe_ids {
            if let Some(slot) = self.resident_slots.remove(&probe_id) {
                self.free_slots.insert(slot);
                self.evictable_probes
                    .retain(|candidate| *candidate != probe_id);
                return true;
            }
        }
        false
    }
}
