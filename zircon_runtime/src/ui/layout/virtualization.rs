//! 固定步长列表的逻辑窗口、几何公式与有限物理槽映射。
//! `UiVirtualListSlotMap` 只规划索引映射；实体节点的创建、绑定和布局投影发布由表面所有者执行。
//! 物理容量预算先由调用方检查，再调用映射协调方法；请求容量不能代替该预算门。
use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

#[path = "virtualization/materialization.rs"]
mod materialization;

pub use materialization::{
    fixed_extent_slot_capacity, UiVirtualListSlotChange, UiVirtualListSlotMap,
};

/// 返回包含 overscan 的半开逻辑索引区间；不创建节点，也不移动物理槽位。
/// 生产调用方先钳制偏移并提供有限正步长；gap 应已计入 item_extent，不能再次单独叠加。
///
/// # 参数与约束
/// `offset` 是沿滚动轴的偏移，`viewport_extent` 是视口长度；`item_extent` 是含间距的有限正步长。
/// `item_count` 是逻辑总数，`overscan` 是两端额外项数。生产调用方应先把有限偏移限制到内容范围；
/// 本函数只钳制负偏移，不统一拒绝所有非有限输入。
///
/// # 返回与参考
/// 返回带 overscan 的半开区间，空列表或非正视口/步长返回默认空窗口。
/// 本函数不物化节点；有限物理槽的协调见 `UiVirtualListSlotMap::reconcile`，容量请求见 `fixed_extent_slot_capacity`。
pub fn compute_virtual_list_window(
    offset: f32,
    viewport_extent: f32,
    item_extent: f32,
    item_count: usize,
    overscan: usize,
) -> UiVirtualListWindow {
    if item_count == 0 || item_extent <= 0.0 || viewport_extent <= 0.0 {
        return UiVirtualListWindow::default();
    }

    let offset = offset.max(0.0);
    let first_visible = (offset / item_extent).floor() as usize;
    let last_visible_exclusive = ((offset + viewport_extent) / item_extent).ceil() as usize;

    UiVirtualListWindow {
        first_visible: first_visible.saturating_sub(overscan),
        last_visible_exclusive: last_visible_exclusive
            .saturating_add(overscan)
            .min(item_count),
    }
}

pub(crate) fn fixed_extent_virtual_list_step_extent(item_extent: f32, gap: f32) -> f32 {
    if !item_extent.is_finite() || item_extent <= 0.0 {
        return 0.0;
    }
    let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
    ((item_extent as f64 + gap as f64).min(f32::MAX as f64)) as f32
}

/// 内容几何使用逻辑总数而非当前物理子节点数，避免滚动条随窗口重绑改变长度。
pub(crate) fn fixed_extent_virtual_list_content_extent(
    logical_count: usize,
    item_extent: f32,
    gap: f32,
) -> f32 {
    if logical_count == 0 || !item_extent.is_finite() || item_extent <= 0.0 {
        return 0.0;
    }
    let gap = if gap.is_finite() { gap.max(0.0) } else { 0.0 };
    let item_total = logical_count as f64 * item_extent as f64;
    let gap_total = logical_count.saturating_sub(1) as f64 * gap as f64;
    ((item_total + gap_total).min(f32::MAX as f64)) as f32
}

pub(crate) fn fixed_extent_virtual_list_item_offset(
    logical_index: usize,
    item_extent: f32,
    gap: f32,
) -> f32 {
    let step_extent = fixed_extent_virtual_list_step_extent(item_extent, gap);
    ((logical_index as f64 * step_extent as f64).min(f32::MAX as f64)) as f32
}

#[cfg(test)]
#[path = "tests/virtualization.rs"]
mod tests;
