use crate::core::math::{UVec2, Vec2};
use crate::text::atlas::{GlyphAtlasFormat, GlyphAtlasStorageFormat};

pub(super) const ALPHA_MASK_CHANNELS: u8 = 1;
pub(super) const COLOR_BITMAP_CHANNELS: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GlyphBitmapContent {
    AlphaMask,
    SubpixelMask,
    Color,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum GlyphColorBitmapAlphaMode {
    Straight,
    Premultiplied,
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
        Self::color_with_alpha_mode(
            size,
            bearing,
            px_size,
            data,
            GlyphColorBitmapAlphaMode::Straight,
        )
    }

    pub(super) fn color_with_alpha_mode(
        size: UVec2,
        bearing: Vec2,
        px_size: f32,
        data: Vec<u8>,
        alpha_mode: GlyphColorBitmapAlphaMode,
    ) -> Result<Self, GlyphBitmapError> {
        let mut bitmap = Self::new(size, bearing, px_size, data, GlyphBitmapContent::Color)?;
        if alpha_mode == GlyphColorBitmapAlphaMode::Premultiplied {
            unpremultiply_rgba8_in_place(&mut bitmap.data);
        }
        Ok(bitmap)
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
        Some(self.required_atlas_format())
    }

    pub(super) fn required_atlas_format(&self) -> GlyphAtlasFormat {
        self.content.atlas_format()
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

fn unpremultiply_rgba8_in_place(data: &mut [u8]) {
    for pixel in data.chunks_exact_mut(COLOR_BITMAP_CHANNELS as usize) {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 {
            pixel[..3].fill(0);
            continue;
        }
        if alpha == u32::from(u8::MAX) {
            continue;
        }

        for channel in &mut pixel[..3] {
            let straight = (u32::from(*channel) * u32::from(u8::MAX) + alpha / 2) / alpha;
            *channel = straight.min(u32::from(u8::MAX)) as u8;
        }
    }
}
