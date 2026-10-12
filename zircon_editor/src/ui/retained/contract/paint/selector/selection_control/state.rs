//! 为具体选择控件映射共享状态族；热态、不可用和焦点边线判断分开，避免勾选或普通悬停借用键盘焦点环。

use super::model::WorkbenchSelectionControlKind;
use crate::ui::retained_host::host_contract::data::TemplatePaneNodeData;
use zircon_runtime_interface::ui::style::{UiPainterFamily, UiPainterResolvedState};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn family_for_kind(
    kind: WorkbenchSelectionControlKind,
) -> UiPainterFamily {
    match kind {
        WorkbenchSelectionControlKind::Checkbox => UiPainterFamily::Checkbox,
        WorkbenchSelectionControlKind::Radio => UiPainterFamily::Radio,
        WorkbenchSelectionControlKind::Toggle => UiPainterFamily::Toggle,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_unavailable_selection_state(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Disabled | UiPainterResolvedState::Loading
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_hot(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Hovered
            | UiPainterResolvedState::Pressed
            | UiPainterResolvedState::Dragging
            | UiPainterResolvedState::DropHovered
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn uses_focus_outline(
    state: UiPainterResolvedState,
) -> bool {
    matches!(
        state,
        UiPainterResolvedState::Focused | UiPainterResolvedState::DropHovered
    )
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn selection_node_is_hot(
    node: &TemplatePaneNodeData,
) -> bool {
    node.hovered || node.pressed || node.dragging || node.drop_hovered || node.active_drag_target
}
