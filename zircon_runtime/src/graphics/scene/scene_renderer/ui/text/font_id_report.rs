use crate::text::FontFaceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextFontIdReport {
    pub(crate) text_batch_count: usize,
    pub(crate) glyph_count: usize,
    pub(crate) fallback_glyph_count: usize,
    pub(crate) unmapped_glyph_count: usize,
}

/// Accumulates faces from the handle-resolution batch already needed to project glyph runs.
///
/// The first resolved face in a text batch is its primary face. A later resolved face differs
/// only when shaping selected a fallback; unresolved handles remain visible as diagnostics rather
/// than triggering a renderer-local shape attempt.
pub(super) fn accumulate_resolved_glyph_faces(
    report: &mut ScreenSpaceUiTextFontIdReport,
    faces: impl IntoIterator<Item = Option<FontFaceId>>,
) {
    let mut glyph_count = 0;
    let mut fallback_glyph_count = 0;
    let mut unmapped_glyph_count = 0;
    let mut primary = None;

    for face in faces {
        glyph_count += 1;
        match face {
            Some(face) if primary.is_none() => primary = Some(face),
            Some(face) if primary == Some(face) => {}
            Some(_) => fallback_glyph_count += 1,
            None => unmapped_glyph_count += 1,
        }
    }

    if glyph_count > 0 {
        report.text_batch_count += 1;
        report.glyph_count += glyph_count;
        report.fallback_glyph_count += fallback_glyph_count;
        report.unmapped_glyph_count += unmapped_glyph_count;
    }
}

#[cfg(test)]
mod tests {
    use super::{ScreenSpaceUiTextFontIdReport, accumulate_resolved_glyph_faces};
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
}
