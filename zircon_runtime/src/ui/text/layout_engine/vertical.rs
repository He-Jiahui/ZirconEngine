use crate::core::framework::render::VerticalMode;
use crate::graphics::text::layout::{layout_vertical_rl_columns, TextLineMetrics};
use crate::graphics::text::shaping::{TextShapeRunProvider, VerticalTextShapeRunProvider};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextAlign, UiTextOverflow,
    UiTextRange,
};

use super::super::rich_text::UiParsedText;
use super::direction::resolve_direction;
use super::ellipsis::{
    ellipsize_line_with_provider, is_ellipsis_overflow, line_overflows_horizontally_with_provider,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use super::line_box::{resolve_line_widths_with_provider, text_advance, MIN_TEXT_FONT_SIZE};
use super::paragraph_layout::{self, ColumnConstraints};
use super::visual_order;
use super::wrapping::wrap_source_runs_with_provider;

pub(super) fn layout_vertical_text_with_provider<P>(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    metrics: TextLineMetrics,
    provider: &mut P,
) -> UiResolvedTextLayout
where
    P: TextShapeRunProvider + ?Sized,
{
    let text = parsed.text.as_str();
    let direction = resolve_direction(text, style.text_direction);
    if let Some(layout) = super::rich_inline_vertical::layout_inline_vertical_text_with_provider(
        parsed, style, frame, clip_frame, font_size, direction, provider,
    ) {
        return layout;
    }
    let mut vertical_provider = VerticalTextShapeRunProvider::new(provider, VerticalMode::Mixed);
    let column_advance = metrics.line_height.max(font_size.max(MIN_TEXT_FONT_SIZE));
    let column_width = font_size.max(MIN_TEXT_FONT_SIZE);
    let max_column_height = frame.height.max(text_advance(font_size));
    let block_layout = paragraph_layout::has_block_layout(parsed);
    let mut columns = if block_layout {
        paragraph_layout::wrap_block_paragraphs_with_provider(
            parsed,
            style,
            frame.height,
            &mut vertical_provider,
        )
    } else {
        wrap_source_runs_with_provider(
            &parsed.runs,
            style.wrap,
            max_column_height,
            style,
            &mut vertical_provider,
        )
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
            let last_index = columns.len() - 1;
            let available_height = vertical_column_constraints(
                parsed,
                style,
                frame.height,
                &columns,
                last_index,
                &mut vertical_provider,
            )
            .max_height;
            let last = &mut columns[last_index];
            ellipsize_line_with_provider(
                last,
                available_height,
                style,
                style.text_overflow,
                &mut vertical_provider,
            );
        }
    }
    if is_ellipsis_overflow(style.text_overflow) {
        for index in 0..columns.len() {
            let available_height = vertical_column_constraints(
                parsed,
                style,
                frame.height,
                &columns,
                index,
                &mut vertical_provider,
            )
            .max_height;
            let column = &mut columns[index];
            if !column.ellipsized
                && line_overflows_horizontally_with_provider(
                    column,
                    available_height,
                    style,
                    &mut vertical_provider,
                )
            {
                ellipsize_line_with_provider(
                    column,
                    available_height,
                    style,
                    style.text_overflow,
                    &mut vertical_provider,
                );
                overflow_clipped = true;
            }
        }
    }

    for column in &mut columns {
        visual_order::apply_visual_order(column, text, direction);
    }

    let measured_columns = columns
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let is_last_column = index + 1 == columns.len();
            let constraints = vertical_column_constraints(
                parsed,
                style,
                frame.height,
                &columns,
                index,
                &mut vertical_provider,
            );
            let mut column_style = style.clone();
            column_style.text_align = constraints.align;
            let (measured_height, glyph_advances, content_height) =
                resolve_line_widths_with_provider(
                    column,
                    &column_style,
                    constraints.max_height.max(0.0),
                    is_last_column,
                    &mut vertical_provider,
                );
            let column_height = if column.text.is_empty() {
                metrics.line_height
            } else {
                content_height
            };
            (measured_height, glyph_advances, column_height, constraints)
        })
        .collect::<Vec<_>>();
    let column_heights = measured_columns
        .iter()
        .map(|(_, _, column_height, _)| *column_height)
        .collect::<Vec<_>>();
    let column_layout = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &column_heights,
    );

    let mut resolved_lines = Vec::new();
    let mut visible_column_main_extents = Vec::new();
    for ((column, (measured_height, glyph_advances, column_height, constraints)), column_frame) in
        columns
            .iter()
            .zip(measured_columns)
            .zip(column_layout.frames)
    {
        let column_frame = UiFrame::new(
            column_frame.x,
            aligned_column_y(frame, column_height, constraints),
            column_frame.width,
            column_frame.height,
        );
        if column_frame.intersection(clip).is_some() {
            visible_column_main_extents.push(measured_height);
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

    let visible_layout = layout_vertical_rl_columns(
        frame.x,
        frame.y,
        frame.width,
        column_width,
        column_advance,
        &visible_column_main_extents,
    );
    UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height: metrics.line_height,
        measured_width: visible_layout.measured_width,
        measured_height: visible_layout.measured_height,
        source_range: UiTextRange {
            start: 0,
            end: text.len(),
        },
        lines: resolved_lines,
        boxes: Vec::new(),
        overflow_clipped,
        editable: None,
    }
}

fn vertical_column_constraints<P>(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame_height: f32,
    columns: &[super::candidate_line::CandidateLine],
    index: usize,
    provider: &mut P,
) -> ColumnConstraints
where
    P: TextShapeRunProvider + ?Sized,
{
    let column = &columns[index];
    let paragraph_start =
        paragraph_layout::physical_paragraph_start(&parsed.text, column.source_range.start);
    let first_physical_column = index == 0
        || paragraph_layout::physical_paragraph_start(
            &parsed.text,
            columns[index - 1].source_range.start,
        ) != paragraph_start;
    paragraph_layout::column_constraints_with_provider(
        parsed,
        style,
        frame_height,
        column.source_range.start,
        first_physical_column,
        provider,
    )
}

fn aligned_column_y(frame: UiFrame, column_height: f32, constraints: ColumnConstraints) -> f32 {
    let remaining = (constraints.max_height - column_height).max(0.0);
    let alignment_offset = match constraints.align {
        UiTextAlign::Center => remaining * 0.5,
        UiTextAlign::Right | UiTextAlign::End => remaining,
        UiTextAlign::Left | UiTextAlign::Start | UiTextAlign::Justify => 0.0,
    };
    frame.y + constraints.inset + alignment_offset
}
