use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi) fn evict_one(
        &mut self,
        probe_ids: impl IntoIterator<Item = u32>,
    ) -> bool {
        for probe_id in probe_ids {
            if let Some(slot) = self.remove_resident_probe_slot(probe_id) {
                self.insert_free_slot(slot);
                self.remove_evictable_probe(probe_id);
                return true;
            }
        }
        false
    }
}
