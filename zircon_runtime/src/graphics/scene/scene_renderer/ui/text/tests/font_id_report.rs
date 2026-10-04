use super::{accumulate_resolved_glyph_faces, ScreenSpaceUiTextFontIdReport};
use crate::text::FontFaceId;

#[test]
fn native_font_id_report_uses_canonical_shaped_glyph_faces() {
    let primary = FontFaceId(7);
    let fallback = FontFaceId(11);
    let mut report = ScreenSpaceUiTextFontIdReport::default();
    accumulate_resolved_glyph_faces(
        &mut report,
        [Some(primary), Some(primary), Some(fallback), None],
    );

    assert_eq!(report.text_batch_count, 1);
    assert_eq!(report.glyph_count, 4);
    assert_eq!(report.fallback_glyph_count, 1);
    assert_eq!(report.unmapped_glyph_count, 1);
}
