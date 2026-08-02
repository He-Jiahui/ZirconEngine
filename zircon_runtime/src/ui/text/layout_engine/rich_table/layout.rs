use crate::text::RichTable;
use crate::text::SharedTextLayoutSession;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiResolvedTextLayout, UiTextRange},
};

use super::super::super::rich_text::UiParsedText;
use super::super::{
    layout_parsed_text_with_provider, layout_parsed_text_without_tables_with_provider,
};
use super::{
    axes::TableAxes,
    cell_layout::{
        PreparedTableCellLayout, preferred_column_extent, resolved_cell_boxes,
        resolved_cell_padding, row_extent_constraint, track_origins, track_span_extent,
        translate_layout_and_clip,
    },
    grid::TableGrid,
    sizing::{PreferredColumnExtent, resolve_column_extents, resolve_row_extents},
    source_slice::{
        layout_range_with_provider, shift_layout_source_ranges, slice_parsed,
        slice_parsed_with_table_depth,
    },
};

const TABLE_COLUMN_GAP_EM: f32 = 0.2;
const MIN_TABLE_COLUMN_EM: f32 = 1.0;
const MAX_PROVISIONAL_CELL_BLOCK_EXTENT: f32 = f32::MAX / 4.0;

struct PreparedTableCellSource {
    source_start: usize,
    parsed: UiParsedText,
}

pub(in super::super) fn layout_rich_tables_with_provider(
    parsed: &UiParsedText,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip_frame: Option<UiFrame>,
    provider: &mut SharedTextLayoutSession,
) -> Option<UiResolvedTextLayout> {
    let axes = TableAxes::from_style(style);
    let top_level_tables: Vec<&RichTable> = parsed
        .tables()
        .filter(|table| table.depth == parsed.table_root_depth() && !table.cells.is_empty())
        .collect();
    if top_level_tables.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    let mut cursor = 0_usize;
    let mut consumed_block = 0.0_f32;
    let mut measured_inline = 0.0_f32;
    let mut overflow_clipped = false;
    let mut boxes = Vec::new();
    let clip = clip_frame.unwrap_or(frame);

    for table in top_level_tables {
        let table_start = usize::try_from(table.byte_range.0)
            .ok()?
            .saturating_sub(parsed.source_offset())
            .min(parsed.text().len());
        let table_end = usize::try_from(table.byte_range.1)
            .ok()?
            .saturating_sub(parsed.source_offset())
            .min(parsed.text().len());
        if cursor < table_start {
            let segment_range = trim_block_delimiters(parsed.text(), cursor, table_start);
            if segment_range.start < segment_range.end {
                let block = layout_range_with_provider(
                    parsed,
                    segment_range,
                    style,
                    axes.remaining_frame(frame, consumed_block),
                    clip,
                    provider,
                );
                consumed_block += axes.layout_block_extent(&block);
                measured_inline = measured_inline.max(axes.layout_inline_extent(&block));
                overflow_clipped |= block.overflow_clipped;
                boxes.extend(block.boxes);
                lines.extend(block.lines);
            }
        }

        let table_layout = layout_table_with_provider(
            parsed,
            table,
            style,
            axes.remaining_frame(frame, consumed_block),
            clip,
            axes,
            provider,
        );
        consumed_block += axes.layout_block_extent(&table_layout);
        measured_inline = measured_inline.max(axes.layout_inline_extent(&table_layout));
        overflow_clipped |= table_layout.overflow_clipped;
        boxes.extend(table_layout.boxes);
        lines.extend(table_layout.lines);
        cursor = cursor.max(table_end);
    }

    let tail = trim_block_delimiters(parsed.text(), cursor, parsed.text().len());
    if tail.start < tail.end {
        let block = layout_range_with_provider(
            parsed,
            tail,
            style,
            axes.remaining_frame(frame, consumed_block),
            clip,
            provider,
        );
        consumed_block += axes.layout_block_extent(&block);
        measured_inline = measured_inline.max(axes.layout_inline_extent(&block));
        overflow_clipped |= block.overflow_clipped;
        boxes.extend(block.boxes);
        lines.extend(block.lines);
    }

    let baseline = layout_parsed_text_without_tables_with_provider(
        &slice_parsed(parsed, 0..0),
        style,
        frame,
        Some(clip),
        provider,
    );
    let (measured_width, measured_height) = axes.physical_extents(measured_inline, consumed_block);
    Some(UiResolvedTextLayout {
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
    })
}

