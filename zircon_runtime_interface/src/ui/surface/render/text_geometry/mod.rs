use std::{collections::HashMap, ops::Range};

use crate::ui::{layout::UiFrame, surface::UiTextPreeditClauseKind};

use super::{
    UiEditableTextState, UiResolvedTextLayout, UiResolvedTextLine, UiTextCaret,
    UiTextCaretAffinity, UiTextPaintDecoration, UiTextPaintDecorationKind, UiTextRange,
    UiTextWritingMode,
};

mod source_map;

pub use source_map::{UiTextLineSourceMap, UiTextVisualBoundaryBias, UiTextVisualSpan};

#[cfg(test)]
#[path = "tests/source_map_tests.rs"]
mod source_map_tests;

#[cfg(test)]
#[path = "tests/caret_performance_tests.rs"]
mod caret_performance_tests;

#[cfg(test)]
#[path = "tests/decoration_performance_tests.rs"]
mod decoration_performance_tests;

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

    let mut source_maps =
        (!range_decorations.is_empty()).then(|| TextDecorationLineSourceMaps::new(&layout.lines));
    if let Some(source_maps) = source_maps.as_mut() {
        append_range_decorations_with_source_maps(
            &mut decorations,
            layout,
            &range_decorations,
            source_maps,
        );
    }

    let caret_frame = match source_maps.as_mut() {
        Some(source_maps) => caret_frame_with_source_maps(layout, &editable.caret, source_maps),
        None => caret_frame(layout, &editable.caret),
    };
    if let Some(frame) = caret_frame {
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

struct TextDecorationLineSourceMaps<'a> {
    lines: &'a [UiResolvedTextLine],
    maps: HashMap<usize, UiTextLineSourceMap<'a>>,
    source_ranges_are_ordered: bool,
    #[cfg(test)]
    initialized_count: usize,
}

impl<'a> TextDecorationLineSourceMaps<'a> {
    fn new(lines: &'a [UiResolvedTextLine]) -> Self {
        Self {
            lines,
            maps: HashMap::new(),
            // `UiResolvedTextLayout` is an interface DTO, so callers may
            // provide lines whose source ranges are not monotonic. Keep the
            // binary range path for the normal shaped-layout contract and
            // remember when a safe linear fallback is required.
            source_ranges_are_ordered: lines.windows(2).all(|pair| {
                pair[0].source_range.start <= pair[1].source_range.start
                    && pair[0].source_range.end <= pair[1].source_range.end
            }),
            #[cfg(test)]
            initialized_count: 0,
        }
    }

    fn for_line(
        &mut self,
        line_index: usize,
    ) -> Option<(&'a UiResolvedTextLine, &UiTextLineSourceMap<'a>)> {
        let line = self.lines.get(line_index)?;
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
        if source_maps.source_ranges_are_ordered {
            for line_index in intersecting_line_range(source_maps.lines, decoration.range) {
                append_range_decoration_for_line(
                    decorations,
                    layout,
                    *decoration,
                    source_maps,
                    line_index,
                );
            }
        } else {
            // Keep declaration and source order for malformed/foreign DTOs;
            // unlike a guessed binary search, this cannot silently skip a
            // line whose range was published out of order.
            let line_count = source_maps.lines.len();
            for line_index in 0..line_count {
                let intersects = {
                    let line = &source_maps.lines[line_index];
                    decoration.range.start < line.source_range.end
                        && line.source_range.start < decoration.range.end
                };
                if intersects {
                    append_range_decoration_for_line(
                        decorations,
                        layout,
                        *decoration,
                        source_maps,
                        line_index,
                    );
                }
            }
        }
    }
}

