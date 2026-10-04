use zircon_runtime_interface::ui::layout::{
    UiScrollState, UiScrollableBoxConfig, UiVirtualListWindow,
};

use super::{compute_virtual_list_window, fixed_extent_virtual_list_step_extent};

#[derive(Clone, Copy, Debug, PartialEq)]
/// 滚轮更新和重新安排共用的状态方案，使可见范围失效与实际偏移钳制使用同一套几何。
pub(crate) struct UiScrollVirtualizationPlan {
    pub scroll_state: UiScrollState,
    pub virtual_window: Option<UiVirtualListWindow>,
    pub visible_range_changed: bool,
}

/// 固定高度或宽度列表的窗口投影；item_extent 与 gap 合成步长，child_count 应使用逻辑行数。
/// 调用方应先按内容范围钳制偏移，未启用虚拟化时返回 None。
pub fn virtual_window_for_scrollable_box(
    config: UiScrollableBoxConfig,
    offset: f32,
    child_count: usize,
    viewport_extent: f32,
) -> Option<UiVirtualListWindow> {
    let virtualization = config.virtualization?;
    let step_extent = fixed_extent_virtual_list_step_extent(virtualization.item_extent, config.gap);
    Some(compute_virtual_list_window(
        offset,
        viewport_extent,
        step_extent,
        child_count,
        virtualization.overscan,
    ))
}

/// 先准备新状态再由树或安排阶段提交；范围变化标志供表面判断是否需要重绑物理槽位。
pub(crate) fn plan_scrollable_virtual_window(
    config: UiScrollableBoxConfig,
    previous_state: UiScrollState,
    previous_window: Option<UiVirtualListWindow>,
    requested_offset: f32,
    child_count: usize,
    viewport_extent: f32,
    content_extent: f32,
) -> UiScrollVirtualizationPlan {
    let viewport_extent = viewport_extent.max(0.0);
    let content_extent = content_extent.max(0.0);
    let max_offset = (content_extent - viewport_extent).max(0.0);
    let offset = requested_offset.max(0.0).min(max_offset);
    let scroll_state = UiScrollState {
        offset,
        viewport_extent,
        content_extent,
    };
    let virtualization_enabled = config.virtualization.is_some();
    let virtual_window =
        virtual_window_for_scrollable_box(config, offset, child_count, viewport_extent).or(Some(
            UiVirtualListWindow {
                first_visible: 0,
                last_visible_exclusive: child_count,
            },
        ));

    UiScrollVirtualizationPlan {
        scroll_state,
        virtual_window,
        visible_range_changed: virtualization_enabled
            && (previous_window != virtual_window
                || (previous_state.viewport_extent - viewport_extent).abs() > f32::EPSILON
                || (previous_state.content_extent - content_extent).abs() > f32::EPSILON),
    }
}
