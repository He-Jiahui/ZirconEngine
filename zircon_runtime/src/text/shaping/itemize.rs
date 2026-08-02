use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::{
    BackendShapeRequest, FontFaceId, HardLine, InstancedFaceId, ShapedGlyph,
    ShapedGlyphClusterFlags, ShapedGlyphScript, TextRange, VerticalMode,
};

use super::bidi::BidiParagraph;
use super::fallback_spans::FallbackTextSpan;
use super::script_segment::{script_for_range, shaped_script_for_cluster, ScriptSegment};
use super::vertical::{vertical_shape_orientation, VerticalShapeOrientation};

#[derive(Clone, Copy)]
pub(super) struct LogicalSegment {
    pub(super) range: TextRange,
    pub(super) face: FontFaceId,
    pub(super) instance: Option<InstancedFaceId>,
    pub(super) direction: TextDirection,
    pub(super) bidi_level: u8,
    pub(super) script: ShapedGlyphScript,
    pub(super) vertical_orientation: VerticalShapeOrientation,
}

pub(super) fn logical_segments_for_line(
    text: &str,
    line_range: Range<usize>,
    fallback_spans: &[FallbackTextSpan],
    scripts: &[ScriptSegment],
    bidi: &BidiParagraph<'_>,
    vertical_mode: Option<VerticalMode>,
) -> Option<Vec<LogicalSegment>> {
    let line_text = text.get(line_range.clone())?;
    let mut segments = Vec::<LogicalSegment>::new();
    for (relative_start, cluster_text) in line_text.grapheme_indices(true) {
        let range = TextRange {
            start: line_range.start + relative_start,
            end: line_range.start + relative_start + cluster_text.len(),
        };
        let span = fallback_span_for_range(fallback_spans, range)?;
        let face = span.face?;
        let bidi_level = bidi.level_for_range(range);
        let direction = direction_for_bidi_level(bidi_level);
        let script = shaped_script_for_cluster(cluster_text, script_for_range(scripts, range));
        let vertical_orientation = vertical_mode
            .map(|mode| vertical_shape_orientation(mode, cluster_text))
            .unwrap_or(VerticalShapeOrientation::Upright);
        if let Some(previous) = segments.last_mut() {
            if previous.range.end == range.start
                && previous.face == face
                && previous.instance == span.instance
                && previous.direction == direction
                && previous.bidi_level == bidi_level
                && previous.script == script
                && previous.vertical_orientation == vertical_orientation
            {
                previous.range.end = range.end;
                continue;
            }
        }
        segments.push(LogicalSegment {
            range,
            face,
            instance: span.instance,
            direction,
            bidi_level,
            script,
            vertical_orientation,
        });
    }
    Some(segments)
}

fn fallback_span_for_range(
    fallback_spans: &[FallbackTextSpan],
    range: TextRange,
) -> Option<&FallbackTextSpan> {
    fallback_spans
        .get(fallback_spans.partition_point(|span| span.range.end <= range.start))
        .filter(|span| span.range.start <= range.start && span.range.end >= range.end)
}

pub(super) fn virtual_hard_break_glyph(
    request: BackendShapeRequest<'_>,
    line: &HardLine,
    bidi: &BidiParagraph<'_>,
    scripts: &[ScriptSegment],
) -> Option<ShapedGlyph> {
    if line.separator.is_empty() {
        return None;
    }
    let cluster_text = request.text.get(line.separator.clone())?;
    let local_range = TextRange {
        start: line.separator.start,
        end: line.separator.end,
    };
    let bidi_level = bidi.level_for_range(local_range);
    let direction = direction_for_bidi_level(bidi_level);
    Some(ShapedGlyph {
        glyph_id: 0,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange {
            start: request.source_range.start + line.separator.start,
            end: request.source_range.start + line.separator.end,
        },
        visual_range: TextRange {
            start: line.separator.start.saturating_sub(line.content.start),
            end: line.separator.end.saturating_sub(line.content.start),
        },
        advance: 0.0,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction,
        bidi_level,
        cluster_flags: ShapedGlyphClusterFlags {
            cluster_start: true,
            rtl: matches!(direction, TextDirection::RightToLeft),
            whitespace: cluster_text.chars().any(char::is_whitespace),
            mandatory_break: true,
            virtual_glyph: true,
            ..ShapedGlyphClusterFlags::default()
        },
        rotation: crate::text::ShapedGlyphRotation::None,
        script: shaped_script_for_cluster(cluster_text, script_for_range(scripts, local_range)),
    })
}

fn direction_for_bidi_level(level: u8) -> TextDirection {
    if level % 2 == 1 {
        TextDirection::RightToLeft
    } else {
        TextDirection::LeftToRight
    }
}

pub(super) fn restore_backend_cluster_logical_order<T>(
    glyphs: &mut [T],
    direction: TextDirection,
    source_offset: impl Fn(&T) -> usize + Copy,
) -> Option<()> {
    let offsets_are_monotonic = glyphs.windows(2).all(|glyphs| match direction {
        TextDirection::RightToLeft => source_offset(&glyphs[0]) >= source_offset(&glyphs[1]),
        TextDirection::Auto | TextDirection::LeftToRight | TextDirection::Mixed => {
            source_offset(&glyphs[0]) <= source_offset(&glyphs[1])
        }
    });
    offsets_are_monotonic.then_some(())?;
    if !matches!(direction, TextDirection::RightToLeft) {
        return Some(());
    }

    glyphs.reverse();
    let mut cluster_start = 0;
    while cluster_start < glyphs.len() {
        let offset = source_offset(&glyphs[cluster_start]);
        let cluster_len =
            glyphs[cluster_start..].partition_point(|glyph| source_offset(glyph) == offset);
        let cluster_end = cluster_start + cluster_len;
        glyphs[cluster_start..cluster_end].reverse();
        cluster_start = cluster_end;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use crate::core::framework::text::TextDirection;

    use super::restore_backend_cluster_logical_order;

    #[derive(Clone, Copy)]
    struct BackendGlyph {
        source_offset: usize,
        glyph_id: u32,
    }

    #[test]
    fn rtl_backend_clusters_restore_logical_order_without_reversing_cluster_glyphs() {
        let mut glyphs = vec![
            backend_glyph(4, 40),
            backend_glyph(2, 20),
            backend_glyph(2, 21),
            backend_glyph(0, 10),
        ];

        restore_backend_cluster_logical_order(&mut glyphs, TextDirection::RightToLeft, |glyph| {
            glyph.source_offset
        })
        .expect("monotonic RTL clusters restore to logical order");

        assert_eq!(
            glyphs
                .iter()
                .map(|glyph| (glyph.source_offset, glyph.glyph_id))
                .collect::<Vec<_>>(),
            vec![(0, 10), (2, 20), (2, 21), (4, 40)]
        );
    }

    const fn backend_glyph(source_offset: usize, glyph_id: u32) -> BackendGlyph {
        BackendGlyph {
            source_offset,
            glyph_id,
        }
    }
}
