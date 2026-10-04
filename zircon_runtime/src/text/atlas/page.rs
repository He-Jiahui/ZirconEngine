use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use crate::core::math::UVec2;

use super::page_residency::{
    apply_page_residency_decision, page_rebuild_residency_decision, page_residency_decision,
    GlyphAtlasPageReservation, GlyphAtlasPageResidencyDecision, GlyphAtlasResidentPage,
};
#[cfg(test)]
use super::page_shadow::GlyphAtlasBitmapPageShadowPatch;
use super::page_shadow::{
    GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPageShadowReport,
    GlyphAtlasBitmapPageShadowStore,
};
use super::slot_cache::{GlyphAtlasPersistentSlot, GlyphAtlasSlotCache};
use super::{GlyphAtlasAllocation, GlyphRasterKey};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlyphAtlasFormat {
    AlphaMask,
    SubpixelMask,
    Sdf,
    Msdf,
    Color,
}

impl GlyphAtlasFormat {
    pub(crate) fn supported_formats() -> [Self; 5] {
        [
            Self::AlphaMask,
            Self::SubpixelMask,
            Self::Sdf,
            Self::Msdf,
            Self::Color,
        ]
    }

    pub(crate) fn storage_format(self) -> GlyphAtlasStorageFormat {
        match self {
            Self::AlphaMask | Self::Sdf => GlyphAtlasStorageFormat::R8Unorm,
            Self::SubpixelMask | Self::Msdf | Self::Color => GlyphAtlasStorageFormat::Rgba8Unorm,
        }
    }

