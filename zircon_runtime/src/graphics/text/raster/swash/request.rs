use super::error::SwashRasterError;
use ::swash::scale::{Source as SwashSource, StrikeWith};
use ::swash::zeno::Format as SwashRenderFormat;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SwashRasterRequest {
    pub(crate) face_index: usize,
    pub(crate) glyph_id: u16,
    pub(crate) px_size: f32,
    pub(crate) hint: bool,
    pub(crate) source: SwashRasterSource,
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
            source: SwashRasterSource::AlphaOutline,
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
            source: SwashRasterSource::SubpixelOutline,
        }
    }

    pub(super) fn validate(self) -> Result<(), SwashRasterError> {
        if !self.px_size.is_finite() || self.px_size <= 0.0 {
            return Err(SwashRasterError::InvalidPxSize);
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

    pub(super) fn render_format(self) -> SwashRenderFormat {
        match self {
            Self::SubpixelOutline => SwashRenderFormat::Subpixel,
            Self::AlphaOutline
            | Self::AlphaBitmap(_)
            | Self::ColorOutline { .. }
            | Self::ColorBitmap(_) => SwashRenderFormat::Alpha,
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
