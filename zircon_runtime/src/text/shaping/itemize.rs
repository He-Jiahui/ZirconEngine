use std::ops::Range;

use unicode_segmentation::UnicodeSegmentation;

use crate::core::framework::text::TextDirection;
use crate::text::{
    BackendShapeRequest, FontFaceId, HardLine, InstancedFaceId, ShapedGlyph,
    ShapedGlyphClusterFlags, ShapedGlyphScript, TextRange, VerticalMode,
};

use super::bidi::{BidiInvariantError, BidiParagraph};
use super::fallback_spans::FallbackTextSpan;
use super::script_segment::ParagraphTextAnalysis;
use super::vertical::{vertical_shape_orientation, VerticalShapeOrientation};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub(super) enum ItemizationError {
    #[error("text itemization source range is invalid: {range:?}")]
    InvalidSourceRange { range: TextRange },
    #[error("text itemization has no fallback span for: {range:?}")]
    MissingFallbackSpan { range: TextRange },
    #[error("text itemization bidi invariant failed: {0:?}")]
    BidiInvariant(BidiInvariantError),
}

impl From<BidiInvariantError> for ItemizationError {
    fn from(error: BidiInvariantError) -> Self {
        Self::BidiInvariant(error)
    }
}

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
    analysis: &ParagraphTextAnalysis,
    bidi: &BidiParagraph<'_>,
    vertical_mode: Option<VerticalMode>,
) -> Result<Vec<LogicalSegment>, ItemizationError> {
    let line_source_range = TextRange {
        start: line_range.start,
        end: line_range.end,
    };
    let line_text = text
        .get(line_range.clone())
        .ok_or(ItemizationError::InvalidSourceRange {
            range: line_source_range,
        })?;
    let mut segments = Vec::<LogicalSegment>::new();
    for (relative_start, cluster_text) in line_text.grapheme_indices(true) {
        let range = TextRange {
            start: line_range.start + relative_start,
            end: line_range.start + relative_start + cluster_text.len(),
        };
        let span = fallback_span_for_range(fallback_spans, range)
            .ok_or(ItemizationError::MissingFallbackSpan { range })?;
        let face = span.resolution.face();
        let bidi_level = bidi.level_for_range(range)?;
        let direction = direction_for_bidi_level(bidi_level);
        let script = analysis.shaped_script_for_range(range);
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
    Ok(segments)
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
    analysis: &ParagraphTextAnalysis,
) -> Result<Option<ShapedGlyph>, ItemizationError> {
    if line.separator.is_empty() {
        return Ok(None);
    }
    let local_range = TextRange {
        start: line.separator.start,
        end: line.separator.end,
    };
    let cluster_text = request
        .text
        .get(line.separator.clone())
        .ok_or(ItemizationError::InvalidSourceRange { range: local_range })?;
    let bidi_level = bidi.level_for_range(local_range)?;
    let direction = direction_for_bidi_level(bidi_level);
    Ok(Some(ShapedGlyph {
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
            line_break: crate::text::ShapedGlyphLineBreakReceipt::mandatory_control(),
            ..ShapedGlyphClusterFlags::default()
        },
        rotation: crate::text::ShapedGlyphRotation::None,
        script: analysis.shaped_script_for_range(local_range),
    }))
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
#[path = "tests/itemize.rs"]
mod tests;
