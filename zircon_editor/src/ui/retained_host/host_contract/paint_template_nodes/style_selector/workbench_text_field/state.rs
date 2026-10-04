//! 文本框使用共享 TextField 状态族与宿主可见焦点契约；加载和禁用统一限制后续颜色覆盖。

use super::super::resolved_state_for_node;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn resolved_text_field_state(
    node: &TemplatePaneNodeData,
) -> UiPainterResolvedState {
    resolved_state_for_node(node).resolved_state_for_family(UiPainterFamily::TextField)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_text_field_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}
