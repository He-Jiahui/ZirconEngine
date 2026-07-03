use crate::core::framework::render::{
    CompositeFontDescriptor, FontFaceId, FontFamilyName, FontQuery, FontScript, FontStretch,
    FontStyle, FontWeight, ShapedGlyph, ShapedGlyphRun,
};
use crate::graphics::text::font::FontDatabase;
use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextRange};

pub(crate) fn font_query_for_style(style: &UiResolvedStyle) -> FontQuery {
    if let Some(family) = style.font_family.as_deref().or(style.font.as_deref()) {
        if !family.trim().is_empty() {
            return font_query_for_family(family, style.font_weight);
        }
    }

    font_query_for_family("", style.font_weight)
}

fn font_query_for_family(family: &str, font_weight: u16) -> FontQuery {
    FontQuery {
        families: vec![FontFamilyName::from(family)],
        weight: FontWeight::clamped(UiResolvedStyle::normalized_font_weight(font_weight)),
        style: FontStyle::Normal,
        stretch: FontStretch::NORMAL,
    }
}

pub(crate) fn annotate_fallback_font_ids(
    run: &mut ShapedGlyphRun,
    primary: FontFaceId,
    query: &FontQuery,
    font_database: &FontDatabase,
    composite: Option<&CompositeFontDescriptor>,
) {
    let source_text = run.source_text.clone();
    let source_start = run.source_range.start;

    for line in &mut run.lines {
        let mut previous_cluster: Option<(UiTextRange, FontFaceId)> = None;
        for glyph in &mut line.glyphs {
            if !glyph.cluster_flags.cluster_start {
                if let Some((range, face)) = previous_cluster {
                    if range == glyph.source_range {
                        glyph.font_id = Some(face);
                        continue;
                    }
                }
            }

            let face = resolve_glyph_face(
                glyph,
                &source_text,
                source_start,
                primary,
                query,
                font_database,
                composite,
            );
            glyph.font_id = Some(face);
            previous_cluster = Some((glyph.source_range, face));
        }
    }
}

fn resolve_glyph_face(
    glyph: &ShapedGlyph,
    source_text: &str,
    source_start: usize,
    primary: FontFaceId,
    query: &FontQuery,
    font_database: &FontDatabase,
    composite: Option<&CompositeFontDescriptor>,
) -> FontFaceId {
    let codepoints = codepoints_for_range(source_text, source_start, glyph.source_range);
    let script = script_for_glyph(glyph, &codepoints);
    font_database.resolve_fallback_face_for_cluster(primary, script, &codepoints, query, composite)
}

fn codepoints_for_range(
    source_text: &str,
    source_start: usize,
    source_range: UiTextRange,
) -> Vec<char> {
    let local_start = source_range.start.saturating_sub(source_start);
    let local_end = source_range.end.saturating_sub(source_start);
    if local_start > local_end
        || local_end > source_text.len()
        || !source_text.is_char_boundary(local_start)
        || !source_text.is_char_boundary(local_end)
    {
        return Vec::new();
    }

    source_text[local_start..local_end].chars().collect()
}

fn script_for_glyph(glyph: &ShapedGlyph, codepoints: &[char]) -> FontScript {
    match glyph.script.iso15924.as_str() {
        "Latn" => FontScript::Latin,
        "Cyrl" => FontScript::Cyrillic,
        "Grek" => FontScript::Greek,
        "Hani" => FontScript::Han,
        "Hira" => FontScript::Hiragana,
        "Kana" => FontScript::Katakana,
        "Hang" => FontScript::Hangul,
        "Arab" => FontScript::Arabic,
        "Hebr" => FontScript::Hebrew,
        "Deva" => FontScript::Devanagari,
        _ => codepoints
            .first()
            .map(|codepoint| FontScript::Other(*codepoint as u32))
            .unwrap_or(FontScript::Other(0)),
    }
}
