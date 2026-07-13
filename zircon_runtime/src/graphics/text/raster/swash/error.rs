use super::bitmap::GlyphBitmapError;
use super::request::SwashRasterSource;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SwashRasterError {
    InvalidFontFace {
        face_index: usize,
    },
    InvalidPxSize,
    InvalidOffset,
    InvalidVariationCoordinate,
    MissingGlyphImage {
        glyph_id: u16,
        source: SwashRasterSource,
    },
    InvalidGlyphBitmap(GlyphBitmapError),
}
