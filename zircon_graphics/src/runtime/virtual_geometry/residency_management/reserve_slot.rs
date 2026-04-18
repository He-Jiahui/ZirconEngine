use super::super::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::runtime::virtual_geometry::residency_management) fn reserve_slot(
        &mut self,
        slot: u32,
    ) {
        if self.free_slots.remove(&slot) {
            return;
        }

        if slot >= self.next_slot {
            for free_slot in self.next_slot..slot {
                self.free_slots.insert(free_slot);
            }
            self.next_slot = slot.saturating_add(1);
        }
    }
}
