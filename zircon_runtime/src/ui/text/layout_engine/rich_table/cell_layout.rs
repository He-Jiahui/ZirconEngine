//! 保存表格单元的两阶段布局结果，并复用含间隙的轨道几何。
//! 本模块生成拥有的盒子和布局数据；最终发布前的预算准入由主布局调用层负责。

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

/// 未显式指定内边距时，物理左右内边距各使用的字号比例。
const TABLE_CELL_INLINE_PADDING_EM: f32 = 0.35;
/// 未显式指定内边距时，物理上下内边距各使用的字号比例。
const TABLE_CELL_BLOCK_PADDING_EM: f32 = 0.2;

/// 单元测量完成、行轨道尚未最终定位时保存的拥有数据。
/// 行列坐标及跨度来自布局网格；主布局随后按最终轨道位置消费本结构。
pub(super) struct PreparedTableCellLayout {
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
    pub padding: ResolvedCellPadding,
    /// 首次内容测量使用的物理框，最终定位用它计算平移量。
    pub provisional_content_frame: UiFrame,
    /// 已回移源范围的拥有布局；最终定位会平移并消费其中的行和盒子。
    pub layout: UiResolvedTextLayout,
}

/// 按物理左、上、右、下保存的内边距；逻辑轴求和时再应用书写方向。
#[derive(Clone, Copy)]
pub(super) struct ResolvedCellPadding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// 拥有轨道长度、对应起点及总长，使同一布局阶段复用一致的间隙几何。
/// 总长和跨轨道长度只含内部间隙，不含最后一条轨道后的间隙。
#[derive(Clone, Debug, PartialEq)]
pub(super) struct TrackMetrics {
    extents: Vec<f32>,
    origins: Vec<f32>,
    total_extent: f32,
}

impl TrackMetrics {
    /// 接管轨道长度并在给定预算内构造起点表。
    ///
    /// # 参数与返回
    /// `gap` 是相邻轨道间距；`budget` 分别约束单轴长度和累计长度。
    /// 成功返回拥有输入长度和新起点表的指标；任何间距、轨道或累计值拒绝都返回 violation。
    /// 空轨道仍验证间距，成功时总长为零；此处不向会话写入拒绝诊断。
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

    /// 借用指标读取轨道起点，索引超出轨道数时返回空值。
    pub(super) fn origin(&self, index: usize) -> Option<f32> {
        self.extents.get(index).map(|_| self.origins[index])
    }

    /// 返回夹取到现有轨道后的跨度长度，空跨度或越界起点返回零。
    /// 起点加跨度采用饱和计算，结果包含跨度内部间隙；查询不重新验证几何预算。
    pub(super) fn span_extent(&self, start: usize, span: usize) -> f32 {
        let start = start.min(self.extents.len());
        let end = start.saturating_add(span).min(self.extents.len());
        if start == end {
            return 0.0;
        }
        let last = end - 1;
        (self.origins[last] + self.extents[last] - self.origins[start]).max(0.0)
    }

    /// 返回构造时保存的总长；末尾不追加间距。
    pub(super) fn total_extent(&self) -> f32 {
        self.total_extent
    }
}

impl ResolvedCellPadding {
    /// 将物理内边距投影到行内方向并相加，不在此验证数值。
    pub(super) fn inline_sum(self, axes: TableAxes) -> f32 {
        match axes {
            TableAxes::HorizontalTb => self.left + self.right,
            TableAxes::VerticalRl => self.top + self.bottom,
        }
    }

    /// 将物理内边距投影到块方向并相加，不在此验证数值。
    pub(super) fn block_sum(self, axes: TableAxes) -> f32 {
        match axes {
            TableAxes::HorizontalTb => self.top + self.bottom,
            TableAxes::VerticalRl => self.left + self.right,
        }
    }
}

/// 优先复制单元显式内边距，否则以至少为一的字号计算默认物理内边距。
/// 返回值不持有单元借用；几何是否合法仍由后续预算准入判断。
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

/// 将已测内容的行内长度与对应内边距合成首轮列宽需求。
/// 列号和跨度随需求保留，交由列求解器处理；本函数不做预算准入。
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

/// 将最终单元内容转为行轨道约束，并加上块向内边距。
/// 竖排内容的块长度至少取字号，横排使用测量高度；行号与跨度原样交给行求解器。
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

/// 为带背景或边框的单元生成拥有的装饰盒子。
///
/// # 参数与返回
/// 网格、行列指标与按单元次序提供的源范围共同确定盒子；源范围必须与全部单元一一对应。
/// 即使单元没有装饰，也先消费其源范围；范围不足或有剩余都返回布局失败。
///
/// # 约束
/// 行号零起算并据奇偶选择行背景，边框存在时使用固定宽度。
/// 本函数不验证源范围内容或执行裁剪；调用层随后验证生成盒子的几何。
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

/// 原地平移布局，再删除与裁剪框完全不相交的行和盒子。
///
/// # 参数与返回
/// `delta_x`、`delta_y` 是物理平移量；行按放置框、盒子按自身框判断相交。
/// 仅当行或盒子数量减少时返回真，调用方据此累计溢出标记。
///
/// # 约束
/// 保留项的矩形不会被切小，源范围和测量尺寸也不改；平移采用有限值恢复，仍需后续几何准入。
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

/// 平移物理原点并使用共享有限值求和，保持宽高不变。
fn translate_frame(frame: &mut UiFrame, delta_x: f32, delta_y: f32) {
    frame.x = finite_sum([frame.x, delta_x]);
    frame.y = finite_sum([frame.y, delta_y]);
}

/// 将四个向量分量按原次序传给界面颜色构造器。
fn ui_rgba_color(color: Vec4) -> UiRgbaColor {
    UiRgbaColor::new(color.x, color.y, color.z, color.w)
}

#[cfg(test)]
#[path = "tests/cell_layout.rs"]
mod tests;
