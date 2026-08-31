use crate::core::math::UVec2;

use super::{SdfOfflineArtifactError, SdfOfflineArtifactIdentity, codec};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SdfOfflineRect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

impl SdfOfflineRect {
    pub(crate) const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SdfOfflineGlyphMetrics {
    pub(crate) bitmap_left: f32,
    pub(crate) bitmap_bottom: f32,
    pub(crate) advance: f32,
    pub(crate) ascent: f32,
}

impl SdfOfflineGlyphMetrics {
    pub(super) fn is_finite(self) -> bool {
        self.bitmap_left.is_finite()
            && self.bitmap_bottom.is_finite()
            && self.advance.is_finite()
            && self.ascent.is_finite()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SdfOfflineGlyph {
    pub(crate) glyph_id: u32,
    pub(crate) codepoint: u32,
    pub(crate) page_index: u32,
    pub(crate) rect: SdfOfflineRect,
    pub(crate) metrics: SdfOfflineGlyphMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SdfOfflinePage {
    pub(crate) page_index: u32,
    pub(crate) pixels: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SdfOfflineArtifact {
    identity: SdfOfflineArtifactIdentity,
    page_size: UVec2,
    pages: Vec<SdfOfflinePage>,
    glyphs: Vec<SdfOfflineGlyph>,
}

impl SdfOfflineArtifact {
    pub(crate) fn new(
        identity: SdfOfflineArtifactIdentity,
        page_size: UVec2,
        mut pages: Vec<SdfOfflinePage>,
        mut glyphs: Vec<SdfOfflineGlyph>,
    ) -> Result<Self, SdfOfflineArtifactError> {
        let identity = identity.normalized()?;
        if page_size.x == 0 || page_size.y == 0 {
            return Err(SdfOfflineArtifactError::InvalidPageSize);
        }
        if pages.is_empty() {
            return Err(SdfOfflineArtifactError::MissingPages);
        }
        pages.sort_by_key(|page| page.page_index);
        let expected_page_len = page_byte_len(page_size, identity.params.mode.channel_count())?;
        for (expected, page) in pages.iter().enumerate() {
            let expected =
                u32::try_from(expected).map_err(|_| SdfOfflineArtifactError::LengthOverflow)?;
            if page.page_index < expected {
                return Err(SdfOfflineArtifactError::DuplicatePageIndex(page.page_index));
            }
            if page.page_index != expected {
                return Err(SdfOfflineArtifactError::NonContiguousPageIndex {
                    expected,
                    actual: page.page_index,
                });
            }
            if page.pixels.len() != expected_page_len {
                return Err(SdfOfflineArtifactError::InvalidPageByteLength {
                    page_index: page.page_index,
                    expected: expected_page_len,
                    actual: page.pixels.len(),
                });
            }
        }

        glyphs.sort_by_key(|glyph| glyph.glyph_id);
        let mut previous_glyph_id = None;
        for glyph in &glyphs {
            if previous_glyph_id == Some(glyph.glyph_id) {
                return Err(SdfOfflineArtifactError::DuplicateGlyphId(glyph.glyph_id));
            }
            previous_glyph_id = Some(glyph.glyph_id);
            if char::from_u32(glyph.codepoint).is_none() {
                return Err(SdfOfflineArtifactError::InvalidCodepoint {
                    glyph_id: glyph.glyph_id,
                    codepoint: glyph.codepoint,
                });
            }
            if !glyph.metrics.is_finite() {
                return Err(SdfOfflineArtifactError::NonFiniteGlyphMetric {
                    glyph_id: glyph.glyph_id,
                });
            }
            if pages.get(glyph.page_index as usize).is_none() {
                return Err(SdfOfflineArtifactError::MissingGlyphPage {
                    glyph_id: glyph.glyph_id,
                    page_index: glyph.page_index,
                });
            }
            if !rect_fits(glyph.rect, page_size) {
                return Err(SdfOfflineArtifactError::GlyphRectOutOfBounds {
                    glyph_id: glyph.glyph_id,
                    page_index: glyph.page_index,
                });
            }
        }

        Ok(Self {
            identity,
            page_size,
            pages,
            glyphs,
        })
    }

    pub(crate) fn identity(&self) -> &SdfOfflineArtifactIdentity {
        &self.identity
    }

    pub(crate) fn page_size(&self) -> UVec2 {
        self.page_size
    }

    pub(crate) fn pages(&self) -> &[SdfOfflinePage] {
        &self.pages
    }

    pub(crate) fn glyphs(&self) -> &[SdfOfflineGlyph] {
        &self.glyphs
    }

    pub(crate) fn glyph(&self, glyph_id: u32) -> Option<&SdfOfflineGlyph> {
        self.glyphs
            .binary_search_by_key(&glyph_id, |glyph| glyph.glyph_id)
            .ok()
            .map(|index| &self.glyphs[index])
    }

    pub(crate) fn glyph_pixels(&self, glyph_id: u32) -> Option<Vec<u8>> {
        let glyph = self.glyph(glyph_id)?;
        let page = self.pages.get(glyph.page_index as usize)?;
        let channels = usize::from(self.identity.params.mode.channel_count());
        let page_width = usize::try_from(self.page_size.x).ok()?;
        let row_byte_len = usize::try_from(glyph.rect.width)
            .ok()?
            .checked_mul(channels)?;
        let mut pixels =
            Vec::with_capacity(row_byte_len.checked_mul(usize::try_from(glyph.rect.height).ok()?)?);
        for row in glyph.rect.y..glyph.rect.y.checked_add(glyph.rect.height)? {
            let pixel_index = usize::try_from(row)
                .ok()?
                .checked_mul(page_width)?
                .checked_add(usize::try_from(glyph.rect.x).ok()?)?;
            let start = pixel_index.checked_mul(channels)?;
            let end = start.checked_add(row_byte_len)?;
            pixels.extend_from_slice(page.pixels.get(start..end)?);
        }
        Some(pixels)
    }

    pub(crate) fn validate_identity(
        &self,
        expected: &SdfOfflineArtifactIdentity,
    ) -> Result<(), SdfOfflineArtifactError> {
        self.identity.validate_matches(expected)
    }

    pub(crate) fn encode(&self) -> Result<Vec<u8>, SdfOfflineArtifactError> {
        codec::encode(self)
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, SdfOfflineArtifactError> {
        codec::decode(bytes)
    }
}

fn page_byte_len(page_size: UVec2, channels: u8) -> Result<usize, SdfOfflineArtifactError> {
    usize::try_from(page_size.x)
        .ok()
        .and_then(|width| {
            usize::try_from(page_size.y)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(usize::from(channels)))
        .ok_or(SdfOfflineArtifactError::LengthOverflow)
}

fn rect_fits(rect: SdfOfflineRect, page_size: UVec2) -> bool {
    rect.width > 0
        && rect.height > 0
        && rect
            .x
            .checked_add(rect.width)
            .is_some_and(|right| right <= page_size.x)
        && rect
            .y
            .checked_add(rect.height)
            .is_some_and(|bottom| bottom <= page_size.y)
}
