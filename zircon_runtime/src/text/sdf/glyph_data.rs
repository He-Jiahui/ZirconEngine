use crate::core::math::UVec2;

use super::{SdfGlyphGenerationError, SdfMode};

/// Keeps a full default scheduler batch within its 32 MiB completion budget.
pub(super) const MAX_SDF_GLYPH_BYTE_COUNT: usize = 1024 * 1024;

pub(super) fn sdf_glyph_byte_len(
    size: UVec2,
    channels: u8,
) -> Result<usize, SdfGlyphGenerationError> {
    if size.x == 0 || size.y == 0 {
        return Err(SdfGlyphGenerationError::InvalidDimensions(size));
    }
    let byte_count = u64::from(size.x)
        .checked_mul(u64::from(size.y))
        .and_then(|pixel_count| pixel_count.checked_mul(u64::from(channels)))
        .and_then(|byte_count| usize::try_from(byte_count).ok())
        .filter(|byte_count| *byte_count <= MAX_SDF_GLYPH_BYTE_COUNT)
        .ok_or(SdfGlyphGenerationError::InvalidDimensions(size))?;
    Ok(byte_count)
}

/// Baked glyph pixels and metrics expressed in bake-pixel units.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SdfGlyphData {
    pub(crate) size: UVec2,
    pub(crate) bitmap_left: f32,
    pub(crate) bitmap_bottom: f32,
    pub(crate) advance: f32,
    pub(crate) ascent: f32,
    pub(crate) pixels: Vec<u8>,
    pub(crate) channels: u8,
    pub(crate) spread_px: f32,
    pub(crate) mode: SdfMode,
}

impl SdfGlyphData {
    pub(crate) fn validate(&self) -> Result<(), SdfGlyphGenerationError> {
        let expected_channels = self.mode.channel_count();
        if self.channels != expected_channels {
            return Err(SdfGlyphGenerationError::InvalidChannelCount {
                expected: expected_channels,
                actual: self.channels,
            });
        }
        let expected = sdf_glyph_byte_len(self.size, expected_channels)?;
        if self.pixels.len() != expected {
            return Err(SdfGlyphGenerationError::InvalidOutputLength {
                expected,
                actual: self.pixels.len(),
            });
        }
        Ok(())
    }
}
