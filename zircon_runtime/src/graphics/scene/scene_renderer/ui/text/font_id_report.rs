use crate::graphics::scene::scene_renderer::ui::render::ScreenSpaceUiTextBatch;
use crate::graphics::text::font::FontDatabase;
use crate::graphics::text::shaping::{
    annotate_fallback_font_ids, font_query_for_style, shape_horizontal_line,
};
use crate::ui::text::shaper::resolve_text_render_mode;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRange, UiTextRenderMode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ScreenSpaceUiTextFontIdReport {
    pub(super) text_batch_count: usize,
    pub(super) glyph_count: usize,
    pub(super) fallback_glyph_count: usize,
}

pub(super) fn accumulate_text_font_id_report(
    report: &mut ScreenSpaceUiTextFontIdReport,
    text: &ScreenSpaceUiTextBatch,
    family_name: Option<&str>,
    font_database: &FontDatabase,
) {
    if text.text.is_empty() {
        return;
    }

    let style = resolved_style_for_text_batch(text, family_name);
    let query = font_query_for_style(&style);
    let Some(primary) = font_database
        .match_face(&query)
        .map(|font_match| font_match.face)
    else {
        return;
    };

    let mut shaped = shape_horizontal_line(
        &text.text,
        &style,
        text.text_direction,
        UiTextRange {
            start: 0,
            end: text.text.len(),
        },
    );
    annotate_fallback_font_ids(&mut shaped, primary, &query, font_database, None);

    let mut glyph_count = 0;
    let mut fallback_glyph_count = 0;
    for glyph in shaped.lines.iter().flat_map(|line| line.glyphs.iter()) {
        glyph_count += 1;
        if glyph.font_id.is_some_and(|font_id| font_id != primary) {
            fallback_glyph_count += 1;
        }
    }

    if glyph_count > 0 {
        report.text_batch_count += 1;
        report.glyph_count += glyph_count;
        report.fallback_glyph_count += fallback_glyph_count;
    }
}

fn resolved_style_for_text_batch(
    text: &ScreenSpaceUiTextBatch,
    family_name: Option<&str>,
) -> UiResolvedStyle {
    UiResolvedStyle {
        font: text.font.clone(),
        font_family: family_name
            .map(str::to_string)
            .or_else(|| text.font_family.clone()),
        font_weight: text.font_weight,
        font_size: text.font_size,
        line_height: text.line_height,
        text_align: text.text_align,
        text_direction: text.text_direction,
        wrap: text.wrap,
        text_render_mode: resolve_text_render_mode(UiTextRenderMode::Native, None),
        ..UiResolvedStyle::default()
    }
}
