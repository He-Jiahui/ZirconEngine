use std::ops::Range;

use crate::core::framework::text::TextLayoutError;
use crate::core::math::Vec4;
use crate::text::layout_geometry::finite_sum;
use crate::text::{RichTableCell, TextLayoutGeometryBudget, TextLayoutGeometryViolation};
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    style::UiRgbaColor,
    surface::{UiResolvedTextBox, UiResolvedTextLayout, UiTextRange},
};

use super::{
    axes::TableAxes,
    grid::TableGrid,
    sizing::{PreferredColumnExtent, RowExtentConstraint},
};

const TABLE_CELL_INLINE_PADDING_EM: f32 = 0.35;
const TABLE_CELL_BLOCK_PADDING_EM: f32 = 0.2;

pub(super) struct PreparedTableCellLayout {
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
    pub padding: ResolvedCellPadding,
    pub provisional_content_frame: UiFrame,
    pub layout: UiResolvedTextLayout,
}

#[derive(Clone, Copy)]
pub(super) struct ResolvedCellPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct TrackMetrics {
    extents: Vec<f32>,
    origins: Vec<f32>,
    total_extent: f32,
}

impl TrackMetrics {
    pub(super) fn new(
        extents: Vec<f32>,
        gap: f32,
        budget: TextLayoutGeometryBudget,
    ) -> Result<Self, TextLayoutGeometryViolation> {
        let gap = budget.admit_axis_extent(gap)?;
        let mut origins = Vec::with_capacity(extents.len());
        let mut cursor = 0.0;
        for (index, extent) in extents.iter().enumerate() {
            budget.admit_axis_extent(*extent)?;
            origins.push(cursor);
            cursor = budget.checked_add_accumulated(cursor, *extent)?;
            if index + 1 < extents.len() {
                cursor = budget.checked_add_accumulated(cursor, gap)?;
            }
        }
        Ok(Self {
            extents,
            origins,
            total_extent: cursor,
        })
    }

    pub(super) fn origin(&self, index: usize) -> Option<f32> {
        self.extents.get(index).map(|_| self.origins[index])
    }

    pub(super) fn span_extent(&self, start: usize, span: usize) -> f32 {
        let start = start.min(self.extents.len());
        let end = start.saturating_add(span).min(self.extents.len());
        if start == end {
            return 0.0;
        }
        let last = end - 1;
        (self.origins[last] + self.extents[last] - self.origins[start]).max(0.0)
    }

    pub(super) fn total_extent(&self) -> f32 {
        self.total_extent
    }
}

impl ResolvedCellPadding {
    pub(super) fn inline_sum(self, axes: TableAxes) -> f32 {
        match axes {
            TableAxes::HorizontalTb => self.left + self.right,
            TableAxes::VerticalRl => self.top + self.bottom,
        }
    }

    pub(super) fn block_sum(self, axes: TableAxes) -> f32 {
        match axes {
            TableAxes::HorizontalTb => self.top + self.bottom,
            TableAxes::VerticalRl => self.left + self.right,
        }
    }
}

pub(super) fn resolved_cell_padding(cell: &RichTableCell, font_size: f32) -> ResolvedCellPadding {
    cell.box_style
        .padding
        .map(|padding| ResolvedCellPadding {
            left: padding.left,
            top: padding.top,
            right: padding.right,
            bottom: padding.bottom,
        })
        .unwrap_or_else(|| {
            let font_size = font_size.max(1.0);
            ResolvedCellPadding {
                left: font_size * TABLE_CELL_INLINE_PADDING_EM,
                top: font_size * TABLE_CELL_BLOCK_PADDING_EM,
                right: font_size * TABLE_CELL_INLINE_PADDING_EM,
                bottom: font_size * TABLE_CELL_BLOCK_PADDING_EM,
            }
        })
}

