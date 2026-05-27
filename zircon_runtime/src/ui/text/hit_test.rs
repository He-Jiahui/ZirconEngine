use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{
        UiResolvedTextLayout, UiResolvedTextLine, UiResolvedTextRun, UiTextCaretAffinity,
        UiTextDirection,
    },
};

use super::grapheme::{grapheme_count, grapheme_indices};
use super::layout_engine::text_advance;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct UiTextHitTest {
    pub line_index: Option<usize>,
    pub source_offset: usize,
    pub visual_grapheme_index: usize,
    pub affinity: UiTextCaretAffinity,
    pub inside_line: bool,
}

/// Converts a surface-space text point into the nearest source byte caret.
///
/// The helper intentionally consumes `UiResolvedTextLayout` instead of raw text
/// so pointer selection, render extract, and later shaping backends share one
/// geometry source.
pub(crate) fn hit_test_text_layout(layout: &UiResolvedTextLayout, point: UiPoint) -> UiTextHitTest {
    let Some(line_index) = text_line_index_for_y(layout, point.y) else {
        return UiTextHitTest {
            line_index: None,
            source_offset: layout.source_range.start,
            visual_grapheme_index: 0,
            affinity: UiTextCaretAffinity::Downstream,
            inside_line: false,
        };
    };
    let line = &layout.lines[line_index];
    let advance = text_advance(layout.font_size);
    let grapheme_index = visual_grapheme_index_for_x(line, point.x, advance);

    UiTextHitTest {
        line_index: Some(line_index),
        source_offset: line_source_offset_for_grapheme_index(line, grapheme_index),
        visual_grapheme_index: grapheme_index,
        affinity: if point.x <= line.frame.x {
            UiTextCaretAffinity::Upstream
        } else {
            UiTextCaretAffinity::Downstream
        },
        inside_line: line.frame.contains_point(point),
    }
}

fn text_line_index_for_y(layout: &UiResolvedTextLayout, y: f32) -> Option<usize> {
    let first = layout.lines.first()?;
    if y <= first.frame.y {
        return Some(0);
    }
    layout
        .lines
        .iter()
        .position(|line| y <= line.frame.bottom())
        .or_else(|| layout.lines.len().checked_sub(1))
}

fn visual_grapheme_index_for_x(
    line: &UiResolvedTextLine,
    point_x: f32,
    char_advance: f32,
) -> usize {
    let grapheme_count = grapheme_count(&line.text);
    if grapheme_count == 0 {
        return 0;
    }

    let relative_x = match line.direction {
        UiTextDirection::RightToLeft => line.frame.right() - point_x,
        UiTextDirection::LeftToRight | UiTextDirection::Mixed | UiTextDirection::Auto => {
            point_x - line.frame.x
        }
    };
    let measured_x = relative_x.clamp(0.0, line.measured_width.max(0.0));
    ((measured_x / char_advance) + 0.5)
        .floor()
        .clamp(0.0, grapheme_count as f32) as usize
}

fn line_source_offset_for_grapheme_index(line: &UiResolvedTextLine, index: usize) -> usize {
    if index == 0 {
        return line
            .runs
            .first()
            .map(|run| run.source_range.start)
            .unwrap_or(line.source_range.start);
    }

    let mut consumed = 0;
    let mut last_end = line.source_range.start;
    for run in &line.runs {
        if let Some(offset) = run_source_offset_for_grapheme_index(run, &mut consumed, index) {
            return offset;
        }
        last_end = run.source_range.end;
    }
    last_end.max(line.source_range.end)
}

fn run_source_offset_for_grapheme_index(
    run: &UiResolvedTextRun,
    consumed: &mut usize,
    target_index: usize,
) -> Option<usize> {
    for (byte_index, grapheme) in grapheme_indices(&run.text) {
        if *consumed == target_index {
            return Some(run.source_range.start + byte_index);
        }
        *consumed += 1;
        let end = run.source_range.start + byte_index + grapheme.len();
        if *consumed == target_index {
            return Some(end);
        }
    }
    None
}
