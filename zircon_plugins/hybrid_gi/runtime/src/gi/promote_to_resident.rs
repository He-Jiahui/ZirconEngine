use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::hybrid_gi) fn promote_to_resident(&mut self, probe_id: u32) {
        if self.has_resident_probe(probe_id) {
            return;
        }

        self.clear_pending_update(probe_id);

        let slot = self
            .take_free_slot()
            .unwrap_or_else(|| self.allocate_next_slot());
        self.insert_resident_probe_slot(probe_id, slot);
    }
}
