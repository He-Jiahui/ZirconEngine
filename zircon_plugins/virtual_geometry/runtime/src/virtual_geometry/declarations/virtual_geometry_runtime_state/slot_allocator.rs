use super::runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::virtual_geometry) fn free_slot_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.free_slots.iter().copied()
    }

    pub(in crate::virtual_geometry) fn first_free_slot(&self) -> Option<u32> {
        self.free_slots.iter().next().copied()
    }

    pub(in crate::virtual_geometry) fn has_free_slot(&self, slot: u32) -> bool {
        self.free_slots.contains(&slot)
    }

    pub(in crate::virtual_geometry) fn insert_free_slot(&mut self, slot: u32) -> bool {
        self.free_slots.insert(slot)
    }

    pub(in crate::virtual_geometry) fn remove_free_slot(&mut self, slot: u32) -> bool {
        self.free_slots.remove(&slot)
    }

    pub(in crate::virtual_geometry) fn next_slot(&self) -> u32 {
        self.next_slot
    }

    pub(in crate::virtual_geometry) fn allocate_next_slot(&mut self) -> u32 {
        let slot = self.next_slot;
        self.next_slot = self.next_slot.saturating_add(1);
        slot
    }

    pub(in crate::virtual_geometry) fn advance_next_slot_past(&mut self, slot: u32) {
        self.next_slot = slot.saturating_add(1);
    }
}
