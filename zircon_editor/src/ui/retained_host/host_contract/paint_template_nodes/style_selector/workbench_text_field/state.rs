use super::super::resolved_state_for_node;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolved_text_field_state(
    node: &TemplatePaneNodeData,
) -> UiPainterResolvedState {
    let mut painter_state = resolved_state_for_node(node);
    if node.control_id.as_str() == "WorkbenchInputFocused" {
        painter_state.focused = true;
    }
    painter_state.resolved_state_for_family(UiPainterFamily::TextField)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_text_field_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
