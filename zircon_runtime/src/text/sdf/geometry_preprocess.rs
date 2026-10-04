//! 在交给 FDSM 距离查询之前拒绝空或无效轮廓；调用方以此把字体轮廓问题转成可诊断的生成错误。

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
