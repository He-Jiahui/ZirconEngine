use std::collections::{BTreeMap, HashMap, HashSet};

use crate::core::math::UVec2;

use super::{
    GlyphAtlasAllocation, GlyphAtlasPageKey, GlyphAtlasRect, GlyphAtlasShelfAllocator,
    GlyphRasterKey,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasPersistentSlot {
    pub(crate) page_key: GlyphAtlasPageKey,
    pub(crate) page_generation: u64,
    pub(crate) inserted_frame_index: u64,
    pub(crate) rect: GlyphAtlasRect,
    pub(crate) content_size: UVec2,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct GlyphAtlasSlotCache {
    slots: HashMap<GlyphRasterKey, GlyphAtlasPersistentSlot>,
    page_slots: HashMap<GlyphAtlasPageKey, HashSet<GlyphRasterKey>>,
    allocators: HashMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
}

impl GlyphAtlasSlotCache {
    pub(super) fn slot(&self, key: GlyphRasterKey) -> Option<GlyphAtlasPersistentSlot> {
        self.slots.get(&key).copied()
    }

    pub(super) fn remove_slot(&mut self, key: GlyphRasterKey) {
        if let Some(slot) = self.slots.remove(&key) {
            self.remove_page_slot(slot.page_key, key);
        }
    }

    pub(super) fn insert_slot(&mut self, key: GlyphRasterKey, slot: GlyphAtlasPersistentSlot) {
        if let Some(previous) = self.slots.insert(key, slot) {
            if previous.page_key != slot.page_key {
                self.remove_page_slot(previous.page_key, key);
            }
        }
        self.page_slots
            .entry(slot.page_key)
            .or_default()
            .insert(key);
    }

    pub(super) fn page_key_for_slot(&self, key: GlyphRasterKey) -> Option<GlyphAtlasPageKey> {
        self.slots.get(&key).map(|slot| slot.page_key)
    }

    pub(super) fn allocate(
        &mut self,
        page_key: GlyphAtlasPageKey,
        page_size: UVec2,
        padding_px: u32,
        content_size: UVec2,
    ) -> Option<GlyphAtlasAllocation> {
        let allocator = self
            .allocators
            .entry(page_key)
            .or_insert_with(|| GlyphAtlasShelfAllocator::new(page_key, page_size, padding_px));
        if !allocator.matches_configuration(page_key, page_size, padding_px) {
            return None;
        }

        let mut trial = allocator.clone();
        let allocation = trial.allocate(content_size)?;
        *allocator = trial;
        Some(allocation)
    }

    pub(super) fn invalidate_page(&mut self, page_key: GlyphAtlasPageKey) -> Vec<GlyphRasterKey> {
        self.allocators.remove(&page_key);
        let invalidated_keys = self
            .page_slots
            .remove(&page_key)
            .map_or_else(Vec::new, |keys| keys.into_iter().collect());
        for key in &invalidated_keys {
            self.slots.remove(key);
        }
        invalidated_keys
    }

    fn remove_page_slot(&mut self, page_key: GlyphAtlasPageKey, key: GlyphRasterKey) {
        let remove_page = if let Some(keys) = self.page_slots.get_mut(&page_key) {
            keys.remove(&key);
            keys.is_empty()
        } else {
            false
        };
        if remove_page {
            self.page_slots.remove(&page_key);
        }
    }

    pub(super) fn slot_rects_by_page(&self) -> BTreeMap<GlyphAtlasPageKey, Vec<GlyphAtlasRect>> {
        let mut rects_by_page = BTreeMap::new();
        for slot in self.slots.values() {
            rects_by_page
                .entry(slot.page_key)
                .or_insert_with(Vec::new)
                .push(slot.rect);
        }
        rects_by_page
    }

    #[cfg(test)]
    pub(super) fn slot_count(&self) -> usize {
        self.slots.len()
    }
}

#[cfg(test)]
#[path = "slot_cache/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "slot_cache/hash_allocator_tests.rs"]
mod hash_allocator_tests;
