//! 将表格输入坐标投影为布局轨道索引，拥有放置描述但借用原单元。
//! 坐标与跨度夹取只限制布局索引，不代替完整表语义或资源预算验证。

use crate::text::{RichTable, RichTableCell, MAX_RICH_TABLE_ROW_SPAN};

/// 一个单元的派生放置坐标；`cell` 的借用不长于传入表格。
/// 行列和跨度使用投影后的值，原单元的坐标字段不被修改。
#[derive(Clone, Copy, Debug)]
pub(super) struct PlacedTableCell<'a> {
    pub cell: &'a RichTableCell,
    pub row: usize,
    pub column: usize,
    pub row_span: usize,
    pub column_span: usize,
}

/// 拥有单元放置向量和轨道计数；向量中的单元引用继续借用原表格。
/// 列数至少为一，行数由放置单元的末端决定；空单元集合的行数为零。
#[derive(Clone, Debug)]
pub(super) struct TableGrid<'a> {
    pub column_count: usize,
    pub row_count: usize,
    pub cells: Vec<PlacedTableCell<'a>>,
}

impl<'a> TableGrid<'a> {
    /// 按输入单元次序构造布局网格，不复制单元内容或改变输入表格。
    ///
    /// # 参数与返回
    /// `table` 必须在网格使用期间存活；返回网格拥有新分配的放置描述。
    /// 列坐标夹入列数，行上限由单元数量与行跨度上限推导；跨度按可用轨道夹取且至少为一。
    /// 这不检查单元重叠、源范围或完整表结构是否合法。
    /// Projects resolved parser coordinates into allocation-safe layout indices.
    pub(super) fn from_table(table: &'a RichTable) -> Self {
        let column_count = table.columns.len().max(1);
        let row_limit = table
            .cells
            .len()
            .saturating_mul(usize::from(MAX_RICH_TABLE_ROW_SPAN))
            .max(1);
        let mut row_count = 0;
        let cells = table
            .cells
            .iter()
            .map(|cell| {
                let column = usize::from(cell.column_index).min(column_count - 1);
                let column_span = usize::from(cell.column_span.max(1))
                    .min(column_count.saturating_sub(column))
                    .max(1);
                let row = usize::try_from(cell.row_index)
                    .unwrap_or(row_limit - 1)
                    .min(row_limit - 1);
                let row_span = usize::from(cell.row_span.max(1))
                    .min(usize::from(MAX_RICH_TABLE_ROW_SPAN))
                    .min(row_limit.saturating_sub(row))
                    .max(1);
                row_count = row_count.max(row.saturating_add(row_span));
                PlacedTableCell {
                    cell,
                    row,
                    column,
                    row_span,
                    column_span,
                }
            })
            .collect();
        Self {
            column_count,
            row_count,
            cells,
        }
    }
}
