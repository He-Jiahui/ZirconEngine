//! 在几何预算内求解表格逻辑列宽和行高。
//! 固定的收缩策略与最小长度可以优先于可用宽度；成功准入不等于表格一定装入容器。

use crate::text::{
    RichTableColumn, TextLayoutAxisConstraint, TextLayoutGeometryBudget,
    TextLayoutGeometryViolation,
};

/// 收缩比例搜索的固定迭代次数，提供有限步骤的近似解。
const SHRINK_SCALE_SEARCH_STEPS: usize = 24;

/// 一个单元对逻辑列轨道的长度需求；跨度需求包含其内部列间距。
#[derive(Clone, Copy, Debug)]
pub(super) struct PreferredColumnExtent {
    pub column: usize,
    pub column_span: usize,
    pub extent: f32,
}

/// 一个单元对逻辑行轨道的长度需求；当前行求解不另加行间距。
#[derive(Clone, Copy, Debug)]
pub(super) struct RowExtentConstraint {
    pub row: usize,
    pub row_span: usize,
    pub extent: f32,
}

/// 汇合单列和跨列需求，返回拥有的列长度向量。
///
/// # 参数
/// `available_track_extent` 已由调用层扣除总列间距；`gap` 用于解释跨列单元需求中的内部间距。
/// 空列策略按一列默认策略处理，`minimum` 与输入长度均须满足几何预算。
///
/// # 返回与约束
/// 先处理单列需求，再按跨度由小到大补足跨列需求，最后应用收缩或扩张策略。
/// 失败返回几何 violation，不修改输入策略或需求；成功仍可能因最小值或禁止收缩而超过可用宽度。
pub(super) fn resolve_column_extents(
    columns: &[RichTableColumn],
    preferred_cells: &[PreferredColumnExtent],
    available_track_extent: TextLayoutAxisConstraint,
    gap: f32,
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<Vec<f32>, TextLayoutGeometryViolation> {
    let gap = budget.admit_axis_extent(gap)?;
    let minimum = budget.admit_axis_extent(minimum)?;
    let policies = if columns.is_empty() {
        vec![RichTableColumn::default()]
    } else {
        columns.to_vec()
    };
    let mut widths = vec![minimum; policies.len()];

    for cell in preferred_cells.iter().filter(|cell| cell.column_span == 1) {
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        if let Some(width) = widths.get_mut(cell.column) {
            *width = width.max(cell_extent);
        }
    }

    let mut spanning_cells = preferred_cells
        .iter()
        .filter(|cell| cell.column_span > 1)
        .collect::<Vec<_>>();
    spanning_cells.sort_by_key(|cell| cell.column_span);
    for cell in spanning_cells {
        let end = cell
            .column
            .saturating_add(cell.column_span)
            .min(widths.len());
        if cell.column >= end {
            continue;
        }
        let track_extent =
            checked_sum_accumulated(widths[cell.column..end].iter().copied(), budget)?;
        let internal_gap_extent =
            budget.checked_scale_accumulated(gap, end.saturating_sub(cell.column + 1))?;
        let track_extent = budget.checked_add_accumulated(track_extent, internal_gap_extent)?;
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        let deficit = (cell_extent - track_extent).max(0.0);
        distribute_column_deficit(
            &mut widths[cell.column..end],
            &policies[cell.column..end],
            deficit,
            budget,
        )?;
    }

    fit_columns_to_available_width(
        &mut widths,
        &policies,
        available_track_extent,
        minimum,
        budget,
    )?;
    for width in &widths {
        budget.admit_axis_extent(*width)?;
    }
    Ok(widths)
}

/// 将单行和跨行需求合成为新的行长度向量。
/// 行数由布局网格提供；先取单行最大需求，再把跨行缺额均分到夹取后的有效范围。
/// 无有效覆盖范围的需求被跳过；所有采用的长度与累计量接受预算检查，失败返回 violation。
pub(super) fn resolve_row_extents(
    row_count: usize,
    cells: &[RowExtentConstraint],
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<Vec<f32>, TextLayoutGeometryViolation> {
    let minimum = budget.admit_axis_extent(minimum)?;
    let mut heights = vec![minimum; row_count];
    for cell in cells.iter().filter(|cell| cell.row_span == 1) {
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        if let Some(height) = heights.get_mut(cell.row) {
            *height = height.max(cell_extent);
        }
    }

    let mut spanning_cells = cells
        .iter()
        .filter(|cell| cell.row_span > 1)
        .collect::<Vec<_>>();
    spanning_cells.sort_by_key(|cell| cell.row_span);
    for cell in spanning_cells {
        let end = cell.row.saturating_add(cell.row_span).min(heights.len());
        if cell.row >= end {
            continue;
        }
        let current = checked_sum_accumulated(heights[cell.row..end].iter().copied(), budget)?;
        let cell_extent = budget.admit_axis_extent(cell.extent)?;
        let deficit = (cell_extent - current).max(0.0);
        let share = deficit / end.saturating_sub(cell.row).max(1) as f32;
        budget.admit_axis_extent(share)?;
        for height in &mut heights[cell.row..end] {
            *height = budget
                .checked_add_accumulated(*height, share)
                .and_then(|height| budget.admit_axis_extent(height))?;
        }
    }
    Ok(heights)
}

/// 就地分配跨列缺额：有扩张权重时按权重，否则在所有覆盖列间均分。
/// 非正缺额或空范围不调整；失败可能发生在部分列已更新后，调用层应放弃局部求解结果。
fn distribute_column_deficit(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    deficit: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    budget.admit_accumulated_extent(deficit)?;
    if deficit <= 0.0 || widths.is_empty() {
        return Ok(());
    }
    let expand_ratio = policies
        .iter()
        .filter(|column| column.expand)
        .map(|column| u32::from(column.expand_ratio))
        .sum::<u32>();
    if expand_ratio > 0 {
        for (width, column) in widths.iter_mut().zip(policies) {
            if column.expand {
                let share = deficit * f32::from(column.expand_ratio) / expand_ratio as f32;
                *width = budget
                    .checked_add_accumulated(*width, share)
                    .and_then(|width| budget.admit_axis_extent(width))?;
            }
        }
    } else {
        let share = deficit / widths.len() as f32;
        budget.admit_axis_extent(share)?;
        for width in widths {
            *width = budget
                .checked_add_accumulated(*width, share)
                .and_then(|width| budget.admit_axis_extent(width))?;
        }
    }
    Ok(())
}

/// 就地拟合已扣间距的可用轨道长度；无界约束直接保留自然长度。
/// 策略与长度须一一对应；仅允许收缩的列参与缩减，扩张则由扩张权重决定。
/// 最小长度或禁止收缩的列可使结果仍超过可用长度；错误不承诺回滚已修改的列。
fn fit_columns_to_available_width(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    available_width: TextLayoutAxisConstraint,
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    let Some(available_width) = available_width.bounded_extent() else {
        return Ok(());
    };
    let preferred_total = checked_sum_accumulated(widths.iter().copied(), budget)?;
    if preferred_total > available_width && preferred_total > 0.0 {
        let fixed_total = checked_sum_accumulated(
            widths
                .iter()
                .zip(policies)
                .filter(|(_, column)| !column.shrink)
                .map(|(width, _)| *width),
            budget,
        )?;
        let shrink_budget = (available_width - fixed_total).max(0.0);
        shrink_columns_to_budget(widths, policies, shrink_budget, minimum, budget)?;
    } else if preferred_total < available_width {
        let expand_ratio = policies
            .iter()
            .filter(|column| column.expand)
            .map(|column| u32::from(column.expand_ratio))
            .sum::<u32>();
        if expand_ratio > 0 {
            let extra = available_width - preferred_total;
            for (width, column) in widths.iter_mut().zip(policies) {
                if column.expand {
                    let share = extra * f32::from(column.expand_ratio) / expand_ratio as f32;
                    *width = budget
                        .checked_add_accumulated(*width, share)
                        .and_then(|width| budget.admit_axis_extent(width))?;
                }
            }
        }
    }
    Ok(())
}

/// 在保留列最小长度的前提下缩减允许收缩的列，其他列不变。
/// 先检查最小总量；若目标不可行，仍保留最小长度，否则用固定轮数的二分搜索近似比例。
/// 输入宽度与策略须对应，失败不保证恢复此前已修改的宽度。
fn shrink_columns_to_budget(
    widths: &mut [f32],
    policies: &[RichTableColumn],
    shrink_budget: f32,
    minimum: f32,
    budget: TextLayoutGeometryBudget,
) -> Result<(), TextLayoutGeometryViolation> {
    let shrinkable_count = policies.iter().filter(|column| column.shrink).count();
    if shrinkable_count == 0 {
        return Ok(());
    }

    let minimum_total = budget.checked_scale_accumulated(minimum, shrinkable_count)?;
    if shrink_budget <= minimum_total {
        for (width, column) in widths.iter_mut().zip(policies) {
            if column.shrink {
                *width = minimum;
            }
        }
        return Ok(());
    }

    let preferred_shrink_total = checked_sum_accumulated(
        widths
            .iter()
            .zip(policies)
            .filter(|(_, column)| column.shrink)
            .map(|(width, _)| *width),
        budget,
    )?;
    if preferred_shrink_total <= shrink_budget {
        return Ok(());
    }

    let mut lower_scale = 0.0_f32;
    let mut upper_scale = 1.0_f32;
    for _ in 0..SHRINK_SCALE_SEARCH_STEPS {
        let scale = (lower_scale + upper_scale) * 0.5;
        let resolved_shrink_total = checked_sum_accumulated(
            widths
                .iter()
                .zip(policies)
                .filter(|(_, column)| column.shrink)
                .map(|(width, _)| (*width * scale).max(minimum)),
            budget,
        )?;
        if resolved_shrink_total > shrink_budget {
            upper_scale = scale;
        } else {
            lower_scale = scale;
        }
    }

    for (width, column) in widths.iter_mut().zip(policies) {
        if column.shrink {
            *width = budget.admit_axis_extent((*width * lower_scale).max(minimum))?;
        }
    }
    Ok(())
}

/// 消费长度迭代器，逐项检查单轴限制，并检查累计长度；拒绝时立即返回 violation。
fn checked_sum_accumulated(
    mut values: impl Iterator<Item = f32>,
    budget: TextLayoutGeometryBudget,
) -> Result<f32, TextLayoutGeometryViolation> {
    values.try_fold(0.0, |total, value| {
        let value = budget.admit_axis_extent(value)?;
        budget.checked_add_accumulated(total, value)
    })
}

#[cfg(test)]
#[path = "tests/sizing.rs"]
mod tests;
