use std::collections::BTreeMap;
use std::sync::Arc;

use crate::core::math::UVec2;

use super::page_residency::{
    GlyphAtlasPageReservation, GlyphAtlasPageResidencyDecision, GlyphAtlasResidentPage,
    apply_page_residency_decision, page_rebuild_residency_decision, page_residency_decision,
};
#[cfg(test)]
use super::page_shadow::GlyphAtlasBitmapPageShadowPatch;
use super::page_shadow::{GlyphAtlasBitmapPageShadowCommit, GlyphAtlasBitmapPageShadowStore};
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

    pub(crate) fn invalidate_bitmap_page_upload_state<I>(&mut self, page_keys: I)
    where
        I: IntoIterator<Item = GlyphAtlasPageKey>,
    {
        for page_key in page_keys {
            self.slot_cache.invalidate_page(page_key);
            self.invalidate_bitmap_page_shadow(page_key);
        }
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
        let existing_pages = self
            .pages
            .iter()
            .filter(|page| page.key().format == key.format)
            .map(|page| (page.key(), page.spec().size, page.spec().generation))
            .collect::<Vec<_>>();
        for (page_key, resident_size, page_generation) in existing_pages {
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
            self.slot_cache.invalidate_page(page_key);
            self.invalidate_bitmap_page_shadow(page_key);
        }
    }

    fn invalidate_bitmap_page_shadow(&mut self, page_key: GlyphAtlasPageKey) {
        Arc::make_mut(&mut self.bitmap_page_shadow).remove_page(page_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_text_atlas_subpixel_mask_uses_distinct_rgba_page_format() {
        let page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
            UVec2::new(256, 256),
        );

        assert!(GlyphAtlasFormat::supported_formats().contains(&GlyphAtlasFormat::SubpixelMask));
        assert_eq!(page.key.format, GlyphAtlasFormat::SubpixelMask);
        assert_eq!(page.storage_format, GlyphAtlasStorageFormat::Rgba8Unorm);
        assert_eq!(
            page.sampling_semantics,
            GlyphAtlasSamplingSemantics::SubpixelCoverage
        );
        assert_eq!(page.storage_format.bytes_per_pixel(), 4);
    }

    #[test]
    fn render_text_atlas_rgba_storage_keeps_color_and_subpixel_blend_semantics_distinct() {
        let subpixel_page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::SubpixelMask, 0),
            UVec2::new(128, 128),
        );
        let color_page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0),
            UVec2::new(128, 128),
        );

        assert_eq!(subpixel_page.storage_format, color_page.storage_format);
        assert_eq!(
            subpixel_page.sampling_semantics,
            GlyphAtlasSamplingSemantics::SubpixelCoverage
        );
        assert_eq!(
            color_page.sampling_semantics,
            GlyphAtlasSamplingSemantics::ColorRgba
        );
    }

    #[test]
    fn render_text_atlas_resident_page_bytes_account_for_storage_format() {
        let atlas = GlyphAtlasSet::default()
            .with_page(GlyphAtlasPageSpec::new(
                GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
                UVec2::new(16, 8),
            ))
            .with_page(GlyphAtlasPageSpec::new(
                GlyphAtlasPageKey::new(GlyphAtlasFormat::Color, 0),
                UVec2::new(16, 8),
            ));

        assert_eq!(atlas.page_count(), 2);
        assert_eq!(atlas.resident_page_byte_len(), 16 * 8 * (1 + 4));
    }

    #[test]
    fn bitmap_page_shadow_commits_only_accepted_zero_initialized_pages() {
        let page = GlyphAtlasPageSpec::new(
            GlyphAtlasPageKey::new(GlyphAtlasFormat::AlphaMask, 0),
            UVec2::new(8, 8),
        )
        .with_generation(3);
        let mut atlas = GlyphAtlasSet::from_page(page.clone());
        let patch = GlyphAtlasBitmapPageShadowPatch {
            page_key: page.key,
            page_generation: page.generation,
            target_rect: GlyphAtlasRect {
                x: 2,
                y: 1,
                width: 2,
                height: 2,
            },
            bytes_per_row: 2,
            bytes: vec![0x7F; 4],
        };
        let mut failed = GlyphAtlasBitmapPageShadowCommit::default();
        failed.zero_initialized_pages.insert(page.key);
        failed.failed_zero_initialized_pages.insert(page.key);
        failed.patches.push(patch.clone());
        atlas.commit_bitmap_page_shadow(failed);
        assert!(atlas.bitmap_page_shadow_bytes(&page).is_none());

        let mut accepted = GlyphAtlasBitmapPageShadowCommit::default();
        accepted.zero_initialized_pages.insert(page.key);
        accepted.patches.push(patch);
        atlas.commit_bitmap_page_shadow(accepted);

        let shadow = atlas.bitmap_page_shadow_bytes(&page).unwrap();
        assert_eq!(shadow.len(), page.byte_len());
        assert_eq!(shadow[1 * 8 + 2], 0x7F);
        assert_eq!(shadow[2 * 8 + 3], 0x7F);
        assert_eq!(shadow[0], 0);

        atlas.invalidate_bitmap_page_upload_state([page.key]);
        assert!(atlas.bitmap_page_shadow_bytes(&page).is_none());
    }
}
