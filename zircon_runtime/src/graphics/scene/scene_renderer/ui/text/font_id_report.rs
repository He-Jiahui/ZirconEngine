use glyphon::Buffer;

use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch;
use crate::text::font::FontDatabase;
use crate::text::{FontFamilyName, FontQuery, FontStretch, FontStyle, FontWeight};
use zircon_runtime_interface::ui::surface::{
    resolve_ui_text_render_mode, UiResolvedStyle, UiTextRenderMode,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ScreenSpaceUiTextFontIdReport {
    pub(crate) text_batch_count: usize,
    pub(crate) glyph_count: usize,
    pub(crate) fallback_glyph_count: usize,
    pub(crate) unmapped_glyph_count: usize,
}

impl ScreenSpaceUiTextFontIdReport {
    pub(super) fn accumulate(&mut self, report: crate::text::NativeTextFontIdReport) {
        self.text_batch_count = self
            .text_batch_count
            .saturating_add(report.text_batch_count);
        self.glyph_count = self.glyph_count.saturating_add(report.glyph_count);
        self.fallback_glyph_count = self
            .fallback_glyph_count
            .saturating_add(report.fallback_glyph_count);
        self.unmapped_glyph_count = self
            .unmapped_glyph_count
            .saturating_add(report.unmapped_glyph_count);
    }
}

pub(super) fn accumulate_text_font_id_report(
    report: &mut ScreenSpaceUiTextFontIdReport,
    style: &UiResolvedStyle,
    buffer: &Buffer,
    font_database: &FontDatabase,
) {
    let query = font_query_for_style(style);
    let Some(primary) = font_database
        .match_face(&query)
        .map(|font_match| font_match.face)
    else {
        return;
    };

    accumulate_backend_glyphs(report, buffer, primary, font_database);
}

fn accumulate_backend_glyphs(
    report: &mut ScreenSpaceUiTextFontIdReport,
    buffer: &Buffer,
    primary: crate::text::FontFaceId,
    font_database: &FontDatabase,
) {
    let mut glyph_count = 0;
    let mut fallback_glyph_count = 0;
    let mut unmapped_glyph_count = 0;
    for glyph in buffer.layout_runs().flat_map(|run| run.glyphs.iter()) {
        glyph_count += 1;
        match font_database.font_face_id(glyph.font_id) {
            Some(face) if face != primary => fallback_glyph_count += 1,
            Some(_) => {}
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

fn font_query_for_style(style: &UiResolvedStyle) -> FontQuery {
    let family = style
        .font_family
        .as_deref()
        .or(style.font.as_deref())
        .unwrap_or_default();
    FontQuery {
        families: vec![FontFamilyName::from(family)],
        weight: FontWeight::clamped(UiResolvedStyle::normalized_font_weight(style.font_weight)),
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    }
}

pub(super) fn resolved_style_for_text_batch(
    text: &ScreenSpaceUiTextBatch,
    family_name: Option<&str>,
) -> UiResolvedStyle {
    UiResolvedStyle {
        font: text.font.clone(),
        font_family: family_name
            .map(str::to_string)
            .or_else(|| text.font_family.clone()),
        language: text.language.clone(),
        font_weight: text.font_weight,
        font_size: text.font_size,
        line_height: text.line_height,
        text_align: text.text_align,
        text_direction: text.text_direction,
        wrap: text.wrap,
        text_render_mode: resolve_ui_text_render_mode(UiTextRenderMode::Native, None),
        ..UiResolvedStyle::default()
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use glyphon::{Attrs, Buffer, FontSystem, Metrics, Shaping};

    use super::{accumulate_backend_glyphs, ScreenSpaceUiTextFontIdReport};
    use crate::text::font::FontDatabase;

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
        accumulate_backend_glyphs(&mut report, &buffer, primary, &database);

        assert!(report.glyph_count > 0);
        assert_eq!(report.fallback_glyph_count, 0);
        assert_eq!(report.unmapped_glyph_count, 0);
    }
}
