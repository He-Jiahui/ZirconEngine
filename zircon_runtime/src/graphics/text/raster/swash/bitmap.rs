use crate::core::math::{UVec2, Vec2};
use crate::graphics::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};

pub(super) const ALPHA_MASK_CHANNELS: u8 = 1;
pub(super) const COLOR_BITMAP_CHANNELS: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphBitmapContent {
    AlphaMask,
    SubpixelMask,
    Color,
}

impl GlyphBitmapContent {
    fn channels(self) -> u8 {
        match self {
            Self::AlphaMask => ALPHA_MASK_CHANNELS,
            Self::SubpixelMask | Self::Color => COLOR_BITMAP_CHANNELS,
        }
    }

    fn atlas_format(self) -> GlyphAtlasFormat {
        match self {
            Self::AlphaMask => GlyphAtlasFormat::AlphaMask,
            Self::SubpixelMask => GlyphAtlasFormat::SubpixelMask,
            Self::Color => GlyphAtlasFormat::Color,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct GlyphBitmap {
    pub(crate) size: UVec2,
    pub(crate) bearing: Vec2,
    pub(crate) px_size: f32,
    pub(crate) data: Vec<u8>,
    pub(crate) channels: u8,
    pub(crate) content: GlyphBitmapContent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum GlyphBitmapError {
    EmptySize { size: UVec2 },
    InvalidBearing,
    InvalidPxSize,
    UnsupportedChannelCount { channels: u8 },
    DataLengthMismatch { expected: usize, actual: usize },
}

impl GlyphBitmap {
    pub(crate) fn alpha_mask(
        size: UVec2,
        bearing: Vec2,
        px_size: f32,
        data: Vec<u8>,
    ) -> Result<Self, GlyphBitmapError> {
        Self::new(size, bearing, px_size, data, GlyphBitmapContent::AlphaMask)
    }

    pub(crate) fn subpixel_mask(
        size: UVec2,
        bearing: Vec2,
        px_size: f32,
        data: Vec<u8>,
    ) -> Result<Self, GlyphBitmapError> {
        Self::new(
            size,
            bearing,
            px_size,
            data,
            GlyphBitmapContent::SubpixelMask,
        )
    }

    pub(crate) fn color(
        size: UVec2,
        bearing: Vec2,
        px_size: f32,
        data: Vec<u8>,
    ) -> Result<Self, GlyphBitmapError> {
        Self::new(size, bearing, px_size, data, GlyphBitmapContent::Color)
    }

    fn new(
        size: UVec2,
        bearing: Vec2,
        px_size: f32,
        data: Vec<u8>,
        content: GlyphBitmapContent,
    ) -> Result<Self, GlyphBitmapError> {
        let bitmap = Self {
            size,
            bearing,
            px_size,
            data,
            channels: content.channels(),
            content,
        };
        bitmap.validate()?;
        Ok(bitmap)
    }

    fn validate(&self) -> Result<(), GlyphBitmapError> {
        if self.size.x == 0 || self.size.y == 0 {
            return Err(GlyphBitmapError::EmptySize { size: self.size });
        }

        if !self.bearing.x.is_finite() || !self.bearing.y.is_finite() {
            return Err(GlyphBitmapError::InvalidBearing);
        }

        if !self.px_size.is_finite() || self.px_size <= 0.0 {
            return Err(GlyphBitmapError::InvalidPxSize);
        }

        if self.channels != self.content.channels() {
            return Err(GlyphBitmapError::UnsupportedChannelCount {
                channels: self.channels,
            });
        }

        let expected = self.expected_data_len();
        let actual = self.data.len();
        if actual != expected {
            return Err(GlyphBitmapError::DataLengthMismatch { expected, actual });
        }

        Ok(())
    }

    pub(crate) fn atlas_format(&self) -> Option<GlyphAtlasFormat> {
        Some(self.content.atlas_format())
    }

    pub(crate) fn storage_format(&self) -> Option<GlyphAtlasStorageFormat> {
        self.atlas_format().map(GlyphAtlasFormat::storage_format)
    }

    pub(crate) fn expected_data_len(&self) -> usize {
        (self.size.x as usize)
            .saturating_mul(self.size.y as usize)
            .saturating_mul(self.channels as usize)
    }

    pub(crate) fn has_expected_data_len(&self) -> bool {
        self.data.len() == self.expected_data_len()
    }
}
