use zircon_runtime_interface::ui::layout::UiVirtualListWindow;

mod materialization;

pub use materialization::{
    fixed_extent_slot_capacity, UiVirtualListSlotChange, UiVirtualListSlotMap,
};

/// 返回包含 overscan 的半开逻辑索引区间；不创建节点，也不移动物理槽位。
/// 生产调用方先钳制偏移并提供有限正步长；gap 应已计入 item_extent，不能再次单独叠加。
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
