use crate::text::SharedTextLayoutSession;
use crate::text::VerticalMode;
use crate::text::layout::{TextLineMetrics, layout_vertical_rl_columns};
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextOverflow, UiTextRange,
};

use super::super::rich_text::UiParsedText;
use super::direction::resolve_direction;
use super::ellipsis::{
    ellipsize_line_with_provider, is_ellipsis_overflow, line_overflows_horizontally_with_provider,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use super::layout_result::LayoutWithoutArtifact;
use super::line_box::{
    MIN_TEXT_FONT_SIZE, available_wrap_extent, materialize_arabic_tatweels_for_justified_line,
    resolve_line_widths_with_provider,
};
use super::paragraph_layout;
use super::visual_order;
use super::wrapping::wrap_source_runs_with_provider;

pub(super) fn layout_vertical_text_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    metrics: TextLineMetrics,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<LayoutWithoutArtifact> {
    let text = parsed.text();
    let direction = resolve_direction(text, style.text_direction);
    match super::rich_layout_vertical::layout_rich_vertical_text_with_provider(
        parsed, style, frame, clip_frame, font_size, direction, provider,
    ) {
        TextShapingOutcome::Ready(Some(layout)) => return TextShapingOutcome::Ready(layout),
        TextShapingOutcome::Ready(None) => {}
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    }
    let mut vertical_provider = provider.vertical_scope(VerticalMode::Mixed);
    let column_advance = metrics.line_height.max(font_size.max(MIN_TEXT_FONT_SIZE));
    let column_width = font_size.max(MIN_TEXT_FONT_SIZE);
    let max_column_height = available_wrap_extent(frame.height);
    let block_layout = paragraph_layout::has_block_layout(parsed);
    let mut columns = match if block_layout {
        paragraph_layout::wrap_block_paragraphs_with_provider(
            parsed,
            style,
            max_column_height,
            &mut *vertical_provider,
        )
    } else {
        wrap_source_runs_with_provider(
            &parsed.runs,
            style.wrap,
            max_column_height,
            style,
            &mut *vertical_provider,
        )
    } {
        TextShapingOutcome::Ready(columns) => columns,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let paragraph_constraints =
        match paragraph_layout::resolve_paragraph_column_constraints_with_provider(
            parsed,
            style,
            frame.height,
            &mut *vertical_provider,
        ) {
            TextShapingOutcome::Ready(constraints) => constraints,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
    let clip = clip_frame.unwrap_or(frame);
    let column_capacity = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &[],
    )
    .column_capacity;
    let mut overflow_clipped = columns.len() > column_capacity;

    if is_ellipsis_overflow(style.text_overflow) && overflow_clipped {
        if matches!(
            style.text_overflow,
            UiTextOverflow::EllipsisStart | UiTextOverflow::EllipsisMiddle
        ) {
            merge_clipped_lines_for_tail_preserving_ellipsis(&mut columns, column_capacity);
        }
        columns.truncate(column_capacity);
        if !columns.is_empty() {
            let column_constraints = paragraph_constraints.for_candidates(&columns);
            let last_index = columns.len() - 1;
            let available_height = column_constraints[last_index].max_height;
            let last = &mut columns[last_index];
            match ellipsize_line_with_provider(
                last,
                available_height,
                style,
                style.text_overflow,
                &mut *vertical_provider,
            ) {
                TextShapingOutcome::Ready(()) => {}
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            }
        }
    }
    let column_constraints = paragraph_constraints.for_candidates(&columns);
    if is_ellipsis_overflow(style.text_overflow) {
        for index in 0..columns.len() {
            let available_height = column_constraints[index].max_height;
            let column = &mut columns[index];
            let overflows = match line_overflows_horizontally_with_provider(
                column,
                available_height,
                style,
                &mut *vertical_provider,
            ) {
                TextShapingOutcome::Ready(overflows) => overflows,
                TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            if !column.ellipsized && overflows {
                match ellipsize_line_with_provider(
                    column,
                    available_height,
                    style,
                    style.text_overflow,
                    &mut *vertical_provider,
                ) {
                    TextShapingOutcome::Ready(()) => {}
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                }
                overflow_clipped = true;
            }
        }
    }

    for index in 0..columns.len() {
        let is_last_column = index + 1 == columns.len();
        let constraints = column_constraints[index];
        let mut column_style = style.clone();
        column_style.text_align = constraints.align;
        match materialize_arabic_tatweels_for_justified_line(
            &mut columns[index],
            &column_style,
            constraints.max_height.max(0.0),
            is_last_column,
            &mut *vertical_provider,
        ) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
    }
    for column in &mut columns {
        if visual_order::apply_visual_order(column, text, direction).is_err() {
            return TextShapingOutcome::failed(
                crate::core::framework::text::TextLayoutError::BidiInvariant,
            );
        }
    }

    let column_count = columns.len();
    let mut measured_columns = Vec::with_capacity(column_count);
    for (index, column) in columns.iter().enumerate() {
        let is_last_column = index + 1 == column_count;
        let constraints = column_constraints[index];
        let mut column_style = style.clone();
        column_style.text_align = constraints.align;
        let (measured_height, glyph_advances, content_height) =
            match resolve_line_widths_with_provider(
                column,
                &column_style,
                constraints.max_height.max(0.0),
                is_last_column,
                None,
                &mut *vertical_provider,
            ) {
                TextShapingOutcome::Ready(widths) => widths,
                TextShapingOutcome::Deferred(error) => {
                    return TextShapingOutcome::Deferred(error);
                }
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
        let alignment_height = if column.text.is_empty() {
            metrics.line_height
        } else {
            content_height
        };
        let natural_height = if column.text.is_empty() {
            metrics.line_height
        } else {
            measured_height
        };
        measured_columns.push((
            measured_height,
            glyph_advances,
            natural_height,
            alignment_height,
            constraints,
        ));
    }
    let column_heights = measured_columns
        .iter()
        .map(|(_, _, natural_height, _, _)| *natural_height)
        .collect::<Vec<_>>();
    let column_layout = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &column_heights,
    );
    let measured_width = column_layout.measured_width;
    let measured_height = column_layout.measured_height;

    let mut resolved_lines = Vec::new();
    for (
        (column, (measured_height, glyph_advances, natural_height, alignment_height, constraints)),
        column_frame,
    ) in columns
        .iter()
        .zip(measured_columns)
        .zip(column_layout.frames)
    {
        let placement_frame = UiFrame::new(
            column_frame.x,
            frame.y + constraints.inset,
            column_frame.width,
            constraints.max_height,
        );
        let column_frame = UiFrame::new(
            column_frame.x,
            paragraph_layout::aligned_column_y(frame, alignment_height, constraints),
            column_frame.width,
            natural_height,
        );
        if placement_frame.intersection(clip).is_some() {
            resolved_lines.push(UiResolvedTextLine {
                text: column.text.clone(),
                frame: column_frame,
                placement_frame,
                source_range: column.source_range,
                visual_range: UiTextRange {
                    start: 0,
                    end: column.text.len(),
                },
                measured_width: measured_height,
                glyph_advances,
                baseline: column_width * 0.5,
                direction,
                runs: column.runs.clone(),
                ellipsized: column.ellipsized,
            });
        } else {
            overflow_clipped = true;
        }
    }

    TextShapingOutcome::Ready(LayoutWithoutArtifact::without_retained_fragments(
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
            boxes: Vec::new(),
            overflow_clipped,
            editable: None,
            rich_text_artifact: None,
        },
    ))
}