fn append_range_decoration_for_line(
    decorations: &mut Vec<UiTextPaintDecoration>,
    layout: &UiResolvedTextLayout,
    decoration: TextRangeDecoration,
    source_maps: &mut TextDecorationLineSourceMaps<'_>,
    line_index: usize,
) {
    // Both callers have already established intersection: the ordered path
    // derives `line_index` from `intersecting_line_range`, while the fallback
    // checks the DTO line explicitly. Avoid repeating that range predicate
    // before touching the lazy source-map cache.
    let Some((line, source_map)) = source_maps.for_line(line_index) else {
        return;
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

fn intersecting_line_range(lines: &[UiResolvedTextLine], range: UiTextRange) -> Range<usize> {
    let start = lines.partition_point(|line| line.source_range.end <= range.start);
    let end = lines.partition_point(|line| line.source_range.start < range.end);
    start.min(end)..end
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
    caret_frame_from_source_map(layout, caret, line, &map)
}

fn caret_frame_with_source_maps(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    source_maps: &mut TextDecorationLineSourceMaps<'_>,
) -> Option<UiFrame> {
    let line_index = caret_line_index(layout, caret)?;
    let (line, map) = source_maps.for_line(line_index)?;
    caret_frame_from_source_map(layout, caret, line, map)
}

fn caret_frame_from_source_map(
    layout: &UiResolvedTextLayout,
    caret: &UiTextCaret,
    line: &UiResolvedTextLine,
    map: &UiTextLineSourceMap<'_>,
) -> Option<UiFrame> {
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
    let candidate = match caret.affinity {
        UiTextCaretAffinity::Upstream => {
            let index = layout
                .lines
                .partition_point(|line| line.source_range.end < caret.offset);
            layout.lines.get(index)
        }
        UiTextCaretAffinity::Downstream => {
            let end = layout
                .lines
                .partition_point(|line| line.source_range.start <= caret.offset);
            end.checked_sub(1).and_then(|index| layout.lines.get(index))
        }
    };
    candidate
        .filter(|line| line_contains_caret(line, caret.offset))
        .or_else(|| linear_caret_line(layout, caret))
}

fn caret_line_index(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<usize> {
    let candidate = match caret.affinity {
        UiTextCaretAffinity::Upstream => {
            let index = layout
                .lines
                .partition_point(|line| line.source_range.end < caret.offset);
            Some(index).filter(|index| {
                layout
                    .lines
                    .get(*index)
                    .is_some_and(|line| line_contains_caret(line, caret.offset))
            })
        }
        UiTextCaretAffinity::Downstream => {
            let end = layout
                .lines
                .partition_point(|line| line.source_range.start <= caret.offset);
            end.checked_sub(1).filter(|index| {
                layout
                    .lines
                    .get(*index)
                    .is_some_and(|line| line_contains_caret(line, caret.offset))
            })
        }
    };
    candidate.or_else(|| linear_caret_line_index(layout, caret))
}

fn linear_caret_line<'a>(
    layout: &'a UiResolvedTextLayout,
    caret: &UiTextCaret,
) -> Option<&'a UiResolvedTextLine> {
    linear_caret_line_index(layout, caret).and_then(|index| layout.lines.get(index))
}

fn linear_caret_line_index(layout: &UiResolvedTextLayout, caret: &UiTextCaret) -> Option<usize> {
    match caret.affinity {
        UiTextCaretAffinity::Upstream => layout
            .lines
            .iter()
            .position(|line| line_contains_caret(line, caret.offset)),
        UiTextCaretAffinity::Downstream => layout
            .lines
            .iter()
            .rposition(|line| line_contains_caret(line, caret.offset)),
    }
    .or_else(|| {
        layout
            .lines
            .first()
            .filter(|line| caret.offset < line.source_range.start)
            .map(|_| 0)
    })
    .or_else(|| layout.lines.len().checked_sub(1))
}

fn line_contains_caret(line: &UiResolvedTextLine, offset: usize) -> bool {
    offset >= line.source_range.start && offset <= line.source_range.end
}

#[cfg(test)]
#[path = "tests/mod_performance_tests.rs"]
mod performance_tests;
