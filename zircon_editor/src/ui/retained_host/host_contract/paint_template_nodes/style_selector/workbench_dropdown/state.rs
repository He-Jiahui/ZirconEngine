use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::UiPainterResolvedState;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_dropdown_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_node_is_hot(
    node: &TemplatePaneNodeData,
) -> bool {
    node.hovered || node.dragging || node.drop_hovered || node.active_drag_target
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn dropdown_node_is_open(
    node: &TemplatePaneNodeData,
) -> bool {
    node.popup_open
}
