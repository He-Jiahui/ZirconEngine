use fdsm::shape::{Contour, Shape};

use super::SdfGlyphGenerationError;

/// Rejects outlines that cannot participate in fdsm distance queries.
pub(super) fn validate_outline_shape(
    shape: &Shape<Contour>,
    glyph_id: u16,
) -> Result<(), SdfGlyphGenerationError> {
    if shape.contours.is_empty()
        || shape
            .contours
            .iter()
            .any(|contour| contour.segments.is_empty())
    {
        return Err(SdfGlyphGenerationError::EmptyGlyphBounds(glyph_id));
    }
    Ok(())
}
