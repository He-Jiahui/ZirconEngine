use unicode_segmentation::UnicodeSegmentation;

use crate::ui::layout::UiFrame;

use super::{
    UiEditableTextState, UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextCaret,
    UiTextCaretAffinity, UiTextPaintDecoration, UiTextPaintDecorationKind, UiTextRange,
    UiTextWritingMode,
};

const TEXT_SELECTION_COLOR: &str = "#4d89ff66";
const TEXT_CARET_COLOR: &str = "#e8eef7";
const TEXT_COMPOSITION_UNDERLINE_COLOR: &str = "#4d89ff";
const TEXT_CARET_WIDTH: f32 = 1.0;
const TEXT_COMPOSITION_UNDERLINE_HEIGHT: f32 = 2.0;

pub(super) fn editable_text_decorations(
    layout: &UiResolvedTextLayout,
    editable: &UiEditableTextState,
) -> Vec<UiTextPaintDecoration> {
    let mut decorations = Vec::new();
    if let Some(selection) = editable.selection.as_ref() {
        let range = selection.range();
        if range.start < range.end {
            for frame in text_range_frames(layout, range, TextDecorationMetric::Selection) {
                decorations.push(UiTextPaintDecoration::selection(
                    range,
                    frame,
                    TEXT_SELECTION_COLOR,
                ));
            }
        }
    }

    if let Some(composition) = editable.composition.as_ref() {
        for frame in text_range_frames(
            layout,
            composition.range,
            TextDecorationMetric::CompositionUnderline,
        ) {
            decorations.push(UiTextPaintDecoration::composition_underline(
                composition.range,
                frame,
                TEXT_COMPOSITION_UNDERLINE_COLOR,
            ));
        }
    }

    if let Some(frame) = caret_frame(layout, &editable.caret) {
        decorations.push(UiTextPaintDecoration {
            kind: UiTextPaintDecorationKind::Caret,
            range: UiTextRange {
                start: editable.caret.offset,
                end: editable.caret.offset,
            },
            frame,
            color: TEXT_CARET_COLOR.to_string(),
            thickness: TEXT_CARET_WIDTH,
        });
    }
    decorations
}

#[derive(Clone, Copy)]
enum TextDecorationMetric {
    Selection,
    CompositionUnderline,
}

#[derive(Clone, Copy)]
enum SourceVisualBias {
    Leading,
    Trailing,
}

fn text_range_frames(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    metric: TextDecorationMetric,
) -> Vec<UiFrame> {
    let mut frames = Vec::new();
    for line in &layout.lines {
        for run in &line.runs {
            let start = range.start.max(run.source_range.start);
            let end = range.end.min(run.source_range.end);
            if start >= end {
                continue;
            }
            let visual_start =
                run_visual_offset_for_source_offset(run, start, SourceVisualBias::Leading);
            let visual_end =
                run_visual_offset_for_source_offset(run, end, SourceVisualBias::Trailing);
            if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
                let y0 = visual_y(line, visual_start);
                let y1 = visual_y(line, visual_end);
                let y = y0.min(y1);
                let height = (y1 - y0).abs().max(TEXT_CARET_WIDTH);
                let (x, width) = match metric {
                    TextDecorationMetric::Selection => (line.frame.x, line.frame.width),
                    TextDecorationMetric::CompositionUnderline => (
                        line.frame.right() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                        TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                    ),
                };
                frames.push(UiFrame::new(x, y, width, height));
                continue;
            }
            let x0 = visual_x(line, visual_start);
            let x1 = visual_x(line, visual_end);
            let (y, height) = match metric {
                TextDecorationMetric::Selection => (line.frame.y, line.frame.height),
                TextDecorationMetric::CompositionUnderline => (
                    line.frame.bottom() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                    TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                ),
            };
            frames.push(UiFrame::new(
                x0.min(x1),
                y,
                (x1 - x0).abs().max(TEXT_CARET_WIDTH),
                height,
            ));
        }
    }
    frames
}

fn caret_frame(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<UiFrame> {
    let offset = caret.offset;
    let line = layout
        .lines
        .iter()
        .find(|line| offset >= line.source_range.start && offset <= line.source_range.end)
        .or_else(|| layout.lines.last())?;
    let bias = match caret.affinity {
        UiTextCaretAffinity::Upstream => SourceVisualBias::Leading,
        UiTextCaretAffinity::Downstream => SourceVisualBias::Trailing,
    };
    let visual_offset = line
        .runs
        .iter()
        .find_map(|run| {
            (offset >= run.source_range.start && offset <= run.source_range.end)
                .then(|| run_visual_offset_for_source_offset(run, offset, bias))
        })
        .unwrap_or(line.visual_range.end);
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        return Some(UiFrame::new(
            line.frame.x,
            visual_y(line, visual_offset),
            line.frame.width.max(TEXT_CARET_WIDTH),
            TEXT_CARET_WIDTH,
        ));
    }
    Some(UiFrame::new(
        visual_x(line, visual_offset),
        line.frame.y,
        TEXT_CARET_WIDTH,
        line.frame.height,
    ))
}