pub(super) fn preferred_column_extent(
    axes: TableAxes,
    column: usize,
    column_span: usize,
    layout: &UiResolvedTextLayout,
    padding: ResolvedCellPadding,
) -> PreferredColumnExtent {
    PreferredColumnExtent {
        column,
        column_span,
        extent: axes.layout_inline_extent(layout) + padding.inline_sum(axes),
    }
}

pub(super) fn row_extent_constraint(
    axes: TableAxes,
    row: usize,
    row_span: usize,
    layout: &UiResolvedTextLayout,
    padding: ResolvedCellPadding,
) -> RowExtentConstraint {
    let content_extent = match axes {
        TableAxes::HorizontalTb => layout.measured_height,
        TableAxes::VerticalRl => layout.measured_width.max(layout.font_size),
    };
    RowExtentConstraint {
        row,
        row_span,
        extent: content_extent + padding.block_sum(axes),
    }
}

pub(super) fn resolved_cell_boxes<I>(
    axes: TableAxes,
    table_frame: UiFrame,
    grid: &TableGrid<'_>,
    column_metrics: &TrackMetrics,
    row_metrics: &TrackMetrics,
    cell_source_ranges: I,
) -> Result<Vec<UiResolvedTextBox>, TextLayoutError>
where
    I: Iterator<Item = Range<usize>>,
{
    let mut cell_source_ranges = cell_source_ranges;
    let mut boxes = Vec::with_capacity(grid.cells.len());
    for placed in &grid.cells {
        let source_range = cell_source_ranges
            .next()
            .ok_or(TextLayoutError::LayoutFailed)?;
        let style = &placed.cell.box_style;
        let background = if placed.row % 2 == 0 {
            style.odd_row_background
        } else {
            style.even_row_background
        };
        if background.is_none() && style.border_color.is_none() {
            continue;
        }
        boxes.push(UiResolvedTextBox {
            range: UiTextRange {
                start: source_range.start,
                end: source_range.end,
            },
            frame: axes.physical_frame(
                table_frame,
                column_metrics.origin(placed.column).unwrap_or_default(),
                row_metrics.origin(placed.row).unwrap_or_default(),
                column_metrics.span_extent(placed.column, placed.column_span),
                row_metrics.span_extent(placed.row, placed.row_span),
            ),
            background_color: background.map(ui_rgba_color),
            border_color: style.border_color.map(ui_rgba_color),
            border_width: style.border_color.map_or(0.0, |_| 1.0),
        });
    }
    if cell_source_ranges.next().is_some() {
        return Err(TextLayoutError::LayoutFailed);
    }
    Ok(boxes)
}

pub(super) fn translate_layout_and_clip(
    layout: &mut UiResolvedTextLayout,
    delta_x: f32,
    delta_y: f32,
    clip: UiFrame,
) -> bool {
    let original_line_count = layout.lines.len();
    let original_box_count = layout.boxes.len();
    for line in &mut layout.lines {
        translate_frame(&mut line.frame, delta_x, delta_y);
        translate_frame(&mut line.placement_frame, delta_x, delta_y);
    }
    for text_box in &mut layout.boxes {
        translate_frame(&mut text_box.frame, delta_x, delta_y);
    }
    layout
        .lines
        .retain(|line| line.placement_frame.intersection(clip).is_some());
    layout
        .boxes
        .retain(|text_box| text_box.frame.intersection(clip).is_some());
    original_line_count != layout.lines.len() || original_box_count != layout.boxes.len()
}

fn translate_frame(frame: &mut UiFrame, delta_x: f32, delta_y: f32) {
    frame.x = finite_sum([frame.x, delta_x]);
    frame.y = finite_sum([frame.y, delta_y]);
}

fn ui_rgba_color(color: Vec4) -> UiRgbaColor {
    UiRgbaColor::new(color.x, color.y, color.z, color.w)
}

#[cfg(test)]
#[path = "tests/cell_layout.rs"]
mod tests;
