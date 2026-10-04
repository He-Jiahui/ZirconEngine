use crate::ui::workbench::layout::{ActivityDrawerMode, ActivityDrawerSlot};
use crate::ui::workbench::model::WorkbenchViewModel;

use super::super::super::{LogicalRegionPreferredExtents, ShellRegionId};

/// 全局抽屉显隐与stack内容共同决定壳区域是否占位。
pub(super) fn tool_region_has_tabs(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> bool {
    let drawers_visible = model.drawer_ring.visible;
    drawers_visible
        && slots.iter().any(|slot| {
            model
                .tool_windows
                .get(slot)
                .is_some_and(|stack| stack.visible && !stack.tabs.is_empty())
        })
}

/// 同侧只要一个可见stack展开即可保留内容区；折叠状态不靠tab active推断。
pub(super) fn tool_region_is_expanded(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> bool {
    let drawers_visible = model.drawer_ring.visible;
    drawers_visible
        && slots.iter().any(|slot| {
            model.tool_windows.get(slot).is_some_and(|stack| {
                stack.visible
                    && !stack.tabs.is_empty()
                    && stack.mode != ActivityDrawerMode::Collapsed
            })
        })
}

/// 拖动临时值优先于持久化，再回退主题令牌；此边界收到的查询均为logical单位。
pub(super) fn tool_region_extent(
    model: &WorkbenchViewModel,
    region: ShellRegionId,
    slots: &[ActivityDrawerSlot],
    transient_region_preferred: LogicalRegionPreferredExtents<'_>,
    token_region_preferred: LogicalRegionPreferredExtents<'_>,
) -> f32 {
    transient_region_preferred
        .get(region)
        .or_else(|| persisted_tool_region_extent(model, slots))
        .or_else(|| token_region_preferred.get(region))
        .unwrap_or(0.0)
}

/// 同壳侧的多个可见drawer共享主轴，取最大保存偏好以容纳各slot。
fn persisted_tool_region_extent(
    model: &WorkbenchViewModel,
    slots: &[ActivityDrawerSlot],
) -> Option<f32> {
    slots
        .iter()
        .filter_map(|slot| model.drawer_ring.drawers.get(slot))
        .filter(|drawer| drawer.visible)
        .map(|drawer| drawer.extent)
        .fold(None, |maximum, extent| {
            Some(maximum.map_or(extent, |current: f32| current.max(extent)))
        })
}
