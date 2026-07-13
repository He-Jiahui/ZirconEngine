use super::error::SwashRasterError;
use crate::core::math::Vec2;
use ::glyphon::cosmic_text::{CacheKey as GlyphonCacheKey, CacheKeyFlags as GlyphonCacheKeyFlags};
use ::swash::scale::{Source as SwashSource, StrikeWith};
use ::swash::zeno::{Angle, Format as SwashRenderFormat, Transform as SwashTransform};

use crate::core::framework::render::VariationCoords;

const SWASH_RASTER_SOURCE_CAPACITY: usize = 3;
const FAKE_ITALIC_SKEW_DEGREES: f32 = 14.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SwashRasterRequest {
    pub(crate) face_index: usize,
    pub(crate) glyph_id: u16,
    pub(crate) px_size: f32,
    pub(crate) hint: bool,
    pub(crate) offset: Vec2,
    pub(crate) render_format: SwashRenderFormat,
    pub(crate) fake_italic: bool,
    pub(crate) variations: VariationCoords,
    sources: [SwashRasterSource; SWASH_RASTER_SOURCE_CAPACITY],
    source_count: usize,
}

impl SwashRasterRequest {
    pub(crate) fn alpha_outline(
        face_index: usize,
        glyph_id: u16,
        px_size: f32,
        hint: bool,
    ) -> Self {
        Self {
            face_index,
            glyph_id,
            px_size,
            hint,
            offset: Vec2::ZERO,
            render_format: SwashRenderFormat::Alpha,
            fake_italic: false,
            variations: VariationCoords::default(),
            sources: [SwashRasterSource::AlphaOutline; SWASH_RASTER_SOURCE_CAPACITY],
            source_count: 1,
        }
    }

    pub(crate) fn subpixel_outline(
        face_index: usize,
        glyph_id: u16,
        px_size: f32,
        hint: bool,
    ) -> Self {
        Self {
            face_index,
            glyph_id,
            px_size,
            hint,
            offset: Vec2::ZERO,
            render_format: SwashRenderFormat::Subpixel,
            fake_italic: false,
            variations: VariationCoords::default(),
            sources: [SwashRasterSource::SubpixelOutline; SWASH_RASTER_SOURCE_CAPACITY],
            source_count: 1,
        }
    }

    pub(crate) fn glyphon_cache_key(face_index: usize, cache_key: GlyphonCacheKey) -> Self {
        Self {
            face_index,
            glyph_id: cache_key.glyph_id,
            px_size: f32::from_bits(cache_key.font_size_bits),
            hint: !cache_key
                .flags
                .contains(GlyphonCacheKeyFlags::DISABLE_HINTING),
            offset: glyphon_cache_key_offset(cache_key),
            render_format: SwashRenderFormat::Alpha,
            fake_italic: cache_key.flags.contains(GlyphonCacheKeyFlags::FAKE_ITALIC),
            variations: VariationCoords(vec![(
                u32::from_be_bytes(*b"wght"),
                f32::from(cache_key.font_weight.0),
            )]),
            sources: [
                SwashRasterSource::ColorOutline { palette_index: 0 },
                SwashRasterSource::ColorBitmap(SwashBitmapStrike::BestFit),
                SwashRasterSource::AlphaOutline,
            ],
            source_count: 3,
        }
    }

    pub(crate) fn sources(&self) -> &[SwashRasterSource] {
        &self.sources[..self.source_count]
    }

    pub(crate) fn with_variations(mut self, variations: VariationCoords) -> Self {
        self.variations = variations;
        self
    }

    pub(super) fn swash_sources(&self) -> [SwashSource; SWASH_RASTER_SOURCE_CAPACITY] {
        [
            self.sources[0].to_swash_source(),
            self.sources[1].to_swash_source(),
            self.sources[2].to_swash_source(),
        ]
    }

    pub(super) fn source_count(&self) -> usize {
        self.source_count
    }

    pub(super) fn primary_source(&self) -> SwashRasterSource {
        self.sources[0]
    }

    pub(super) fn fake_italic_transform(&self) -> Option<SwashTransform> {
        self.fake_italic.then(|| {
            SwashTransform::skew(
                Angle::from_degrees(FAKE_ITALIC_SKEW_DEGREES),
                Angle::from_degrees(0.0),
            )
        })
    }

    pub(super) fn validate(&self) -> Result<(), SwashRasterError> {
        if !self.px_size.is_finite() || self.px_size <= 0.0 {
            return Err(SwashRasterError::InvalidPxSize);
        }
        if !self.offset.x.is_finite() || !self.offset.y.is_finite() {
            return Err(SwashRasterError::InvalidOffset);
        }
        if self
            .variations
            .0
            .iter()
            .any(|(_, value)| !value.is_finite())
        {
            return Err(SwashRasterError::InvalidVariationCoordinate);
        }

        Ok(())
    }
}

fn glyphon_cache_key_offset(cache_key: GlyphonCacheKey) -> Vec2 {
    if cache_key.flags.contains(GlyphonCacheKeyFlags::PIXEL_FONT) {
        Vec2::new(
            cache_key.x_bin.as_float().round() + 1.0,
            cache_key.y_bin.as_float().round(),
        )
    } else {
        Vec2::new(cache_key.x_bin.as_float(), cache_key.y_bin.as_float())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SwashRasterSource {
    AlphaOutline,
    SubpixelOutline,
    AlphaBitmap(SwashBitmapStrike),
    ColorOutline { palette_index: u16 },
    ColorBitmap(SwashBitmapStrike),
}

impl SwashRasterSource {
    pub(super) fn to_swash_source(self) -> SwashSource {
        match self {
            Self::AlphaOutline | Self::SubpixelOutline => SwashSource::Outline,
            Self::AlphaBitmap(strike) => SwashSource::Bitmap(strike.into()),
            Self::ColorOutline { palette_index } => SwashSource::ColorOutline(palette_index),
            Self::ColorBitmap(strike) => SwashSource::ColorBitmap(strike.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SwashBitmapStrike {
    ExactSize,
    BestFit,
    LargestSize,
    Index(u32),
}

impl From<SwashBitmapStrike> for StrikeWith {
    fn from(strike: SwashBitmapStrike) -> Self {
        match strike {
            SwashBitmapStrike::ExactSize => StrikeWith::ExactSize,
            SwashBitmapStrike::BestFit => StrikeWith::BestFit,
            SwashBitmapStrike::LargestSize => StrikeWith::LargestSize,
            SwashBitmapStrike::Index(index) => StrikeWith::Index(index),
        }
    }
}
