//! 选中项颜色受禁用和加载优先级约束；热态查询供 Focused 分支恢复同时存在的悬停、展开或拖放反馈。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_segmented_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn segmented_node_is_hot(
    node: &TemplatePaneNodeData,
) -> bool {
    node.hovered || node.popup_open || node.dragging || node.drop_hovered || node.active_drag_target
}
