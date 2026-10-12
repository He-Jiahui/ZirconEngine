use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry::residency_management) fn reserve_slot(&mut self, slot: u32) {
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
