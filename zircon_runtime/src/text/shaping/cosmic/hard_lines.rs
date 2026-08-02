use crate::text::{BackendShapeRequest, HardLine, ShapedTextLine, TextRange};

use super::resolved_line_height;
use crate::text::shaping::bidi::BidiParagraph;
use crate::text::shaping::itemize::virtual_hard_break_glyph;
use crate::text::shaping::script_segment::ScriptSegment;

pub(super) fn normalize_cosmic_hard_lines(
    request: BackendShapeRequest<'_>,
    bidi: &BidiParagraph<'_>,
    scripts: &[ScriptSegment],
    hard_lines: &[HardLine],
    raw_lines: Vec<ShapedTextLine>,
) -> Vec<ShapedTextLine> {
    let fallback_line_height = resolved_line_height(request);
    let fallback_baseline = request.style.font_size.max(1.0) * 0.8;
    let mut has_backend_metrics = vec![false; hard_lines.len()];
    let mut lines = hard_lines
        .iter()
        .enumerate()
        .map(|(line_index, hard_line)| {
            let source_range = hard_line.source_range();
            ShapedTextLine {
                line_index,
                source_range: TextRange {
                    start: request.source_range.start + source_range.start,
                    end: request.source_range.start + source_range.end,
                },
                visual_range: TextRange {
                    start: 0,
                    end: source_range.end.saturating_sub(source_range.start),
                },
                measured_width: 0.0,
                baseline: fallback_baseline,
                line_height: fallback_line_height,
                glyphs: Vec::new(),
            }
        })
        .collect::<Vec<_>>();

    for raw_line in raw_lines {
        let baseline = raw_line.baseline;
        let line_height = raw_line.line_height;
        for glyph in raw_line.glyphs {
            let Some(local_start) = glyph
                .source_range
                .start
                .checked_sub(request.source_range.start)
            else {
                continue;
            };
            let Some(local_end) = glyph
                .source_range
                .end
                .checked_sub(request.source_range.start)
            else {
                continue;
            };
            let line_index = hard_lines.partition_point(|line| line.content.end <= local_start);
            let Some(hard_line) = hard_lines.get(line_index) else {
                continue;
            };
            if local_start < hard_line.content.start
                || local_end > hard_line.content.end
                || local_start >= local_end
            {
                continue;
            }
            let line = &mut lines[line_index];
            if has_backend_metrics[line_index] {
                line.baseline = line.baseline.max(baseline);
                line.line_height = line.line_height.max(line_height);
            } else {
                line.baseline = baseline;
                line.line_height = line_height.max(fallback_line_height);
                has_backend_metrics[line_index] = true;
            }
            line.glyphs.push(glyph);
        }
    }

    for (hard_line, line) in hard_lines.iter().zip(&mut lines) {
        if let Some(separator) = virtual_hard_break_glyph(request, hard_line, bidi, scripts) {
            line.glyphs.push(separator);
        }
        line.glyphs
            .sort_by_key(|glyph| (glyph.source_range.start, glyph.source_range.end));
        let mut cursor = 0.0_f32;
        for glyph in &mut line.glyphs {
            glyph.x = cursor;
            cursor += glyph.advance.max(0.0);
        }
        line.measured_width = cursor;
    }
    lines
}
