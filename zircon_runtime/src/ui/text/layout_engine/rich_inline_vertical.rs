use crate::text::SharedTextLayoutSession;
use crate::text::VerticalMode;
use crate::text::layout::{
    ELLIPSIS, layout_vertical_rl_columns, measured_grapheme_widths_with_provider,
    rich_vertical_columns_with_provider,
};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextDirection, UiTextRange,
    UiTextWritingMode,
};

use super::super::rich_text::UiParsedText;
use super::candidate_line::CandidateLine;
use super::ellipsis::{
    ellipsize_line_with_advances, force_ellipsize_line_with_advances, is_ellipsis_overflow,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use super::paragraph_layout;
use super::rich_inline::{append_soft_hyphen_break_suffix, resolved_runs_for_line};
use super::visual_order::apply_visual_order_with_advances;
use crate::text::text_style;

pub(super) fn layout_inline_vertical_text_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    direction: UiTextDirection,
    provider: &mut SharedTextLayoutSession,
) -> Option<UiResolvedTextLayout> {
    if !parsed.runs.iter().any(|run| run.inline().is_some())
        || !matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
    {
        return None;
    }

    let mut vertical_provider = provider.vertical_scope(VerticalMode::Mixed);
    let neutral_style = text_style(style);
    let paragraph_constraints =
        paragraph_layout::resolve_paragraph_column_constraints_with_provider(
            parsed,
            style,
            frame.height,
            &mut *vertical_provider,
        );
    let column_metrics = rich_vertical_columns_with_provider(
        parsed,
        &neutral_style,
        |forced_range, column_index| {
            paragraph_constraints
                .for_source_offset(
                    usize::try_from(forced_range.0).unwrap_or(usize::MAX),
                    column_index == 0,
                )
                .max_height
        },
        &mut *vertical_provider,
    );
    let mut columns = Vec::with_capacity(column_metrics.len());
    let mut column_advances = Vec::with_capacity(column_metrics.len());
    let mut column_cross_extents = Vec::with_capacity(column_metrics.len());
    for metrics in column_metrics {
        let source_range = UiTextRange {
            start: usize::try_from(metrics.source_range.0).ok()?,
            end: usize::try_from(metrics.source_range.1).ok()?,
        };
        let mut column = CandidateLine {
            text: parsed
                .text()
                .get(source_range.start..source_range.end)?
                .to_string(),
            source_range,
            runs: resolved_runs_for_line(parsed, source_range, direction),
            pending_break_suffix: None,
            ellipsized: false,
        };
        let mut advances = metrics.advances;
        append_soft_hyphen_break_suffix(
            &mut column,
            &mut advances,
            parsed,
            style,
            source_range.end,
            &mut *vertical_provider,
        );
        columns.push(column);
        column_advances.push(advances);
        column_cross_extents.push(metrics.cross_extent);
    }

    let column_width = column_cross_extents
        .iter()
        .copied()
        .fold(font_size.max(1.0), f32::max);
    let column_advance = style.line_height.max(column_width);
    let column_capacity = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &[],
    )
    .column_capacity;
    let clipped_columns = columns.len() > column_capacity;
    let mut overflow_clipped = clipped_columns;

    if clipped_columns {
        if matches!(
            style.text_overflow,
            zircon_runtime_interface::ui::surface::UiTextOverflow::EllipsisStart
                | zircon_runtime_interface::ui::surface::UiTextOverflow::EllipsisMiddle
        ) {
            let merged_source_end = columns
                .last()
                .map(|column| column.source_range.end)
                .unwrap_or_default();
            let tail_advances = column_advances
                .iter()
                .skip(column_capacity)
                .flatten()
                .copied()
                .collect::<Vec<_>>();
            let tail_cross_extent = column_cross_extents
                .iter()
                .skip(column_capacity)
                .copied()
                .fold(0.0_f32, f32::max);
            merge_clipped_lines_for_tail_preserving_ellipsis(&mut columns, column_capacity);
            if let Some(column) = columns.last_mut() {
                column.source_range.end = merged_source_end.max(column.source_range.end);
            }
            column_advances.truncate(column_capacity);
            column_cross_extents.truncate(column_capacity);
            if let Some(advances) = column_advances.last_mut() {
                advances.extend(tail_advances);
            }
            if let Some(cross_extent) = column_cross_extents.last_mut() {
                *cross_extent = cross_extent.max(tail_cross_extent);
            }
        } else {
            columns.truncate(column_capacity);
            column_advances.truncate(column_capacity);
            column_cross_extents.truncate(column_capacity);
        }
    }

    let column_constraints = paragraph_constraints.for_candidates(&columns);

    let ellipsis_advance =
        measured_grapheme_widths_with_provider(ELLIPSIS, &neutral_style, &mut *vertical_provider)
            .into_iter()
            .next()
            .unwrap_or_default();
    let last_visible_index = columns.len().saturating_sub(1);
    for (index, ((column, advances), constraints)) in columns
        .iter_mut()
        .zip(&mut column_advances)
        .zip(&column_constraints)
        .enumerate()
    {
        apply_visual_order_with_advances(column, parsed.text(), direction, advances);
        if is_ellipsis_overflow(style.text_overflow) {
            let was_ellipsized = column.ellipsized;
            if clipped_columns && index == last_visible_index {
                force_ellipsize_line_with_advances(
                    column,
                    advances,
                    constraints.max_height,
                    ellipsis_advance,
                    style.text_overflow,
                );
            } else {
                ellipsize_line_with_advances(
                    column,
                    advances,
                    constraints.max_height,
                    ellipsis_advance,
                    style.text_overflow,
                );
            }
            overflow_clipped |= !was_ellipsized && column.ellipsized;
        } else {
            overflow_clipped |= advances.iter().copied().sum::<f32>() > constraints.max_height;
        }
    }

    let column_heights = column_advances
        .iter()
        .map(|advances| advances.iter().copied().sum::<f32>())
        .collect::<Vec<_>>();
    let column_layout = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &column_heights,
    );
    let clip = clip_frame.unwrap_or(frame);
    let mut resolved_lines = Vec::new();
    let mut visible_column_heights = Vec::new();
    for ((((column, glyph_advances), column_height), constraints), column_frame) in columns
        .into_iter()
        .zip(column_advances)
        .zip(column_heights)
        .zip(column_constraints)
        .zip(column_layout.frames)
    {
        let column_frame = UiFrame::new(
            column_frame.x,
            paragraph_layout::aligned_column_y(frame, column_height, constraints),
            column_frame.width,
            column_frame.height,
        );
        if column_frame.intersection(clip).is_none() {
            overflow_clipped = true;
            continue;
        }
        visible_column_heights.push(column_height);
        resolved_lines.push(UiResolvedTextLine {
            text: column.text,
            frame: column_frame,
            source_range: column.source_range,
            visual_range: UiTextRange {
                start: 0,
                end: column
                    .runs
                    .last()
                    .map(|run| run.visual_range.end)
                    .unwrap_or_default(),
            },
            measured_width: column_height,
            glyph_advances,
            baseline: column_width * 0.5,
            direction,
            runs: column.runs,
            ellipsized: column.ellipsized,
        });
    }

    let visible_layout = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &visible_column_heights,
    );
    Some(UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height: column_advance,
        measured_width: visible_layout.measured_width,
        measured_height: visible_layout.measured_height,
        source_range: UiTextRange {
            start: 0,
            end: parsed.text().len(),
        },
        lines: resolved_lines,
        boxes: Vec::new(),
        overflow_clipped,
        editable: None,
        rich_text_artifact: None,
    })
}