fn run_visual_offset_for_source_offset(
    run: &UiResolvedTextRun,
    offset: usize,
    bias: SourceVisualBias,
) -> usize {
    if offset <= run.source_range.start {
        return run.visual_range.start;
    }
    if offset >= run.source_range.end {
        return run.visual_range.end;
    }

    let source_len = run.source_range.end.saturating_sub(run.source_range.start);
    if source_len == 0 {
        return match bias {
            SourceVisualBias::Leading => run.visual_range.start,
            SourceVisualBias::Trailing => run.visual_range.end,
        };
    }

    let run_visual_len = run.visual_range.end.saturating_sub(run.visual_range.start);
    let local_source_offset = offset.saturating_sub(run.source_range.start);
    let local_visual_offset = if source_len == run.text.len() {
        match bias {
            SourceVisualBias::Leading => grapheme_floor(run.text.as_str(), local_source_offset),
            SourceVisualBias::Trailing => grapheme_ceil(run.text.as_str(), local_source_offset),
        }
    } else {
        non_isomorphic_local_visual_offset(run.text.as_str(), local_source_offset, source_len, bias)
    };

    run.visual_range.start + local_visual_offset.min(run_visual_len)
}

fn non_isomorphic_local_visual_offset(
    text: &str,
    local_source_offset: usize,
    source_len: usize,
    bias: SourceVisualBias,
) -> usize {
    let grapheme_count = text.graphemes(true).count();
    if grapheme_count == 0 {
        return 0;
    }

    let progress = local_source_offset.min(source_len) as f32 / source_len as f32;
    let visual_index = match bias {
        SourceVisualBias::Leading => (progress * grapheme_count as f32).floor() as usize,
        SourceVisualBias::Trailing => (progress * grapheme_count as f32).ceil() as usize,
    };
    grapheme_boundary_by_index(text, visual_index.min(grapheme_count))
}

fn grapheme_boundary_by_index(text: &str, index: usize) -> usize {
    if index == 0 {
        return 0;
    }
    text.grapheme_indices(true)
        .nth(index)
        .map(|(start, _)| start)
        .unwrap_or(text.len())
}

fn visual_y(line: &UiResolvedTextLine, visual_offset: usize) -> f32 {
    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count();
    let before_units = text[..offset].graphemes(true).count();
    if line.glyph_advances.len() == total_units {
        return line.frame.y
            + line
                .glyph_advances
                .iter()
                .take(before_units)
                .map(|advance| sanitized_advance(*advance))
                .sum::<f32>();
    }

    let total_units = total_units.max(1) as f32;
    let before_units = before_units as f32;
    line.frame.y + (line.frame.height.max(0.0) * before_units / total_units)
}

fn visual_x(line: &UiResolvedTextLine, visual_offset: usize) -> f32 {
    let text = line.text.as_str();
    let offset = grapheme_floor(text, visual_offset.min(text.len()));
    let total_units = text.graphemes(true).count();
    let before_units = text[..offset].graphemes(true).count();
    if line.glyph_advances.len() == total_units {
        return line.frame.x
            + line
                .glyph_advances
                .iter()
                .take(before_units)
                .map(|advance| sanitized_advance(*advance))
                .sum::<f32>();
    }

    let total_units = total_units.max(1) as f32;
    let before_units = before_units as f32;
    line.frame.x + (line.frame.width.max(0.0) * before_units / total_units)
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}

fn grapheme_floor(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset > 0 && !text.is_char_boundary(offset) {
        offset -= 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return start;
        }
        if start >= offset {
            break;
        }
    }
    offset
}

fn grapheme_ceil(text: &str, offset: usize) -> usize {
    let mut offset = offset.min(text.len());
    while offset < text.len() && !text.is_char_boundary(offset) {
        offset += 1;
    }
    for (start, grapheme) in text.grapheme_indices(true) {
        let end = start + grapheme.len();
        if start < offset && offset < end {
            return end;
        }
        if start >= offset {
            break;
        }
    }
    offset
}
