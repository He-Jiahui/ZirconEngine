use glyphon::Buffer;

use crate::text::FontFaceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextFontIdReport {
    pub(crate) text_batch_count: usize,
    pub(crate) glyph_count: usize,
    pub(crate) fallback_glyph_count: usize,
    pub(crate) unmapped_glyph_count: usize,
}

pub(super) fn accumulate_text_font_id_report(
    report: &mut ScreenSpaceUiTextFontIdReport,
    buffer: &Buffer,
    primary: Option<FontFaceId>,
    resolve_face: impl FnMut(glyphon::fontdb::ID) -> Option<FontFaceId>,
) {
    accumulate_backend_glyphs(report, buffer, primary, resolve_face);
}

fn accumulate_backend_glyphs(
    report: &mut ScreenSpaceUiTextFontIdReport,
    buffer: &Buffer,
    primary: Option<FontFaceId>,
    mut resolve_face: impl FnMut(glyphon::fontdb::ID) -> Option<FontFaceId>,
) {
    let mut glyph_count = 0;
    let mut fallback_glyph_count = 0;
    let mut unmapped_glyph_count = 0;
    for glyph in buffer.layout_runs().flat_map(|run| run.glyphs.iter()) {
        glyph_count += 1;
        match (resolve_face(glyph.font_id), primary) {
            (Some(face), Some(primary)) if face == primary => {}
            (Some(_), _) => fallback_glyph_count += 1,
            (None, _) => unmapped_glyph_count += 1,
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
    use std::path::Path;

    use glyphon::{Attrs, Buffer, FontSystem, Metrics, Shaping};

    use super::{accumulate_text_font_id_report, ScreenSpaceUiTextFontIdReport};
    use crate::text::font::FontDatabase;
    use crate::text::FontFaceId;

    #[test]
    fn native_font_id_report_uses_actual_layout_glyph_face() {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraMono-subset.ttf");
        let mut database = FontDatabase::default();
        let primary = database
            .register_font_file(&source, Some("Fira Mono"), 0)
            .unwrap();
        let mut font_system = FontSystem::new();
        database
            .load_face_into_font_system(primary, &mut font_system)
            .unwrap();
        let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
        buffer.set_text(
            &mut font_system,
            "Actual backend face",
            &Attrs::new().family(glyphon::Family::Name("Fira Mono")),
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut font_system, false);

        let mut report = ScreenSpaceUiTextFontIdReport::default();
        accumulate_text_font_id_report(&mut report, &buffer, Some(primary), |backend| {
            database.font_face_id(backend)
        });

        assert!(report.glyph_count > 0);
        assert_eq!(report.fallback_glyph_count, 0);
        assert_eq!(report.unmapped_glyph_count, 0);

        let mut different_primary = ScreenSpaceUiTextFontIdReport::default();
        accumulate_text_font_id_report(
            &mut different_primary,
            &buffer,
            Some(FontFaceId(primary.0 + 1)),
            |backend| database.font_face_id(backend),
        );
        assert_eq!(different_primary.glyph_count, report.glyph_count);
        assert_eq!(
            different_primary.fallback_glyph_count,
            different_primary.glyph_count
        );
        assert_eq!(different_primary.unmapped_glyph_count, 0);

        let mut unresolved_primary = ScreenSpaceUiTextFontIdReport::default();
        accumulate_text_font_id_report(&mut unresolved_primary, &buffer, None, |backend| {
            database.font_face_id(backend)
        });
        assert_eq!(unresolved_primary.glyph_count, report.glyph_count);
        assert_eq!(
            unresolved_primary.fallback_glyph_count,
            unresolved_primary.glyph_count
        );
        assert_eq!(unresolved_primary.unmapped_glyph_count, 0);
    }
}
