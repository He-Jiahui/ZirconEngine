use std::collections::HashSet;

use super::{RawBakedGlyph, SdfAtlasGlyphKey, SdfAtlasSlot, SdfFontBakeCache};

const MAX_RESIDENT_BAKED_GLYPH_COUNT: usize = 4 * 1024;
const MAX_RESIDENT_BAKED_GLYPH_BYTE_COUNT: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SdfBakedGlyphCacheReport {
    pub(super) resident_count: usize,
    pub(super) resident_byte_count: usize,
    pub(super) eviction_count: usize,
    pub(super) oldest_idle_access_count: u64,
}

impl SdfFontBakeCache {
    pub(super) fn contains_baked_glyph(&mut self, key: &SdfAtlasGlyphKey) -> bool {
        let contains = self.glyphs.contains_key(key);
        if contains {
            self.touch_cached_glyph_key(key.clone());
        }
        contains
    }

    pub(super) fn cached_baked_glyph(&mut self, key: &SdfAtlasGlyphKey) -> Option<RawBakedGlyph> {
        let glyph = self.glyphs.get(key).cloned();
        if glyph.is_some() {
            self.touch_cached_glyph_key(key.clone());
        }
        glyph
    }

    pub(super) fn insert_baked_glyph(&mut self, key: SdfAtlasGlyphKey, glyph: RawBakedGlyph) {
        self.measured_glyphs.insert(key.clone(), glyph.metrics);
        if let Some(previous) = self.glyphs.insert(key.clone(), glyph) {
            self.resident_baked_glyph_byte_count = self
                .resident_baked_glyph_byte_count
                .saturating_sub(previous.bitmap.len());
        }
        self.resident_baked_glyph_byte_count = self
            .resident_baked_glyph_byte_count
            .saturating_add(self.glyphs.get(&key).map_or(0, |glyph| glyph.bitmap.len()));
        self.touch_cached_glyph_key(key);
    }

    pub(super) fn enforce_baked_glyph_budget(&mut self, slots: &[SdfAtlasSlot]) {
        if !self.baked_glyph_cache_over_budget() {
            return;
        }
        let protected = slots
            .iter()
            .map(|slot| slot.key.clone())
            .collect::<HashSet<_>>();
        while self.baked_glyph_cache_over_budget() {
            let Some(victim) = self.oldest_unprotected_baked_glyph(&protected) else {
                break;
            };
            if let Some(glyph) = self.glyphs.remove(&victim) {
                self.resident_baked_glyph_byte_count = self
                    .resident_baked_glyph_byte_count
                    .saturating_sub(glyph.bitmap.len());
            }
            self.measured_glyphs.remove(&victim);
            self.face_resolutions.remove(&victim);
            self.shaped_face_resolutions.remove(&victim);
            if let Some(epoch) = self.baked_glyph_recency.remove(&victim) {
                self.baked_glyph_recency_order.remove(&(epoch, victim));
            }
            self.baked_glyph_eviction_count = self.baked_glyph_eviction_count.saturating_add(1);
        }
    }

    pub(super) fn report_baked_glyph_cache(&mut self) -> SdfBakedGlyphCacheReport {
        let eviction_count = self
            .baked_glyph_eviction_count
            .saturating_sub(self.reported_baked_glyph_eviction_count);
        self.reported_baked_glyph_eviction_count = self.baked_glyph_eviction_count;
        SdfBakedGlyphCacheReport {
            resident_count: self.baked_glyph_recency.len(),
            resident_byte_count: self.resident_baked_glyph_byte_count,
            eviction_count,
            oldest_idle_access_count: self
                .baked_glyph_recency_order
                .first()
                .map(|(epoch, _)| self.baked_glyph_access_epoch.saturating_sub(*epoch))
                .unwrap_or(0),
        }
    }

    pub(super) fn clear_cached_glyph_entries(&mut self) {
        self.glyphs.clear();
        self.measured_glyphs.clear();
        self.face_resolutions.clear();
        self.shaped_face_resolutions.clear();
        self.baked_glyph_recency.clear();
        self.baked_glyph_recency_order.clear();
        self.baked_glyph_access_epoch = 0;
        self.resident_baked_glyph_byte_count = 0;
        self.baked_glyph_eviction_count = 0;
        self.reported_baked_glyph_eviction_count = 0;
    }

    pub(super) fn touch_cached_glyph_slots(&mut self, slots: &[SdfAtlasSlot]) {
        for slot in slots {
            if self.baked_glyph_recency.contains_key(&slot.key) {
                self.touch_cached_glyph_key(slot.key.clone());
            }
        }
    }

    pub(super) fn touch_cached_glyph_key(&mut self, key: SdfAtlasGlyphKey) {
        self.baked_glyph_access_epoch = self.baked_glyph_access_epoch.saturating_add(1).max(1);
        if let Some(previous_epoch) = self
            .baked_glyph_recency
            .insert(key.clone(), self.baked_glyph_access_epoch)
        {
            self.baked_glyph_recency_order
                .remove(&(previous_epoch, key.clone()));
        }
        self.baked_glyph_recency_order
            .insert((self.baked_glyph_access_epoch, key));
    }

    fn baked_glyph_cache_over_budget(&self) -> bool {
        self.baked_glyph_recency.len() > MAX_RESIDENT_BAKED_GLYPH_COUNT
            || (self.glyphs.len() > 1
                && self.resident_baked_glyph_byte_count > MAX_RESIDENT_BAKED_GLYPH_BYTE_COUNT)
    }

    fn oldest_unprotected_baked_glyph(
        &self,
        protected: &HashSet<SdfAtlasGlyphKey>,
    ) -> Option<SdfAtlasGlyphKey> {
        self.baked_glyph_recency_order
            .iter()
            .find(|(_, key)| !protected.contains(key))
            .map(|(_, key)| key.clone())
    }
}
