use crate::core::framework::text::TextLayoutError;
use crate::text::RichTable;
use crate::text::SharedTextLayoutSession;
use crate::text::TextLayoutAxisConstraint;
use crate::text::TextLayoutGeometryOwner;
use crate::text::is_hard_line_separator;
use crate::text::shaping::{TextLayoutOutcome, TextShapingOutcome};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextRange},
};

use super::super::super::rich_text::UiParsedText;
use super::super::geometry_admission::{
    validate_resolved_layout_geometry, validate_resolved_text_boxes_geometry,
};
use super::super::measurement::{
    bounded_inline_measurement_frame_with_provider, intrinsic_measurement_frame_with_provider,
};
use super::super::{
    layout_parsed_text_with_provider_outcome, layout_parsed_text_without_tables_with_provider,
};
use super::{
    axes::TableAxes,
    cell_layout::{
        PreparedTableCellLayout, TrackMetrics, preferred_column_extent, resolved_cell_boxes,
        resolved_cell_padding, row_extent_constraint, translate_layout_and_clip,
    },
    geometry::{admit_aggregate_layout_geometry, finite_max_zero, whole_parsed_source_range},
    grid::TableGrid,
    sizing::{PreferredColumnExtent, resolve_column_extents, resolve_row_extents},
    source_slice::{
        layout_range_with_provider, shift_layout_source_ranges, slice_parsed,
        slice_parsed_with_table_depth,
    },
};

const TABLE_COLUMN_GAP_EM: f32 = 0.2;
const MIN_TABLE_COLUMN_EM: f32 = 1.0;

struct PreparedTableCellSource {
    source_start: usize,
    source_range: std::ops::Range<usize>,
    parsed: UiParsedText,
}

