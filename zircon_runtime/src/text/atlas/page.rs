use crate::core::math::UVec2;

use super::page_residency::{
    apply_page_residency_decision, page_rebuild_residency_decision, page_residency_decision,
    GlyphAtlasPageReservation, GlyphAtlasResidentPage,
};

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
}

impl GlyphAtlasSet {
    #[cfg(test)]
    pub(crate) fn from_page(page: GlyphAtlasPageSpec) -> Self {
        Self::default().with_page(page)
    }

    #[cfg(test)]
    pub(crate) fn with_page(mut self, page: GlyphAtlasPageSpec) -> Self {
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
        apply_page_residency_decision(&mut self.pages, decision, page_size, frame_index)
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
}
