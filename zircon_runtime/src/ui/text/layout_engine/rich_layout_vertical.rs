use crate::core::framework::text::TextLayoutError;
use crate::text::SharedTextLayoutSession;
use crate::text::VerticalMode;
use crate::text::layout::{
    ELLIPSIS, layout_vertical_rl_columns, measured_grapheme_widths_with_provider,
    rich_vertical_columns_with_provider,
};
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use zircon_runtime_interface::ui::layout::UiFrame;
use zircon_runtime_interface::ui::surface::{
    UiResolvedStyle, UiResolvedTextLayout, UiResolvedTextLine, UiTextDirection, UiTextRange,
    UiTextWritingMode,
};

use super::super::rich_text::UiParsedText;
use super::candidate_line::CandidateLine;
use super::ellipsis::{
    ellipsis_style_owner_source_range, ellipsize_line_with_advances_and_style_owner,
    force_ellipsize_line_with_advances_and_style_owner, is_ellipsis_overflow,
    merge_clipped_lines_for_tail_preserving_ellipsis,
};
use super::layout_result::LayoutWithoutArtifact;
use super::paragraph_layout;
use super::rich_layout::{append_soft_hyphen_break_suffix, resolved_runs_for_line};
use super::{virtual_fragment_sequence, visual_order};
use crate::text::text_style;

pub(super) fn layout_rich_vertical_text_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    font_size: f32,
    direction: UiTextDirection,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<LayoutWithoutArtifact>> {
    if matches!(
        style.rich_text_format,
        zircon_runtime_interface::ui::surface::UiRichTextFormat::Plain
    ) || !matches!(style.text_writing_mode, UiTextWritingMode::VerticalRl)
    {
        return TextShapingOutcome::Ready(None);
    }

    let mut vertical_provider = provider.vertical_scope(VerticalMode::Mixed);
    let neutral_style = text_style(style);
    let inline_source_ranges = parsed
        .runs
        .iter()
        .filter(|run| run.inline().is_some())
        .map(|run| run.source_range)
        .collect::<Vec<_>>();
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
    let column_metrics = match rich_vertical_columns_with_provider(
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
    ) {
        TextShapingOutcome::Ready(metrics) => metrics,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let mut columns = Vec::with_capacity(column_metrics.len());
    let mut column_advances = Vec::with_capacity(column_metrics.len());
    let mut column_cross_extents = Vec::with_capacity(column_metrics.len());
    for metrics in column_metrics {
        let (Ok(start), Ok(end)) = (
            usize::try_from(metrics.source_range.0),
            usize::try_from(metrics.source_range.1),
        ) else {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        };
        let source_range = UiTextRange { start, end };
        let Some(column_text) = parsed.text().get(source_range.start..source_range.end) else {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        };
        let mut column = CandidateLine {
            text: column_text.to_string(),
            source_range,
            runs: resolved_runs_for_line(parsed, source_range, direction),
            virtual_source_receipts: Vec::new(),
            pending_break_suffix: None,
            ellipsized: false,
        };
        let mut advances = metrics.advances;
        match append_soft_hyphen_break_suffix(
            &mut column,
            &mut advances,
            parsed,
            style,
            source_range.end,
            &mut *vertical_provider,
        ) {
            TextShapingOutcome::Ready(()) => {}
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        }
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

    let last_visible_index = columns.len().saturating_sub(1);
    let mut column_virtual_line_sequences = Vec::with_capacity(columns.len());
    for (index, ((column, advances), constraints)) in columns
        .iter_mut()
        .zip(&mut column_advances)
        .zip(&column_constraints)
        .enumerate()
    {
        if is_ellipsis_overflow(style.text_overflow) {
            let style_owner_source_range = ellipsis_style_owner_source_range(
                column,
                advances,
                constraints.max_height,
                style.text_overflow,
            );
            let ellipsis_style = style_owner_source_range
                .and_then(|source_range| {
                    parsed.runs.iter().find(|run| {
                        run.source_range.start <= source_range.start
                            && source_range.end <= run.source_range.end
                    })
                })
                .map_or_else(
                    || neutral_style.clone(),
                    |run| crate::text::layout::resolve_rich_run_style(&neutral_style, run.style()),
                );
            let ellipsis_advance = match measured_grapheme_widths_with_provider(
                ELLIPSIS,
                &ellipsis_style,
                &mut *vertical_provider,
            ) {
                TextShapingOutcome::Ready(advances) => {
                    advances.into_iter().next().unwrap_or_default()
                }
                TextShapingOutcome::Deferred(error) => {
                    return TextShapingOutcome::Deferred(error);
                }
                TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
            };
            let was_ellipsized = column.ellipsized;
            if clipped_columns && index == last_visible_index {
                force_ellipsize_line_with_advances_and_style_owner(
                    column,
                    advances,
                    constraints.max_height,
                    ellipsis_advance,
                    style.text_overflow,
                    style_owner_source_range,
                );
            } else {
                ellipsize_line_with_advances_and_style_owner(
                    column,
                    advances,
                    constraints.max_height,
                    ellipsis_advance,
                    style.text_overflow,
                    style_owner_source_range,
                );
            }
            overflow_clipped |= !was_ellipsized && column.ellipsized;
        } else {
            overflow_clipped |= advances.iter().copied().sum::<f32>() > constraints.max_height;
        }
        let mut virtual_sequence = virtual_fragment_sequence::capture_with_external_source_ranges(
            column,
            direction,
            &inline_source_ranges,
        );
        if visual_order::apply_visual_order_with_virtual_sequence(
            column,
            direction,
            virtual_sequence.as_mut(),
            Some(advances),
        )
        .is_err()
        {
            return TextShapingOutcome::failed(TextLayoutError::BidiInvariant);
        }
        column_virtual_line_sequences.push(virtual_sequence);
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
    let measured_width = column_layout.measured_width;
    let measured_height = column_layout.measured_height;
    let clip = clip_frame.unwrap_or(frame);
    let mut resolved_lines = Vec::new();
    let mut retained_virtual_line_sequences = Vec::new();
    for (
        ((((column, glyph_advances), column_height), constraints), column_frame),
        virtual_sequence,
    ) in columns
        .into_iter()
        .zip(column_advances)
        .zip(column_heights)
        .zip(column_constraints)
        .zip(column_layout.frames)
        .zip(column_virtual_line_sequences)
    {
        let placement_frame = UiFrame::new(
            column_frame.x,
            frame.y + constraints.inset,
            column_frame.width,
            constraints.max_height,
        );
        let column_frame = UiFrame::new(
            column_frame.x,
            paragraph_layout::aligned_column_y(frame, column_height, constraints),
            column_frame.width,
            column_frame.height,
        );
        if placement_frame.intersection(clip).is_none() {
            overflow_clipped = true;
            continue;
        }
        retained_virtual_line_sequences.push(virtual_sequence);
        resolved_lines.push(UiResolvedTextLine {
            text: column.text,
            frame: column_frame,
            placement_frame,
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

    let layout = UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size,
        line_height: column_advance,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: 0,
            end: parsed.text().len(),
        },
        lines: resolved_lines,
        boxes: Vec::new(),
        overflow_clipped,
        editable: None,
        rich_text_artifact: None,
    };
    TextShapingOutcome::Ready(Some(LayoutWithoutArtifact::with_virtual_line_sequences(
        layout,
        retained_virtual_line_sequences,
    )))
}
