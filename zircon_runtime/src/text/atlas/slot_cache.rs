use std::collections::{BTreeMap, HashMap};

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
    allocators: BTreeMap<GlyphAtlasPageKey, GlyphAtlasShelfAllocator>,
}

impl GlyphAtlasSlotCache {
    pub(super) fn slot(&self, key: GlyphRasterKey) -> Option<GlyphAtlasPersistentSlot> {
        self.slots.get(&key).copied()
    }

    pub(super) fn remove_slot(&mut self, key: GlyphRasterKey) {
        self.slots.remove(&key);
    }

    pub(super) fn insert_slot(&mut self, key: GlyphRasterKey, slot: GlyphAtlasPersistentSlot) {
        self.slots.insert(key, slot);
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

    pub(super) fn invalidate_page(&mut self, page_key: GlyphAtlasPageKey) {
        self.allocators.remove(&page_key);
        self.slots.retain(|_, slot| slot.page_key != page_key);
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
