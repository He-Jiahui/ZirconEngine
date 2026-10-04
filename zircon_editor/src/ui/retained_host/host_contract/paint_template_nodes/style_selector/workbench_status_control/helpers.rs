//! 状态条选择器共享的节点语义查询；持久选择、热态和主动图标含义分别用于 Focused 配方。
//! 零透明度的原始声明色视为未提供覆盖；此约定仅适用于使用该辅助函数的颜色通道。

use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use crate::ui::retained_host::primitives::Color;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn declared_color(
    color: Color,
) -> Option<[u8; 4]> {
    (color.a > 0).then_some([color.r, color.g, color.b, color.a])
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_status_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_node_is_selected(
    node: &TemplatePaneNodeData,
) -> bool {
    node.selected || node.checked
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_node_is_hot(
    node: &TemplatePaneNodeData,
) -> bool {
    node.hovered || node.popup_open || node.dragging || node.drop_hovered || node.active_drag_target
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn status_node_uses_active_glyph(
    node: &TemplatePaneNodeData,
) -> bool {
    status_node_is_selected(node)
        || node.popup_open
        || node.dragging
        || node.drop_hovered
        || node.active_drag_target
}
