use std::borrow::Cow;

use crate::text::{layout::measured_grapheme_widths, text_style};
use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextCaretAffinity,
        UiTextDirection, UiTextLineSourceMap, UiTextVisualBoundaryBias, UiTextWritingMode,
    },
};

use super::grapheme::grapheme_count;

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
    let vertical_rl = is_vertical_rl(layout);
    let Some(line_index) = containing_line_index(layout, point).or_else(|| {
        if vertical_rl {
            text_column_index_for_vertical_rl_x(layout, point.x)
        } else {
            text_line_index_for_y(layout, point.y)
        }
    }) else {
        return UiTextHitTest {
            line_index: None,
            source_offset: layout.source_range.start,
            visual_grapheme_index: 0,
            affinity: UiTextCaretAffinity::Downstream,
            inside_line: false,
        };
    };
    let line = &layout.lines[line_index];
    let style = style_for_layout(layout, line);
    let (grapheme_index, boundary_bias) = if vertical_rl {
        visual_grapheme_boundary_for_y(line, point.y, &style)
    } else {
        visual_grapheme_boundary_for_x(line, point.x, &style)
    };
    let source_map = UiTextLineSourceMap::new(line);
    let fallback_source_offset = if grapheme_index == 0 {
        line.source_range.start
    } else {
        line.source_range.end
    };
    let resolved_caret =
        source_map.caret_for_visual_boundary(grapheme_index, boundary_bias, fallback_source_offset);
    let affinity =
        if (vertical_rl && point.y <= line.frame.y) || (!vertical_rl && point.x <= line.frame.x) {
            UiTextCaretAffinity::Upstream
        } else {
            resolved_caret.affinity
        };

    UiTextHitTest {
        line_index: Some(line_index),
        source_offset: resolved_caret.offset,
        visual_grapheme_index: grapheme_index,
        affinity,
        inside_line: line.frame.contains_point(point),
    }
}

/// Prefers the resolved physical line that actually contains the point.
///
/// Rich-table cells can share the writing-mode block coordinate, so selecting
/// only by y (HorizontalTb) or x (VerticalRl) can choose a sibling cell before
/// the full-frame containment check. The nearest-axis fallback remains below
/// for ordinary caret placement outside all resolved line frames.
fn containing_line_index(layout: &UiResolvedTextLayout, point: UiPoint) -> Option<usize> {
    layout
        .lines
        .iter()
        .position(|line| line.frame.contains_point(point))
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

fn text_column_index_for_vertical_rl_x(layout: &UiResolvedTextLayout, x: f32) -> Option<usize> {
    let mut nearest = None;
    for (index, line) in layout.lines.iter().enumerate() {
        if x >= line.frame.x && x <= line.frame.right() {
            return Some(index);
        }
        let distance = if x < line.frame.x {
            line.frame.x - x
        } else {
            x - line.frame.right()
        };
        if nearest
            .map(|(_, nearest_distance): (usize, f32)| distance < nearest_distance)
            .unwrap_or(true)
        {
            nearest = Some((index, distance));
        }
    }
    nearest.map(|(index, _)| index)
}

fn visual_grapheme_boundary_for_x(
    line: &UiResolvedTextLine,
    point_x: f32,
    style: &UiResolvedStyle,
) -> (usize, UiTextVisualBoundaryBias) {
    let grapheme_count = grapheme_count(&line.text);
    if grapheme_count == 0 {
        return (0, UiTextVisualBoundaryBias::LeadingCurrent);
    }

    // `line.text` and `glyph_advances` are already in post-UAX#9 visual order.
    // Logical RTL recovery belongs to the source map, not to physical hit coordinates.
    let relative_x = point_x - line.frame.x;
    let advances = resolved_grapheme_advances(line, style, grapheme_count);
    let advance_width = advances.iter().sum::<f32>();
    let measured_x = relative_x.clamp(0.0, line.measured_width.max(advance_width).max(0.0));
    let mut cursor_x = 0.0_f32;
    for (index, width) in advances.iter().copied().enumerate() {
        if index > 0 && measured_x < cursor_x {
            return (index, UiTextVisualBoundaryBias::TrailingPrevious);
        }
        if measured_x <= cursor_x + width * 0.5 {
            return (index, UiTextVisualBoundaryBias::LeadingCurrent);
        }
        cursor_x += width;
    }
    (grapheme_count, UiTextVisualBoundaryBias::TrailingPrevious)
}

fn visual_grapheme_boundary_for_y(
    line: &UiResolvedTextLine,
    point_y: f32,
    style: &UiResolvedStyle,
) -> (usize, UiTextVisualBoundaryBias) {
    let grapheme_count = grapheme_count(&line.text);
    if grapheme_count == 0 {
        return (0, UiTextVisualBoundaryBias::LeadingCurrent);
    }

    let relative_y = point_y - line.frame.y;
    let advances = resolved_grapheme_advances(line, style, grapheme_count);
    let advance_height = advances.iter().sum::<f32>();
    let measured_y = relative_y.clamp(
        0.0,
        line.measured_width
            .max(line.frame.height)
            .max(advance_height)
            .max(0.0),
    );
    let mut cursor_y = 0.0_f32;
    for (index, height) in advances.iter().copied().enumerate() {
        if index > 0 && measured_y < cursor_y {
            return (index, UiTextVisualBoundaryBias::TrailingPrevious);
        }
        if measured_y <= cursor_y + height * 0.5 {
            return (index, UiTextVisualBoundaryBias::LeadingCurrent);
        }
        cursor_y += height;
    }
    (grapheme_count, UiTextVisualBoundaryBias::TrailingPrevious)
}

fn resolved_grapheme_advances<'a>(
    line: &'a UiResolvedTextLine,
    style: &UiResolvedStyle,
    grapheme_count: usize,
) -> Cow<'a, [f32]> {
    if line.glyph_advances.len() == grapheme_count {
        if line
            .glyph_advances
            .iter()
            .all(|advance| advance.is_finite() && *advance >= 0.0)
            && line.glyph_advances.iter().any(|advance| *advance > 0.0)
        {
            return Cow::Borrowed(&line.glyph_advances);
        }
        let advances = line
            .glyph_advances
            .iter()
            .map(|advance| sanitized_advance(*advance))
            .collect::<Vec<_>>();
        if advances.iter().any(|advance| *advance > 0.0) {
            return Cow::Owned(advances);
        }
    }
    Cow::Owned(measured_grapheme_widths(&line.text, &text_style(style)))
}

fn sanitized_advance(advance: f32) -> f32 {
    if advance.is_finite() {
        advance.max(0.0)
    } else {
        0.0
    }
}

fn style_for_layout(layout: &UiResolvedTextLayout, line: &UiResolvedTextLine) -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: layout.font_size,
        line_height: layout.line_height,
        text_direction: line.direction,
        text_writing_mode: layout.writing_mode,
        ..UiResolvedStyle::default()
    }
}

fn is_vertical_rl(layout: &UiResolvedTextLayout) -> bool {
    matches!(layout.writing_mode, UiTextWritingMode::VerticalRl)
}
