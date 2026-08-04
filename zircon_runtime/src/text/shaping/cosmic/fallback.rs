use crate::core::framework::text::TextDirection;
use crate::text::{
    BackendShapeRequest, ShapedGlyph, ShapedGlyphRotation, ShapedGlyphRun, ShapedTextLine,
    TextRange,
};
use unicode_segmentation::UnicodeSegmentation;

use super::super::bidi::BidiParagraph;
use super::super::itemize::virtual_hard_break_glyph;
use super::super::line_break::LineBreakOpportunityMap;
use super::super::normalize::ShapingTextView;
use super::super::script_segment::{script_for_range, script_segments, shaped_script_for_cluster};
use super::{absolute_range, cluster_flags, resolved_line_height};

const DEFAULT_FALLBACK_ADVANCE_EM: f32 = 0.56;

pub(super) fn fallback_shape(
    request: BackendShapeRequest<'_>,
    text_view: &ShapingTextView<'_>,
    bidi: &BidiParagraph<'_>,
) -> ShapedGlyphRun {
    let line_height = resolved_line_height(request);
    let baseline = request.style.font_size.max(1.0) * 0.8;
    let line_breaks = LineBreakOpportunityMap::new(text_view.shaping_text());
    let scripts = script_segments(text_view.shaping_text());
    let hard_lines = crate::text::hard_lines(text_view.shaping_text());
    let mut lines = Vec::with_capacity(hard_lines.len());
    for (line_index, hard_line) in hard_lines.iter().enumerate() {
        let mut x = 0.0_f32;
        let mut glyphs = Vec::new();
        let content = text_view
            .shaping_text()
            .get(hard_line.content.clone())
            .unwrap_or_default();
        for (relative_start, grapheme) in content.grapheme_indices(true) {
            let visual_start = hard_line.content.start + relative_start;
            let visual_end = visual_start + grapheme.len();
            let advance = fallback_grapheme_advance(grapheme, request.style.font_size.max(1.0));
            let local_range = TextRange {
                start: visual_start,
                end: visual_end,
            };
            let bidi_level = bidi.level_for_range(local_range);
            let direction = if bidi_level % 2 == 1 {
                TextDirection::RightToLeft
            } else {
                TextDirection::LeftToRight
            };
            glyphs.push(ShapedGlyph {
                glyph_id: synthetic_glyph_id(grapheme),
                font_id: None,
                font_instance_id: None,
                source_range: {
                    let projected =
                        text_view.source_range_for_shaping_range(visual_start..visual_end);
                    absolute_range(request.source_range.start, projected.start, projected.end)
                },
                visual_range: TextRange {
                    start: visual_start.saturating_sub(hard_line.content.start),
                    end: visual_end.saturating_sub(hard_line.content.start),
                },
                advance,
                x,
                y: 0.0,
                offset_x: 0.0,
                offset_y: 0.0,
                direction,
                bidi_level,
                cluster_flags: cluster_flags(
                    grapheme,
                    direction,
                    true,
                    line_breaks.flags_for_cluster(visual_start, visual_end),
                ),
                rotation: ShapedGlyphRotation::None,
                script: shaped_script_for_cluster(
                    grapheme,
                    script_for_range(&scripts, local_range),
                ),
            });
            x += advance;
        }
        if let Some(mut separator) = virtual_hard_break_glyph(request, hard_line, bidi, &scripts) {
            separator.x = x;
            glyphs.push(separator);
        }
        let full_range = hard_line.source_range();
        let projected_range = text_view.source_range_for_shaping_range(full_range.clone());
        lines.push(ShapedTextLine {
            line_index,
            source_range: absolute_range(
                request.source_range.start,
                projected_range.start,
                projected_range.end,
            ),
            visual_range: TextRange {
                start: 0,
                end: full_range.end.saturating_sub(full_range.start),
            },
            measured_width: x,
            baseline,
            line_height,
            glyphs,
        });
    }

    let measured_width = lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    let measured_height = lines.iter().map(|line| line.line_height).sum();

    ShapedGlyphRun {
        source_text: request.shared_source_text(),
        source_range: request.source_range,
        direction: bidi.resolved_base_direction(),
        orientation: request.orientation,
        vertical_mode: request.vertical_mode,
        include_kerning: request.include_kerning,
        measured_width,
        measured_height,
        lines,
    }
}

fn fallback_grapheme_advance(grapheme: &str, font_size: f32) -> f32 {
    if grapheme.chars().all(char::is_whitespace) {
        return font_size * 0.33;
    }
    if grapheme.chars().any(is_wide_fallback_grapheme) {
        return font_size;
    }
    if grapheme
        .chars()
        .all(|ch| matches!(ch, 'i' | 'l' | 'I' | '!' | '|' | '.' | ','))
    {
        return font_size * 0.3;
    }
    if grapheme
        .chars()
        .any(|ch| matches!(ch, 'W' | 'M' | 'w' | 'm'))
    {
        return font_size * 0.85;
    }
    font_size * DEFAULT_FALLBACK_ADVANCE_EM
}

fn is_wide_fallback_grapheme(ch: char) -> bool {
    matches!(
        ch as u32,
        0x1100..=0x11FF
            | 0x2E80..=0xA4CF
            | 0xAC00..=0xD7AF
            | 0xF900..=0xFAFF
            | 0xFE10..=0xFE6F
            | 0xFF00..=0xFFEF
            | 0x1F300..=0x1FAFF
    )
}

fn synthetic_glyph_id(grapheme: &str) -> u32 {
    let mut hash = 2_166_136_261_u32;
    for byte in grapheme.as_bytes() {
        hash ^= *byte as u32;
        hash = hash.wrapping_mul(16_777_619);
    }
    hash.max(1)
}