pub(in super::super) fn layout_rich_tables_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Option<UiResolvedTextLayout>> {
    let axes = TableAxes::from_style(style);
    let top_level_tables: Vec<&RichTable> = parsed
        .tables()
        .filter(|table| table.depth == parsed.table_root_depth() && !table.cells.is_empty())
        .collect();
    if top_level_tables.is_empty() {
        return TextShapingOutcome::Ready(None);
    }

    let mut lines = Vec::new();
    let mut cursor = 0_usize;
    let mut consumed_block = 0.0_f32;
    let mut measured_inline = 0.0_f32;
    let mut overflow_clipped = false;
    let mut boxes = Vec::new();
    let clip = clip_frame.unwrap_or(frame);

    for table in top_level_tables {
        let table_range = match checked_table_source_range(parsed, table, None) {
            Ok(range) => range,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        let table_start = table_range.start;
        let table_end = table_range.end;
        if table_start < cursor {
            return TextShapingOutcome::failed(TextLayoutError::LayoutFailed);
        }
        if cursor < table_start {
            let segment_range = match trim_block_delimiters(parsed.text(), cursor, table_start) {
                Ok(range) => range,
                Err(error) => return TextShapingOutcome::failed(error),
            };
            if segment_range.start < segment_range.end {
                let block = match layout_range_with_provider(
                    parsed,
                    segment_range,
                    style,
                    axes.remaining_frame(frame, consumed_block),
                    clip,
                    provider,
                ) {
                    TextShapingOutcome::Ready(layout) => layout,
                    TextShapingOutcome::Deferred(error) => {
                        return TextShapingOutcome::Deferred(error);
                    }
                    TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
                };
                (consumed_block, measured_inline) = match admit_aggregate_layout_geometry(
                    &block,
                    axes,
                    consumed_block,
                    measured_inline,
                    whole_parsed_source_range(parsed),
                    provider,
                ) {
                    Ok(extents) => extents,
                    Err(error) => return TextShapingOutcome::failed(error),
                };
                overflow_clipped |= block.overflow_clipped;
                boxes.extend(block.boxes);
                lines.extend(block.lines);
            }
        }

        let table_layout = match layout_table_with_provider(
            parsed,
            table,
            style,
            axes.remaining_frame(frame, consumed_block),
            clip,
            axes,
            provider,
        ) {
            TextShapingOutcome::Ready(layout) => layout,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        (consumed_block, measured_inline) = match admit_aggregate_layout_geometry(
            &table_layout,
            axes,
            consumed_block,
            measured_inline,
            Some(table.byte_range),
            provider,
        ) {
            Ok(extents) => extents,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        overflow_clipped |= table_layout.overflow_clipped;
        boxes.extend(table_layout.boxes);
        lines.extend(table_layout.lines);
        cursor = cursor.max(table_end);
    }

    let tail = match trim_block_delimiters(parsed.text(), cursor, parsed.text().len()) {
        Ok(range) => range,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    if tail.start < tail.end {
        let block = match layout_range_with_provider(
            parsed,
            tail,
            style,
            axes.remaining_frame(frame, consumed_block),
            clip,
            provider,
        ) {
            TextShapingOutcome::Ready(layout) => layout,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        (consumed_block, measured_inline) = match admit_aggregate_layout_geometry(
            &block,
            axes,
            consumed_block,
            measured_inline,
            whole_parsed_source_range(parsed),
            provider,
        ) {
            Ok(extents) => extents,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        overflow_clipped |= block.overflow_clipped;
        boxes.extend(block.boxes);
        lines.extend(block.lines);
    }

    let empty = match slice_parsed(parsed, 0..0) {
        Ok(empty) => empty,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    let baseline = match layout_parsed_text_without_tables_with_provider(
        &empty,
        style,
        frame,
        Some(clip),
        provider,
    ) {
        TextShapingOutcome::Ready(layout) => layout,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let (measured_width, measured_height) = axes.physical_extents(measured_inline, consumed_block);
    TextShapingOutcome::Ready(Some(UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction: baseline.direction,
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size: baseline.font_size,
        line_height: baseline.line_height,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: 0,
            end: parsed.text().len(),
        },
        lines,
        boxes,
        overflow_clipped: overflow_clipped || consumed_block > axes.block_extent(frame),
        editable: None,
        rich_text_artifact: None,
    }))
}

fn layout_table_with_provider(
    parsed: &UiParsedText,
    table: &RichTable,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    axes: TableAxes,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<UiResolvedTextLayout> {
    let table_source_range = match checked_table_source_range(parsed, table, None) {
        Ok(range) => range,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    let grid = TableGrid::from_table(table);
    provider.record_table_layout_attempt(
        table_source_range
            .end
            .saturating_sub(table_source_range.start),
        grid.cells.len(),
    );
    let column_count = grid.column_count;
    let geometry_budget = provider.geometry_budget();
    let geometry_source_range = Some(table.byte_range);
    let geometry_work_units = grid.cells.len();
    let column_gap =
        match geometry_budget.admit_axis_extent(style.font_size.max(1.0) * TABLE_COLUMN_GAP_EM) {
            Ok(gap) => gap,
            Err(violation) => {
                return TextShapingOutcome::failed(provider.reject_geometry(
                    TextLayoutGeometryOwner::TableAvailableTrackExtent,
                    violation,
                    geometry_source_range,
                    geometry_work_units,
                ));
            }
        };
    let available_inline = match TextLayoutAxisConstraint::from_request_extent(
        axes.inline_extent(frame),
        geometry_budget,
    ) {
        Ok(constraint) => constraint,
        Err(violation) => {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableAvailableTrackExtent,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
    };
    let total_column_gap = match geometry_budget
        .checked_scale_accumulated(column_gap, column_count.saturating_sub(1))
    {
        Ok(extent) => extent,
        Err(violation) => {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableAvailableTrackExtent,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
    };
    let available_track_extent =
        match available_inline.subtract_accumulated(total_column_gap, geometry_budget) {
            Ok(constraint) => constraint,
            Err(violation) => {
                return TextShapingOutcome::failed(provider.reject_geometry(
                    TextLayoutGeometryOwner::TableAvailableTrackExtent,
                    violation,
                    geometry_source_range,
                    geometry_work_units,
                ));
            }
        };
    let cell_sources = match prepare_cell_sources(parsed, &grid, table, table.depth) {
        Ok(sources) => sources,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    let preferred_cells = match preferred_cell_extents(&cell_sources, &grid, style, axes, provider)
    {
        TextShapingOutcome::Ready(extents) => extents,
        TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
        TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
    };
    let natural_minimum_column_extent = style.font_size.max(1.0) * MIN_TABLE_COLUMN_EM;
    let minimum_column_extent = available_track_extent
        .bounded_extent()
        .map_or(natural_minimum_column_extent, |available| {
            natural_minimum_column_extent.min(available / column_count.max(1) as f32)
        });
    let column_extents = match resolve_column_extents(
        &table.columns,
        &preferred_cells,
        available_track_extent,
        column_gap,
        minimum_column_extent,
        geometry_budget,
    ) {
        Ok(extents) => extents,
        Err(violation) => {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableColumnTracks,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
    };
    let column_metrics = match TrackMetrics::new(column_extents, column_gap, geometry_budget) {
        Ok(metrics) => metrics,
        Err(violation) => {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableColumnTracks,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
    };

    let mut prepared_cells = Vec::with_capacity(grid.cells.len());
    let mut row_constraints = Vec::with_capacity(grid.cells.len());
    let mut overflow_clipped = false;
    for (cell_index, placed) in grid.cells.iter().enumerate() {
        let padding = resolved_cell_padding(placed.cell, style.font_size);
        let span_extent = column_metrics.span_extent(placed.column, placed.column_span);
        let (content_origin_x, content_origin_y, content_inline_extent) = match axes {
            TableAxes::HorizontalTb => (
                frame.x + column_metrics.origin(placed.column).unwrap_or_default() + padding.left,
                0.0,
                finite_max_zero(span_extent - padding.left - padding.right),
            ),
            TableAxes::VerticalRl => (
                0.0,
                frame.y + column_metrics.origin(placed.column).unwrap_or_default() + padding.top,
                finite_max_zero(span_extent - padding.top - padding.bottom),
            ),
        };
        let provisional_content_frame = match bounded_inline_measurement_frame_with_provider(
            style,
            content_origin_x,
            content_origin_y,
            content_inline_extent,
            TextLayoutGeometryOwner::TableCellFrame,
            Some(placed.cell.byte_range),
            cell_sources[cell_index].parsed.text().len(),
            provider,
        ) {
            Ok(frame) => frame,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        provider.record_table_final_cell_layout(cell_sources[cell_index].parsed.text().len());
        let mut layout = match layout_parsed_text_with_provider_outcome(
            &cell_sources[cell_index].parsed,
            style,
            provisional_content_frame,
            Some(provisional_content_frame),
            provider,
        ) {
            TextShapingOutcome::Ready(layout) => layout,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        if let Err(violation) =
            validate_resolved_layout_geometry(&layout, provider.geometry_budget())
        {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableCellFrame,
                violation,
                Some(placed.cell.byte_range),
                cell_sources[cell_index].parsed.text().len(),
            ));
        }
        shift_layout_source_ranges(&mut layout, cell_sources[cell_index].source_start);
        overflow_clipped |= layout.overflow_clipped;
        row_constraints.push(row_extent_constraint(
            axes,
            placed.row,
            placed.row_span,
            &layout,
            padding,
        ));
        prepared_cells.push(PreparedTableCellLayout {
            row: placed.row,
            column: placed.column,
            row_span: placed.row_span,
            column_span: placed.column_span,
            padding,
            provisional_content_frame,
            layout,
        });
    }

    let minimum_row_extent = style.line_height.max(1.0);
    let row_extents = match resolve_row_extents(
        grid.row_count,
        &row_constraints,
        minimum_row_extent,
        geometry_budget,
    ) {
        Ok(extents) => extents,
        Err(violation) => {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableRowTracks,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
    };
    let row_metrics = match TrackMetrics::new(row_extents, 0.0, geometry_budget) {
        Ok(metrics) => metrics,
        Err(violation) => {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableRowTracks,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
    };
    provider.record_table_layout_tracks(grid.column_count, grid.row_count);
    let mut boxes = match resolved_cell_boxes(
        axes,
        frame,
        &grid,
        &column_metrics,
        &row_metrics,
        cell_sources
            .iter()
            .map(|source| source.source_range.clone()),
    ) {
        Ok(boxes) => boxes,
        Err(error) => return TextShapingOutcome::failed(error),
    };
    if let Err(violation) = validate_resolved_text_boxes_geometry(&boxes, geometry_budget) {
        return TextShapingOutcome::failed(provider.reject_geometry(
            TextLayoutGeometryOwner::TableCellFrame,
            violation,
            geometry_source_range,
            geometry_work_units,
        ));
    }
    let mut lines = Vec::new();
    for mut prepared in prepared_cells {
        let cell_frame = axes.physical_frame(
            frame,
            column_metrics.origin(prepared.column).unwrap_or_default(),
            row_metrics.origin(prepared.row).unwrap_or_default(),
            column_metrics.span_extent(prepared.column, prepared.column_span),
            row_metrics.span_extent(prepared.row, prepared.row_span),
        );
        let target_content_frame = inset_physical_frame(cell_frame, prepared.padding);
        let content_clip = target_content_frame
            .intersection(clip)
            .unwrap_or(UiFrame::new(clip.x, clip.y, 0.0, 0.0));
        let delta_x = match axes {
            TableAxes::HorizontalTb => {
                target_content_frame.x - prepared.provisional_content_frame.x
            }
            TableAxes::VerticalRl => {
                target_content_frame.right() - vertical_layout_right_anchor(&prepared.layout)
            }
        };
        let translated_outside_clip = translate_layout_and_clip(
            &mut prepared.layout,
            delta_x,
            target_content_frame.y - prepared.provisional_content_frame.y,
            content_clip,
        );
        overflow_clipped |= translated_outside_clip;
        if let Err(violation) = validate_resolved_layout_geometry(&prepared.layout, geometry_budget)
        {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TableCellFrame,
                violation,
                geometry_source_range,
                geometry_work_units,
            ));
        }
        boxes.extend(prepared.layout.boxes);
        lines.extend(prepared.layout.lines);
    }
    let total_inline_extent = column_metrics.total_extent();
    let total_block_extent = row_metrics.total_extent();
    let (measured_width, measured_height) =
        axes.physical_extents(total_inline_extent, total_block_extent);

    let layout = UiResolvedTextLayout {
        text_align: style.text_align,
        wrap: style.wrap,
        direction: lines.first().map(|line| line.direction).unwrap_or_default(),
        writing_mode: style.text_writing_mode,
        overflow: style.text_overflow,
        font_size: style.font_size,
        line_height: style.line_height,
        measured_width,
        measured_height,
        source_range: UiTextRange {
            start: table_source_range.start,
            end: table_source_range.end,
        },
        lines,
        boxes,
        overflow_clipped: overflow_clipped
            || total_inline_extent > axes.inline_extent(frame)
            || total_block_extent > axes.block_extent(frame),
        editable: None,
        rich_text_artifact: None,
    };
    if let Err(violation) = validate_resolved_layout_geometry(&layout, geometry_budget) {
        return TextShapingOutcome::failed(provider.reject_geometry(
            TextLayoutGeometryOwner::TableAggregate,
            violation,
            geometry_source_range,
            geometry_work_units,
        ));
    }
    provider.record_table_layout_output(layout.lines.len(), layout.boxes.len());
    TextShapingOutcome::Ready(layout)
}

fn preferred_cell_extents(
    cell_sources: &[PreparedTableCellSource],
    grid: &TableGrid<'_>,
    style: &UiResolvedStyle,
    axes: TableAxes,
    provider: &mut SharedTextLayoutSession,
) -> TextLayoutOutcome<Vec<PreferredColumnExtent>> {
    let mut preferred = Vec::with_capacity(grid.cells.len());
    for (cell_index, placed) in grid.cells.iter().enumerate() {
        let padding = resolved_cell_padding(placed.cell, style.font_size);
        let intrinsic_style = UiResolvedStyle {
            wrap: zircon_runtime_interface::ui::surface::UiTextWrap::None,
            text_overflow: zircon_runtime_interface::ui::surface::UiTextOverflow::Clip,
            text_align: zircon_runtime_interface::ui::surface::UiTextAlign::Left,
            ..style.clone()
        };
        let measure_frame = match intrinsic_measurement_frame_with_provider(
            cell_sources[cell_index].parsed.text(),
            &intrinsic_style,
            provider,
        ) {
            Ok(frame) => frame,
            Err(error) => return TextShapingOutcome::failed(error),
        };
        provider.record_table_preferred_cell_layout(cell_sources[cell_index].parsed.text().len());
        let layout = match layout_parsed_text_with_provider_outcome(
            &cell_sources[cell_index].parsed,
            &intrinsic_style,
            measure_frame,
            None,
            provider,
        ) {
            TextShapingOutcome::Ready(layout) => layout,
            TextShapingOutcome::Deferred(error) => return TextShapingOutcome::Deferred(error),
            TextShapingOutcome::Failed(error) => return TextShapingOutcome::Failed(error),
        };
        if let Err(violation) =
            validate_resolved_layout_geometry(&layout, provider.geometry_budget())
        {
            return TextShapingOutcome::failed(provider.reject_geometry(
                TextLayoutGeometryOwner::TablePreferredCell,
                violation,
                Some(placed.cell.byte_range),
                cell_sources[cell_index].parsed.text().len(),
            ));
        }
        preferred.push(preferred_column_extent(
            axes,
            placed.column,
            placed.column_span,
            &layout,
            padding,
        ));
    }
    TextShapingOutcome::Ready(preferred)
}

fn prepare_cell_sources(
    parsed: &UiParsedText,
    grid: &TableGrid<'_>,
    table: &RichTable,
    table_depth: u16,
) -> Result<Vec<PreparedTableCellSource>, TextLayoutError> {
    let table_range = checked_table_source_range(parsed, table, None)?;
    let mut previous_cell_end = table_range.start;
    grid.cells
        .iter()
        .map(|placed| {
            let range = checked_table_source_range(parsed, table, Some(placed.cell.byte_range))?;
            if range.start < previous_cell_end {
                return Err(TextLayoutError::LayoutFailed);
            }
            previous_cell_end = range.end;
            Ok(PreparedTableCellSource {
                source_start: range.start,
                source_range: range.clone(),
                parsed: slice_parsed_with_table_depth(parsed, range, Some(table_depth))?,
            })
        })
        .collect()
}

fn checked_table_source_range(
    parsed: &UiParsedText,
    table: &RichTable,
    cell_range: Option<(u32, u32)>,
) -> Result<std::ops::Range<usize>, TextLayoutError> {
    let source_start = parsed.source_offset();
    let source_end = source_start
        .checked_add(parsed.text().len())
        .ok_or(TextLayoutError::LayoutFailed)?;
    let table_start =
        usize::try_from(table.byte_range.0).map_err(|_| TextLayoutError::LayoutFailed)?;
    let table_end =
        usize::try_from(table.byte_range.1).map_err(|_| TextLayoutError::LayoutFailed)?;
    let (start, end) = cell_range.unwrap_or((table.byte_range.0, table.byte_range.1));
    let start = usize::try_from(start).map_err(|_| TextLayoutError::LayoutFailed)?;
    let end = usize::try_from(end).map_err(|_| TextLayoutError::LayoutFailed)?;
    let containing_end = if cell_range.is_some() {
        table_end
    } else {
        source_end
    };
    let containing_start = if cell_range.is_some() {
        table_start
    } else {
        source_start
    };
    if start > end
        || start < containing_start
        || end > containing_end
        || start < source_start
        || end > source_end
    {
        return Err(TextLayoutError::LayoutFailed);
    }
    let local_start = start
        .checked_sub(source_start)
        .ok_or(TextLayoutError::LayoutFailed)?;
    let local_end = end
        .checked_sub(source_start)
        .ok_or(TextLayoutError::LayoutFailed)?;
    parsed
        .text()
        .get(local_start..local_end)
        .map(|_| local_start..local_end)
        .ok_or(TextLayoutError::LayoutFailed)
}

fn vertical_layout_right_anchor(layout: &UiResolvedTextLayout) -> f32 {
    layout
        .lines
        .iter()
        .map(|line| line.frame.right())
        .chain(layout.boxes.iter().map(|text_box| text_box.frame.right()))
        .reduce(f32::max)
        .unwrap_or_default()
}

fn inset_physical_frame(
    frame: UiFrame,
    padding: super::cell_layout::ResolvedCellPadding,
) -> UiFrame {
    UiFrame::new(
        frame.x + padding.left,
        frame.y + padding.top,
        (frame.width - padding.left - padding.right).max(0.0),
        (frame.height - padding.top - padding.bottom).max(0.0),
    )
}

fn trim_block_delimiters(
    text: &str,
    start: usize,
    end: usize,
) -> Result<std::ops::Range<usize>, TextLayoutError> {
    if start > end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err(TextLayoutError::LayoutFailed);
    }
    let mut start = start;
    let mut end = end;
    while start < end {
        let Some(character) = text.get(start..end).and_then(|slice| slice.chars().next()) else {
            break;
        };
        if !is_hard_line_separator(character) {
            break;
        }
        start += character.len_utf8();
    }
    while start < end {
        let Some((index, character)) = text[..end].char_indices().next_back() else {
            break;
        };
        if !is_hard_line_separator(character) {
            break;
        }
        end = index;
    }
    Ok(start..end)
}

#[cfg(test)]
mod tests {
    use super::trim_block_delimiters;

    #[test]
    fn trim_block_delimiters_uses_all_canonical_hard_line_separators() {
        let text = "\r\n\u{2028}content\u{0085}\u{000b}";

        assert_eq!(
            trim_block_delimiters(text, 0, text.len()).expect("valid text range"),
            "\r\n\u{2028}".len().."\r\n\u{2028}content".len()
        );
    }

    #[test]
    fn trim_block_delimiters_rejects_reversed_or_non_boundary_ranges() {
        let text = "a\u{4e2d}b";

        assert!(trim_block_delimiters(text, 3, 2).is_err());
        assert!(trim_block_delimiters(text, 2, text.len()).is_err());
        assert!(trim_block_delimiters(text, 0, text.len() + 1).is_err());
    }
}
