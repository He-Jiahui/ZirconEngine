use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi) fn promote_to_resident_in_slot(&mut self, probe_id: u32, slot: u32) {
        self.clear_pending_update(probe_id);
        if let Some(previous_slot) = self.remove_resident_probe_slot(probe_id) {
            if previous_slot != slot {
                self.insert_free_slot(previous_slot);
            }
        }
        if let Some(conflicting_probe) = self.probe_in_slot(slot) {
            if conflicting_probe != probe_id {
                self.evict_one([conflicting_probe]);
            }
        }
        self.reserve_slot(slot);
        self.insert_resident_probe_slot(probe_id, slot);
    }
}