    pub(crate) fn sampling_semantics(self) -> GlyphAtlasSamplingSemantics {
        match self {
            Self::AlphaMask => GlyphAtlasSamplingSemantics::AlphaCoverage,
            Self::SubpixelMask => GlyphAtlasSamplingSemantics::SubpixelCoverage,
            Self::Sdf => GlyphAtlasSamplingSemantics::SignedDistance,
            Self::Msdf => GlyphAtlasSamplingSemantics::MultiChannelSignedDistance,
            Self::Color => GlyphAtlasSamplingSemantics::ColorRgba,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlyphAtlasStorageFormat {
    R8Unorm,
    Rgba8Unorm,
}

impl GlyphAtlasStorageFormat {
    pub(crate) fn bytes_per_pixel(self) -> u32 {
        match self {
            Self::R8Unorm => 1,
            Self::Rgba8Unorm => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum GlyphAtlasSamplingSemantics {
    AlphaCoverage,
    SubpixelCoverage,
    SignedDistance,
    MultiChannelSignedDistance,
    ColorRgba,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct GlyphAtlasPageKey {
    pub(crate) format: GlyphAtlasFormat,
    pub(crate) page_index: u32,
}

impl GlyphAtlasPageKey {
    pub(crate) fn new(format: GlyphAtlasFormat, page_index: u32) -> Self {
        Self { format, page_index }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct GlyphAtlasPageSpec {
    pub(crate) key: GlyphAtlasPageKey,
    pub(crate) size: UVec2,
    pub(crate) generation: u64,
    pub(crate) storage_format: GlyphAtlasStorageFormat,
    pub(crate) sampling_semantics: GlyphAtlasSamplingSemantics,
}

impl GlyphAtlasPageSpec {
    pub(crate) fn new(key: GlyphAtlasPageKey, size: UVec2) -> Self {
        debug_assert!(GlyphAtlasFormat::supported_formats().contains(&key.format));
        Self {
            key,
            size,
            generation: 0,
            storage_format: key.format.storage_format(),
            sampling_semantics: key.format.sampling_semantics(),
        }
    }

    pub(crate) fn with_generation(mut self, generation: u64) -> Self {
        self.generation = generation;
        self
    }

    pub(crate) fn byte_len(&self) -> usize {
        let width = usize::try_from(self.size.x).unwrap_or(usize::MAX);
        let height = usize::try_from(self.size.y).unwrap_or(usize::MAX);
        width
            .saturating_mul(height)
            .saturating_mul(self.storage_format.bytes_per_pixel() as usize)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl GlyphAtlasRect {
    pub(crate) fn union(self, other: Self) -> Self {
        let min_x = self.x.min(other.x);
        let min_y = self.y.min(other.y);
        let max_x = self
            .x
            .saturating_add(self.width)
            .max(other.x.saturating_add(other.width));
        let max_y = self
            .y
            .saturating_add(self.height)
            .max(other.y.saturating_add(other.height));
        Self {
            x: min_x,
            y: min_y,
            width: max_x.saturating_sub(min_x),
            height: max_y.saturating_sub(min_y),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct GlyphAtlasSet {
    pages: Vec<GlyphAtlasResidentPage>,
    slot_cache: GlyphAtlasSlotCache,
    bitmap_page_shadow: Arc<GlyphAtlasBitmapPageShadowStore>,
    pending_invalidated_bitmap_raster_keys: Vec<GlyphRasterKey>,
}

impl GlyphAtlasSet {
    #[cfg(test)]
    pub(crate) fn from_page(page: GlyphAtlasPageSpec) -> Self {
        Self::default().with_page(page)
    }

    #[cfg(test)]
    pub(crate) fn with_page(mut self, page: GlyphAtlasPageSpec) -> Self {
        self.slot_cache.invalidate_page(page.key);
        self.invalidate_bitmap_page_shadow(page.key);
        if let Some(existing) = self
            .pages
            .iter_mut()
            .find(|existing| existing.key() == page.key)
        {
            existing.replace_spec(page);
        } else {
            self.pages
                .push(GlyphAtlasResidentPage::from_existing_page(page));
        }
        self
    }

    pub(crate) fn page_count(&self) -> usize {
        self.pages.len()
    }

    pub(crate) fn resident_page_byte_len(&self) -> usize {
        self.pages.iter().fold(0, |byte_len, page| {
            byte_len.saturating_add(page.spec().byte_len())
        })
    }

    pub(crate) fn bitmap_page_shadow_report(&self) -> GlyphAtlasBitmapPageShadowReport {
        self.bitmap_page_shadow.report()
    }

    pub(crate) fn page(
        &self,
        format: GlyphAtlasFormat,
        page_index: u32,
    ) -> Option<&GlyphAtlasPageSpec> {
        let key = GlyphAtlasPageKey::new(format, page_index);
        self.pages
            .iter()
            .find(|page| page.key() == key)
            .map(|page| page.spec())
    }

    pub(crate) fn begin_frame(&mut self) {
        for page in &mut self.pages {
            page.clear_frame_reference();
        }
    }

    pub(crate) fn mark_page_used(&mut self, key: GlyphAtlasPageKey, frame_index: u64) -> bool {
        if let Some(page) = self.pages.iter_mut().find(|page| page.key() == key) {
            page.mark_used(frame_index);
            true
        } else {
            false
        }
    }

    pub(crate) fn reserve_page_for_format(
        &mut self,
        format: GlyphAtlasFormat,
        page_size: UVec2,
        frame_index: u64,
        max_pages_per_format: usize,
    ) -> GlyphAtlasPageReservation {
        let decision = page_residency_decision(&self.pages, format, max_pages_per_format);
        self.invalidate_evicted_page(decision);
        apply_page_residency_decision(&mut self.pages, decision, page_size, frame_index)
    }

    pub(crate) fn reserve_rebuildable_page_for_format(
        &mut self,
        format: GlyphAtlasFormat,
        page_size: UVec2,
        frame_index: u64,
        max_pages_per_format: usize,
    ) -> GlyphAtlasPageReservation {
        let decision = page_rebuild_residency_decision(&self.pages, format, max_pages_per_format);
        self.invalidate_evicted_page(decision);
        apply_page_residency_decision(&mut self.pages, decision, page_size, frame_index)
    }

    pub(crate) fn persistent_bitmap_slot(
        &mut self,
        key: GlyphRasterKey,
        content_size: UVec2,
        page_size: UVec2,
        frame_index: u64,
    ) -> Option<GlyphAtlasPersistentSlot> {
        let slot = self.slot_cache.slot(key)?;
        let Some((resident_size, resident_generation)) = self
            .page(slot.page_key.format, slot.page_key.page_index)
            .map(|page| (page.size, page.generation))
        else {
            self.slot_cache.remove_slot(key);
            return None;
        };
        if key.format != slot.page_key.format
            || slot.content_size != content_size
            || resident_size != page_size
            || slot.page_generation != resident_generation
        {
            self.slot_cache.remove_slot(key);
            return None;
        }

        self.mark_page_used(slot.page_key, frame_index);
        Some(slot)
    }

    pub(crate) fn persistent_bitmap_slot_rects_by_page(
        &self,
    ) -> BTreeMap<GlyphAtlasPageKey, Vec<GlyphAtlasRect>> {
        self.slot_cache.slot_rects_by_page()
    }

    pub(crate) fn bitmap_page_shadow_bytes(&self, page: &GlyphAtlasPageSpec) -> Option<&[u8]> {
        self.bitmap_page_shadow.bytes_for_page(page)
    }

    pub(crate) fn bitmap_page_shadow_pages(&self) -> impl Iterator<Item = &GlyphAtlasPageSpec> {
        self.pages
            .iter()
            .map(GlyphAtlasResidentPage::spec)
            .filter(|page| self.bitmap_page_shadow_bytes(page).is_some())
    }

    pub(crate) fn has_bitmap_page_shadow(&self, page_key: GlyphAtlasPageKey) -> bool {
        self.page(page_key.format, page_key.page_index)
            .and_then(|page| self.bitmap_page_shadow_bytes(page))
            .is_some()
    }

    pub(crate) fn commit_bitmap_page_shadow(&mut self, commit: GlyphAtlasBitmapPageShadowCommit) {
        let pages = self
            .pages
            .iter()
            .map(|page| page.spec().clone())
            .collect::<Vec<_>>();
        Arc::make_mut(&mut self.bitmap_page_shadow).apply(&pages, commit);
    }

    pub(crate) fn invalidate_bitmap_page_upload_state<I>(
        &mut self,
        page_keys: I,
    ) -> Vec<GlyphRasterKey>
    where
        I: IntoIterator<Item = GlyphAtlasPageKey>,
    {
        let mut invalidated_keys = Vec::new();
        for page_key in page_keys.into_iter().collect::<BTreeSet<_>>() {
            invalidated_keys.extend(self.invalidate_bitmap_page_contents(page_key));
        }
        invalidated_keys
    }

    pub(crate) fn invalidate_bitmap_raster_keys<I>(&mut self, raster_keys: I) -> Vec<GlyphRasterKey>
    where
        I: IntoIterator<Item = GlyphRasterKey>,
    {
        let page_keys = raster_keys
            .into_iter()
            .filter_map(|key| self.slot_cache.page_key_for_slot(key))
            .collect::<BTreeSet<_>>();
        self.invalidate_bitmap_page_upload_state(page_keys)
    }

    pub(crate) fn take_pending_invalidated_bitmap_raster_keys(&mut self) -> Vec<GlyphRasterKey> {
        std::mem::take(&mut self.pending_invalidated_bitmap_raster_keys)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn allocate_persistent_bitmap_slot(
        &mut self,
        key: GlyphRasterKey,
        content_size: UVec2,
        page_size: UVec2,
        frame_index: u64,
        max_pages_per_format: usize,
        padding_px: u32,
    ) -> Option<(
        GlyphAtlasPersistentSlot,
        Option<GlyphAtlasPageResidencyDecision>,
    )> {
        for page_index in 0..self.pages.len() {
            let (page_key, resident_size, page_generation) = {
                let page = &self.pages[page_index];
                (page.key(), page.spec().size, page.spec().generation)
            };
            if page_key.format != key.format {
                continue;
            }
            if resident_size != page_size {
                continue;
            }
            if let Some(allocation) =
                self.slot_cache
                    .allocate(page_key, page_size, padding_px, content_size)
            {
                let slot = self.insert_persistent_bitmap_slot(
                    key,
                    content_size,
                    page_generation,
                    allocation,
                    frame_index,
                );
                return Some((slot, None));
            }
        }

        let reservation =
            self.reserve_page_for_format(key.format, page_size, frame_index, max_pages_per_format);
        let page = reservation.page?;
        let allocation = self
            .slot_cache
            .allocate(page.key, page_size, padding_px, content_size)?;
        let slot = self.insert_persistent_bitmap_slot(
            key,
            content_size,
            page.generation,
            allocation,
            frame_index,
        );
        Some((slot, Some(reservation.decision)))
    }

    fn insert_persistent_bitmap_slot(
        &mut self,
        key: GlyphRasterKey,
        content_size: UVec2,
        page_generation: u64,
        allocation: GlyphAtlasAllocation,
        frame_index: u64,
    ) -> GlyphAtlasPersistentSlot {
        let slot = GlyphAtlasPersistentSlot {
            page_key: allocation.page_key,
            page_generation,
            inserted_frame_index: frame_index,
            rect: allocation.rect,
            content_size,
        };
        self.slot_cache.insert_slot(key, slot);
        self.mark_page_used(allocation.page_key, frame_index);
        slot
    }

    fn invalidate_evicted_page(&mut self, decision: GlyphAtlasPageResidencyDecision) {
        if let GlyphAtlasPageResidencyDecision::Evict(page_key) = decision {
            self.pending_invalidated_bitmap_raster_keys
                .extend(self.slot_cache.invalidate_page(page_key));
            self.invalidate_bitmap_page_shadow(page_key);
        }
    }

    fn invalidate_bitmap_page_contents(
        &mut self,
        page_key: GlyphAtlasPageKey,
    ) -> Vec<GlyphRasterKey> {
        let invalidated_keys = self.slot_cache.invalidate_page(page_key);
        self.invalidate_bitmap_page_shadow(page_key);
        if let Some(page) = self.pages.iter_mut().find(|page| page.key() == page_key) {
            page.invalidate_contents();
        }
        invalidated_keys
    }

    fn invalidate_bitmap_page_shadow(&mut self, page_key: GlyphAtlasPageKey) {
        Arc::make_mut(&mut self.bitmap_page_shadow).remove_page(page_key);
    }
}

#[cfg(test)]
#[path = "tests/page.rs"]
mod tests;
