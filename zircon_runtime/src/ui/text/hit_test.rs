use crate::text::font::{shared_font_collection_snapshot, FontCollectionSnapshot};
use crate::text::{
    resolve_resolved_text_glyph_artifact, resolved_text_glyph_artifact_caret_at_advance,
};
use zircon_runtime_interface::ui::{
    layout::UiPoint,
    surface::{
        UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextCaretAffinity,
        UiTextLineSourceMap, UiTextVisualBoundaryBias, UiTextWritingMode,
    },
};

use super::geometry::source_metrics_caret_at_advance;
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
    hit_test_text_layout_inner(layout, point, None)
}

pub(crate) fn hit_test_text_layout_with_source_metrics(
    layout: &UiResolvedTextLayout,
    point: UiPoint,
    text: &str,
    style: &UiResolvedStyle,
) -> UiTextHitTest {
    let font_collection = shared_font_collection_snapshot();
    hit_test_text_layout_with_font_collection(layout, point, text, style, &font_collection)
}

pub(crate) fn hit_test_text_layout_with_font_collection(
    layout: &UiResolvedTextLayout,
    point: UiPoint,
    text: &str,
    style: &UiResolvedStyle,
    font_collection: &FontCollectionSnapshot,
) -> UiTextHitTest {
    hit_test_text_layout_inner(layout, point, Some((text, style, font_collection)))
}

fn hit_test_text_layout_inner(
    layout: &UiResolvedTextLayout,
    point: UiPoint,
    measure_context: Option<(&str, &UiResolvedStyle, &FontCollectionSnapshot)>,
) -> UiTextHitTest {
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
    let visual_advance = if vertical_rl {
        point.y - line.frame.y
    } else {
        point.x - line.frame.x
    };
    let artifact_caret = resolved_glyph_artifact(layout).and_then(|artifact| {
        resolved_text_glyph_artifact_caret_at_advance(
            artifact.as_ref(),
            line_index,
            line,
            visual_advance,
        )
    });
    let source_caret = if artifact_caret.is_none() {
        measure_context.and_then(|(text, style, font_collection)| {
            source_metrics_caret_at_advance(
                layout,
                line,
                visual_advance,
                text,
                style,
                font_collection,
            )
        })
    } else {
        None
    };
    let (grapheme_index, boundary_bias) = if let Some((_, visual_index)) = &source_caret {
        (*visual_index, UiTextVisualBoundaryBias::LeadingCurrent)
    } else if vertical_rl {
        visual_grapheme_boundary_for_y(line, point.y)
    } else {
        visual_grapheme_boundary_for_x(line, point.x)
    };
    let resolved_caret = artifact_caret
        .or_else(|| source_caret.map(|(caret, _)| caret))
        .unwrap_or_else(|| {
            // A valid shaped artifact already resolves the source caret. Keep the
            // allocation-backed source map on the stale-or-missing artifact path.
            let source_map = UiTextLineSourceMap::new(line);
            let fallback_source_offset = if grapheme_index == 0 {
                line.source_range.start
            } else {
                line.source_range.end
            };
            source_map.caret_for_visual_boundary(
                grapheme_index,
                boundary_bias,
                fallback_source_offset,
            )
        });
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
        inside_line: line.hit_frame().contains_point(point),
    }
}

/// Prefers the resolved physical line whose placement slot contains the point.
///
/// Rich-table cells can share the writing-mode block coordinate, so selecting
/// only by y (HorizontalTb) or x (VerticalRl) can choose a sibling cell before
/// the full-slot containment check. The nearest-axis fallback remains below
/// for ordinary caret placement outside all resolved line frames.
fn containing_line_index(layout: &UiResolvedTextLayout, point: UiPoint) -> Option<usize> {
    layout
        .lines
        .iter()
        .position(|line| line.placement_frame.contains_point(point))
}

fn text_line_index_for_y(layout: &UiResolvedTextLayout, y: f32) -> Option<usize> {
    let first = layout.lines.first()?;
    if y <= first.placement_frame.y {
        return Some(0);
    }
    layout
        .lines
        .iter()
        .position(|line| y <= line.placement_frame.bottom())
        .or_else(|| layout.lines.len().checked_sub(1))
}

fn text_column_index_for_vertical_rl_x(layout: &UiResolvedTextLayout, x: f32) -> Option<usize> {
    let mut nearest = None;
    for (index, line) in layout.lines.iter().enumerate() {
        if x >= line.placement_frame.x && x <= line.placement_frame.right() {
            return Some(index);
        }
        let distance = if x < line.placement_frame.x {
            line.placement_frame.x - x
        } else {
            x - line.placement_frame.right()
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
) -> (usize, UiTextVisualBoundaryBias) {
    let grapheme_count = grapheme_count(&line.text);
    if grapheme_count == 0 {
        return (0, UiTextVisualBoundaryBias::LeadingCurrent);
    }

    // `line.text` and `glyph_advances` are already in post-UAX#9 visual order.
    // Logical RTL recovery belongs to the source map, not to physical hit coordinates.
    let relative_x = point_x - line.frame.x;
    let Some(advances) = resolved_grapheme_advances(line, grapheme_count) else {
        return endpoint_boundary(relative_x, line.measured_width, grapheme_count);
    };
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
) -> (usize, UiTextVisualBoundaryBias) {
    let grapheme_count = grapheme_count(&line.text);
    if grapheme_count == 0 {
        return (0, UiTextVisualBoundaryBias::LeadingCurrent);
    }

    let relative_y = point_y - line.frame.y;
    let Some(advances) = resolved_grapheme_advances(line, grapheme_count) else {
        return endpoint_boundary(
            relative_y,
            line.measured_width.max(line.frame.height),
            grapheme_count,
        );
    };
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

fn resolved_grapheme_advances(line: &UiResolvedTextLine, grapheme_count: usize) -> Option<&[f32]> {
    (line.glyph_advances.len() == grapheme_count
        && line
            .glyph_advances
            .iter()
            .all(|advance| advance.is_finite() && *advance >= 0.0))
    .then_some(line.glyph_advances.as_slice())
}

fn endpoint_boundary(
    relative_advance: f32,
    measured_advance: f32,
    grapheme_count: usize,
) -> (usize, UiTextVisualBoundaryBias) {
    let measured_advance = if measured_advance.is_finite() {
        measured_advance.max(0.0)
    } else {
        0.0
    };
    if relative_advance <= measured_advance * 0.5 {
        (0, UiTextVisualBoundaryBias::LeadingCurrent)
    } else {
        (grapheme_count, UiTextVisualBoundaryBias::TrailingPrevious)
    }
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
