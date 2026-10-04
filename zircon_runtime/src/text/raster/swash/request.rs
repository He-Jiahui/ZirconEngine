use std::sync::Arc;

use super::error::SwashRasterError;
use crate::core::math::Vec2;
use swash::scale::{Source as SwashSource, StrikeWith};
use swash::zeno::{Angle, Format as SwashRenderFormat, Transform as SwashTransform};

use crate::core::framework::text::{
    TextGlyphRasterError, TextGlyphRasterHinting, TextGlyphRasterMode, TextGlyphRasterRequest,
    TextGlyphRasterSmoothing,
};
use crate::text::VariationCoords;

const SWASH_RASTER_SOURCE_CAPACITY: usize = 3;
const FAKE_ITALIC_SKEW_DEGREES: f32 = 14.0;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SwashRasterRequest {
    pub(crate) face_index: usize,
    /// Stable identity for ScaleContext cache reuse when a worker owns the font bytes.
    pub(crate) font_identity: Option<[u64; 2]>,
    pub(crate) glyph_id: u16,
    pub(crate) px_size: f32,
    pub(crate) hint: bool,
    pub(crate) offset: Vec2,
    pub(crate) render_format: SwashRenderFormat,
    pub(crate) fake_italic: bool,
    pub(crate) variations: Arc<VariationCoords>,
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
            font_identity: None,
            glyph_id,
            px_size,
            hint,
            offset: Vec2::ZERO,
            render_format: SwashRenderFormat::Alpha,
            fake_italic: false,
            variations: Arc::new(VariationCoords::default()),
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
            font_identity: None,
            glyph_id,
            px_size,
            hint,
            offset: Vec2::ZERO,
            render_format: SwashRenderFormat::Subpixel,
            fake_italic: false,
            variations: Arc::new(VariationCoords::default()),
            sources: [SwashRasterSource::SubpixelOutline; SWASH_RASTER_SOURCE_CAPACITY],
            source_count: 1,
        }
    }

    pub(crate) fn color_preferred(
        face_index: usize,
        glyph_id: u16,
        px_size: f32,
        hint: bool,
        subpixel_fallback: bool,
    ) -> Self {
        Self {
            face_index,
            font_identity: None,
            glyph_id,
            px_size,
            hint,
            offset: Vec2::ZERO,
            render_format: if subpixel_fallback {
                SwashRenderFormat::Subpixel
            } else {
                SwashRenderFormat::Alpha
            },
            fake_italic: false,
            variations: Arc::new(VariationCoords::default()),
            sources: [
                SwashRasterSource::ColorOutline { palette_index: 0 },
                SwashRasterSource::ColorBitmap(SwashBitmapStrike::BestFit),
                SwashRasterSource::AlphaOutline,
            ],
            source_count: 3,
        }
    }

    /// Adapts the backend-neutral Runtime request for either synchronous or worker execution.
    pub(crate) fn from_text_glyph_request(
        face_index: usize,
        request: TextGlyphRasterRequest,
    ) -> Result<Self, TextGlyphRasterError> {
        let glyph_id =
            u16::try_from(request.glyph_id).map_err(|_| TextGlyphRasterError::InvalidGlyphId)?;
        if request.physical_ppem == 0 {
            return Err(TextGlyphRasterError::InvalidPhysicalPpem);
        }
        if request.horizontal_phase >= TextGlyphRasterRequest::HORIZONTAL_PHASE_COUNT
            || request.vertical_phase >= TextGlyphRasterRequest::VERTICAL_PHASE_COUNT
        {
            return Err(TextGlyphRasterError::InvalidSubpixelPhase);
        }
        let hint = request.hinting != TextGlyphRasterHinting::None;
        let subpixel = request.smoothing == TextGlyphRasterSmoothing::Subpixel;
        let mut backend_request = match request.mode {
            TextGlyphRasterMode::Outline if subpixel => {
                Self::subpixel_outline(face_index, glyph_id, request.physical_ppem as f32, hint)
            }
            TextGlyphRasterMode::Outline => {
                Self::alpha_outline(face_index, glyph_id, request.physical_ppem as f32, hint)
            }
            TextGlyphRasterMode::ColorPreferred => Self::color_preferred(
                face_index,
                glyph_id,
                request.physical_ppem as f32,
                hint,
                subpixel,
            ),
        };
        backend_request.offset = Vec2::new(
            f32::from(request.horizontal_phase)
                / f32::from(TextGlyphRasterRequest::HORIZONTAL_PHASE_COUNT),
            f32::from(request.vertical_phase)
                / f32::from(TextGlyphRasterRequest::VERTICAL_PHASE_COUNT),
        );
        backend_request.fake_italic = request.synthetic.oblique;
        Ok(backend_request)
    }

    pub(crate) fn sources(&self) -> &[SwashRasterSource] {
        &self.sources[..self.source_count]
    }

    pub(crate) fn with_variations(mut self, variations: Arc<VariationCoords>) -> Self {
        self.variations = variations;
        self
    }

    pub(crate) fn with_font_identity(mut self, font_identity: [u64; 2]) -> Self {
        self.font_identity = Some(font_identity);
        self
    }

    pub(crate) fn shares_scaler_configuration_with(&self, other: &Self) -> bool {
        self.face_index == other.face_index
            && self.font_identity == other.font_identity
            && self.px_size.to_bits() == other.px_size.to_bits()
            && self.hint == other.hint
            && self.variations == other.variations
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
