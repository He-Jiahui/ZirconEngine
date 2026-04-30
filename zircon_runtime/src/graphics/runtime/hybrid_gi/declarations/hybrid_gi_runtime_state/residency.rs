use super::runtime_state::HybridGiRuntimeState;

impl HybridGiRuntimeState {
    pub(in crate::graphics::runtime::hybrid_gi) fn first_free_slot(&self) -> Option<u32> {
        self.free_slots.iter().next().copied()
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn insert_free_slot(&mut self, slot: u32) -> bool {
        self.free_slots.insert(slot)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn remove_free_slot(&mut self, slot: u32) -> bool {
        self.free_slots.remove(&slot)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn next_slot(&self) -> u32 {
        self.next_slot
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn allocate_next_slot(&mut self) -> u32 {
        let slot = self.next_slot;
        self.next_slot = self.next_slot.saturating_add(1);
        slot
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn advance_next_slot_past(&mut self, slot: u32) {
        self.next_slot = slot.saturating_add(1);
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn resident_probe_count(&self) -> usize {
        self.resident_slots.len()
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn has_resident_probe(
        &self,
        probe_id: u32,
    ) -> bool {
        self.resident_slots.contains_key(&probe_id)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn resident_probe_ids(
        &self,
    ) -> impl Iterator<Item = u32> + '_ {
        self.resident_slots.keys().copied()
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn resident_probe_slots(
        &self,
    ) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.resident_slots
            .iter()
            .map(|(&probe_id, &slot)| (probe_id, slot))
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn insert_resident_probe_slot(
        &mut self,
        probe_id: u32,
        slot: u32,
    ) -> Option<u32> {
        self.resident_slots.insert(probe_id, slot)
    }

    pub(in crate::graphics::runtime::hybrid_gi) fn remove_resident_probe_slot(
        &mut self,
        probe_id: u32,
    ) -> Option<u32> {
        self.resident_slots.remove(&probe_id)
    }
}
