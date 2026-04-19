use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi) fn promote_to_resident(&mut self, probe_id: u32) {
        if self.resident_slots.contains_key(&probe_id) {
            return;
        }

        self.clear_pending_update(probe_id);

        let slot = self.take_free_slot().unwrap_or_else(|| {
            let slot = self.next_slot;
            self.next_slot += 1;
            slot
        });
        self.resident_slots.insert(probe_id, slot);
    }
}
