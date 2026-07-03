use crate::graphics::text::layout::TextLineMetrics;
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextOverflow, UiTextRange,
};

use super::super::rich_text::parse_source_runs;
use super::direction::resolve_direction;
use super::ellipsis::{
    ellipsize_line, is_ellipsis_overflow, line_overflows_horizontally,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use super::line_box::{resolve_line_widths, text_advance, MIN_TEXT_FONT_SIZE};
use super::visual_order;
use super::wrapping::wrap_source_runs;

pub(super) fn layout_vertical_text(
    text: &str,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    metrics: TextLineMetrics,
) -> UiResolvedTextLayout {
    let direction = resolve_direction(text, style.text_direction);
    let source_runs = parse_source_runs(text, style.rich_text);
    let column_advance = metrics.line_height.max(font_size.max(MIN_TEXT_FONT_SIZE));
    let column_width = font_size.max(MIN_TEXT_FONT_SIZE);
    let max_column_height = frame.height.max(text_advance(font_size));
    let mut columns = wrap_source_runs(&source_runs, style.wrap, max_column_height, style);
    let clip = clip_frame.unwrap_or(frame);
    let column_capacity = (frame.width.max(column_advance) / column_advance)
        .floor()
        .max(1.0) as usize;
    let mut overflow_clipped = columns.len() > column_capacity;

    if is_ellipsis_overflow(style.text_overflow) && overflow_clipped {
        if matches!(
            style.text_overflow,
            UiTextOverflow::EllipsisStart | UiTextOverflow::EllipsisMiddle
        ) {
            merge_clipped_lines_for_tail_preserving_ellipsis(&mut columns, column_capacity);
        }
        columns.truncate(column_capacity);
        if let Some(last) = columns.last_mut() {
            ellipsize_line(last, max_column_height, style, style.text_overflow);
        }
    }
    if is_ellipsis_overflow(style.text_overflow) {
        for column in &mut columns {
            if !column.ellipsized && line_overflows_horizontally(column, max_column_height, style) {
                ellipsize_line(column, max_column_height, style, style.text_overflow);
                overflow_clipped = true;
            }
        }
    }

    for column in &mut columns {
        visual_order::apply_visual_order(column, direction);
    }

    let mut resolved_lines = Vec::new();
    for (index, column) in columns.iter().enumerate() {
        let is_last_column = index + 1 == columns.len();
        let (measured_height, glyph_advances, content_height) =
            resolve_line_widths(column, style, max_column_height.max(0.0), is_last_column);
        let column_height = if column.text.is_empty() {
            metrics.line_height
        } else {
            content_height
        };
        let column_frame = UiFrame::new(
            vertical_rl_column_x(frame, index, column_advance),
            frame.y,
            column_width,
            column_height,
        );
        if column_frame.intersection(clip).is_some() {
            resolved_lines.push(UiResolvedTextLine {
                text: column.text.clone(),
                frame: column_frame,
                source_range: column.source_range,
                visual_range: UiTextRange {
                    start: 0,
                    end: column.text.len(),
                },
                measured_width: measured_height,
                glyph_advances,
                baseline: metrics.baseline,
                direction,
                runs: column.runs.clone(),
                ellipsized: column.ellipsized,
            });
        } else {
            overflow_clipped = true;
        }
    }

    let measured_width = resolved_lines.len() as f32 * column_advance;
    let measured_height = resolved_lines
        .iter()
        .map(|line| line.measured_width)
        .fold(0.0_f32, f32::max);
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height: metrics.line_height,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        lines: resolved_lines,
        overflow_clipped,
        editable: None,
    }
}

fn vertical_rl_column_x(frame: UiFrame, column_index: usize, column_advance: f32) -> f32 {
    frame.right() - (column_index + 1) as f32 * column_advance
}