fn layout_table_with_provider(
    parsed: &UiParsedText,
    table: &RichTable,
    style: &UiResolvedStyle,
    frame: UiFrame,
    clip: UiFrame,
    axes: TableAxes,
    provider: &mut SharedTextLayoutSession,
) -> UiResolvedTextLayout {
    let grid = TableGrid::from_table(table);
    let column_count = grid.column_count;
    let column_gap = style.font_size.max(1.0) * TABLE_COLUMN_GAP_EM;
    let available_track_extent =
        (axes.inline_extent(frame) - column_gap * column_count.saturating_sub(1) as f32).max(0.0);
    let cell_sources = prepare_cell_sources(parsed, &grid, table.depth);
    let preferred_cells = preferred_cell_extents(&cell_sources, &grid, style, axes, provider);
    let minimum_column_extent = (style.font_size.max(1.0) * MIN_TABLE_COLUMN_EM)
        .min(available_track_extent / column_count.max(1) as f32);
    let column_extents = resolve_column_extents(
        &table.columns,
        &preferred_cells,
        available_track_extent,
        column_gap,
        minimum_column_extent,
    );
    let column_origins = track_origins(&column_extents);

    let mut prepared_cells = Vec::with_capacity(grid.cells.len());
    let mut row_constraints = Vec::with_capacity(grid.cells.len());
    let mut overflow_clipped = false;
    for (cell_index, placed) in grid.cells.iter().enumerate() {
        let padding = resolved_cell_padding(placed.cell, style.font_size);
        let range = usize::try_from(placed.cell.byte_range.0).unwrap_or(0)
            ..usize::try_from(placed.cell.byte_range.1).unwrap_or(0);
        let span_extent = track_span_extent(
            &column_extents,
            placed.column,
            placed.column_span,
            column_gap,
        );
        let provisional_content_frame = provisional_content_frame(
            axes,
            frame,
            column_origins[placed.column],
            span_extent,
            padding,
            provisional_block_extent(range.len(), style),
        );
        let mut layout = layout_parsed_text_with_provider(
            &cell_sources[cell_index].parsed,
            style,
            provisional_content_frame,
            Some(provisional_content_frame),
            provider,
        );
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
    let row_extents = resolve_row_extents(grid.row_count, &row_constraints, minimum_row_extent);
    let row_origins = track_origins(&row_extents);
    let mut boxes = resolved_cell_boxes(
        axes,
        frame,
        &grid,
        parsed.source_offset(),
        &column_origins,
        &column_extents,
        &row_origins,
        &row_extents,
        column_gap,
    );
    let mut lines = Vec::new();
    for mut prepared in prepared_cells {
        let cell_frame = axes.physical_frame(
            frame,
            column_origins
                .get(prepared.column)
                .copied()
                .unwrap_or_default(),
            row_origins.get(prepared.row).copied().unwrap_or_default(),
            track_span_extent(
                &column_extents,
                prepared.column,
                prepared.column_span,
                column_gap,
            ),
            track_span_extent(&row_extents, prepared.row, prepared.row_span, 0.0),
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
        boxes.extend(prepared.layout.boxes);
        lines.extend(prepared.layout.lines);
    }
    let total_inline_extent =
        column_extents.iter().sum::<f32>() + column_gap * column_count.saturating_sub(1) as f32;
    let total_block_extent = row_extents.iter().sum::<f32>();
    let (measured_width, measured_height) =
        axes.physical_extents(total_inline_extent, total_block_extent);

    UiResolvedTextLayout {
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
            start: (table.byte_range.0 as usize).saturating_sub(parsed.source_offset()),
            end: (table.byte_range.1 as usize).saturating_sub(parsed.source_offset()),
        },
        lines,
        boxes,
        overflow_clipped: overflow_clipped
            || total_inline_extent > axes.inline_extent(frame)
            || total_block_extent > axes.block_extent(frame),
        editable: None,
        rich_text_artifact: None,
    }
}

fn preferred_cell_extents(
    cell_sources: &[PreparedTableCellSource],
    grid: &TableGrid<'_>,
    style: &UiResolvedStyle,
    axes: TableAxes,
    provider: &mut SharedTextLayoutSession,
) -> Vec<PreferredColumnExtent> {
    grid.cells
        .iter()
        .enumerate()
        .map(|(cell_index, placed)| {
            let padding = resolved_cell_padding(placed.cell, style.font_size);
            let range = placed.cell.byte_range.0 as usize..placed.cell.byte_range.1 as usize;
            let measure_frame = preferred_measure_frame(axes, range.len(), style);
            let layout = layout_parsed_text_with_provider(
                &cell_sources[cell_index].parsed,
                &UiResolvedStyle {
                    wrap: zircon_runtime_interface::ui::surface::UiTextWrap::None,
                    ..style.clone()
                },
                measure_frame,
                Some(measure_frame),
                provider,
            );
            preferred_column_extent(axes, placed.column, placed.column_span, &layout, padding)
        })
        .collect()
}

fn prepare_cell_sources(
    parsed: &UiParsedText,
    grid: &TableGrid<'_>,
    table_depth: u16,
) -> Vec<PreparedTableCellSource> {
    grid.cells
        .iter()
        .map(|placed| {
            let start = usize::try_from(placed.cell.byte_range.0)
                .unwrap_or_default()
                .saturating_sub(parsed.source_offset())
                .min(parsed.text().len());
            let end = usize::try_from(placed.cell.byte_range.1)
                .unwrap_or_default()
                .saturating_sub(parsed.source_offset())
                .min(parsed.text().len())
                .max(start);
            PreparedTableCellSource {
                source_start: start,
                parsed: slice_parsed_with_table_depth(parsed, start..end, Some(table_depth)),
            }
        })
        .collect()
}

fn preferred_measure_frame(axes: TableAxes, source_len: usize, style: &UiResolvedStyle) -> UiFrame {
    match axes {
        TableAxes::HorizontalTb => UiFrame::new(
            0.0,
            0.0,
            MAX_PROVISIONAL_CELL_BLOCK_EXTENT,
            MAX_PROVISIONAL_CELL_BLOCK_EXTENT,
        ),
        TableAxes::VerticalRl => {
            let extent = provisional_block_extent(source_len, style);
            UiFrame::new(0.0, 0.0, extent, extent)
        }
    }
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

fn provisional_content_frame(
    axes: TableAxes,
    table_frame: UiFrame,
    inline_origin: f32,
    inline_extent: f32,
    padding: super::cell_layout::ResolvedCellPadding,
    block_extent: f32,
) -> UiFrame {
    match axes {
        TableAxes::HorizontalTb => UiFrame::new(
            table_frame.x + inline_origin + padding.left,
            0.0,
            (inline_extent - padding.left - padding.right).max(0.0),
            block_extent,
        ),
        TableAxes::VerticalRl => UiFrame::new(
            0.0,
            table_frame.y + inline_origin + padding.top,
            block_extent,
            (inline_extent - padding.top - padding.bottom).max(0.0),
        ),
    }
}

fn provisional_block_extent(source_len: usize, style: &UiResolvedStyle) -> f32 {
    (source_len.max(1) as f32 * style.line_height.max(style.font_size).max(1.0))
        .min(MAX_PROVISIONAL_CELL_BLOCK_EXTENT)
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

fn trim_block_delimiters(text: &str, start: usize, end: usize) -> std::ops::Range<usize> {
    let mut start = start.min(text.len());
    let mut end = end.min(text.len()).max(start);
    while start < end && text.as_bytes()[start] == b'\n' {
        start += 1;
    }
    while start < end && text.as_bytes()[end - 1] == b'\n' {
        end -= 1;
    }
    start..end
}
