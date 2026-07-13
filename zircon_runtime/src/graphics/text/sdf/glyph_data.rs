use crate::core::math::UVec2;

use super::{SdfGlyphGenerationError, SdfMode};

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
        if self.size.x == 0 || self.size.y == 0 {
            return Err(SdfGlyphGenerationError::InvalidDimensions(self.size));
        }
        let expected = self
            .size
            .x
            .checked_mul(self.size.y)
            .and_then(|pixel_count| pixel_count.checked_mul(self.channels as u32))
            .map(|byte_count| byte_count as usize)
            .ok_or(SdfGlyphGenerationError::InvalidDimensions(self.size))?;
        if self.pixels.len() != expected {
            return Err(SdfGlyphGenerationError::InvalidOutputLength {
                expected,
                actual: self.pixels.len(),
            });
        }
        Ok(())
    }
}
