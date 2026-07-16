use crate::core::math::Vec4;
use crate::text::RichTableCell;
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

pub(super) fn resolved_cell_boxes(
    axes: TableAxes,
    table_frame: UiFrame,
    grid: &TableGrid<'_>,
    column_origins: &[f32],
    column_extents: &[f32],
    row_origins: &[f32],
    row_extents: &[f32],
    column_gap: f32,
) -> Vec<UiResolvedTextBox> {
    grid.cells
        .iter()
        .filter_map(|placed| {
            let style = &placed.cell.box_style;
            let background = if placed.row % 2 == 0 {
                style.odd_row_background
            } else {
                style.even_row_background
            };
            if background.is_none() && style.border_color.is_none() {
                return None;
            }
            Some(UiResolvedTextBox {
                range: UiTextRange {
                    start: placed.cell.byte_range.0 as usize,
                    end: placed.cell.byte_range.1 as usize,
                },
                frame: axes.physical_frame(
                    table_frame,
                    column_origins
                        .get(placed.column)
                        .copied()
                        .unwrap_or_default(),
                    row_origins.get(placed.row).copied().unwrap_or_default(),
                    track_span_extent(
                        column_extents,
                        placed.column,
                        placed.column_span,
                        column_gap,
                    ),
                    track_span_extent(row_extents, placed.row, placed.row_span, 0.0),
                ),
                background_color: background.map(ui_rgba_color),
                border_color: style.border_color.map(ui_rgba_color),
                border_width: style.border_color.map_or(0.0, |_| 1.0),
            })
        })
        .collect()
}

pub(super) fn track_span_extent(extents: &[f32], start: usize, span: usize, gap: f32) -> f32 {
    let end = start.saturating_add(span).min(extents.len());
    extents[start.min(end)..end].iter().sum::<f32>()
        + gap * end.saturating_sub(start.saturating_add(1)) as f32
}

pub(super) fn track_origins(sizes: &[f32]) -> Vec<f32> {
    let mut cursor = 0.0;
    sizes
        .iter()
        .map(|size| {
            let origin = cursor;
            cursor += *size;
            origin
        })
        .collect()
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
        line.frame.x += delta_x;
        line.frame.y += delta_y;
    }
    for text_box in &mut layout.boxes {
        text_box.frame.x += delta_x;
        text_box.frame.y += delta_y;
    }
    layout
        .lines
        .retain(|line| line.frame.intersection(clip).is_some());
    layout
        .boxes
        .retain(|text_box| text_box.frame.intersection(clip).is_some());
    original_line_count != layout.lines.len() || original_box_count != layout.boxes.len()
}

fn ui_rgba_color(color: Vec4) -> UiRgbaColor {
    UiRgbaColor::new(color.x, color.y, color.z, color.w)
}
