use crate::ui::{layout::UiFrame, surface::UiTextPreeditClauseKind};

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
const TEXT_COMPOSITION_HIGHLIGHT_COLOR: &str = "#4d89ff24";
const TEXT_COMPOSITION_UNDERLINE_COLOR: &str = "#4d89ff";
const TEXT_COMPOSITION_CONVERTED_UNDERLINE_COLOR: &str = "#72b7f2";
const TEXT_COMPOSITION_TARGET_CONVERTED_UNDERLINE_COLOR: &str = "#42bf77";
const TEXT_COMPOSITION_TARGET_NOT_CONVERTED_UNDERLINE_COLOR: &str = "#e05a5a";
const TEXT_CARET_WIDTH: f32 = 1.0;
const TEXT_COMPOSITION_UNDERLINE_HEIGHT: f32 = 2.0;

pub(super) fn editable_text_decorations(
    layout: &UiResolvedTextLayout,
    editable: &UiEditableTextState,
) -> Vec<UiTextPaintDecoration> {
    let mut decorations = Vec::new();
    let composition = editable.composition.as_ref();
    let mut range_decorations = Vec::with_capacity(
        usize::from(editable.selection.is_some())
            + composition.map_or(0, |value| value.preedit_clauses.len().max(1) + 1),
    );

    if let Some(composition) = composition {
        range_decorations.push(TextRangeDecoration::composition_highlight(
            composition.range,
        ));
    }

    if let Some(selection) = editable.selection.as_ref() {
        let range = selection.range();
        if range.start < range.end {
            range_decorations.push(TextRangeDecoration::selection(range));
        }
    }

    if let Some(composition) = composition {
        if composition.preedit_clauses.is_empty() {
            range_decorations.push(TextRangeDecoration::composition_underline(
                composition.range,
                TEXT_COMPOSITION_UNDERLINE_COLOR,
            ));
        } else {
            for clause in &composition.preedit_clauses {
                let start = composition
                    .range
                    .start
                    .saturating_add(clause.range.start_byte as usize)
                    .min(composition.range.end);
                let range = UiTextRange {
                    start,
                    end: composition
                        .range
                        .start
                        .saturating_add(clause.range.end_byte as usize)
                        .min(composition.range.end)
                        .max(start),
                };
                range_decorations.push(TextRangeDecoration::composition_underline(
                    range,
                    composition_underline_color(clause.kind),
                ));
            }
        }
    }

    append_range_decorations(&mut decorations, layout, &range_decorations);

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
enum TextRangeDecorationKind {
    Selection,
    CompositionHighlight,
    CompositionUnderline,
}

#[derive(Clone, Copy)]
struct TextRangeDecoration {
    range: UiTextRange,
    color: &'static str,
    kind: TextRangeDecorationKind,
}

impl TextRangeDecoration {
    const fn selection(range: UiTextRange) -> Self {
        Self {
            range,
            color: TEXT_SELECTION_COLOR,
            kind: TextRangeDecorationKind::Selection,
        }
    }

    const fn composition_underline(range: UiTextRange, color: &'static str) -> Self {
        Self {
            range,
            color,
            kind: TextRangeDecorationKind::CompositionUnderline,
        }
    }

    const fn composition_highlight(range: UiTextRange) -> Self {
        Self {
            range,
            color: TEXT_COMPOSITION_HIGHLIGHT_COLOR,
            kind: TextRangeDecorationKind::CompositionHighlight,
        }
    }

    const fn metric(self) -> TextDecorationMetric {
        match self.kind {
            TextRangeDecorationKind::Selection | TextRangeDecorationKind::CompositionHighlight => {
                TextDecorationMetric::Selection
            }
            TextRangeDecorationKind::CompositionUnderline => {
                TextDecorationMetric::CompositionUnderline
            }
        }
    }

    fn paint(self, frame: UiFrame) -> UiTextPaintDecoration {
        match self.kind {
            TextRangeDecorationKind::Selection => {
                UiTextPaintDecoration::selection(self.range, frame, self.color)
            }
            TextRangeDecorationKind::CompositionHighlight => {
                UiTextPaintDecoration::composition_highlight(self.range, frame, self.color)
            }
            TextRangeDecorationKind::CompositionUnderline => {
                UiTextPaintDecoration::composition_underline(self.range, frame, self.color)
            }
        }
    }
}

fn append_range_decorations(
    decorations: &mut Vec<UiTextPaintDecoration>,
    layout: &UiResolvedTextLayout,
    range_decorations: &[TextRangeDecoration],
) {
    if range_decorations.is_empty() {
        return;
    }

    // Reuse each line's cluster projection and exact-advance cache across the
    // selection and every IME clause while retaining declaration order.
    let source_maps = layout
        .lines
        .iter()
        .map(UiTextLineSourceMap::new)
        .collect::<Vec<_>>();
    for decoration in range_decorations {
        for (line, source_map) in layout.lines.iter().zip(&source_maps) {
            for span in source_map.visual_spans_for_source_range(decoration.range) {
                let start = source_map.advance_to_visual_offset(span.visual_range.start);
                let end = source_map.advance_to_visual_offset(span.visual_range.end);
                decorations.push(decoration.paint(decoration_frame(
                    layout,
                    line,
                    start,
                    end,
                    decoration.metric(),
                )));
            }
        }
    }
}

fn composition_underline_color(kind: UiTextPreeditClauseKind) -> &'static str {
    match kind {
        UiTextPreeditClauseKind::Input => TEXT_COMPOSITION_UNDERLINE_COLOR,
        UiTextPreeditClauseKind::Converted => TEXT_COMPOSITION_CONVERTED_UNDERLINE_COLOR,
        UiTextPreeditClauseKind::TargetConverted => {
            TEXT_COMPOSITION_TARGET_CONVERTED_UNDERLINE_COLOR
        }
        UiTextPreeditClauseKind::TargetNotConverted => {
            TEXT_COMPOSITION_TARGET_NOT_CONVERTED_UNDERLINE_COLOR
        }
    }
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
