use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_icon_button_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_node_is_selected(
    node: &TemplatePaneNodeData,
) -> bool {
    node.selected || node.checked
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_node_is_hot(
    node: &TemplatePaneNodeData,
) -> bool {
    node.hovered || node.popup_open || node.dragging || node.drop_hovered || node.active_drag_target
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn icon_button_node_uses_active_glyph(
    node: &TemplatePaneNodeData,
) -> bool {
    icon_button_node_is_selected(node)
        || node.popup_open
        || node.dragging
        || node.drop_hovered
        || node.active_drag_target
}
