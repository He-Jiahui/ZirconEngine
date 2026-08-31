use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextCaret,
        UiTextCaretAffinity, UiTextLineSourceMap, UiTextRange, UiTextWritingMode,
    },
};

use crate::text::font::{shared_font_collection_snapshot, FontCollectionSnapshot};
use crate::text::{
    resolve_resolved_text_glyph_artifact, resolved_text_glyph_artifact_caret_advance,
    resolved_text_glyph_artifact_range_advance_spans,
};

#[path = "geometry/source_metrics.rs"]
mod source_metrics;

use source_metrics::{SourceLineGeometry, SourceMeasureContext};

#[cfg(test)]
use source_metrics::{line_accepts_source_measure, source_prefix_range_for_visual_offset};

const TEXT_CARET_WIDTH: f32 = 1.0;

pub(crate) fn caret_frame_for_text_layout(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<UiFrame> {
    caret_frame_for_text_layout_inner(layout, caret, None)
}

pub(crate) fn caret_frame_for_text_layout_with_source_metrics(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    text: &str,
    style: &UiResolvedStyle,
) -> Option<UiFrame> {
    let font_collection = shared_font_collection_snapshot();
    caret_frame_for_text_layout_with_font_collection(layout, caret, text, style, &font_collection)
}

pub(crate) fn caret_frame_for_text_layout_with_font_collection(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    text: &str,
    style: &UiResolvedStyle,
    font_collection: &FontCollectionSnapshot,
) -> Option<UiFrame> {
    caret_frame_for_text_layout_inner(
        layout,
        caret,
        Some(SourceMeasureContext {
            text,
            style,
            font_collection,
        }),
    )
}

fn caret_frame_for_text_layout_inner(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> Option<UiFrame> {
    let (line_index, line) = caret_line(layout, caret)?;
    let source_map = UiTextLineSourceMap::new(line);
    let visual_offset = source_map.visual_offset_for_caret(caret);
    let artifact_advance = resolved_glyph_artifact(layout)
        .as_deref()
        .and_then(|artifact| {
            resolved_text_glyph_artifact_caret_advance(artifact, line_index, line, caret)
        });
    let resolved_advance = artifact_advance.or_else(|| {
        SourceLineGeometry::for_line(layout, line, measure_context)
            .map(|geometry| geometry.caret_advance(caret))
    });
    if is_vertical_rl(layout) {
        return Some(UiFrame::new(
            line.frame.x,
            visual_y(line, &source_map, visual_offset, resolved_advance),
            line.frame.width.max(TEXT_CARET_WIDTH),
            TEXT_CARET_WIDTH,
        ));
    }
    Some(UiFrame::new(
        visual_x(line, &source_map, visual_offset, resolved_advance),
        line.frame.y,
        TEXT_CARET_WIDTH,
        line.frame.height.max(TEXT_CARET_WIDTH),
    ))
}

pub(crate) fn text_range_frames_for_text_layout(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
) -> Vec<UiFrame> {
    text_range_frames_for_text_layout_inner(layout, range, None)
}

pub(crate) fn text_range_frames_for_text_layout_with_source_metrics(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    text: &str,
    style: &UiResolvedStyle,
) -> Vec<UiFrame> {
    let font_collection = shared_font_collection_snapshot();
    text_range_frames_for_text_layout_with_font_collection(
        layout,
        range,
        text,
        style,
        &font_collection,
    )
}

pub(crate) fn text_range_frames_for_text_layout_with_font_collection(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    text: &str,
    style: &UiResolvedStyle,
    font_collection: &FontCollectionSnapshot,
) -> Vec<UiFrame> {
    text_range_frames_for_text_layout_inner(
        layout,
        range,
        Some(SourceMeasureContext {
            text,
            style,
            font_collection,
        }),
    )
}

pub(super) fn source_metrics_caret_at_advance(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    visual_advance: f32,
    text: &str,
    style: &UiResolvedStyle,
    font_collection: &FontCollectionSnapshot,
) -> Option<(UiTextCaret, usize)> {
    SourceLineGeometry::for_line(
        layout,
        line,
        Some(SourceMeasureContext {
            text,
            style,
            font_collection,
        }),
    )?
    .caret_at_advance(visual_advance)
}

fn text_range_frames_for_text_layout_inner(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    measure_context: Option<SourceMeasureContext<'_>>,
) -> Vec<UiFrame> {
    if range.start == range.end {
        return caret_frame_for_text_layout_inner(
            layout,
            &UiTextCaret {
                offset: range.start,
                affinity: UiTextCaretAffinity::Downstream,
            },
            measure_context,
        )
        .into_iter()
        .collect();
    }

    let mut frames = Vec::new();
    let artifact = resolved_glyph_artifact(layout);
    for (line_index, line) in layout.lines.iter().enumerate() {
        if range.start >= line.source_range.end || line.source_range.start >= range.end {
            continue;
        }
        if let Some(spans) = artifact.as_deref().and_then(|artifact| {
            resolved_text_glyph_artifact_range_advance_spans(artifact, line_index, line, range)
        }) {
            for (start, end) in spans {
                frames.push(range_frame(layout, line, start, end));
            }
            continue;
        }
        if let Some((start, end)) = SourceLineGeometry::for_line(layout, line, measure_context)
            .and_then(|geometry| geometry.range_advance_span(range))
        {
            frames.push(range_frame(layout, line, start, end));
            continue;
        }
        let source_map = UiTextLineSourceMap::new(line);
        for span in source_map.visual_spans_for_source_range(range) {
            let visual_start = span.visual_range.start;
            let visual_end = span.visual_range.end;
            let start = resolved_visual_advance(&source_map, visual_start, None);
            let end = resolved_visual_advance(&source_map, visual_end, None);
            frames.push(range_frame(layout, line, start, end));
        }
    }
    frames
}

fn visual_x(
    line: &UiResolvedTextLine,
    source_map: &UiTextLineSourceMap<'_>,
    visual_offset: usize,
    resolved_advance: Option<f32>,
) -> f32 {
    line.frame.x + resolved_visual_advance(source_map, visual_offset, resolved_advance)
}

fn visual_y(
    line: &UiResolvedTextLine,
    source_map: &UiTextLineSourceMap<'_>,
    visual_offset: usize,
    resolved_advance: Option<f32>,
) -> f32 {
    line.frame.y + resolved_visual_advance(source_map, visual_offset, resolved_advance)
}

fn resolved_visual_advance(
    source_map: &UiTextLineSourceMap<'_>,
    visual_offset: usize,
    resolved_advance: Option<f32>,
) -> f32 {
    resolved_advance.unwrap_or_else(|| source_map.advance_to_visual_offset(visual_offset))
}

fn range_frame(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    start: f32,
    end: f32,
) -> UiFrame {
    if is_vertical_rl(layout) {
        return UiFrame::new(
            line.frame.x,
            line.frame.y + start.min(end),
            line.frame.width.max(TEXT_CARET_WIDTH),
            (end - start).abs().max(TEXT_CARET_WIDTH),
        );
    }
    UiFrame::new(
        line.frame.x + start.min(end),
        line.frame.y,
        (end - start).abs().max(TEXT_CARET_WIDTH),
        line.frame.height.max(TEXT_CARET_WIDTH),
    )
}

fn resolved_glyph_artifact(
    layout: &UiResolvedTextLayout,
) -> Option<std::sync::Arc<crate::text::ResolvedTextGlyphArtifact>> {
    layout
        .rich_text_artifact
        .as_ref()
        .and_then(resolve_resolved_text_glyph_artifact)
        .filter(|artifact| artifact.writing_mode == layout.writing_mode)
}

fn is_vertical_rl(layout: &UiResolvedTextLayout) -> bool {
    matches!(layout.writing_mode, UiTextWritingMode::VerticalRl)
}

fn caret_line<'a>(
    layout: &'a UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<(usize, &'a UiResolvedTextLine)> {
    let matching = |line: &&UiResolvedTextLine| {
        caret.offset >= line.source_range.start && caret.offset <= line.source_range.end
    };
    match caret.affinity {
        UiTextCaretAffinity::Upstream => layout
            .lines
            .iter()
            .enumerate()
            .find(|(_, line)| matching(line)),
        UiTextCaretAffinity::Downstream => layout
            .lines
            .iter()
            .enumerate()
            .rev()
            .find(|(_, line)| matching(line)),
    }
    .or_else(|| {
        layout
            .lines
            .first()
            .filter(|line| caret.offset < line.source_range.start)
            .map(|line| (0, line))
    })
    .or_else(|| layout.lines.iter().enumerate().last())
}

#[cfg(test)]
#[path = "geometry/tests/mixed_bidi.rs"]
mod mixed_bidi_tests;

#[cfg(test)]
#[path = "geometry/tests.rs"]
mod tests;
