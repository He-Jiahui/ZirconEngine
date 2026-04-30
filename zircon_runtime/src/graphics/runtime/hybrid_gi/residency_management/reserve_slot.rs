use super::super::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi::residency_management) fn reserve_slot(
        &mut self,
        slot: u32,
    ) {
        if self.remove_free_slot(slot) {
            return;
        }

        if slot >= self.next_slot() {
            for free_slot in self.next_slot()..slot {
                self.insert_free_slot(free_slot);
            }
            self.advance_next_slot_past(slot);
        }
    }
}
