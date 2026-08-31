use std::collections::HashMap;

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

    let mut source_maps = TextDecorationLineSourceMaps::new(&layout.lines);
    append_range_decorations_with_source_maps(
        decorations,
        layout,
        range_decorations,
        &mut source_maps,
    );
}

struct TextDecorationLineSourceMaps<'a> {
    lines: &'a [UiResolvedTextLine],
    maps: HashMap<usize, UiTextLineSourceMap<'a>>,
    #[cfg(test)]
    initialized_count: usize,
}

impl<'a> TextDecorationLineSourceMaps<'a> {
    fn new(lines: &'a [UiResolvedTextLine]) -> Self {
        Self {
            lines,
            maps: HashMap::new(),
            #[cfg(test)]
            initialized_count: 0,
        }
    }

    fn for_source_range(
        &mut self,
        line_index: usize,
        range: UiTextRange,
    ) -> Option<(&'a UiResolvedTextLine, &UiTextLineSourceMap<'a>)> {
        let line = self.lines.get(line_index)?;
        if range.start >= line.source_range.end || line.source_range.start >= range.end {
            return None;
        }

        let source_map = match self.maps.entry(line_index) {
            std::collections::hash_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::hash_map::Entry::Vacant(entry) => {
                #[cfg(test)]
                {
                    self.initialized_count += 1;
                }
                entry.insert(UiTextLineSourceMap::new(line))
            }
        };
        Some((line, source_map))
    }

    #[cfg(test)]
    fn initialized_count(&self) -> usize {
        self.initialized_count
    }
}

fn append_range_decorations_with_source_maps(
    decorations: &mut Vec<UiTextPaintDecoration>,
    layout: &UiResolvedTextLayout,
    range_decorations: &[TextRangeDecoration],
    source_maps: &mut TextDecorationLineSourceMaps<'_>,
) {
    // Reuse each touched line's cluster projection and exact-advance cache
    // across the selection and every IME clause while retaining declaration order.
    for decoration in range_decorations {
        for line_index in 0..source_maps.lines.len() {
            let Some((line, source_map)) =
                source_maps.for_source_range(line_index, decoration.range)
            else {
                continue;
            };
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

#[cfg(test)]
mod performance_tests {
    use super::*;
    use crate::ui::surface::{UiResolvedTextRun, UiTextDirection, UiTextRunKind};

    #[test]
    fn localized_selection_and_preedit_share_one_intersecting_line_source_map() {
        let lines = (0..128)
            .map(|line_index| UiResolvedTextLine {
                text: "x".to_string(),
                placement_frame: UiFrame::default(),
                frame: UiFrame::new(0.0, line_index as f32 * 12.0, 8.0, 12.0),
                source_range: UiTextRange {
                    start: line_index,
                    end: line_index + 1,
                },
                visual_range: UiTextRange { start: 0, end: 1 },
                measured_width: 8.0,
                glyph_advances: vec![8.0],
                baseline: 9.0,
                direction: UiTextDirection::LeftToRight,
                runs: vec![UiResolvedTextRun {
                    kind: UiTextRunKind::Plain,
                    text: "x".to_string(),
                    source_range: UiTextRange {
                        start: line_index,
                        end: line_index + 1,
                    },
                    visual_range: UiTextRange { start: 0, end: 1 },
                    direction: UiTextDirection::LeftToRight,
                }],
                ellipsized: false,
            })
            .collect::<Vec<_>>();
        let layout = UiResolvedTextLayout {
            lines,
            ..Default::default()
        };
        let range = UiTextRange { start: 64, end: 65 };
        let mut decorations = Vec::new();
        let mut source_maps = TextDecorationLineSourceMaps::new(&layout.lines);

        append_range_decorations_with_source_maps(
            &mut decorations,
            &layout,
            &[
                TextRangeDecoration::composition_highlight(range),
                TextRangeDecoration::selection(range),
                TextRangeDecoration::composition_underline(range, TEXT_COMPOSITION_UNDERLINE_COLOR),
            ],
            &mut source_maps,
        );

        assert_eq!(source_maps.initialized_count(), 1);
        assert_eq!(
            decorations
                .iter()
                .map(|decoration| decoration.kind)
                .collect::<Vec<_>>(),
            vec![
                UiTextPaintDecorationKind::CompositionHighlight,
                UiTextPaintDecorationKind::Selection,
                UiTextPaintDecorationKind::CompositionUnderline,
            ]
        );
        assert!(decorations
            .iter()
            .all(|decoration| decoration.range == range));
        assert_eq!(decorations[0].frame, UiFrame::new(0.0, 768.0, 8.0, 12.0));
        assert_eq!(decorations[1].frame, UiFrame::new(0.0, 768.0, 8.0, 12.0));
        assert_eq!(decorations[2].frame, UiFrame::new(0.0, 778.0, 8.0, 2.0));
    }
}
