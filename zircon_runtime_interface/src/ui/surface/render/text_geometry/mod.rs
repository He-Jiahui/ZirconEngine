use crate::ui::layout::UiFrame;

use super::{
    UiEditableTextState, UiResolvedTextLayout, UiResolvedTextLine, UiTextCaret,
    UiTextCaretAffinity, UiTextPaintDecoration, UiTextPaintDecorationKind, UiTextRange,
    UiTextWritingMode,
};

mod source_map;

pub use source_map::{UiTextLineSourceMap, UiTextVisualBoundaryBias, UiTextVisualSpan};

#[cfg(test)]
mod source_map_tests;

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

fn text_range_frames(
    layout: &UiResolvedTextLayout,
    range: UiTextRange,
    metric: TextDecorationMetric,
) -> Vec<UiFrame> {
    let mut frames = Vec::new();
    for line in &layout.lines {
        let map = UiTextLineSourceMap::new(line);
        for span in map.visual_spans_for_source_range(range) {
            let start = map.advance_to_visual_offset(span.visual_range.start);
            let end = map.advance_to_visual_offset(span.visual_range.end);
            frames.push(decoration_frame(layout, line, start, end, metric));
        }
    }
    frames
}

fn decoration_frame(
    layout: &UiResolvedTextLayout,
    line: &UiResolvedTextLine,
    start: f32,
    end: f32,
    metric: TextDecorationMetric,
) -> UiFrame {
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        let (x, width) = match metric {
            TextDecorationMetric::Selection => (line.frame.x, line.frame.width),
            TextDecorationMetric::CompositionUnderline => (
                line.frame.right() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
                TEXT_COMPOSITION_UNDERLINE_HEIGHT,
            ),
        };
        return UiFrame::new(
            x,
            line.frame.y + start.min(end),
            width,
            (end - start).abs().max(TEXT_CARET_WIDTH),
        );
    }

    let (y, height) = match metric {
        TextDecorationMetric::Selection => (line.frame.y, line.frame.height),
        TextDecorationMetric::CompositionUnderline => (
            line.frame.bottom() - TEXT_COMPOSITION_UNDERLINE_HEIGHT,
            TEXT_COMPOSITION_UNDERLINE_HEIGHT,
        ),
    };
    UiFrame::new(
        line.frame.x + start.min(end),
        y,
        (end - start).abs().max(TEXT_CARET_WIDTH),
        height,
    )
}

fn caret_frame(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<UiFrame> {
    let line = caret_line(layout, caret)?;
    let map = UiTextLineSourceMap::new(line);
    let main_offset = map.advance_to_visual_offset(map.visual_offset_for_caret(caret));
    if matches!(layout.writing_mode, UiTextWritingMode::VerticalRl) {
        return Some(UiFrame::new(
            line.frame.x,
            line.frame.y + main_offset,
            line.frame.width.max(TEXT_CARET_WIDTH),
            TEXT_CARET_WIDTH,
        ));
    }
    Some(UiFrame::new(
        line.frame.x + main_offset,
        line.frame.y,
        TEXT_CARET_WIDTH,
        line.frame.height.max(TEXT_CARET_WIDTH),
    ))
}

fn caret_line<'a>(
    layout: &'a UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<&'a UiResolvedTextLine> {
    let matching = |line: &&UiResolvedTextLine| {
        caret.offset >= line.source_range.start && caret.offset <= line.source_range.end
    };
    match caret.affinity {
        UiTextCaretAffinity::Upstream => layout.lines.iter().find(matching),
        UiTextCaretAffinity::Downstream => layout.lines.iter().rev().find(matching),
    }
    .or_else(|| {
        layout
            .lines
            .first()
            .filter(|line| caret.offset < line.source_range.start)
    })
    .or_else(|| layout.lines.last())
}
