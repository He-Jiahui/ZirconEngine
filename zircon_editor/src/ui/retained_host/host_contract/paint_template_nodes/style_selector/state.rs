//! 把宿主节点的实时标志与按钮声明状态合并为画家共用状态，供各控件选择器统一判定视觉优先级。
//! 焦点可见性在已投影的运行时输入中以模态标志为准；静态预览继续保留声明的焦点外观。

use super::super::super::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{ButtonInteractionState, UiPainterState};

/// 供各控件在绘制前统一合成状态；同时接纳实时节点标志与声明的按钮交互态。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolved_state_for_node(
    node: &TemplatePaneNodeData,
) -> UiPainterState {
    let style_state = node.button_style.interaction_state;
    UiPainterState {
        hovered: node.hovered || matches!(style_state, ButtonInteractionState::Hover),
        pressed: node.pressed
            || node.enter_pressed
            || matches!(style_state, ButtonInteractionState::Pressed),
        focused: node.focused || matches!(style_state, ButtonInteractionState::Focused),
        // Runtime focus modality is authoritative once projected. Legacy/static previews retain
        // their authored focus appearance until a live pointer or keyboard cause is known.
        focus_visible: focus_visible_for_node(node),
        disabled: node.disabled
            || node.button_style.disabled
            || matches!(style_state, ButtonInteractionState::Disabled),
        checked: node.checked,
        selected: node.selected,
        open: node.popup_open,
        dragging: node.dragging,
        drop_hovered: node.drop_hovered || node.active_drag_target,
        loading: node.button_style.loading
            || matches!(style_state, ButtonInteractionState::Loading),
    }
}

/// 使用已知的运行时焦点模态；未知模态的静态预览回退到声明焦点。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn focus_visible_for_node(
    node: &TemplatePaneNodeData,
) -> bool {
    if node.focus_visible_known {
        node.focus_visible
    } else {
        node.focus_visible
            || node.focused
            || matches!(
                node.button_style.interaction_state,
                ButtonInteractionState::Focused
            )
    }
}

#[cfg(test)]
#[path = "tests/state.rs"]
mod tests;
