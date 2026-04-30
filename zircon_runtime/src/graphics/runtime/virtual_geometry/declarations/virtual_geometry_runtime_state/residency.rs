use super::runtime_state::VirtualGeometryRuntimeState;

impl VirtualGeometryRuntimeState {
    pub(in crate::graphics::runtime::virtual_geometry) fn resident_page_count(&self) -> usize {
        self.resident_slots.len()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn resident_slot(
        &self,
        page_id: u32,
    ) -> Option<u32> {
        self.resident_slots.get(&page_id).copied()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn has_resident_page(
        &self,
        page_id: u32,
    ) -> bool {
        self.resident_slots.contains_key(&page_id)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn resident_page_ids(
        &self,
    ) -> impl Iterator<Item = u32> + '_ {
        self.resident_slots.keys().copied()
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn resident_page_slots(
        &self,
    ) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.resident_slots
            .iter()
            .map(|(&page_id, &slot)| (page_id, slot))
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn insert_resident_page_slot(
        &mut self,
        page_id: u32,
        slot: u32,
    ) -> Option<u32> {
        self.resident_slots.insert(page_id, slot)
    }

    pub(in crate::graphics::runtime::virtual_geometry) fn remove_resident_page_slot(
        &mut self,
        page_id: u32,
    ) -> Option<u32> {
        self.resident_slots.remove(&page_id)
    }
}
